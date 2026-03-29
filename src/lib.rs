//! tui01 - 基于 ratatui 的 Rust 终端界面框架。
//!
//! 对外契约收敛到 `prelude`、`field` 和 `host::RuntimeHost`；
//! 其余模块只作为扩展层或 crate 内部实现存在。

// Internal implementation layers stay available within the crate without
// advertising them as part of the external contract.
mod app;
mod components;
pub mod controls;
pub mod host;
mod infra;
pub mod prelude;
mod runtime;
pub mod spec;
pub mod theme;

// Keep internal module paths stable without widening the external facade again.
pub(crate) use app::{action, showcase};
pub(crate) use infra::{event, tui};
pub(crate) use spec::builder;
pub use spec::field;
pub use spec::schema;
