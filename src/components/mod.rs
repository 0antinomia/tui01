//! Component trait 及其实现

mod content_panel;
mod controls;
mod menu;
mod quadrant;
mod status_panel;
mod title_panel;

pub use content_panel::{
    ContentBlock, ContentBlueprint, ContentControl, ContentPanel, ContentSection,
};
pub use controls::{ControlKind, SelectControl, TextInputControl, ToggleControl};
pub use menu::{MenuComponent, MenuItem, MenuState};
pub use quadrant::{QuadrantConfig, QuadrantLayout};
pub use status_panel::StatusPanel;
pub use title_panel::TitlePanel;

use crate::action::Action;
use crate::event::Event;
use ratatui::{layout::Rect, Frame};

/// Component trait，用于有独立状态和渲染的 UI 区域
///
/// 基于 The Elm Architecture (TEA) 模式
pub trait Component {
    /// 初始化组件
    fn init(&mut self) -> color_eyre::Result<()> {
        Ok(())
    }

    /// 检查组件是否可以接受焦点
    fn can_focus(&self) -> bool {
        false
    }

    /// 检查组件当前是否拥有焦点
    fn is_focused(&self) -> bool {
        false
    }

    /// 授予组件焦点
    fn focus(&mut self) {}

    /// 移除组件焦点
    fn blur(&mut self) {}

    /// 处理事件并返回 Action
    fn handle_events(&mut self, _event: Option<Event>) -> Action {
        Action::Noop
    }

    /// 根据 Action 更新组件状态，可选地返回新的 Action
    ///
    /// 返回 `None` 表示没有后续动作
    /// 返回 `Some(action)` 表示需要触发链式 Action
    fn update(&mut self, _action: Action) -> Option<Action> {
        None
    }

    /// 渲染组件到 frame
    fn render(&mut self, f: &mut Frame, rect: Rect);
}
