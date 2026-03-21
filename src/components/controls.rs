//! Unified form controls for the right-bottom content panel.

use crate::event::Key;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlKind {
    TextInput(TextInputControl),
    Select(SelectControl),
    Toggle(ToggleControl),
    StaticText(String),
}

impl ControlKind {
    pub fn render(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool) {
        match self {
            Self::TextInput(control) => control.render(area, buf, selected, active),
            Self::Select(control) => control.render(area, buf, selected, active),
            Self::Toggle(control) => control.render(area, buf, selected, active),
            Self::StaticText(text) => {
                let widget = Paragraph::new(text.as_str())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Cyan));
                Widget::render(widget, area, buf);
            }
        }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        match self {
            Self::TextInput(control) => control.handle_key(key),
            Self::Select(control) => control.handle_key(key),
            Self::Toggle(control) => control.handle_key(key),
            Self::StaticText(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputControl {
    pub value: String,
    pub placeholder: String,
}

impl TextInputControl {
    pub fn new(value: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            placeholder: placeholder.into(),
        }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        match key {
            Key::Char(ch) if !ch.is_control() => {
                self.value.push(ch);
                true
            }
            Key::Backspace => self.value.pop().is_some(),
            _ => false,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool) {
        let area = left_aligned_control_rect(area, 22);
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
        let display = if active {
            format!("{display}▏")
        } else {
            display.to_string()
        };
        let block = framed_block(selected, active);
        let widget = Paragraph::new(display)
            .block(block)
            .alignment(Alignment::Left)
            .style(style);
        Widget::render(widget, area, buf);
    }
}

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

    fn render(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool) {
        let area = left_aligned_control_rect(area, 22);
        if area.width <= 2 || area.height == 0 {
            return;
        }

        if active && area.height >= 3 {
            self.render_expanded(area, buf, selected);
        } else {
            self.render_collapsed(area, buf, selected, active);
        }
    }

    fn render_collapsed(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool) {
        let current = self
            .options
            .get(self.selected)
            .map(|s| s.as_str())
            .unwrap_or("-");
        let block = framed_block(selected, active);
        Widget::render(block, area, buf);

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
        let value_style = Style::default().fg(if active { Color::Cyan } else { Color::White });
        let arrow_style = Style::default().fg(if active {
            Color::Cyan
        } else if selected {
            Color::White
        } else {
            Color::Gray
        });

        buf.set_stringn(inner_x, inner_y, &value, value_width as usize, value_style);
        let arrow_x = area.x + area.width.saturating_sub(3);
        buf.set_string(arrow_x, inner_y, arrow, arrow_style);
    }

    fn render_expanded(&self, area: Rect, buf: &mut Buffer, selected: bool) {
        let border = if selected { Color::Cyan } else { Color::White };
        let base_style = Style::default().fg(Color::Gray);
        let active_style = Style::default().fg(Color::Cyan);

        let rows = [
            self.selected.checked_sub(1),
            Some(self.selected),
            self.selected
                .checked_add(1)
                .filter(|index| *index < self.options.len()),
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
            let style = if option_index == Some(self.selected) {
                active_style
            } else {
                base_style
            };
            let text_width = area.width.saturating_sub(4) as usize;
            let text = truncate_to_chars(text, text_width);
            let text_x = area.x + 1;
            buf.set_stringn(text_x, y, &text, text_width, style);

            if option_index == Some(self.selected) {
                let arrow_x = area.x + area.width.saturating_sub(3);
                buf.set_string(arrow_x, y, "⌄", Style::default().fg(border));
            }
        }
    }
}

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

    fn render(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool) {
        let area = left_aligned_control_rect(area, 10);
        if area.width < 10 || area.height == 0 {
            return;
        }

        let track_y = area.y + area.height.saturating_sub(3) / 2;
        draw_toggle_track(
            buf,
            Rect::new(area.x, track_y, area.width.min(10), 3),
            self.on,
            if active {
                Color::Cyan
            } else if selected {
                Color::White
            } else {
                Color::Gray
            },
            if self.on {
                Color::Cyan
            } else {
                Color::Rgb(120, 126, 132)
            },
            Color::Rgb(250, 252, 254),
        );
    }
}

fn draw_toggle_track(
    buf: &mut Buffer,
    area: Rect,
    on: bool,
    border: Color,
    track_bg: Color,
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

fn framed_block(selected: bool, active: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(if active {
            Color::Cyan
        } else if selected {
            Color::White
        } else {
            Color::Gray
        }))
}

fn left_aligned_control_rect(area: Rect, desired_width: u16) -> Rect {
    let width = desired_width.min(area.width.max(1));
    let x = area.x;
    Rect::new(x, area.y, width, area.height)
}

fn truncate_to_chars(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_string();
    }

    if max_chars == 1 {
        return "…".to_string();
    }

    text.chars().take(max_chars - 1).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::{ControlKind, SelectControl, TextInputControl, ToggleControl};
    use crate::event::Key;

    #[test]
    fn text_input_appends_and_deletes_chars() {
        let mut control = TextInputControl::new("", "placeholder");
        assert!(control.handle_key(Key::Char('a')));
        assert_eq!(control.value, "a");
        assert!(control.handle_key(Key::Backspace));
        assert_eq!(control.value, "");
    }

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

    #[test]
    fn toggle_control_flips() {
        let mut control = ToggleControl::new(false);
        assert!(control.handle_key(Key::Enter));
        assert!(control.on);
    }

    #[test]
    fn control_kind_routes_key_handling() {
        let mut control = ControlKind::TextInput(TextInputControl::new("", "name"));
        assert!(control.handle_key(Key::Char('x')));
        match control {
            ControlKind::TextInput(inner) => assert_eq!(inner.value, "x"),
            _ => panic!("unexpected control kind"),
        }
    }
}
