use std::time::Duration;
use aegis_core::*;
use aegis_executor::Executor;

pub struct HttpServiceExecutor {
    client: reqwest::Client,
    base_url: String,
    target_config: TargetConfig,
}

impl HttpServiceExecutor {
    pub fn new(base_url: String, target_config: TargetConfig) -> Self {
        let client = reqwest::Client::builder().timeout(Duration::from_secs(30)).build().expect("client");
        Self { client, base_url, target_config }
    }
}

impl Executor for HttpServiceExecutor {
    fn execute(&self, input: &dyn Input) -> AegisResult<ExecutionResult> {
        let start = std::time::Instant::now();
        let input_id = uuid::Uuid::new_v4();
        let rt = tokio::runtime::Runtime::new().map_err(|e| AegisError::ExecutionError(format!("{}", e)))?;
        let result = rt.block_on(async {
            match self.client.post(&self.base_url).body(input.as_bytes().to_vec()).send().await {
                Ok(res) => {
                    let status = res.status().as_u16();
                    let body = res.bytes().await.unwrap_or_default().to_vec();
                    let exec_status = if status >= 500 {
                        ExecutionStatus::Crash(CrashInfo { crash_type: CrashType::Unknown(format!("http_{}", status)), address: None, stack_hash: 0, stack_trace: vec![], sanitizer_output: None })
                    } else { ExecutionStatus::Success };
                    ExecutionResult { status: exec_status, exit_code: Some(status as i32), signal: None, exec_time: start.elapsed(), stdout: body, stderr: vec![], coverage: CoverageSnapshot::default(), input_id }
                }
                Err(e) => ExecutionResult { status: ExecutionStatus::Error(format!("{}", e)), exit_code: None, signal: None, exec_time: start.elapsed(), stdout: vec![], stderr: vec![], coverage: CoverageSnapshot::default(), input_id }
            }
        });
        Ok(result)
    }
    fn target(&self) -> &TargetConfig { &self.target_config }
    fn health_check(&self) -> AegisResult<()> { Ok(()) }
}

pub struct EVMContractExecutor {
    target_config: TargetConfig,
    contract_address: String,
}

impl EVMContractExecutor {
    pub fn new(contract_address: String, target_config: TargetConfig) -> Self {
        Self { target_config, contract_address }
    }
    pub fn generate_tx(&self, input: &[u8]) -> Vec<u8> {
        let mut data = if input.len() >= 4 { input[0..4].to_vec() } else { vec![0,0,0,0] };
        if input.len() > 4 { data.extend_from_slice(&input[4..]); }
        data
    }
}

impl Executor for EVMContractExecutor {
    fn execute(&self, input: &dyn Input) -> AegisResult<ExecutionResult> {
        let start = std::time::Instant::now();
        let input_id = uuid::Uuid::new_v4();
        let _tx = self.generate_tx(input.as_bytes());
        Ok(ExecutionResult { status: ExecutionStatus::Success, exit_code: Some(0), signal: None, exec_time: start.elapsed(), stdout: vec![], stderr: vec![], coverage: CoverageSnapshot::default(), input_id })
    }
    fn target(&self) -> &TargetConfig { &self.target_config }
    fn health_check(&self) -> AegisResult<()> { Ok(()) }
}
