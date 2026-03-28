//! 数据展示控件实现。

use crate::event::Key;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Widget},
};

use super::control_trait::ControlTrait;
use super::helpers::{left_aligned_control_rect, truncate_to_chars};
use crate::theme::RenderContext;
use std::any::Any;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDisplayControl {
    pub value: String,
    pub dynamic: bool,
}

impl DataDisplayControl {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            dynamic: false,
        }
    }

    pub fn new_dynamic(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            dynamic: true,
        }
    }
}

impl ControlTrait for DataDisplayControl {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        let area = left_aligned_control_rect(area, 24);
        if area.width <= 2 || area.height == 0 {
            return;
        }

        let border = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(if ctx.active {
                Color::Cyan
            } else if ctx.selected {
                Color::White
            } else {
                Color::Gray
            }));
        Widget::render(border, area, buf);

        let icon = if self.dynamic { "◌" } else { "·" };
        let text = truncate_to_chars(&self.value, area.width.saturating_sub(6) as usize);
        let y = area.y + area.height.saturating_sub(1) / 2;
        buf.set_stringn(
            area.x + 1,
            y,
            format!("{icon} {text}"),
            area.width.saturating_sub(2) as usize,
            Style::default().fg(if self.dynamic { Color::Cyan } else { Color::White }),
        );
    }

    fn handle_key(&mut self, _key: Key) -> bool {
        false
    }

    fn value(&self) -> String {
        self.value.clone()
    }

    fn preferred_width(&self) -> u16 {
        24
    }

    fn is_editable(&self) -> bool {
        false
    }

    fn triggers_on_activate(&self) -> bool {
        false
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
