//! 宿主侧运行时接入面。

use crate::executor::{ActionContext, ActionOutcome, ActionRegistry};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShellPolicy {
    #[default]
    AllowAll,
    RegisteredOnly,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostEvent {
    OperationStarted {
        operation_id: u64,
        screen_index: usize,
        block_index: usize,
        source: String,
    },
    OperationFinished {
        operation_id: u64,
        screen_index: usize,
        block_index: usize,
        source: String,
        success: bool,
        stdout: String,
        stderr: String,
    },
}

type HostEventHook = dyn Fn(HostEvent) + Send + Sync;
type HostLoggerHook = dyn Fn(HostLogRecord) + Send + Sync;

#[derive(Clone, Default)]
pub struct ExecutionPolicy {
    allowed_working_dirs: Vec<PathBuf>,
    allowed_env_keys: Option<HashSet<String>>,
}

impl ExecutionPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_working_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.allowed_working_dirs.push(path.into());
        self
    }

    pub fn allow_env_key(mut self, key: impl Into<String>) -> Self {
        self.allowed_env_keys
            .get_or_insert_with(HashSet::new)
            .insert(key.into());
        self
    }

    pub fn allowed_working_dirs(&self) -> &[PathBuf] {
        &self.allowed_working_dirs
    }

    pub fn allowed_env_keys(&self) -> Option<&HashSet<String>> {
        self.allowed_env_keys.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostLogRecord {
    pub level: HostLogLevel,
    pub target: String,
    pub message: String,
}

#[derive(Clone, Default)]
pub struct ShellRuntime {
    cwd: Option<PathBuf>,
    env: HashMap<String, String>,
}

impl ShellRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn insert_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn cwd(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }
}

#[derive(Clone, Default)]
pub struct RuntimeHost {
    actions: ActionRegistry,
    context: HashMap<String, String>,
    shell: ShellRuntime,
    shell_policy: ShellPolicy,
    execution_policy: ExecutionPolicy,
    framework_log_enabled: bool,
    framework_log_path: Option<PathBuf>,
    event_hook: Option<Arc<HostEventHook>>,
    logger: Option<Arc<HostLoggerHook>>,
}

