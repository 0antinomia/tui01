//! 下拉选择控件实现。

use crate::event::Key;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

use super::control_trait::ControlTrait;
use super::helpers::{
    feedback_accent, framed_block, left_aligned_control_rect, render_feedback_marker,
    truncate_to_chars,
};
use crate::theme::RenderContext;
use std::any::Any;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectControl {
    pub options: Vec<String>,
    pub selected: usize,
}

impl SelectControl {
    pub fn new(options: impl IntoIterator<Item = impl Into<String>>, selected: usize) -> Self {
        let options: Vec<String> = options.into_iter().map(Into::into).collect();
        let selected = selected.min(options.len().saturating_sub(1));
        Self { options, selected }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        if self.options.is_empty() {
            return false;
        }

        match key {
            Key::Left | Key::Up | Key::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                    true
                } else {
                    false
                }
            }
            Key::Right | Key::Down | Key::Char('j') | Key::Char('l') => {
                if self.selected + 1 < self.options.len() {
                    self.selected += 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn render_collapsed(&self, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        let current = self.options.get(self.selected).map(|s| s.as_str()).unwrap_or("-");
        let block = framed_block(ctx.selected, ctx.active, ctx.feedback);
        ratatui::widgets::Widget::render(block, area, buf);

        let inner_y = area.y + area.height.saturating_sub(1) / 2;
        let inner_x = area.x + 1;
        let inner_width = area.width.saturating_sub(2);
        if inner_width == 0 {
            return;
        }

        let arrow = "⌄";
        let arrow_width = 1u16;
        let value_width = inner_width.saturating_sub(arrow_width + 2);
        let value = truncate_to_chars(current, value_width as usize);
        let value_style = Style::default().fg(if ctx.active { Color::Cyan } else { Color::White });
        let arrow_style = Style::default().fg(if ctx.active {
            Color::Cyan
        } else if ctx.selected {
            Color::White
        } else {
            Color::Gray
        });

        buf.set_stringn(inner_x, inner_y, &value, value_width as usize, value_style);
        let arrow_x = area.x + area.width.saturating_sub(3);
        buf.set_string(arrow_x, inner_y, arrow, arrow_style);
        render_feedback_marker(buf, area, ctx.feedback);
    }

    fn render_expanded(&self, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        let border = feedback_accent(ctx.feedback, ctx.selected, true);
        let base_style = Style::default().fg(Color::Gray);
        let active_style = Style::default().fg(border);

        let rows = [
            self.selected.checked_sub(1),
            Some(self.selected),
            self.selected.checked_add(1).filter(|index| *index < self.options.len()),
        ];

        for (offset, option_index) in rows.into_iter().enumerate() {
            let y = area.y + offset as u16;
            if y >= area.y + area.height {
                break;
            }

            let text = option_index
                .and_then(|index| self.options.get(index))
                .map(String::as_str)
                .unwrap_or("");
            let style = if option_index == Some(self.selected) { active_style } else { base_style };
            let text_width = area.width.saturating_sub(4) as usize;
            let text = truncate_to_chars(text, text_width);
            let text_x = area.x + 1;
            buf.set_stringn(text_x, y, &text, text_width, style);

            if option_index == Some(self.selected) {
                let arrow_x = area.x + area.width.saturating_sub(3);
                buf.set_string(arrow_x, y, "⌄", Style::default().fg(border));
            }
        }
        render_feedback_marker(buf, area, ctx.feedback);
    }
}

impl ControlTrait for SelectControl {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        let area = left_aligned_control_rect(area, 22);
        if area.width <= 2 || area.height == 0 {
            return;
        }

        if ctx.active && area.height >= 3 {
            self.render_expanded(area, buf, ctx);
        } else {
            self.render_collapsed(area, buf, ctx);
        }
    }

    fn handle_key(&mut self, key: Key) -> bool {
        self.handle_key(key)
    }

    fn value(&self) -> String {
        self.options.get(self.selected).cloned().unwrap_or_default()
    }

    fn preferred_width(&self) -> u16 {
        22
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
    use super::SelectControl;
    use crate::event::Key;

    #[test]
    fn select_control_moves_between_options() {
        let mut control = SelectControl::new(["A", "B", "C"], 0);
        assert!(control.handle_key(Key::Right));
        assert_eq!(control.selected, 1);
        assert!(control.handle_key(Key::Left));
        assert_eq!(control.selected, 0);
        assert!(!control.handle_key(Key::Enter));
        assert_eq!(control.selected, 0);
    }
}
