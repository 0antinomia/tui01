//! tui01 - 基于 ratatui 的 Rust TUI 框架
//!
//! 提供固定"田"字形四区布局的高层封装，快速将命令行工具 TUI 化。
//!
//! 推荐宿主接入面：
//! - [`prelude`]
//! - [`field`]
//! - [`host::RuntimeHost`]
//!
//! 其余模块当前仍然公开，以支持框架内部组合和渐进迁移，但不作为稳定接入面承诺。

pub mod action;
pub mod app;
/// Internal-oriented builder layer. Prefer [`crate::prelude`] in host applications.
pub mod builder;
/// Internal component implementations. Not part of the recommended host integration surface.
pub mod components;
pub mod event;
pub mod executor;
/// Recommended concise field factories for host applications.
pub mod field;
pub mod framework_log;
pub mod host;
/// Recommended public imports for host applications.
pub mod prelude;
/// Runtime model/state layer. Public for now, but not part of the recommended stable host API.
pub mod runtime;
/// Declarative schema layer. Prefer [`crate::field`] and [`crate::prelude`] in host applications.
pub mod schema;
/// Internal showcase shell. Host applications should build through [`crate::prelude::AppSpec`].
pub mod showcase;
pub mod tui;
