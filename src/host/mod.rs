//! 宿主集成层：运行时接入、操作执行器和日志。

pub mod executor;
pub mod framework_log;
pub mod host_types;

pub use executor::*;
pub use framework_log::*;
pub use host_types::*;
