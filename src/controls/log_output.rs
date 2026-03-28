//! 日志输出控件实现。

use std::fs;
use std::path::{Path, PathBuf};

use crate::event::Key;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Widget},
};

use super::control_trait::ControlTrait;
use super::helpers::{left_aligned_control_rect, wrap_text_lines};
use crate::theme::RenderContext;
use std::any::Any;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogOutputControl {
    pub content: String,
    pub scroll_offset: usize,
    pub file_source: Option<PathBuf>,
    pub tail_lines: Option<usize>,
}

impl LogOutputControl {
    const MAX_LINES: usize = 24;
    const DEFAULT_VISIBLE_LINES: usize = 12;
    const DEFAULT_WRAP_WIDTH: usize = 34;

    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            scroll_offset: 0,
            file_source: None,
            tail_lines: None,
        }
    }

    pub fn from_file(path: impl Into<PathBuf>) -> Self {
        let mut control = Self {
            content: String::new(),
            scroll_offset: 0,
            file_source: Some(path.into()),
            tail_lines: None,
        };
        control.refresh_from_file();
        control
    }

    pub fn set_file_source(&mut self, path: impl Into<PathBuf>) {
        self.file_source = Some(path.into());
        self.refresh_from_file();
    }

    pub fn file_source(&self) -> Option<&Path> {
        self.file_source.as_deref()
    }

    pub fn set_tail_lines(&mut self, tail_lines: usize) {
        self.tail_lines = Some(tail_lines.max(1));
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

        self.content = apply_tail_limit(lines, self.tail_lines).join("\n");
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

    pub fn refresh_from_file(&mut self) {
        let Some(path) = &self.file_source else {
            return;
        };

        match fs::read_to_string(path) {
            Ok(content) => {
                let lines = content.lines().map(ToString::to_string).collect::<Vec<_>>();
                self.content = apply_tail_limit(lines, self.tail_lines).join("\n");
                self.scroll_to_bottom();
            }
            Err(err) => {
                self.content = format!("failed to read log file\n{}\n{}", path.display(), err);
                self.scroll_to_bottom();
            }
        }
    }

    fn max_scroll_offset(&self) -> usize {
        wrap_text_lines(&self.content, Self::DEFAULT_WRAP_WIDTH)
            .len()
            .saturating_sub(Self::DEFAULT_VISIBLE_LINES)
    }
}

fn apply_tail_limit(mut lines: Vec<String>, tail_lines: Option<usize>) -> Vec<String> {
    if let Some(limit) = tail_lines {
        if lines.len() > limit {
            let drain = lines.len() - limit;
            lines.drain(0..drain);
        }
    }
    lines
}

fn log_line_style(line: &str) -> Style {
    let fg = if line.contains(" ERROR ") || line.starts_with("ERROR ") || line.contains("[ERROR]") {
        Color::Red
    } else if line.contains(" WARN ") || line.starts_with("WARN ") || line.contains("[WARN]") {
        Color::Yellow
    } else if line.contains(" INFO ") || line.starts_with("INFO ") || line.contains("[INFO]") {
        Color::Cyan
    } else if line.contains(" DEBUG ") || line.starts_with("DEBUG ") || line.contains("[DEBUG]") {
        Color::DarkGray
    } else {
        Color::White
    };

    Style::default().fg(fg)
}

impl ControlTrait for LogOutputControl {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &RenderContext) {
        let area = left_aligned_control_rect(area, 36);
        if area.width <= 2 || area.height == 0 {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(if ctx.active {
                Color::Cyan
            } else if ctx.selected {
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
                line.as_str(),
                inner.width as usize,
                log_line_style(&line),
            );
            y += 1;
        }
    }

    fn handle_key(&mut self, _key: Key) -> bool {
        false
    }

    fn value(&self) -> String {
        self.content.clone()
    }

    fn preferred_width(&self) -> u16 {
        36
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
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self == o)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::LogOutputControl;
    use std::fs;

    #[test]
    fn log_output_append_moves_scroll_to_bottom() {
        let mut control = LogOutputControl::new(
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
    fn log_output_can_refresh_from_file() {
        let path = std::env::temp_dir().join(format!("tui01-log-{}.log", std::process::id()));
        fs::write(&path, "line-a\nline-b\n").unwrap();

        let mut control = LogOutputControl::from_file(&path);
        assert!(control.content.contains("line-a"));

        fs::write(&path, "line-c\n").unwrap();
        control.refresh_from_file();
        assert!(control.content.contains("line-c"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn log_output_respects_tail_lines_for_file_source() {
        let path = std::env::temp_dir().join(format!("tui01-tail-{}.log", std::process::id()));
        fs::write(&path, "a\nb\nc\nd\n").unwrap();

        let mut control = LogOutputControl::from_file(&path);
        control.set_tail_lines(2);
        control.refresh_from_file();

        assert_eq!(control.content, "c\nd");

        let _ = fs::remove_file(path);
    }
}
