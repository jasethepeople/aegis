use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use rand::SeedableRng;
use tracing::{info, debug, trace, warn};
use aegis_core::*;
use aegis_mutate::MutatorEngine;
use aegis_executor::{ProcessExecutor, ExecutorBuilder};
use aegis_corpus::{InMemoryCorpus, CorpusMinimizer};

pub struct Engine {
    config: CampaignConfig,
    target_config: TargetConfig,
    corpus: Box<dyn Corpus>,
    mutator: MutatorEngine,
    executor: Box<dyn Executor>,
    stats: Mutex<CampaignStats>,
    crashes: Mutex<Vec<CrashReport>>,
    running: AtomicBool,
    exec_count: AtomicU64,
    start_time: Mutex<Option<Instant>>,
}

impl Engine {
    pub fn new(config: CampaignConfig, target_config: TargetConfig) -> AegisResult<Self> {
        let seed = config.seed_inputs.first().and_then(|s| s.first()).copied().unwrap_or(42) as u64;
        let mutator = MutatorEngine::new(seed);
        let corpus: Box<dyn Corpus> = Box::new(InMemoryCorpus::new(config.corpus_config.power_schedule.clone(), config.corpus_config.max_size, seed));
        let executor = ExecutorBuilder::new().target(target_config.clone()).build_process()?;
        Ok(Self { config, target_config, corpus, mutator, executor: Box::new(executor),
                  stats: Mutex::new(CampaignStats::default()), crashes: Mutex::new(Vec::new()),
                  running: AtomicBool::new(false), exec_count: AtomicU64::new(0),
                  start_time: Mutex::new(None) })
    }

    pub fn add_seeds(&mut self, seeds: Vec<Vec<u8>>) -> AegisResult<()> {
        for seed in seeds { let input = ByteInput::new(seed); let coverage = CoverageSnapshot::default(); self.corpus.add(input, &coverage)?; }
        Ok(())
    }

    pub fn run(&mut self) -> AegisResult<CampaignStats> {
        self.running.store(true, Ordering::SeqCst);
        *self.start_time.lock() = Some(Instant::now());
        info!("Starting campaign: {}", self.config.name);

        if self.corpus.is_empty() {
            if !self.config.seed_inputs.is_empty() { self.add_seeds(self.config.seed_inputs.clone())?; }
            else { self.add_seeds(vec![vec![0u8; 10]])?; }
        }

        let mut last_stats = Instant::now();
        let mut iteration: u64 = 0;
        while self.running.load(Ordering::SeqCst) {
            if let Some(max) = self.config.max_iterations { if iteration >= max { info!("Max iterations reached"); break; } }
            if let Some(max_dur) = self.config.max_duration {
                if let Some(start) = *self.start_time.lock() { if start.elapsed() >= max_dur { info!("Max duration reached"); break; } }
            }
            match self.fuzz_iteration(iteration) {
                Ok(found_crash) => { if found_crash && self.config.stop_on_first_crash { break; } }
                Err(e) => warn!("Iteration {} failed: {}", iteration, e),
            }
            iteration += 1;
            self.exec_count.fetch_add(1, Ordering::Relaxed);
            if last_stats.elapsed() >= Duration::from_secs(5) { self.update_stats(); self.print_stats(); last_stats = Instant::now(); }
        }
        self.running.store(false, Ordering::SeqCst);
        self.update_stats();
        let final_stats = self.stats.lock().clone();
        info!("Campaign complete: {} execs, {} crashes", final_stats.total_execs, final_stats.total_crashes);
        Ok(final_stats)
    }

    fn fuzz_iteration(&mut self, iteration: u64) -> AegisResult<bool> {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(iteration.wrapping_add(42));
        let base_input = self.corpus.select(&mut rng).cloned().unwrap_or_else(|| ByteInput::new(vec![0u8; 10]));
        let mutated_bytes = self.mutator.mutate(&base_input)?;
        let mutated_input = ByteInput::new(mutated_bytes);
        let result = self.executor.execute(&mutated_input)?;
        let is_crash = matches!(result.status, ExecutionStatus::Crash(_));
        let is_hang = matches!(result.status, ExecutionStatus::Hang | ExecutionStatus::Timeout);
        if is_crash { self.handle_crash(mutated_input, result)?; return Ok(true); }
        if is_hang { let mut stats = self.stats.lock(); stats.hang_count += 1; }
        if result.coverage.has_new_coverage() { let _ = self.corpus.add(mutated_input, &result.coverage)?; }
        Ok(false)
    }

