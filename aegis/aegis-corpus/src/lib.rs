use std::collections::HashMap;
use rand::RngCore;
use rand::seq::SliceRandom;
use rand::distributions::WeightedIndex;
use rand::distributions::Distribution;
use parking_lot::RwLock;
use aegis_core::*;

#[derive(Clone, Debug)]
pub struct CorpusEntry {
    pub input: ByteInput,
    pub id: uuid::Uuid,
    pub coverage_edges: Vec<u64>,
    pub exec_time: std::time::Duration,
    pub depth: u32,
    pub fav_factor: f64,
    pub selected_count: u64,
    pub new_edges_found: usize,
}

impl CorpusEntry {
    pub fn new(input: ByteInput, coverage: &CoverageSnapshot, exec_time: std::time::Duration) -> Self {
        let new_edges = coverage.new_edges.len();
        let fav_factor = if exec_time.as_secs_f64() > 0.0 { coverage.edge_count as f64 / exec_time.as_secs_f64() } else { coverage.edge_count as f64 };
        Self { id: uuid::Uuid::new_v4(), input, coverage_edges: coverage.new_edges.clone(), exec_time, depth: 0, fav_factor, selected_count: 0, new_edges_found: new_edges }
    }
    pub fn power_score(&self, schedule: &PowerSchedule, total_edges: usize) -> f64 {
        let base = match schedule {
            PowerSchedule::Fast => self.fav_factor * 100.0,
            PowerSchedule::Explore => (self.new_edges_found as f64 + 1.0) * 10.0,
            PowerSchedule::Exploit => if total_edges > 0 { (self.coverage_edges.len() as f64 / total_edges as f64) * 1000.0 } else { 1.0 },
            PowerSchedule::Coe => { let age = (chrono::Utc::now() - chrono::Utc::now()).num_hours() as f64; (self.fav_factor * 100.0) / (1.0 + age) },
            PowerSchedule::Lin => self.coverage_edges.len() as f64 + 1.0,
            PowerSchedule::Quad => { let e = self.coverage_edges.len() as f64; e * e + 1.0 },
        };
        (base / (1.0 + (self.selected_count as f64).sqrt() * 0.1)).max(0.1)
    }
}

pub struct InMemoryCorpus {
    entries: RwLock<Vec<CorpusEntry>>,
    schedule: PowerSchedule,
    max_size: usize,
    total_edges: std::sync::atomic::AtomicUsize,
}

impl InMemoryCorpus {
    pub fn new(schedule: PowerSchedule, max_size: usize, _seed: u64) -> Self {
        Self { entries: RwLock::new(Vec::new()), schedule, max_size, total_edges: std::sync::atomic::AtomicUsize::new(0) }
    }
    pub fn add_seed(&self, data: Vec<u8>) -> AegisResult<()> {
        let input = ByteInput::new(data);
        let coverage = CoverageSnapshot::default();
        let entry = CorpusEntry::new(input, &coverage, std::time::Duration::from_millis(1));
        self.entries.write().push(entry);
        Ok(())
    }
    pub fn len(&self) -> usize { self.entries.read().len() }
    pub fn is_empty(&self) -> bool { self.entries.read().is_empty() }
    pub fn total_edges(&self) -> usize { self.total_edges.load(std::sync::atomic::Ordering::Relaxed) }
}

impl Corpus for InMemoryCorpus {
    fn add(&mut self, input: ByteInput, coverage: &CoverageSnapshot) -> AegisResult<bool> {
        if !coverage.has_new_coverage() { return Ok(false); }
        let mut entries = self.entries.write();
        if entries.len() >= self.max_size {
            if let Some(min_idx) = entries.iter().enumerate().min_by(|(_, a), (_, b)| {
                let sa = a.power_score(&self.schedule, self.total_edges());
                let sb = b.power_score(&self.schedule, self.total_edges());
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            }).map(|(i, _)| i) { entries.remove(min_idx); }
        }
        let new_edges = coverage.new_edges.len();
        entries.push(CorpusEntry::new(input, coverage, std::time::Duration::from_millis(1)));
        drop(entries);
        let current = self.total_edges();
        if current + new_edges > current { self.total_edges.store(current + new_edges, std::sync::atomic::Ordering::Relaxed); }
        Ok(true)
    }
    fn select(&self, rng: &mut dyn RngCore) -> Option<&ByteInput> {
        let entries = self.entries.read();
        if entries.is_empty() { return None; }
        let total_edges = self.total_edges();
        let weights: Vec<f64> = entries.iter().map(|e| e.power_score(&self.schedule, total_edges)).collect();
        if let Ok(dist) = WeightedIndex::new(&weights) {
            let idx = dist.sample(rng);
            Some(&entries[idx].input)
        } else {
            entries.choose(rng).map(|e| &e.input)
        }
    }
    fn inputs(&self) -> Vec<&ByteInput> { self.entries.read().iter().map(|e| &e.input).collect() }
    fn save(&self) -> AegisResult<()> { Ok(()) }
    fn load(&mut self) -> AegisResult<()> { Ok(()) }
}

pub struct CorpusMinimizer;

impl CorpusMinimizer {
    pub fn minimize(input: &[u8], preserve: impl Fn(&[u8]) -> bool) -> Vec<u8> {
        let mut current = input.to_vec();
        let mut changed = true;
        while changed && current.len() > 1 {
            changed = false;
            for i in (0..current.len()).rev() {
                let mut test = current.clone(); test.remove(i);
                if preserve(&test) { current = test; changed = true; }
            }
            let mut chunk_size = current.len() / 2;
            while chunk_size > 0 {
                for i in (0..current.len()).step_by(chunk_size).rev() {
                    if i + chunk_size <= current.len() {
                        let mut test = current.clone(); test.drain(i..i+chunk_size);
                        if preserve(&test) { current = test; changed = true; break; }
                    }
                }
                chunk_size /= 2;
            }
        }
        current
    }
}
