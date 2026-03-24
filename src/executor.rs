//! 异步操作执行器，负责运行真实命令并回传结果。

use crate::host::{HostEvent, HostLogLevel, HostLogRecord, ShellPolicy};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::mpsc;

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

type ActionFuture = Pin<Box<dyn Future<Output = ActionOutcome> + Send>>;
type ActionHandler = dyn Fn(ActionContext) -> ActionFuture + Send + Sync;

#[derive(Clone)]
enum RegisteredAction {
    ShellTemplate(String),
    Handler(Arc<ActionHandler>),
}

#[derive(Clone, Default)]
pub struct ActionRegistry {
    actions: HashMap<String, RegisteredAction>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_shell_action(&mut self, name: impl Into<String>, command: impl Into<String>) {
        self.actions
            .insert(name.into(), RegisteredAction::ShellTemplate(command.into()));
    }

    pub fn register_action_handler<F, Fut>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(ActionContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ActionOutcome> + Send + 'static,
    {
        let handler =
            Arc::new(move |context: ActionContext| -> ActionFuture { Box::pin(handler(context)) });
        self.actions
            .insert(name.into(), RegisteredAction::Handler(handler));
    }

    pub fn has_action(&self, name: &str) -> bool {
        self.actions.contains_key(name)
    }

    fn resolve(&self, name: &str) -> Option<RegisteredAction> {
        self.actions.get(name).cloned()
    }
}

pub struct OperationExecutor {
    registry: ActionRegistry,
    shell_policy: ShellPolicy,
    event_hook: Option<Arc<dyn Fn(HostEvent) + Send + Sync>>,
    logger: Option<Arc<dyn Fn(HostLogRecord) + Send + Sync>>,
    sender: mpsc::UnboundedSender<OperationResult>,
    receiver: mpsc::UnboundedReceiver<OperationResult>,
}

impl OperationExecutor {
    pub fn new() -> Self {
        Self::with_registry(ActionRegistry::new())
    }

    pub fn with_registry(registry: ActionRegistry) -> Self {
        Self::with_runtime(registry, ShellPolicy::AllowAll, None, None)
    }

    pub fn with_runtime(
        registry: ActionRegistry,
        shell_policy: ShellPolicy,
        event_hook: Option<Arc<dyn Fn(HostEvent) + Send + Sync>>,
        logger: Option<Arc<dyn Fn(HostLogRecord) + Send + Sync>>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            registry,
            shell_policy,
            event_hook,
            logger,
            sender,
            receiver,
        }
    }

    pub fn register_shell_action(&mut self, name: impl Into<String>, command: impl Into<String>) {
        self.registry.register_shell_action(name, command);
    }

