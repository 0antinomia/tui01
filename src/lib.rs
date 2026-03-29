//! tui01 - 基于 ratatui 的 Rust 终端界面框架
//!
//! 提供固定"田"字形四区布局的高层封装，快速将命令行工具 TUI 化。

// Internal implementation layers stay available within the crate without
// advertising them as part of the external contract.
mod app;
mod components;
pub mod controls;
pub mod host;
mod infra;
mod runtime;
pub mod spec;
pub mod theme;
pub mod prelude;

// Crate-private bridges keep internal modules stable while the public facade
// contracts around the canonical surface.
pub(crate) use app::{action, showcase};
pub(crate) use infra::{event, tui};
pub(crate) use spec::builder;
pub use spec::field;
pub use spec::schema;
