//! `Component` trait 及其实现。

mod content_panel;
mod menu;
mod quadrant;
mod status_panel;
mod title_panel;

pub use content_panel::ContentPanel;
pub use menu::{MenuComponent, MenuItem};
pub use quadrant::{QuadrantConfig, QuadrantLayout};
pub use status_panel::StatusPanel;
pub use title_panel::TitlePanel;

use crate::action::Action;
use crate::event::Event;
use ratatui::{Frame, layout::Rect};

/// `Component` trait，用于有独立状态和渲染逻辑的界面区域。
///
/// 基于 The Elm Architecture（TEA）模式。
pub trait Component {
    /// 初始化组件。
    #[allow(dead_code)]
    fn init(&mut self) -> color_eyre::Result<()> {
        Ok(())
    }

    /// 检查组件是否可以接受焦点。
    #[allow(dead_code)]
    fn can_focus(&self) -> bool {
        false
    }

    /// 检查组件当前是否拥有焦点。
    #[allow(dead_code)]
    fn is_focused(&self) -> bool {
        false
    }

    /// 授予组件焦点。
    fn focus(&mut self) {}

    /// 移除组件焦点。
    fn blur(&mut self) {}

    /// 处理事件并返回动作。
    fn handle_events(&mut self, _event: Option<Event>) -> Action {
        Action::Noop
    }

    /// 根据动作更新组件状态，并可选返回新的动作。
    ///
    /// 返回 `None` 表示没有后续动作。
    /// 返回 `Some(action)` 表示需要触发链式动作。
    #[allow(dead_code)]
    fn update(&mut self, _action: Action) -> Option<Action> {
        None
    }

    /// 将组件渲染到当前帧。
    fn render(&mut self, f: &mut Frame, rect: Rect);
}
