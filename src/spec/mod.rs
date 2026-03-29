//! 声明式规范层：构建器、模式和字段工厂。

pub mod builder;
pub mod field;
pub mod schema;

// Schema types remain available as extension-layer access, while the builder
// entry points are intentionally routed through the canonical prelude.
pub use schema::{FieldSpec, PageSpec, SectionSpec};
