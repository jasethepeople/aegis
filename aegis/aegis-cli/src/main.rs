use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use aegis_core::*;
use aegis_engine::{Engine, EngineBuilder};
use aegis_target::{TargetFactory, TargetValidator};
use aegis_corpus::CorpusMinimizer;

#[derive(Parser)]
#[command(name = "aegis")]
#[command(about = "Aegis - Production-grade fuzzing platform")]
#[command(version = "0.4.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)] name: String,
        #[arg(short, long)] command: String,
        #[arg(short, long, default_value = "binary")] target_type: String,
        #[arg(short, long, default_value = "stdin")] input_mode: String,
        #[arg(short, long, default_value = "aegis.json")] output: PathBuf,
        #[arg(short, long, default_value = "5")] timeout: u64,
        #[arg(short, long, default_value = "256")] memory: usize,
    },
    Run {
        #[arg(short, long)] target: PathBuf,
        #[arg(short, long)] seeds: Option<PathBuf>,
        #[arg(short, long, default_value = "./corpus")] corpus: String,
        #[arg(short, long)] max_iterations: Option<u64>,
        #[arg(short, long)] max_duration: Option<u64>,
        #[arg(short, long, default_value = "1")] workers: usize,
        #[arg(long)] stop_on_crash: bool,
        #[arg(short, long)] name: Option<String>,
    },
    Validate { target: PathBuf },
    Replay { crash: String, #[arg(short, long)] target: PathBuf },
    Minimize { input: PathBuf, #[arg(short, long)] target: PathBuf, #[arg(short, long)] output: Option<PathBuf> },
    Version,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name, command, target_type, input_mode, output, timeout, memory } => {
            let target = TargetConfig {
                id: uuid::Uuid::new_v4(), name, target_type: TargetType::Binary, command,
                args: vec![], env: vec![], working_dir: None,
                input_mode: match input_mode.as_str() {
                    "file" => InputMode::File { path_template: "/tmp/aegis_input.bin".into() },
                    "network" => InputMode::Network { host: "127.0.0.1".into(), port: 8080 },
                    _ => InputMode::Stdin,
                },
                timeout: Duration::from_secs(timeout), memory_limit_mb: memory, cpu_limit: None,
                instrumentation: InstrumentationConfig::default(),
            };
            std::fs::write(&output, serde_json::to_string_pretty(&target)?)?;
            println!("Target config written to: {}", output.display());
        }
        Commands::Run { target, seeds, corpus, max_iterations, max_duration, workers, stop_on_crash, name } => {
            let target_config = TargetFactory::from_json(&target)?;
            let errors = TargetValidator::validate(&target_config);
            if !errors.is_empty() { for e in errors { eprintln!("Validation error: {}", e); } std::process::exit(1); }

            let mut seed_inputs = Vec::new();
            if let Some(seeds_dir) = seeds {
                if seeds_dir.is_dir() {
                    for entry in std::fs::read_dir(seeds_dir)? {
                        let path = entry?.path();
                        if path.is_file() { seed_inputs.push(std::fs::read(&path)?); }
                    }
                }
            }
            if seed_inputs.is_empty() { seed_inputs.push(vec![0u8; 10]); }

            let campaign = CampaignConfig {
                id: uuid::Uuid::new_v4(),
                name: name.unwrap_or_else(|| "campaign".into()),
                target_id: target_config.id, seed_inputs,
                max_iterations, max_duration: max_duration.map(Duration::from_secs),
                parallel_workers: workers,
                mutation_config: MutationConfig::default(),
                corpus_config: CorpusConfig { max_size: 10000, storage_path: corpus, enable_minimization: true, power_schedule: PowerSchedule::Fast },
                stop_on_first_crash: stop_on_crash,
            };

            let mut engine = EngineBuilder::new().campaign(campaign).target(target_config).build()?;
            let stats = engine.run()?;
            println!("\n=== Campaign Complete ===");
            println!("Total executions: {}", stats.total_execs);
            println!("Exec/sec: {:.1}", stats.execs_per_sec);
            println!("Corpus size: {}", stats.corpus_size);
            println!("Unique crashes: {}", stats.unique_crashes);
        }
        Commands::Validate { target } => {
            let t = TargetFactory::from_json(&target)?;
            let errors = TargetValidator::validate(&t);
            if errors.is_empty() { println!("Target is valid"); } else { for e in errors { println!("Error: {}", e); } }
        }
        Commands::Replay { crash, target } => {
            let target_config = TargetFactory::from_json(&target)?;
            let executor = ExecutorBuilder::new().target(target_config).build_process()?;
            let crash_data = std::fs::read(&crash).unwrap_or_default();
            let input = ByteInput::new(crash_data);
            let result = executor.execute(&input)?;
            match result.status {
                ExecutionStatus::Crash(ref info) => println!("Crash reproduced: {}", info.crash_type),
                ExecutionStatus::Success => println!("No crash - input may be minimized"),
                _ => println!("Result: {:?}", result.status),
            }
        }
        Commands::Minimize { input, target, output } => {
            let target_config = TargetFactory::from_json(&target)?;
            let executor = ExecutorBuilder::new().target(target_config).build_process()?;
            let original = std::fs::read(&input)?;
            println!("Original: {} bytes", original.len());
            let minimized = CorpusMinimizer::minimize(&original, |data| {
                let input = ByteInput::new(data.to_vec());
                match executor.execute(&input) {
                    Ok(r) => matches!(r.status, ExecutionStatus::Crash(_)),
                    Err(_) => false,
                }
            });
            println!("Minimized: {} bytes", minimized.len());
            let out = output.unwrap_or_else(|| { let mut p = input.clone(); p.set_file_name("minimized.bin"); p });
            std::fs::write(&out, &minimized)?;
        }
        Commands::Version => println!("Aegis v0.4.0"),
    }
    Ok(())
}
