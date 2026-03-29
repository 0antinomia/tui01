//! 操作执行器核心：提交、异步执行和结果接收。

use super::registry::{ActionRegistry, RegisteredAction};
use super::shell;
use super::types::{
    ActionContext, ActionOutcome, OperationRequest, OperationResult, OperationSource,
};
use crate::host::framework_log::FrameworkLogger;
use crate::host::host_types::{HostEvent, HostLogLevel, HostLogRecord, ShellPolicy};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct OperationExecutor {
    registry: ActionRegistry,
    shell_policy: ShellPolicy,
    event_hook: Option<Arc<dyn Fn(HostEvent) + Send + Sync>>,
    logger: Option<Arc<dyn Fn(HostLogRecord) + Send + Sync>>,
    framework_logger: FrameworkLogger,
    sender: mpsc::UnboundedSender<OperationResult>,
    receiver: mpsc::UnboundedReceiver<OperationResult>,
}

impl OperationExecutor {
    pub fn new() -> Self {
        Self::with_registry(ActionRegistry::new())
    }

    pub fn with_registry(registry: ActionRegistry) -> Self {
        Self::with_runtime(registry, ShellPolicy::AllowAll, None, None, FrameworkLogger::fallback())
    }

    pub fn with_runtime(
        registry: ActionRegistry,
        shell_policy: ShellPolicy,
        event_hook: Option<Arc<dyn Fn(HostEvent) + Send + Sync>>,
        logger: Option<Arc<dyn Fn(HostLogRecord) + Send + Sync>>,
        framework_logger: FrameworkLogger,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { registry, shell_policy, event_hook, logger, framework_logger, sender, receiver }
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
        let framework_logger = self.framework_logger.clone();
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
        let start_record = HostLogRecord {
            level: HostLogLevel::Info,
            target: "tui01.operation".to_string(),
            message: format!(
                "started op={} screen={} block={} source={}",
                request.operation_id, request.screen_index, request.block_index, source_description
            ),
        };
        framework_logger.log(&start_record);
        if let Some(logger) = &logger {
            logger(start_record);
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
            if let Some(error) = shell::validate_request_permissions(
                request.cwd.as_deref(),
                &request.env,
                &request.allowed_working_dirs,
                request.allowed_env_keys.as_ref(),
            ) {
                if let Some(logger) = &logger {
                    let record = HostLogRecord {
                        level: HostLogLevel::Warn,
                        target: "tui01.operation".to_string(),
                        message: format!(
                            "blocked by execution policy op={} source={} reason={}",
                            request.operation_id, source_description, error
                        ),
                    };
                    framework_logger.log(&record);
                    logger(record);
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
                        let record = HostLogRecord {
                            level: HostLogLevel::Warn,
                            target: "tui01.operation".to_string(),
                            message: format!(
                                "blocked inline shell by policy op={} source={}",
                                request.operation_id, source_description
                            ),
                        };
                        framework_logger.log(&record);
                        logger(record);
                    }
                    ActionOutcome::failure("inline shell commands are disabled by host policy")
                }
                OperationSource::ShellCommand(command) => {
                    shell::run_shell_command(
                        command.clone(),
                        request.cwd.clone(),
                        request.env.clone(),
                    )
                    .await
                }
                OperationSource::RegisteredAction(name) => match registered {
                    Some(RegisteredAction::ShellTemplate(_))
                        if shell_policy == ShellPolicy::Disabled =>
                    {
                        if let Some(logger) = &logger {
                            let record = HostLogRecord {
                                level: HostLogLevel::Warn,
                                target: "tui01.operation".to_string(),
                                message: format!(
                                    "blocked registered shell by policy op={} source={}",
                                    request.operation_id, source_description
                                ),
                            };
                            framework_logger.log(&record);
                            logger(record);
                        }
                        ActionOutcome::failure(
                            "registered shell actions are disabled by host policy",
                        )
                    }
                    Some(RegisteredAction::ShellTemplate(template)) => {
                        let command = shell::render_command_template(
                            &template,
                            &request.params,
                            &request.host,
                        );
                        shell::run_shell_command(command, request.cwd.clone(), request.env.clone())
                            .await
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
            let finish_record = HostLogRecord {
                level: if result.success { HostLogLevel::Info } else { HostLogLevel::Error },
                target: "tui01.operation".to_string(),
                message: format!(
                    "finished op={} screen={} block={} source={} success={}",
                    result.operation_id,
                    result.screen_index,
                    result.block_index,
                    source_description,
                    result.success
                ),
            };
            framework_logger.log(&finish_record);
            if let Some(logger) = &logger {
                logger(finish_record);
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
