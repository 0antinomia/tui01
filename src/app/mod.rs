//! 应用层 / TEA 核心：展示应用、动作枚举和应用壳层。

pub mod showcase;
pub mod action;
pub mod app_impl;

pub use showcase::{ShowcaseApp, ShowcaseScreen, ShowcaseCopy};
pub use action::Action;
pub use app_impl::App;
