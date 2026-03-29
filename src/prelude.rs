//! 推荐给宿主应用使用的公开导入集合。

pub use crate::builder::{AppSpec, AppValidationError, page, screen, section};
pub use crate::controls::ControlTrait;
pub use crate::event::{Event, EventHandler, Key};
pub use crate::field;
pub use crate::host::ControlRegistry;
pub use crate::host::{HostEvent, HostLogLevel, HostLogRecord, RuntimeHost, ShellPolicy};
pub use crate::theme::{LayoutAreas, LayoutStrategy, RenderContext, Theme};