impl RuntimeHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_registry(actions: ActionRegistry) -> Self {
        Self {
            actions,
            context: HashMap::new(),
            shell: ShellRuntime::new(),
            shell_policy: ShellPolicy::AllowAll,
            execution_policy: ExecutionPolicy::new(),
            framework_log_enabled: true,
            framework_log_path: None,
            event_hook: None,
            logger: None,
        }
    }

    pub fn action_registry(&self) -> ActionRegistry {
        self.actions.clone()
    }

    pub fn register_shell_action(&mut self, name: impl Into<String>, command: impl Into<String>) {
        self.actions.register_shell_action(name, command);
    }

    pub fn register_action_handler<F, Fut>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(ActionContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ActionOutcome> + Send + 'static,
    {
        self.actions.register_action_handler(name, handler);
    }

    pub fn has_action(&self, name: &str) -> bool {
        self.actions.has_action(name)
    }

    pub fn set_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn insert_context(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.context.insert(key.into(), value.into());
    }

    pub fn context(&self) -> &HashMap<String, String> {
        &self.context
    }

    pub fn context_value(&self, key: &str) -> Option<&str> {
        self.context.get(key).map(String::as_str)
    }

    pub fn shell(&self) -> &ShellRuntime {
        &self.shell
    }

    pub fn set_working_dir(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.shell = self.shell.clone().set_cwd(cwd);
        self
    }

    pub fn working_dir(&self) -> Option<&Path> {
        self.shell.cwd()
    }

    pub fn insert_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.shell = self.shell.clone().insert_env(key, value);
        self
    }

    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.shell.env.insert(key.into(), value.into());
    }

    pub fn set_shell_policy(mut self, policy: ShellPolicy) -> Self {
        self.shell_policy = policy;
        self
    }

    pub fn set_framework_log_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.framework_log_path = Some(path.into());
        self
    }

    pub fn set_framework_log_enabled(mut self, enabled: bool) -> Self {
        self.framework_log_enabled = enabled;
        self
    }

    pub fn framework_log_enabled(&self) -> bool {
        self.framework_log_enabled
    }

    pub fn framework_log_path(&self) -> Option<&Path> {
        self.framework_log_path.as_deref()
    }

    pub fn shell_policy(&self) -> ShellPolicy {
        self.shell_policy
    }

    pub fn execution_policy(&self) -> &ExecutionPolicy {
        &self.execution_policy
    }

    pub fn allow_working_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.execution_policy = self.execution_policy.clone().allow_working_dir(path);
        self
    }

    pub fn allow_env_key(mut self, key: impl Into<String>) -> Self {
        self.execution_policy = self.execution_policy.clone().allow_env_key(key);
        self
    }

    pub fn on_event<F>(mut self, hook: F) -> Self
    where
        F: Fn(HostEvent) + Send + Sync + 'static,
    {
        self.event_hook = Some(Arc::new(hook));
        self
    }

    pub fn set_event_hook<F>(&mut self, hook: F)
    where
        F: Fn(HostEvent) + Send + Sync + 'static,
    {
        self.event_hook = Some(Arc::new(hook));
    }

    pub fn event_hook(&self) -> Option<Arc<HostEventHook>> {
        self.event_hook.clone()
    }

    pub fn on_log<F>(mut self, logger: F) -> Self
    where
        F: Fn(HostLogRecord) + Send + Sync + 'static,
    {
        self.logger = Some(Arc::new(logger));
        self
    }

    pub fn set_logger<F>(&mut self, logger: F)
    where
        F: Fn(HostLogRecord) + Send + Sync + 'static,
    {
        self.logger = Some(Arc::new(logger));
    }

    pub fn logger(&self) -> Option<Arc<HostLoggerHook>> {
        self.logger.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{HostEvent, HostLogLevel, HostLogRecord, RuntimeHost, ShellPolicy};
    use crate::executor::ActionOutcome;
    use std::sync::{Arc, Mutex};

    #[test]
    fn runtime_host_stores_context_values() {
        let host = RuntimeHost::new()
            .set_context("project_root", "/tmp/demo")
            .set_context("profile", "dev");

        assert_eq!(host.context_value("project_root"), Some("/tmp/demo"));
        assert_eq!(host.context_value("profile"), Some("dev"));
    }

    #[test]
    fn runtime_host_tracks_registered_actions() {
        let mut host = RuntimeHost::new();
        host.register_action_handler("sync", |_| async { ActionOutcome::success("ok") });

        assert!(host.has_action("sync"));
        assert!(!host.has_action("missing"));
    }

    #[test]
    fn runtime_host_stores_shell_runtime_settings() {
        let host = RuntimeHost::new()
            .set_working_dir("/tmp/demo")
            .set_framework_log_enabled(false)
            .set_framework_log_path("/tmp/demo/.logs/framework.log")
            .insert_env("APP_ENV", "dev")
            .insert_env("APP_MODE", "test");

        assert_eq!(
            host.working_dir().and_then(|path| path.to_str()),
            Some("/tmp/demo")
        );
        assert_eq!(
            host.shell().env().get("APP_ENV").map(String::as_str),
            Some("dev")
        );
        assert_eq!(
            host.shell().env().get("APP_MODE").map(String::as_str),
            Some("test")
        );
        assert_eq!(
            host.framework_log_path().and_then(|path| path.to_str()),
            Some("/tmp/demo/.logs/framework.log")
        );
        assert!(!host.framework_log_enabled());
    }

    #[test]
    fn runtime_host_stores_shell_policy_and_event_hook() {
        let events = Arc::new(Mutex::new(Vec::<HostEvent>::new()));
        let capture = events.clone();
        let host = RuntimeHost::new()
            .set_shell_policy(ShellPolicy::RegisteredOnly)
            .on_event(move |event| {
                capture.lock().unwrap().push(event);
            });

        assert_eq!(host.shell_policy(), ShellPolicy::RegisteredOnly);
        host.event_hook().unwrap()(HostEvent::OperationStarted {
            operation_id: 1,
            screen_index: 0,
            block_index: 0,
            source: "action:sync".to_string(),
        });
        assert_eq!(events.lock().unwrap().len(), 1);
    }

    #[test]
    fn runtime_host_stores_logger_hook() {
        let logs = Arc::new(Mutex::new(Vec::<HostLogRecord>::new()));
        let capture = logs.clone();
        let host = RuntimeHost::new().on_log(move |record| {
            capture.lock().unwrap().push(record);
        });

        host.logger().unwrap()(HostLogRecord {
            level: HostLogLevel::Info,
            target: "tui01.host".to_string(),
            message: "ready".to_string(),
        });
        assert_eq!(logs.lock().unwrap().len(), 1);
    }

    #[test]
    fn execution_policy_tracks_allowed_dirs_and_env_keys() {
        let host = RuntimeHost::new()
            .allow_working_dir("/workspace/demo")
            .allow_env_key("APP_ENV")
            .allow_env_key("APP_MODE");

        assert_eq!(host.execution_policy().allowed_working_dirs().len(), 1);
        assert!(host
            .execution_policy()
            .allowed_env_keys()
            .unwrap()
            .contains("APP_ENV"));
        assert!(host
            .execution_policy()
            .allowed_env_keys()
            .unwrap()
            .contains("APP_MODE"));
    }
}
