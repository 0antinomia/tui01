//! Unified form controls for the right-bottom content panel.

use crate::event::Key;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlKind {
    TextInput(TextInputControl),
    NumberInput(NumberInputControl),
    Select(SelectControl),
    Toggle(ToggleControl),
    ActionButton(ActionButtonControl),
    StaticData(DataDisplayControl),
    DynamicData(DataDisplayControl),
    LogOutput(LogOutputControl),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFeedback {
    Idle,
    Running(usize),
    Success,
    Failure,
}

impl ControlKind {
    pub fn preferred_width(&self) -> u16 {
        match self {
            Self::TextInput(_) => 22,
            Self::NumberInput(_) => 14,
            Self::Select(_) => 22,
            Self::Toggle(_) => 10,
            Self::ActionButton(_) => 16,
            Self::StaticData(_) | Self::DynamicData(_) => 24,
            Self::LogOutput(_) => 36,
        }
    }

    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    ) {
        match self {
            Self::TextInput(control) => control.render(area, buf, selected, active, feedback),
            Self::NumberInput(control) => control.render(area, buf, selected, active, feedback),
            Self::Select(control) => control.render(area, buf, selected, active, feedback),
            Self::Toggle(control) => control.render(area, buf, selected, active, feedback),
            Self::ActionButton(control) => control.render(area, buf, selected, active, feedback),
            Self::StaticData(control) => control.render(area, buf, false, false, false),
            Self::DynamicData(control) => control.render(area, buf, true, false, false),
            Self::LogOutput(control) => control.render(area, buf, selected, active),
        }
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        match self {
            Self::TextInput(control) => control.handle_key(key),
            Self::NumberInput(control) => control.handle_key(key),
            Self::Select(control) => control.handle_key(key),
            Self::Toggle(control) => control.handle_key(key),
            Self::ActionButton(control) => control.handle_key(key),
            Self::StaticData(_) | Self::DynamicData(_) | Self::LogOutput(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputControl {
    pub value: String,
    pub placeholder: String,
}

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

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    ) {
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
        let display = if active {
            format!("{display}▏")
        } else {
            display.to_string()
        };
        let block = framed_block(selected, active, feedback);
        let widget = Paragraph::new(display)
            .block(block)
            .alignment(Alignment::Left)
            .style(style);
        Widget::render(widget, area, buf);
        render_feedback_marker(buf, area, feedback);
    }
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

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    ) {
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
        let block = framed_block(selected, active, feedback);
        let widget = Paragraph::new(display)
            .block(block)
            .alignment(Alignment::Left)
            .style(style);
        Widget::render(widget, area, buf);
        render_feedback_marker(buf, area, feedback);
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

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    ) {
        let area = left_aligned_control_rect(area, 22);
        if area.width <= 2 || area.height == 0 {
            return;
        }

        if active && area.height >= 3 {
            self.render_expanded(area, buf, selected, feedback);
        } else {
            self.render_collapsed(area, buf, selected, active, feedback);
        }
    }

    fn render_collapsed(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    ) {
        let current = self
            .options
            .get(self.selected)
            .map(|s| s.as_str())
            .unwrap_or("-");
        let block = framed_block(selected, active, feedback);
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
        render_feedback_marker(buf, area, feedback);
    }

    fn render_expanded(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        feedback: ControlFeedback,
    ) {
        let border = feedback_accent(feedback, selected, true);
        let base_style = Style::default().fg(Color::Gray);
        let active_style = Style::default().fg(border);

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
        render_feedback_marker(buf, area, feedback);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleControl {
    pub on: bool,
    pub on_label: String,
    pub off_label: String,
}

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

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    ) {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDisplayControl {
    pub value: String,
}

impl DataDisplayControl {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, dynamic: bool, selected: bool, active: bool) {
        let area = left_aligned_control_rect(area, 24);
        if area.width <= 2 || area.height == 0 {
            return;
        }

        let border = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(if active {
                Color::Cyan
            } else if selected {
                Color::White
            } else {
                Color::Gray
            }));
        Widget::render(border, area, buf);

        let icon = if dynamic { "◌" } else { "·" };
        let text = truncate_to_chars(&self.value, area.width.saturating_sub(6) as usize);
        let y = area.y + area.height.saturating_sub(1) / 2;
        buf.set_stringn(
            area.x + 1,
            y,
            format!("{icon} {text}"),
            area.width.saturating_sub(2) as usize,
            Style::default().fg(if dynamic { Color::Cyan } else { Color::White }),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogOutputControl {
    pub content: String,
    pub scroll_offset: usize,
}

impl LogOutputControl {
    const MAX_LINES: usize = 24;
    const DEFAULT_VISIBLE_LINES: usize = 12;
    const DEFAULT_WRAP_WIDTH: usize = 34;

    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            scroll_offset: 0,
        }
    }

    pub fn append_entry(&mut self, entry: impl AsRef<str>) {
        let entry = entry.as_ref().trim();
        if entry.is_empty() {
            return;
        }

        let mut lines: Vec<String> = self
            .content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(ToString::to_string)
            .collect();

        lines.extend(entry.lines().map(ToString::to_string));

        if lines.len() > Self::MAX_LINES {
            let drain = lines.len() - Self::MAX_LINES;
            lines.drain(0..drain);
        }

        self.content = lines.join("\n");
        self.scroll_to_bottom();
    }

    pub fn handle_key(&mut self, key: Key) -> bool {
        let max_offset = self.max_scroll_offset();
        match key {
            Key::Up | Key::Char('k') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    true
                } else {
                    false
                }
            }
            Key::Down | Key::Char('j') => {
                if self.scroll_offset < max_offset {
                    self.scroll_offset += 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
    }

    fn max_scroll_offset(&self) -> usize {
        wrap_text_lines(&self.content, Self::DEFAULT_WRAP_WIDTH)
            .len()
            .saturating_sub(Self::DEFAULT_VISIBLE_LINES)
    }

    fn render(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool) {
        let area = left_aligned_control_rect(area, 36);
        if area.width <= 2 || area.height == 0 {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(if active {
                Color::Cyan
            } else if selected {
                Color::White
            } else {
                Color::Gray
            }));
        let inner = block.inner(area);
        Widget::render(block, area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let wrapped_lines = wrap_text_lines(&self.content, inner.width as usize);
        let mut y = inner.y;
        for line in wrapped_lines
            .into_iter()
            .skip(self.scroll_offset)
            .take(inner.height as usize)
        {
            buf.set_stringn(
                inner.x,
                y,
                line,
                inner.width as usize,
                Style::default().fg(Color::White),
            );
            y += 1;
        }
    }
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

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    ) {
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

fn framed_block(selected: bool, active: bool, feedback: ControlFeedback) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(feedback_accent(feedback, selected, active)))
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

fn wrap_text_lines(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return Vec::new();
    }

    let mut wrapped = Vec::new();

    for raw_line in text.lines() {
        if raw_line.is_empty() {
            wrapped.push(String::new());
            continue;
        }

        let mut current = String::new();
        let mut width = 0usize;

        for ch in raw_line.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if ch_width == 0 {
                continue;
            }

            if width + ch_width > max_width && !current.is_empty() {
                wrapped.push(current);
                current = String::new();
                width = 0;
            }

            current.push(ch);
            width += ch_width;

            if width >= max_width {
                wrapped.push(current);
                current = String::new();
                width = 0;
            }
        }

        if current.is_empty() {
            if wrapped.is_empty() || !raw_line.is_empty() {
                continue;
            }
        } else {
            wrapped.push(current);
        }
    }

    if wrapped.is_empty() {
        wrapped.push(String::new());
    }

    wrapped
}

fn feedback_accent(feedback: ControlFeedback, selected: bool, active: bool) -> Color {
    match feedback {
        ControlFeedback::Idle
        | ControlFeedback::Running(_)
        | ControlFeedback::Success
        | ControlFeedback::Failure => {
            if active {
                Color::Cyan
            } else if selected {
                Color::White
            } else {
                Color::Gray
            }
        }
    }
}

fn render_feedback_marker(buf: &mut Buffer, area: Rect, feedback: ControlFeedback) {
    let (symbol, color) = match feedback {
        ControlFeedback::Idle => ("", Color::White),
        ControlFeedback::Running(frame) => {
            const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            (FRAMES[frame % FRAMES.len()], Color::Cyan)
        }
        ControlFeedback::Success => ("✓", Color::Green),
        ControlFeedback::Failure => ("✗", Color::Red),
    };

    if symbol.is_empty() {
        return;
    }

    let x = area.x + area.width.saturating_add(1);
    let y = area
        .y
        .saturating_add(1)
        .min(area.y + area.height.saturating_sub(1));
    buf.set_string(x, y, symbol, Style::default().fg(color));
}

#[cfg(test)]
mod tests {
    use super::{
        wrap_text_lines, ControlKind, NumberInputControl, SelectControl, TextInputControl,
        ToggleControl,
    };
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
    fn number_input_accepts_only_digits() {
        let mut control = NumberInputControl::new("", "0");
        assert!(control.handle_key(Key::Char('8')));
        assert!(!control.handle_key(Key::Char('x')));
        assert_eq!(control.value, "8");
    }

    #[test]
    fn log_output_append_moves_scroll_to_bottom() {
        let mut control = super::LogOutputControl::new(
            (0..24)
                .map(|i| format!("line-{i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        assert_eq!(control.scroll_offset, 0);

        control.append_entry("line-24");
        assert_eq!(control.scroll_offset, 12);
    }

    #[test]
    fn wrap_text_lines_splits_long_lines_by_width() {
        let wrapped = wrap_text_lines("abcdefgh", 3);
        assert_eq!(wrapped, vec!["abc", "def", "gh"]);
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
