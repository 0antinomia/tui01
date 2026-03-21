//! 左上标题区域组件

use crate::components::Component;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub struct TitlePanel {
    text: String,
}

impl TitlePanel {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl Component for TitlePanel {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let widget = Paragraph::new(self.text.as_str())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(widget, rect);
    }
}
