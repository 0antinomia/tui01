//! 宿主集成层：运行时接入、操作执行器和日志。

pub mod executor;
pub mod framework_log;
pub mod host_types;

pub use executor::{ActionContext, ActionOutcome};
pub use host_types::{
    ControlFactory, ControlRegistry, HostEvent, HostLogLevel, HostLogRecord, RuntimeHost,
    ShellPolicy,
};

pub(crate) use executor::{ActionRegistry, OperationExecutor};
pub(crate) use framework_log::FrameworkLogger;
