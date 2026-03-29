//! 数值输入控件实现。

use crate::event::Key;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Widget},
};

use super::control_trait::ControlTrait;
use super::helpers::{framed_block, left_aligned_control_rect, render_feedback_marker};
use crate::theme::RenderContext;
use std::any::Any;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberInputControl {
    pub value: String,
    pub placeholder: String,
}

impl NumberInputControl {
    pub fn new(value: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            placeholder: placeholder.into(),
        }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Char(ch) if ch.is_ascii_digit() => {
                self.value.push(ch);
                true
            }
            Key::Backspace => self.value.pop().is_some(),
            _ => false,
        }
    }
}

impl ControlTrait for NumberInputControl {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        let area = left_aligned_control_rect(area, 14);
        let display = if self.value.is_empty() {
            self.placeholder.as_str()
        } else {
            self.value.as_str()
        };
        let style = if self.value.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };
        let display = if ctx.active {
            format!("{display}▏")
        } else {
            display.to_string()
        };
        let block = framed_block(ctx.selected, ctx.active, ctx.feedback);
        let widget = Paragraph::new(display)
            .block(block)
            .alignment(Alignment::Left)
            .style(style);
        Widget::render(widget, area, buf);
        render_feedback_marker(buf, area, ctx.feedback);
    }

    fn handle_key(&mut self, key: Key) -> bool {
        self.handle_key(key)
    }

    fn value(&self) -> String {
        self.value.clone()
    }

    fn preferred_width(&self) -> u16 {
        14
    }

    fn is_editable(&self) -> bool {
        true
    }

    fn triggers_on_activate(&self) -> bool {
        false
    }

    fn box_clone(&self) -> Box<dyn ControlTrait> {
        Box::new(self.clone())
    }

    fn box_eq(&self, other: &dyn ControlTrait) -> bool {
        other.as_any().downcast_ref::<Self>() == Some(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::NumberInputControl;
    use crate::event::Key;

    #[test]
    fn number_input_accepts_only_digits() {
        let mut control = NumberInputControl::new("", "0");
        assert!(control.handle_key(Key::Char('8')));
        assert!(!control.handle_key(Key::Char('x')));
        assert_eq!(control.value, "8");
    }
}
