//! 操作执行器的公共类型定义。

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationRequest {
    pub operation_id: u64,
    pub screen_index: usize,
    pub block_index: usize,
    pub source: OperationSource,
    pub params: HashMap<String, String>,
    pub host: HashMap<String, String>,
    pub cwd: Option<String>,
    pub env: HashMap<String, String>,
    pub allowed_working_dirs: Vec<String>,
    pub allowed_env_keys: Option<HashSet<String>>,
    pub result_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationSource {
    ShellCommand(String),
    RegisteredAction(String),
}

impl OperationSource {
    pub fn describe(&self) -> String {
        match self {
            Self::ShellCommand(command) => command.clone(),
            Self::RegisteredAction(name) => format!("action:{name}"),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionContext {
    pub operation_id: u64,
    pub screen_index: usize,
    pub block_index: usize,
    pub params: HashMap<String, String>,
    pub host: HashMap<String, String>,
    pub cwd: Option<String>,
    pub env: HashMap<String, String>,
    pub result_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutcome {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl ActionOutcome {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            success: true,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    pub fn failure(stderr: impl Into<String>) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }
}
