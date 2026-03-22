//! Host-side runtime integration surface.

use crate::executor::{ActionContext, ActionOutcome, ActionRegistry};
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};

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
}

#[cfg(test)]
mod tests {
    use super::RuntimeHost;
    use crate::executor::ActionOutcome;

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
    }
}
