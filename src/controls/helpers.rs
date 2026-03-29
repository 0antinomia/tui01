//! 控件渲染共享工具函数。

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};
use unicode_width::UnicodeWidthChar;

use super::control_trait::ControlFeedback;

pub(super) fn framed_block(
    selected: bool,
    active: bool,
    feedback: ControlFeedback,
) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(feedback_accent(feedback, selected, active)))
}

pub(super) fn left_aligned_control_rect(area: Rect, desired_width: u16) -> Rect {
    let width = desired_width.min(area.width.max(1));
    let x = area.x;
    Rect::new(x, area.y, width, area.height)
}

pub fn truncate_to_chars(text: &str, max_chars: usize) -> String {
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

pub fn wrap_text_lines(text: &str, max_width: usize) -> Vec<String> {
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

pub(super) fn feedback_accent(feedback: ControlFeedback, selected: bool, active: bool) -> Color {
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
        },
    }
}

pub(super) fn render_feedback_marker(buf: &mut Buffer, area: Rect, feedback: ControlFeedback) {
    let (symbol, color) = match feedback {
        ControlFeedback::Idle => ("", Color::White),
        ControlFeedback::Running(frame) => {
            const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            (FRAMES[frame % FRAMES.len()], Color::Cyan)
        },
        ControlFeedback::Success => ("✓", Color::Green),
        ControlFeedback::Failure => ("✗", Color::Red),
    };

    if symbol.is_empty() {
        return;
    }

    let x = area.x + area.width.saturating_add(1);
    let y = area.y.saturating_add(1).min(area.y + area.height.saturating_sub(1));
    buf.set_string(x, y, symbol, Style::default().fg(color));
}

#[cfg(test)]
mod tests {
    use super::wrap_text_lines;

    #[test]
    fn wrap_text_lines_splits_long_lines_by_width() {
        let wrapped = wrap_text_lines("abcdefgh", 3);
        assert_eq!(wrapped, vec!["abc", "def", "gh"]);
    }
}