    pub fn register_action_handler<F, Fut>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(ActionContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ActionOutcome> + Send + 'static,
    {
        self.registry.register_action_handler(name, handler);
    }

    pub fn has_action(&self, name: &str) -> bool {
        self.registry.has_action(name)
    }

    pub fn submit(&self, request: OperationRequest) {
        let sender = self.sender.clone();
        let shell_policy = self.shell_policy;
        let event_hook = self.event_hook.clone();
        let logger = self.logger.clone();
        let registered = match &request.source {
            OperationSource::ShellCommand(_) => None,
            OperationSource::RegisteredAction(name) => self.registry.resolve(name),
        };
        let source_description = request.source.describe();
        if let Some(hook) = &event_hook {
            hook(HostEvent::OperationStarted {
                operation_id: request.operation_id,
                screen_index: request.screen_index,
                block_index: request.block_index,
                source: source_description.clone(),
            });
        }
        if let Some(logger) = &logger {
            logger(HostLogRecord {
                level: HostLogLevel::Info,
                target: "tui01.operation".to_string(),
                message: format!(
                    "started op={} screen={} block={} source={}",
                    request.operation_id,
                    request.screen_index,
                    request.block_index,
                    source_description
                ),
            });
        }
        tokio::spawn(async move {
            let context = ActionContext {
                operation_id: request.operation_id,
                screen_index: request.screen_index,
                block_index: request.block_index,
                params: request.params.clone(),
                host: request.host.clone(),
                cwd: request.cwd.clone(),
                env: request.env.clone(),
                result_target: request.result_target.clone(),
            };
            if let Some(error) = validate_request_permissions(
                request.cwd.as_deref(),
                &request.env,
                &request.allowed_working_dirs,
                request.allowed_env_keys.as_ref(),
            ) {
                if let Some(logger) = &logger {
                    logger(HostLogRecord {
                        level: HostLogLevel::Warn,
                        target: "tui01.operation".to_string(),
                        message: format!(
                            "blocked by execution policy op={} source={} reason={}",
                            request.operation_id, source_description, error
                        ),
                    });
                }
                let result = OperationResult {
                    operation_id: request.operation_id,
                    screen_index: request.screen_index,
                    block_index: request.block_index,
                    result_target: request.result_target.clone(),
                    success: false,
                    stdout: String::new(),
                    stderr: error,
                };
                if let Some(hook) = &event_hook {
                    hook(HostEvent::OperationFinished {
                        operation_id: result.operation_id,
                        screen_index: result.screen_index,
                        block_index: result.block_index,
                        source: source_description.clone(),
                        success: false,
                        stdout: String::new(),
                        stderr: result.stderr.clone(),
                    });
                }
                let _ = sender.send(result);
                return;
            }
            let outcome = match &request.source {
                OperationSource::ShellCommand(command) if shell_policy != ShellPolicy::AllowAll => {
                    if let Some(logger) = &logger {
                        logger(HostLogRecord {
                            level: HostLogLevel::Warn,
                            target: "tui01.operation".to_string(),
                            message: format!(
                                "blocked inline shell by policy op={} source={}",
                                request.operation_id, source_description
                            ),
                        });
                    }
                    ActionOutcome::failure("inline shell commands are disabled by host policy")
                }
                OperationSource::ShellCommand(command) => {
                    run_shell_command(command.clone(), request.cwd.clone(), request.env.clone())
                        .await
                }
                OperationSource::RegisteredAction(name) => match registered {
                    Some(RegisteredAction::ShellTemplate(_))
                        if shell_policy == ShellPolicy::Disabled =>
                    {
                        if let Some(logger) = &logger {
                            logger(HostLogRecord {
                                level: HostLogLevel::Warn,
                                target: "tui01.operation".to_string(),
                                message: format!(
                                    "blocked registered shell by policy op={} source={}",
                                    request.operation_id, source_description
                                ),
                            });
                        }
                        ActionOutcome::failure(
                            "registered shell actions are disabled by host policy",
                        )
                    }
                    Some(RegisteredAction::ShellTemplate(template)) => {
                        let command =
                            render_command_template(&template, &request.params, &request.host);
                        run_shell_command(command, request.cwd.clone(), request.env.clone()).await
                    }
                    Some(RegisteredAction::Handler(handler)) => handler(context).await,
                    None => ActionOutcome::failure(format!("unknown action: {name}")),
                },
            };

            let result = OperationResult {
                operation_id: request.operation_id,
                screen_index: request.screen_index,
                block_index: request.block_index,
                result_target: request.result_target.clone(),
                success: outcome.success,
                stdout: outcome.stdout,
                stderr: outcome.stderr,
            };

            if let Some(hook) = &event_hook {
                hook(HostEvent::OperationFinished {
                    operation_id: result.operation_id,
                    screen_index: result.screen_index,
                    block_index: result.block_index,
                    source: source_description.clone(),
                    success: result.success,
                    stdout: result.stdout.clone(),
                    stderr: result.stderr.clone(),
                });
            }
            if let Some(logger) = &logger {
                logger(HostLogRecord {
                    level: if result.success {
                        HostLogLevel::Info
                    } else {
                        HostLogLevel::Error
                    },
                    target: "tui01.operation".to_string(),
                    message: format!(
                        "finished op={} screen={} block={} source={} success={}",
                        result.operation_id,
                        result.screen_index,
                        result.block_index,
                        source_description,
                        result.success
                    ),
                });
            }

            let _ = sender.send(result);
        });
    }

    pub fn try_recv(&mut self) -> Option<OperationResult> {
        self.receiver.try_recv().ok()
    }
}

impl Default for OperationExecutor {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_request_permissions(
    cwd: Option<&str>,
    env: &HashMap<String, String>,
    allowed_working_dirs: &[String],
    allowed_env_keys: Option<&HashSet<String>>,
) -> Option<String> {
    if let Some(cwd) = cwd {
        if !allowed_working_dirs.is_empty()
            && !allowed_working_dirs
                .iter()
                .any(|allowed| Path::new(cwd).starts_with(Path::new(allowed)))
        {
            return Some(format!("working directory is not allowed: {cwd}"));
        }
    }

    if let Some(allowed) = allowed_env_keys {
        for key in env.keys() {
            if !allowed.contains(key) {
                return Some(format!("environment key is not allowed: {key}"));
            }
        }
    }

    None
}

async fn run_shell_command(
    command: String,
    cwd: Option<String>,
    env: HashMap<String, String>,
) -> ActionOutcome {
    let mut child = Command::new("sh");
    child.arg("-lc").arg(&command);
    if let Some(cwd) = cwd {
        child.current_dir(cwd);
    }
    if !env.is_empty() {
        child.envs(env);
    }

    match child.output().await {
        Ok(output) => ActionOutcome {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        },
        Err(err) => ActionOutcome::failure(err.to_string()),
    }
}

fn render_command_template(
    template: &str,
    params: &HashMap<String, String>,
    host: &HashMap<String, String>,
) -> String {
    let mut rendered = String::new();
    let mut cursor = 0usize;

    while let Some(start) = template[cursor..].find("{{") {
        let start = cursor + start;
        rendered.push_str(&template[cursor..start]);
        let value_start = start + 2;

        if let Some(end_rel) = template[value_start..].find("}}") {
            let end = value_start + end_rel;
            let key = template[value_start..end].trim();
            let (raw, key) = if let Some(key) = key.strip_prefix("raw:") {
                (true, key.trim())
            } else {
                (false, key)
            };

            let value = key
                .strip_prefix("host.")
                .and_then(|key| host.get(key))
                .or_else(|| params.get(key));

            if let Some(value) = value {
                if raw {
                    rendered.push_str(value);
                } else {
                    rendered.push_str(&shell_escape(value));
                }
            }
            cursor = end + 2;
        } else {
            rendered.push_str(&template[start..]);
            cursor = template.len();
            break;
        }
    }

    if cursor < template.len() {
        rendered.push_str(&template[cursor..]);
    }

    rendered
}

fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use super::{
        render_command_template, shell_escape, ActionOutcome, ActionRegistry, OperationExecutor,
        OperationRequest, OperationSource,
    };
    use crate::host::{HostEvent, HostLogLevel, HostLogRecord, ShellPolicy};
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    #[test]
    fn action_template_renders_with_shell_escaped_runtime_params() {
        let params = HashMap::from([
            ("project_name".to_string(), "tui 01".to_string()),
            ("port".to_string(), "3000".to_string()),
        ]);
        let host = HashMap::from([("project_root".to_string(), "/workspace/demo".to_string())]);

        let rendered = render_command_template(
            "printf 'workspace=%s port=%s root=%s\\n' {{project_name}} {{port}} {{host.project_root}}",
            &params,
            &host,
        );

        assert_eq!(
            rendered,
            "printf 'workspace=%s port=%s root=%s\\n' 'tui 01' '3000' '/workspace/demo'"
        );
    }

