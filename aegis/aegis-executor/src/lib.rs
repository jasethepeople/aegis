use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::timeout;
use tracing::{debug, trace, warn, error};
use aegis_core::*;
use aegis_sancov::SanCovBackend;

pub struct ProcessExecutor {
    target: TargetConfig,
    coverage: Option<SanCovBackend>,
}

impl ProcessExecutor {
    pub fn new(target: TargetConfig) -> Self {
        let coverage = if target.instrumentation.use_sancov { Some(SanCovBackend::new()) } else { None };
        Self { target, coverage }
    }

    pub async fn execute_async(&self, input: &dyn Input) -> AegisResult<ExecutionResult> {
        let start = Instant::now();
        let input_id = uuid::Uuid::new_v4();
        if let Some(ref cov) = self.coverage { cov.reset(); }

        let mut cmd = Command::new(&self.target.command);
        cmd.args(&self.target.args)
           .envs(self.target.env.iter().cloned())
           .stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::piped())
           .kill_on_drop(true);
        if let Some(ref dir) = self.target.working_dir { cmd.current_dir(dir); }

        let mut child = cmd.spawn().map_err(|e| AegisError::ExecutionError(format!("Spawn failed: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes()).await.map_err(|e| AegisError::ExecutionError(format!("Write failed: {}", e)))?;
            drop(stdin);
        }

        let result = match timeout(self.target.timeout, child.wait()).await {
            Ok(Ok(status)) => {
                let exit_code = status.code();
                let signal = get_signal(&status);
                let mut stdout = Vec::new(); let mut stderr = Vec::new();
                if let Some(mut out) = child.stdout.take() { let _ = out.read_to_end(&mut stdout).await; }
                if let Some(mut err) = child.stderr.take() { let _ = err.read_to_end(&mut stderr).await; }
                let exec_time = start.elapsed();
                let exec_status = if let Some(sig) = signal {
                    match sig {
                        6 => ExecutionStatus::Crash(CrashInfo { crash_type: CrashType::Abort, address: None, stack_hash: 0, stack_trace: vec![], sanitizer_output: extract_sanitizer(&stderr) }),
                        11 => ExecutionStatus::Crash(CrashInfo { crash_type: CrashType::Segfault, address: None, stack_hash: 0, stack_trace: vec![], sanitizer_output: extract_sanitizer(&stderr) }),
                        _ => ExecutionStatus::Crash(CrashInfo { crash_type: CrashType::Unknown(format!("signal_{}", sig)), address: None, stack_hash: 0, stack_trace: vec![], sanitizer_output: extract_sanitizer(&stderr) }),
                    }
                } else if let Some(code) = exit_code {
                    if code == 0 { ExecutionStatus::Success }
                    else if has_sanitizer(&stderr) { ExecutionStatus::Crash(CrashInfo { crash_type: detect_sanitizer(&stderr), address: None, stack_hash: 0, stack_trace: vec![], sanitizer_output: extract_sanitizer(&stderr) }) }
                    else { ExecutionStatus::Error(format!("exit_{}", code)) }
                } else { ExecutionStatus::Error("unknown".into()) };
                ExecutionResult { status: exec_status, exit_code, signal, exec_time, stdout, stderr, coverage: self.collect_coverage(), input_id }
            }
            Ok(Err(e)) => ExecutionResult { status: ExecutionStatus::Error(format!("wait: {}", e)), exit_code: None, signal: None, exec_time: start.elapsed(), stdout: vec![], stderr: vec![], coverage: CoverageSnapshot::default(), input_id },
            Err(_) => { let _ = child.kill().await; ExecutionResult { status: ExecutionStatus::Timeout, exit_code: None, signal: None, exec_time: self.target.timeout, stdout: vec![], stderr: vec![], coverage: self.collect_coverage(), input_id } }
        };
        trace!("Exec completed: {:?}", result.status);
        Ok(result)
    }

    fn collect_coverage(&self) -> CoverageSnapshot {
        if let Some(ref cov) = self.coverage { cov.snapshot() } else { CoverageSnapshot::default() }
    }
}

impl Executor for ProcessExecutor {
    fn execute(&self, input: &dyn Input) -> AegisResult<ExecutionResult> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| AegisError::ExecutionError(format!("Runtime: {}", e)))?;
        rt.block_on(self.execute_async(input))
    }
    fn target(&self) -> &TargetConfig { &self.target }
    fn health_check(&self) -> AegisResult<()> {
        let input = ByteInput::new(vec![]);
        let _ = self.execute(&input)?; Ok(())
    }
}

fn get_signal(status: &std::process::ExitStatus) -> Option<i32> {
    #[cfg(unix)] { use std::os::unix::process::ExitStatusExt; status.signal() }
    #[cfg(not(unix))] { None }
}

fn has_sanitizer(stderr: &[u8]) -> bool {
    let s = String::from_utf8_lossy(stderr);
    s.contains("ERROR: AddressSanitizer") || s.contains("ERROR: MemorySanitizer") || s.contains("ERROR: UndefinedBehavior") || s.contains("ERROR: ThreadSanitizer")
}

fn extract_sanitizer(stderr: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(stderr);
    if has_sanitizer(stderr) {
        Some(s.lines().skip_while(|l| !l.contains("ERROR:")).take(20).collect::<Vec<_>>().join("\n"))
    } else { None }
}

fn detect_sanitizer(stderr: &[u8]) -> CrashType {
    let s = String::from_utf8_lossy(stderr);
    if s.contains("heap-buffer-overflow") { CrashType::HeapBufferOverflow }
    else if s.contains("stack-buffer-overflow") { CrashType::StackBufferOverflow }
    else if s.contains("use-after-free") { CrashType::UseAfterFree }
    else if s.contains("double-free") { CrashType::DoubleFree }
    else if s.contains("SEGV") { CrashType::Segfault }
    else if s.contains("integer-overflow") { CrashType::IntegerOverflow }
    else if s.contains("division-by-zero") { CrashType::DivideByZero }
    else { CrashType::Sanitizer("unknown".into()) }
}

pub struct ExecutorBuilder {
    target: Option<TargetConfig>,
}

impl ExecutorBuilder {
    pub fn new() -> Self { Self { target: None } }
    pub fn target(mut self, target: TargetConfig) -> Self { self.target = Some(target); self }
    pub fn build_process(self) -> AegisResult<ProcessExecutor> {
        Ok(ProcessExecutor::new(self.target.ok_or_else(|| AegisError::ConfigError("No target".into()))?))
    }
}

impl Default for ExecutorBuilder { fn default() -> Self { Self::new() } }
