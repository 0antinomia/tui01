//! 声明式规范层：构建器、模式和字段工厂。

pub mod builder;
pub mod field;
pub mod schema;

pub use builder::{AppSpec, AppValidationError, page, screen, section};
pub use schema::{FieldSpec, PageSpec, SectionSpec};
