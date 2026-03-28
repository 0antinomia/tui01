//! 基础设施层：终端事件处理和 TUI 生命周期管理。

pub mod event;
pub mod tui;

pub use event::{Event, EventHandler, Key};
pub use tui::Tui;
