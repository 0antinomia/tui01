//! 控件模块：各控件类型的独立实现文件和统一导出。

pub mod action_button;
pub mod control_trait;
pub mod data_display;
pub mod helpers;
pub mod log_output;
pub mod number_input;
pub mod select;
pub mod text_input;
pub mod toggle;

pub use action_button::{ActionButtonControl, ActionButtonKind};
pub use control_trait::{ControlFeedback, ControlTrait};
pub use data_display::DataDisplayControl;
pub use helpers::{truncate_to_chars, wrap_text_lines};
pub use log_output::LogOutputControl;
pub use number_input::NumberInputControl;
pub use select::SelectControl;
pub use text_input::TextInputControl;
pub use toggle::ToggleControl;

use crate::event::Key;
use crate::theme::RenderContext;
use ratatui::{buffer::Buffer, layout::Rect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinControl {
    TextInput(TextInputControl),
    NumberInput(NumberInputControl),
    Select(SelectControl),
    Toggle(ToggleControl),
    ActionButton(ActionButtonControl),
    StaticData(DataDisplayControl),
    DynamicData(DataDisplayControl),
    LogOutput(LogOutputControl),
}

impl BuiltinControl {
    pub fn preferred_width(&self) -> u16 {
        match self {
            Self::TextInput(c) => c.preferred_width(),
            Self::NumberInput(c) => c.preferred_width(),
            Self::Select(c) => c.preferred_width(),
            Self::Toggle(c) => c.preferred_width(),
            Self::ActionButton(c) => c.preferred_width(),
            Self::StaticData(c) | Self::DynamicData(c) => c.preferred_width(),
            Self::LogOutput(c) => c.preferred_width(),
        }
    }

    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &RenderContext,
    ) {
        match self {
            Self::TextInput(c) => c.render(area, buf, ctx),
            Self::NumberInput(c) => c.render(area, buf, ctx),
            Self::Select(c) => c.render(area, buf, ctx),
            Self::Toggle(c) => c.render(area, buf, ctx),
            Self::ActionButton(c) => c.render(area, buf, ctx),
            Self::StaticData(c) => c.render(area, buf, ctx),
            Self::DynamicData(c) => c.render(area, buf, ctx),
            Self::LogOutput(c) => c.render(area, buf, ctx),
        }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        match self {
            Self::TextInput(c) => c.handle_key(key),
            Self::NumberInput(c) => c.handle_key(key),
            Self::Select(c) => c.handle_key(key),
            Self::Toggle(c) => c.handle_key(key),
            Self::ActionButton(c) => c.handle_key(key),
            Self::StaticData(c) | Self::DynamicData(c) => c.handle_key(key),
            Self::LogOutput(c) => c.handle_key(key),
        }
    }

    pub fn value(&self) -> String {
        match self {
            Self::TextInput(c) => c.value(),
            Self::NumberInput(c) => c.value(),
            Self::Select(c) => c.value(),
            Self::Toggle(c) => c.value(),
            Self::ActionButton(c) => c.value(),
            Self::StaticData(c) => c.value(),
            Self::DynamicData(c) => c.value(),
            Self::LogOutput(c) => c.value(),
        }
    }

    pub fn is_editable(&self) -> bool {
        match self {
            Self::TextInput(c) => c.is_editable(),
            Self::NumberInput(c) => c.is_editable(),
            Self::Select(c) => c.is_editable(),
            Self::Toggle(c) => c.is_editable(),
            Self::ActionButton(c) => c.is_editable(),
            Self::StaticData(c) => c.is_editable(),
            Self::DynamicData(c) => c.is_editable(),
            Self::LogOutput(c) => c.is_editable(),
        }
    }

    pub fn triggers_on_activate(&self) -> bool {
        match self {
            Self::TextInput(c) => c.triggers_on_activate(),
            Self::NumberInput(c) => c.triggers_on_activate(),
            Self::Select(c) => c.triggers_on_activate(),
            Self::Toggle(c) => c.triggers_on_activate(),
            Self::ActionButton(c) => c.triggers_on_activate(),
            Self::StaticData(c) => c.triggers_on_activate(),
            Self::DynamicData(c) => c.triggers_on_activate(),
            Self::LogOutput(c) => c.triggers_on_activate(),
        }
    }
}

/// 统一控件类型，支持内置控件和宿主应用注册的自定义控件。
///
/// 内置控件通过 `Builtin(BuiltinControl)` 包装，自定义控件通过 `Custom(Box<dyn ControlTrait>)` 包装。
/// 手动实现 Clone、PartialEq、Eq 和 Debug，因为 `Box<dyn ControlTrait>` 不支持 derive 宏。
pub enum AnyControl {
    /// 内置控件（TextInput、Select、Toggle 等）。
    Builtin(BuiltinControl),
    /// 宿主应用注册的自定义控件。
    Custom(Box<dyn ControlTrait>),
}

impl Clone for AnyControl {
    fn clone(&self) -> Self {
        match self {
            Self::Builtin(c) => Self::Builtin(c.clone()),
            Self::Custom(c) => Self::Custom(c.box_clone()),
        }
    }
}

impl PartialEq for AnyControl {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Builtin(a), Self::Builtin(b)) => a == b,
            (Self::Custom(a), Self::Custom(b)) => a.box_eq(b.as_ref()),
            _ => false,
        }
    }
}

impl Eq for AnyControl {}

impl std::fmt::Debug for AnyControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin(c) => write!(f, "AnyControl::Builtin({c:?})"),
            Self::Custom(_) => write!(f, "AnyControl::Custom(<dyn ControlTrait>)"),
        }
    }
}

impl AnyControl {
    pub fn render(&self, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        match self {
            Self::Builtin(bc) => bc.render(area, buf, ctx),
            Self::Custom(cc) => cc.render(area, buf, ctx),
        }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        match self {
            Self::Builtin(bc) => bc.handle_key(key),
            Self::Custom(cc) => cc.handle_key(key),
        }
    }

    pub fn value(&self) -> String {
        match self {
            Self::Builtin(bc) => bc.value(),
            Self::Custom(cc) => cc.value(),
        }
    }

    pub fn is_editable(&self) -> bool {
        match self {
            Self::Builtin(bc) => bc.is_editable(),
            Self::Custom(cc) => cc.is_editable(),
        }
    }

    pub fn triggers_on_activate(&self) -> bool {
        match self {
            Self::Builtin(bc) => bc.triggers_on_activate(),
            Self::Custom(cc) => cc.triggers_on_activate(),
        }
    }

    pub fn preferred_width(&self) -> u16 {
        match self {
            Self::Builtin(bc) => bc.preferred_width(),
            Self::Custom(cc) => cc.preferred_width(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AnyControl, BuiltinControl, TextInputControl};
    use crate::event::Key;

    #[test]
    fn builtin_control_routes_key_handling() {
        let mut control = BuiltinControl::TextInput(TextInputControl::new("", "name"));
        assert!(control.handle_key(Key::Char('x')));
        match control {
            BuiltinControl::TextInput(inner) => assert_eq!(inner.value, "x"),
            _ => panic!("unexpected control kind"),
        }
    }

    #[test]
    fn any_control_builtin_clone_and_eq() {
        let original = AnyControl::Builtin(BuiltinControl::TextInput(TextInputControl::new("hello", "placeholder")));
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn any_control_builtin_and_custom_are_not_equal() {
        let builtin = AnyControl::Builtin(BuiltinControl::TextInput(TextInputControl::new("hello", "placeholder")));
        let custom = AnyControl::Custom(Box::new(TextInputControl::new("hello", "placeholder")));
        assert_ne!(builtin, custom);
    }
}
