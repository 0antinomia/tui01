//! 异步操作执行器，负责运行真实命令并回传结果。

mod executor_core;
mod registry;
mod shell;
mod types;

// Re-export all public types so external callers see the same API
pub use executor_core::OperationExecutor;
pub use registry::ActionRegistry;
pub use types::{ActionContext, ActionOutcome, OperationRequest, OperationResult, OperationSource};

#[cfg(test)]
mod tests {
    use super::super::framework_log::FrameworkLogger;
    use super::super::host_types::{HostEvent, HostLogLevel, HostLogRecord, ShellPolicy};
    use super::shell::{render_command_template, shell_escape};
    use super::{
        ActionOutcome, ActionRegistry, OperationExecutor, OperationRequest, OperationSource,
    };
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    fn test_framework_logger() -> FrameworkLogger {
        FrameworkLogger::new(std::env::temp_dir()).unwrap()
    }

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
                context.params.get("project_name").cloned().unwrap_or_default()
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
            ActionOutcome::success(context.host.get("project_root").cloned().unwrap_or_default())
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
            test_framework_logger(),
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
            test_framework_logger(),
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
        assert!(matches!(events[0], HostEvent::OperationStarted { operation_id: 5, .. }));
        assert!(matches!(
            events[1],
            HostEvent::OperationFinished { operation_id: 5, success: true, .. }
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
            test_framework_logger(),
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
