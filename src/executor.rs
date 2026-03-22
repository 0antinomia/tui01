//! 异步操作执行器，负责运行真实命令并回传结果。

use std::collections::HashMap;
use std::future::Future;
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
    sender: mpsc::UnboundedSender<OperationResult>,
    receiver: mpsc::UnboundedReceiver<OperationResult>,
}

impl OperationExecutor {
    pub fn new() -> Self {
        Self::with_registry(ActionRegistry::new())
    }

    pub fn with_registry(registry: ActionRegistry) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            registry,
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
        let registered = match &request.source {
            OperationSource::ShellCommand(_) => None,
            OperationSource::RegisteredAction(name) => self.registry.resolve(name),
        };
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
            let outcome = match &request.source {
                OperationSource::ShellCommand(command) => {
                    run_shell_command(command.clone(), request.cwd.clone(), request.env.clone())
                        .await
                }
                OperationSource::RegisteredAction(name) => match registered {
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

            let _ = sender.send(result);
        });
    }

    pub fn try_recv(&mut self) -> Option<OperationResult> {
        self.receiver.try_recv().ok()
    }
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
    use std::collections::HashMap;

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
}
