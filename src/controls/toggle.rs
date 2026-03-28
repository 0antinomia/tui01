//! 开关控件实现。

use crate::event::Key;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

use super::control_trait::{ControlFeedback, ControlTrait};
use super::helpers::{left_aligned_control_rect, render_feedback_marker};
use std::any::Any;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleControl {
    pub on: bool,
    pub on_label: String,
    pub off_label: String,
}

impl ToggleControl {
    pub fn new(on: bool) -> Self {
        Self {
            on,
            on_label: "ON".to_string(),
            off_label: "OFF".to_string(),
        }
    }

    pub fn labels(mut self, on_label: impl Into<String>, off_label: impl Into<String>) -> Self {
        self.on_label = on_label.into();
        self.off_label = off_label.into();
        self
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Left | Key::Right | Key::Char('h') | Key::Char('l') | Key::Enter => {
                self.on = !self.on;
                true
            }
            _ => false,
        }
    }
}

fn draw_toggle_track(
    buf: &mut Buffer,
    area: Rect,
    on: bool,
    track_bg: Color,
    border: Color,
    knob_bg: Color,
) {
    if area.width < 10 || area.height < 3 {
        return;
    }

    let inner_width = area.width.saturating_sub(2);
    let knob_width = (inner_width / 2).max(1);
    let inner_x = area.x + 1;
    let knob_x = if on {
        inner_x + inner_width.saturating_sub(knob_width)
    } else {
        inner_x
    };
    let top_y = area.y;
    let middle_y = area.y + 1;
    let bottom_y = area.y + 2;
    let border_style = Style::default().fg(border);
    let track_style = Style::default().bg(track_bg);
    let knob_style = Style::default().bg(knob_bg);

    buf.set_string(area.x, top_y, "╭", border_style);
    for offset in 0..inner_width {
        buf.set_string(inner_x + offset, top_y, "─", border_style);
    }
    buf.set_string(area.x + area.width - 1, top_y, "╮", border_style);

    buf.set_string(area.x, middle_y, "│", border_style);
    for offset in 0..inner_width {
        buf.set_string(inner_x + offset, middle_y, " ", track_style);
    }
    for offset in 0..knob_width {
        buf.set_string(knob_x + offset, middle_y, " ", knob_style);
    }
    buf.set_string(
        area.x + area.width.saturating_sub(1),
        middle_y,
        "│",
        border_style,
    );

    buf.set_string(area.x, bottom_y, "╰", border_style);
    for offset in 0..inner_width {
        buf.set_string(inner_x + offset, bottom_y, "─", border_style);
    }
    buf.set_string(
        area.x + area.width.saturating_sub(1),
        bottom_y,
        "╯",
        border_style,
    );
}

impl ControlTrait for ToggleControl {
    fn render(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool, feedback: ControlFeedback) {
        let area = left_aligned_control_rect(area, 10);
        if area.width < 10 || area.height == 0 {
            return;
        }

        let track_y = area.y + area.height.saturating_sub(3) / 2;
        draw_toggle_track(
            buf,
            Rect::new(area.x, track_y, area.width.min(10), 3),
            self.on,
            if self.on {
                Color::Cyan
            } else {
                Color::Rgb(120, 126, 132)
            },
            if active {
                Color::Cyan
            } else if selected {
                Color::White
            } else {
                Color::Gray
            },
            Color::Rgb(250, 252, 254),
        );
        render_feedback_marker(buf, area, feedback);
    }

    fn handle_key(&mut self, key: Key) -> bool {
        self.handle_key(key)
    }

    fn value(&self) -> String {
        self.on.to_string()
    }

    fn preferred_width(&self) -> u16 {
        10
    }

    fn is_editable(&self) -> bool {
        true
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

#[cfg(test)]
mod tests {
    use super::ToggleControl;
    use crate::event::Key;

    #[test]
    fn toggle_control_flips() {
        let mut control = ToggleControl::new(false);
        assert!(control.handle_key(Key::Enter));
        assert!(control.on);
    }
}
