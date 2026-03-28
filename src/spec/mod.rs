//! 声明式规范层：构建器、模式和字段工厂。

pub mod builder;
pub mod schema;
pub mod field;

pub use builder::{page, screen, section, AppSpec, AppValidationError};
pub use schema::{PageSpec, SectionSpec, FieldSpec};