    fn handle_crash(&self, input: ByteInput, result: ExecutionResult) -> AegisResult<()> {
        let crash_info = match &result.status { ExecutionStatus::Crash(info) => info.clone(), _ => return Ok(()) };
        let report = CrashReport { id: uuid::Uuid::new_v4(), campaign_id: self.config.id, input, result,
            minimized_input: None, dedup_key: format!("{}_{}", crash_info.crash_type, crash_info.stack_hash),
            timestamp: chrono::Utc::now(), triaged: false, severity: None };
        let mut crashes = self.crashes.lock();
        if !crashes.iter().any(|c| c.dedup_key == report.dedup_key) {
            crashes.push(report);
            let mut stats = self.stats.lock(); stats.unique_crashes += 1;
            info!("CRASH: {} (unique)", crash_info.crash_type);
        } else {
            let mut stats = self.stats.lock(); stats.total_crashes += 1;
        }
        Ok(())
    }

    fn update_stats(&self) {
        let mut stats = self.stats.lock();
        stats.total_execs = self.exec_count.load(Ordering::Relaxed);
        if let Some(start) = *self.start_time.lock() {
            let elapsed = start.elapsed().as_secs_f64();
            if elapsed > 0.0 { stats.execs_per_sec = stats.total_execs as f64 / elapsed; }
        }
        stats.corpus_size = self.corpus.len();
        stats.last_update = Some(chrono::Utc::now());
    }

    fn print_stats(&self) {
        let stats = self.stats.lock();
        let elapsed = self.start_time.lock().map(|s| s.elapsed()).unwrap_or(Duration::from_secs(0));
        info!("[{:02}:{:02}:{:02}] execs: {} | {}/sec | corpus: {} | crashes: {} ({} unique) | hangs: {}",
            elapsed.as_secs() / 3600, (elapsed.as_secs() % 3600) / 60, elapsed.as_secs() % 60,
            stats.total_execs, stats.execs_per_sec as u64, stats.corpus_size,
            stats.total_crashes, stats.unique_crashes, stats.hang_count);
    }

    pub fn stop(&mut self) -> AegisResult<()> { self.running.store(false, Ordering::SeqCst); Ok(()) }
    pub fn stats(&self) -> CampaignStats { self.stats.lock().clone() }
    pub fn crashes(&self) -> Vec<CrashReport> { self.crashes.lock().clone() }
    pub fn is_running(&self) -> bool { self.running.load(Ordering::SeqCst) }
}

impl FuzzingEngine for Engine {
    fn run(&mut self, config: &CampaignConfig) -> AegisResult<CampaignStats> { self.config = config.clone(); self.run() }
    fn stop(&mut self) -> AegisResult<()> { self.stop() }
    fn stats(&self) -> CampaignStats { self.stats() }
    fn crashes(&self) -> Vec<CrashReport> { self.crashes() }
}

pub struct EngineBuilder {
    campaign_config: Option<CampaignConfig>,
    target_config: Option<TargetConfig>,
}

impl EngineBuilder {
    pub fn new() -> Self { Self { campaign_config: None, target_config: None } }
    pub fn campaign(mut self, config: CampaignConfig) -> Self { self.campaign_config = Some(config); self }
    pub fn target(mut self, config: TargetConfig) -> Self { self.target_config = Some(config); self }
    pub fn build(self) -> AegisResult<Box<dyn FuzzingEngine>> {
        let campaign = self.campaign_config.ok_or_else(|| AegisError::ConfigError("No campaign".into()))?;
        let target = self.target_config.ok_or_else(|| AegisError::ConfigError("No target".into()))?;
        Ok(Box::new(Engine::new(campaign, target)?))
    }
}

impl Default for EngineBuilder { fn default() -> Self { Self::new() } }
