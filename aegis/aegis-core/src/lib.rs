//! Aegis Core - Shared types, traits, and primitives for the fuzzing platform.

use std::fmt;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AegisError {
    #[error("Execution failed: {0}")]
    ExecutionError(String),
    #[error("Coverage error: {0}")]
    CoverageError(String),
    #[error("Corpus error: {0}")]
    CorpusError(String),
    #[error("Mutation error: {0}")]
    MutationError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
}

pub type AegisResult<T> = Result<T, AegisError>;

pub trait Input: Clone + Send + Sync + fmt::Debug {
    fn as_bytes(&self) -> &[u8];
    fn from_bytes(bytes: &[u8]) -> Result<Self, AegisError> where Self: Sized;
    fn len(&self) -> usize { self.as_bytes().len() }
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputMetadata {
    pub source: InputSource,
    pub generation: u64,
    pub parent_id: Option<Uuid>,
    pub mutation_chain: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputSource { Seed, Generated, Mutated, Minimized, Corpus }
impl Default for InputSource { fn default() -> Self { InputSource::Generated } }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteInput {
    data: Vec<u8>,
    metadata: InputMetadata,
}

impl ByteInput {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, metadata: InputMetadata { timestamp: Utc::now(), ..Default::default() } }
    }
    pub fn metadata(&self) -> &InputMetadata { &self.metadata }
    pub fn data(&self) -> &[u8] { &self.data }
    pub fn into_data(self) -> Vec<u8> { self.data }
}

