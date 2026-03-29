//! 动作注册表：管理 shell 模板和自定义处理器。

use super::types::{ActionContext, ActionOutcome};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type ActionFuture = Pin<Box<dyn Future<Output = ActionOutcome> + Send>>;
type ActionHandler = dyn Fn(ActionContext) -> ActionFuture + Send + Sync;

#[derive(Clone)]
pub(super) enum RegisteredAction {
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
        self.actions.insert(name.into(), RegisteredAction::ShellTemplate(command.into()));
    }

    pub fn register_action_handler<F, Fut>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(ActionContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ActionOutcome> + Send + 'static,
    {
        let handler =
            Arc::new(move |context: ActionContext| -> ActionFuture { Box::pin(handler(context)) });
        self.actions.insert(name.into(), RegisteredAction::Handler(handler));
    }

    pub fn has_action(&self, name: &str) -> bool {
        self.actions.contains_key(name)
    }

    pub(super) fn resolve(&self, name: &str) -> Option<RegisteredAction> {
        self.actions.get(name).cloned()
    }
}