    #[test]
    fn action_template_supports_raw_params() {
        let params = HashMap::from([("flag".to_string(), "--all --force".to_string())]);
        let host = HashMap::new();

        let rendered = render_command_template("command {{raw:flag}}", &params, &host);

        assert_eq!(rendered, "command --all --force");
    }

    #[test]
    fn shell_escape_wraps_and_escapes_single_quotes() {
        assert_eq!(shell_escape(""), "''");
        assert_eq!(shell_escape("simple"), "'simple'");
        assert_eq!(shell_escape("it's"), "'it'\"'\"'s'");
    }

    #[tokio::test]
    async fn registered_handler_action_returns_custom_result() {
        let mut registry = ActionRegistry::new();
        registry.register_action_handler("echo_params", |context| async move {
            ActionOutcome::success(format!(
                "project={}",
                context
                    .params
                    .get("project_name")
                    .cloned()
                    .unwrap_or_default()
            ))
        });
        let mut executor = OperationExecutor::with_registry(registry);

        executor.submit(OperationRequest {
            operation_id: 1,
            screen_index: 0,
            block_index: 0,
            source: OperationSource::RegisteredAction("echo_params".to_string()),
            params: HashMap::from([("project_name".to_string(), "tui01".to_string())]),
            host: HashMap::from([("project_root".to_string(), "/tmp/demo".to_string())]),
            cwd: Some("/tmp".to_string()),
            env: HashMap::from([("APP_ENV".to_string(), "dev".to_string())]),
            allowed_working_dirs: vec![],
            allowed_env_keys: None,
            result_target: None,
        });

        let mut result = None;
        for _ in 0..20 {
            if let Some(value) = executor.try_recv() {
                result = Some(value);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let result = result.expect("expected async action result");
        assert!(result.success);
        assert_eq!(result.stdout, "project=tui01");
    }

    #[tokio::test]
    async fn registered_handler_action_receives_host_context() {
        let mut registry = ActionRegistry::new();
        registry.register_action_handler("host_echo", |context| async move {
            ActionOutcome::success(
                context
                    .host
                    .get("project_root")
                    .cloned()
                    .unwrap_or_default(),
            )
        });
        let mut executor = OperationExecutor::with_registry(registry);

        executor.submit(OperationRequest {
            operation_id: 2,
            screen_index: 0,
            block_index: 0,
            source: OperationSource::RegisteredAction("host_echo".to_string()),
            params: HashMap::new(),
            host: HashMap::from([("project_root".to_string(), "/tmp/demo".to_string())]),
            cwd: Some("/tmp".to_string()),
            env: HashMap::from([("APP_ENV".to_string(), "dev".to_string())]),
            allowed_working_dirs: vec![],
            allowed_env_keys: None,
            result_target: None,
        });

        let mut result = None;
        for _ in 0..20 {
            if let Some(value) = executor.try_recv() {
                result = Some(value);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let result = result.expect("expected async action result");
        assert!(result.success);
        assert_eq!(result.stdout, "/tmp/demo");
    }

    #[tokio::test]
    async fn registered_handler_action_receives_shell_runtime() {
        let mut registry = ActionRegistry::new();
        registry.register_action_handler("env_echo", |context| async move {
            ActionOutcome::success(format!(
                "{}:{}",
                context.cwd.unwrap_or_default(),
                context.env.get("APP_ENV").cloned().unwrap_or_default()
            ))
        });
        let mut executor = OperationExecutor::with_registry(registry);

        executor.submit(OperationRequest {
            operation_id: 3,
            screen_index: 0,
            block_index: 0,
            source: OperationSource::RegisteredAction("env_echo".to_string()),
            params: HashMap::new(),
            host: HashMap::new(),
            cwd: Some("/tmp/demo".to_string()),
            env: HashMap::from([("APP_ENV".to_string(), "dev".to_string())]),
            allowed_working_dirs: vec![],
            allowed_env_keys: None,
            result_target: None,
        });

        let mut result = None;
        for _ in 0..20 {
            if let Some(value) = executor.try_recv() {
                result = Some(value);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let result = result.expect("expected async action result");
        assert!(result.success);
        assert_eq!(result.stdout, "/tmp/demo:dev");
    }

    #[tokio::test]
    async fn shell_policy_blocks_inline_shell_commands() {
        let mut executor = OperationExecutor::with_runtime(
            ActionRegistry::new(),
            ShellPolicy::RegisteredOnly,
            None,
            None,
        );

        executor.submit(OperationRequest {
            operation_id: 4,
            screen_index: 0,
            block_index: 0,
            source: OperationSource::ShellCommand("printf 'ok\\n'".to_string()),
            params: HashMap::new(),
            host: HashMap::new(),
            cwd: None,
            env: HashMap::new(),
            allowed_working_dirs: vec![],
            allowed_env_keys: None,
            result_target: None,
        });

        let mut result = None;
        for _ in 0..20 {
            if let Some(value) = executor.try_recv() {
                result = Some(value);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let result = result.expect("expected policy failure");
        assert!(!result.success);
        assert!(result.stderr.contains("disabled by host policy"));
    }

    #[tokio::test]
    async fn event_hook_receives_start_and_finish_events() {
        let events = Arc::new(Mutex::new(Vec::<HostEvent>::new()));
        let capture = events.clone();
        let hook = Arc::new(move |event| {
            capture.lock().unwrap().push(event);
        });
        let mut executor = OperationExecutor::with_runtime(
            ActionRegistry::new(),
            ShellPolicy::AllowAll,
            Some(hook),
            None,
        );

        executor.submit(OperationRequest {
            operation_id: 5,
            screen_index: 1,
            block_index: 2,
            source: OperationSource::ShellCommand("printf 'ok\\n'".to_string()),
            params: HashMap::new(),
            host: HashMap::new(),
            cwd: None,
            env: HashMap::new(),
            allowed_working_dirs: vec![],
            allowed_env_keys: None,
            result_target: None,
        });

        for _ in 0..20 {
            if executor.try_recv().is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let events = events.lock().unwrap().clone();
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0],
            HostEvent::OperationStarted {
                operation_id: 5,
                ..
            }
        ));
        assert!(matches!(
            events[1],
            HostEvent::OperationFinished {
                operation_id: 5,
                success: true,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn logger_receives_start_and_finish_records() {
        let logs = Arc::new(Mutex::new(Vec::<HostLogRecord>::new()));
        let capture = logs.clone();
        let logger = Arc::new(move |record| {
            capture.lock().unwrap().push(record);
        });
        let mut executor = OperationExecutor::with_runtime(
            ActionRegistry::new(),
            ShellPolicy::AllowAll,
            None,
            Some(logger),
        );

        executor.submit(OperationRequest {
            operation_id: 6,
            screen_index: 0,
            block_index: 0,
            source: OperationSource::ShellCommand("printf 'ok\\n'".to_string()),
            params: HashMap::new(),
            host: HashMap::new(),
            cwd: None,
            env: HashMap::new(),
            allowed_working_dirs: vec![],
            allowed_env_keys: None,
            result_target: None,
        });

        for _ in 0..20 {
            if executor.try_recv().is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let logs = logs.lock().unwrap().clone();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].level, HostLogLevel::Info);
        assert_eq!(logs[0].target, "tui01.operation");
        assert_eq!(logs[1].target, "tui01.operation");
    }

    #[tokio::test]
    async fn execution_policy_blocks_unapproved_working_dir() {
        let mut executor = OperationExecutor::with_registry(ActionRegistry::new());

        executor.submit(OperationRequest {
            operation_id: 7,
            screen_index: 0,
            block_index: 0,
            source: OperationSource::ShellCommand("printf 'ok\\n'".to_string()),
            params: HashMap::new(),
            host: HashMap::new(),
            cwd: Some("/tmp/blocked".to_string()),
            env: HashMap::new(),
            allowed_working_dirs: vec!["/tmp/allowed".to_string()],
            allowed_env_keys: None,
            result_target: None,
        });

        let mut result = None;
        for _ in 0..20 {
            if let Some(value) = executor.try_recv() {
                result = Some(value);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let result = result.expect("expected permission failure");
        assert!(!result.success);
        assert!(result.stderr.contains("working directory is not allowed"));
    }

    #[tokio::test]
    async fn execution_policy_blocks_unapproved_env_key() {
        let mut executor = OperationExecutor::with_registry(ActionRegistry::new());

        executor.submit(OperationRequest {
            operation_id: 8,
            screen_index: 0,
            block_index: 0,
            source: OperationSource::ShellCommand("printf 'ok\\n'".to_string()),
            params: HashMap::new(),
            host: HashMap::new(),
            cwd: None,
            env: HashMap::from([("SECRET_TOKEN".to_string(), "x".to_string())]),
            allowed_working_dirs: vec![],
            allowed_env_keys: Some(HashSet::from(["APP_ENV".to_string()])),
            result_target: None,
        });

        let mut result = None;
        for _ in 0..20 {
            if let Some(value) = executor.try_recv() {
                result = Some(value);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let result = result.expect("expected env failure");
        assert!(!result.success);
        assert!(result.stderr.contains("environment key is not allowed"));
    }
}
