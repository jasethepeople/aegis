use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use aegis_core::*;

pub struct TargetFactory;

impl TargetFactory {
    pub fn binary(name: impl Into<String>, command: impl Into<String>, args: Vec<String>) -> TargetConfig {
        TargetConfig { id: uuid::Uuid::new_v4(), name: name.into(), target_type: TargetType::Binary,
            command: command.into(), args, env: Vec::new(), working_dir: None, input_mode: InputMode::Stdin,
            timeout: Duration::from_secs(5), memory_limit_mb: 256, cpu_limit: None, instrumentation: InstrumentationConfig::default() }
    }
    pub fn service(name: impl Into<String>, host: impl Into<String>, port: u16) -> TargetConfig {
        TargetConfig { id: uuid::Uuid::new_v4(), name: name.into(), target_type: TargetType::Service,
            command: format!("{}:{}", host.into(), port), args: Vec::new(), env: Vec::new(), working_dir: None,
            input_mode: InputMode::Network { host: host.into(), port }, timeout: Duration::from_secs(10),
            memory_limit_mb: 512, cpu_limit: None, instrumentation: InstrumentationConfig { use_sancov: false, use_asan: false, use_ubsan: false, use_tsan: false, sancov_level: 0, trace_pc: false, trace_cmp: false } }
    }
    pub fn from_json(path: impl AsRef<Path>) -> AegisResult<TargetConfig> {
        let content = std::fs::read_to_string(path).map_err(|e| AegisError::ConfigError(format!("Read: {}", e)))?;
        serde_json::from_str(&content).map_err(|e| AegisError::ConfigError(format!("Parse: {}", e)))
    }
}

pub struct TargetValidator;

impl TargetValidator {
    pub fn validate(target: &TargetConfig) -> Vec<TargetValidationError> {
        let mut errors = Vec::new();
        if target.command.is_empty() { errors.push(TargetValidationError::MissingCommand); }
        if target.timeout.as_secs() == 0 { errors.push(TargetValidationError::InvalidTimeout); }
        if target.memory_limit_mb == 0 { errors.push(TargetValidationError::InvalidMemoryLimit); }
        if target.instrumentation.use_sancov && target.instrumentation.sancov_level == 0 {
            errors.push(TargetValidationError::InvalidInstrumentation("sancov enabled but level 0".into()));
        }
        errors
    }
    pub fn is_valid(target: &TargetConfig) -> bool { Self::validate(target).is_empty() }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetValidationError {
    MissingCommand, CommandNotFound(String), InvalidTimeout, InvalidMemoryLimit,
    InvalidInstrumentation(String), UnsupportedTargetType(String),
}

impl std::fmt::Display for TargetValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetValidationError::MissingCommand => write!(f, "Command required"),
            TargetValidationError::CommandNotFound(cmd) => write!(f, "Not found: {}", cmd),
            TargetValidationError::InvalidTimeout => write!(f, "Timeout > 0 required"),
            TargetValidationError::InvalidMemoryLimit => write!(f, "Memory > 0 required"),
            TargetValidationError::InvalidInstrumentation(msg) => write!(f, "Bad instrumentation: {}", msg),
            TargetValidationError::UnsupportedTargetType(t) => write!(f, "Unsupported: {}", t),
        }
    }
}

pub struct TargetRegistry {
    targets: HashMap<uuid::Uuid, TargetConfig>,
}

impl TargetRegistry {
    pub fn new() -> Self { Self { targets: HashMap::new() } }
    pub fn register(&mut self, target: TargetConfig) -> uuid::Uuid { let id = target.id; self.targets.insert(id, target); id }
    pub fn get(&self, id: &uuid::Uuid) -> Option<&TargetConfig> { self.targets.get(id) }
    pub fn list(&self) -> Vec<&TargetConfig> { self.targets.values().collect() }
}

impl Default for TargetRegistry { fn default() -> Self { Self::new() } }
