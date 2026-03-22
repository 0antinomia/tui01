//! 异步操作执行器，负责运行真实命令并回传结果。

use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationRequest {
    pub operation_id: u64,
    pub screen_index: usize,
    pub block_index: usize,
    pub command: String,
    pub result_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationResult {
    pub operation_id: u64,
    pub screen_index: usize,
    pub block_index: usize,
    pub result_target: Option<String>,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub struct OperationExecutor {
    sender: mpsc::UnboundedSender<OperationResult>,
    receiver: mpsc::UnboundedReceiver<OperationResult>,
}

impl OperationExecutor {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    pub fn submit(&self, request: OperationRequest) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let output = Command::new("sh")
                .arg("-lc")
                .arg(&request.command)
                .output()
                .await;

            let result = match output {
                Ok(output) => OperationResult {
                    operation_id: request.operation_id,
                    screen_index: request.screen_index,
                    block_index: request.block_index,
                    result_target: request.result_target.clone(),
                    success: output.status.success(),
                    stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                },
                Err(err) => OperationResult {
                    operation_id: request.operation_id,
                    screen_index: request.screen_index,
                    block_index: request.block_index,
                    result_target: request.result_target.clone(),
                    success: false,
                    stdout: String::new(),
                    stderr: err.to_string(),
                },
            };

            let _ = sender.send(result);
        });
    }

    pub fn try_recv(&mut self) -> Option<OperationResult> {
        self.receiver.try_recv().ok()
    }
}
