//! 右上状态区域组件

use crate::components::Component;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
};

pub struct StatusPanel {
    text: String,
}

impl StatusPanel {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl Component for StatusPanel {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if rect.height == 0 || rect.width == 0 {
            return;
        }

        let widget = Paragraph::new(self.text.as_str())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        f.render_widget(widget, rect);
    }
}
