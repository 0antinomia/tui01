//! 动作按钮控件实现。

use crate::event::Key;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use super::control_trait::{ControlFeedback, ControlTrait};
use super::helpers::{framed_block, left_aligned_control_rect, render_feedback_marker, truncate_to_chars};
use std::any::Any;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionButtonKind {
    Primary,
    Refresh,
    Danger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionButtonControl {
    pub label: String,
    pub kind: ActionButtonKind,
}

impl ActionButtonControl {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: ActionButtonKind::Primary,
        }
    }

    pub fn refresh(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: ActionButtonKind::Refresh,
        }
    }

    pub fn danger(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: ActionButtonKind::Danger,
        }
    }

    pub fn handle_key(&mut self, _key: Key) -> bool {
        false
    }
}

impl ControlTrait for ActionButtonControl {
    fn render(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool, feedback: ControlFeedback) {
        let area = left_aligned_control_rect(area, 16);
        if area.width <= 2 || area.height == 0 {
            return;
        }

        let border = framed_block(selected, active, feedback);
        Widget::render(border, area, buf);

        let icon = match self.kind {
            ActionButtonKind::Primary => "•",
            ActionButtonKind::Refresh => "↻",
            ActionButtonKind::Danger => "!",
        };
        let text = truncate_to_chars(&self.label, area.width.saturating_sub(6) as usize);
        let content = format!("{icon} {text}");
        let y = area.y + area.height.saturating_sub(1) / 2;
        buf.set_stringn(
            area.x + 1,
            y,
            content,
            area.width.saturating_sub(2) as usize,
            Style::default().fg(match self.kind {
                ActionButtonKind::Danger => Color::LightRed,
                _ => Color::White,
            }),
        );
        render_feedback_marker(buf, area, feedback);
    }

    fn handle_key(&mut self, _key: Key) -> bool {
        false
    }

    fn value(&self) -> String {
        self.label.clone()
    }

    fn preferred_width(&self) -> u16 {
        16
    }

    fn is_editable(&self) -> bool {
        false
    }

    fn triggers_on_activate(&self) -> bool {
        true
    }

    fn box_clone(&self) -> Box<dyn ControlTrait> {
        Box::new(self.clone())
    }

    fn box_eq(&self, other: &dyn ControlTrait) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self == o)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
