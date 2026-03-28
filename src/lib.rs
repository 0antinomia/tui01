//! tui01 - 基于 ratatui 的 Rust 终端界面框架
//!
//! 提供固定"田"字形四区布局的高层封装，快速将命令行工具 TUI 化。

// Domain modules (D-01)
pub mod controls;
pub mod components;
pub mod host;
pub mod infra;
pub mod runtime;
pub mod spec;
pub mod app;

pub mod theme;

// Prelude stays at top level
pub mod prelude;

// Backward-compatible re-export aliases (D-06, D-08)
// Old paths like tui01::builder::AppSpec continue to work
pub use spec::builder;
pub use spec::schema;
pub use spec::field;
pub use app::showcase;
pub use app::action;
pub use infra::event;
pub use infra::tui;