impl Input for ByteInput {
    fn as_bytes(&self) -> &[u8] { &self.data }
    fn from_bytes(bytes: &[u8]) -> Result<Self, AegisError> { Ok(Self::new(bytes.to_vec())) }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub exec_time: Duration,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub coverage: CoverageSnapshot,
    pub input_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Crash(CrashInfo),
    Hang,
    Timeout,
    Error(String),
    Oom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrashInfo {
    pub crash_type: CrashType,
    pub address: Option<u64>,
    pub stack_hash: u64,
    pub stack_trace: Vec<String>,
    pub sanitizer_output: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrashType {
    Segfault, StackOverflow, HeapBufferOverflow, StackBufferOverflow,
    UseAfterFree, DoubleFree, IntegerOverflow, DivideByZero,
    AssertionFailure, Abort,
    Sanitizer(String), Unknown(String),
}

impl fmt::Display for CrashType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrashType::Segfault => write!(f, "SEGFAULT"),
            CrashType::HeapBufferOverflow => write!(f, "HEAP_BUFFER_OVERFLOW"),
            CrashType::StackBufferOverflow => write!(f, "STACK_BUFFER_OVERFLOW"),
            CrashType::UseAfterFree => write!(f, "USE_AFTER_FREE"),
            CrashType::DoubleFree => write!(f, "DOUBLE_FREE"),
            CrashType::IntegerOverflow => write!(f, "INTEGER_OVERFLOW"),
            CrashType::DivideByZero => write!(f, "DIVIDE_BY_ZERO"),
            CrashType::AssertionFailure => write!(f, "ASSERTION_FAILURE"),
            CrashType::Abort => write!(f, "ABORT"),
            CrashType::Sanitizer(s) => write!(f, "SANITIZER_{}", s),
            CrashType::Unknown(s) => write!(f, "UNKNOWN_{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CoverageSnapshot {
    pub edge_count: usize,
    pub new_edges: Vec<u64>,
    pub total_edges: usize,
    pub hit_map: Vec<u8>,
    pub path_hash: u64,
}

impl CoverageSnapshot {
    pub fn has_new_coverage(&self) -> bool { !self.new_edges.is_empty() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SessionHandle(pub Uuid);
impl SessionHandle { pub fn new() -> Self { Self(Uuid::new_v4()) } }
impl Default for SessionHandle { fn default() -> Self { Self::new() } }

pub trait CoverageBackend: Send + Sync {
    fn initialize(&mut self, target: &TargetConfig) -> AegisResult<()>;
    fn start_session(&mut self) -> AegisResult<SessionHandle>;
    fn end_session(&mut self, session: SessionHandle) -> AegisResult<CoverageSnapshot>;
    fn is_available(&self) -> bool;
    fn name(&self) -> &str;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TargetConfig {
    pub id: Uuid,
    pub name: String,
    pub target_type: TargetType,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub working_dir: Option<String>,
    pub input_mode: InputMode,
    pub timeout: Duration,
    pub memory_limit_mb: usize,
    pub cpu_limit: Option<f64>,
    pub instrumentation: InstrumentationConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType { Binary, Library, Service, FileParser, SmartContract, Kernel, Custom(String) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    Stdin,
    File { path_template: String },
    Arg { position: usize },
    Network { host: String, port: u16 },
    Function,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentationConfig {
    pub use_sancov: bool,
    pub use_asan: bool,
    pub use_ubsan: bool,
    pub use_tsan: bool,
    pub sancov_level: u8,
    pub trace_pc: bool,
    pub trace_cmp: bool,
}

impl Default for InstrumentationConfig {
    fn default() -> Self {
        Self { use_sancov: true, use_asan: true, use_ubsan: false, use_tsan: false,
               sancov_level: 1, trace_pc: true, trace_cmp: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignConfig {
    pub id: Uuid,
    pub name: String,
    pub target_id: Uuid,
    pub seed_inputs: Vec<Vec<u8>>,
    pub max_iterations: Option<u64>,
    pub max_duration: Option<Duration>,
    pub parallel_workers: usize,
    pub mutation_config: MutationConfig,
    pub corpus_config: CorpusConfig,
    pub stop_on_first_crash: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MutationConfig {
    pub strategies: Vec<String>,
    pub max_input_size: usize,
    pub min_input_size: usize,
    pub mutation_depth: usize,
    pub dictionary: Option<Vec<Vec<u8>>>,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self { strategies: vec!["bit_flip".into(), "byte_flip".into(), "arithmetic".into(),
                                "splice".into(), "insert_delete".into()],
               max_input_size: 1024*1024, min_input_size: 1, mutation_depth: 5, dictionary: None }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorpusConfig {
    pub max_size: usize,
    pub storage_path: String,
    pub enable_minimization: bool,
    pub power_schedule: PowerSchedule,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerSchedule { Fast, Explore, Exploit, Coe, Lin, Quad }

impl Default for CorpusConfig {
    fn default() -> Self {
        Self { max_size: 10000, storage_path: "./corpus".into(),
               enable_minimization: true, power_schedule: PowerSchedule::Fast }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CampaignStats {
    pub total_execs: u64,
    pub execs_per_sec: f64,
    pub total_crashes: u64,
    pub unique_crashes: u64,
    pub corpus_size: usize,
    pub coverage_percent: f64,
    pub total_edges: usize,
    pub edges_found: usize,
    pub start_time: Option<DateTime<Utc>>,
    pub last_update: Option<DateTime<Utc>>,
    pub hang_count: u64,
    pub timeout_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrashReport {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub input: ByteInput,
    pub result: ExecutionResult,
    pub minimized_input: Option<ByteInput>,
    pub dedup_key: String,
    pub timestamp: DateTime<Utc>,
    pub triaged: bool,
    pub severity: Option<Severity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity { Critical, High, Medium, Low, Info }

pub trait Executor: Send + Sync {
    fn execute(&self, input: &dyn Input) -> AegisResult<ExecutionResult>;
    fn target(&self) -> &TargetConfig;
    fn health_check(&self) -> AegisResult<()>;
}

pub trait MutationStrategy: Send + Sync + fmt::Debug {
    fn mutate(&self, input: &mut dyn Input, rng: &mut dyn rand::RngCore);
    fn name(&self) -> &str;
    fn can_mutate(&self, input: &dyn Input) -> bool { !input.is_empty() }
}

pub trait Corpus: Send + Sync {
    fn add(&mut self, input: ByteInput, coverage: &CoverageSnapshot) -> AegisResult<bool>;
    fn select(&self, rng: &mut dyn rand::RngCore) -> Option<&ByteInput>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn inputs(&self) -> Vec<&ByteInput>;
    fn save(&self) -> AegisResult<()>;
    fn load(&mut self) -> AegisResult<()>;
}

pub trait FuzzingEngine: Send + Sync {
    fn run(&mut self, config: &CampaignConfig) -> AegisResult<CampaignStats>;
    fn stop(&mut self) -> AegisResult<()>;
    fn stats(&self) -> CampaignStats;
    fn crashes(&self) -> Vec<CrashReport>;
}
