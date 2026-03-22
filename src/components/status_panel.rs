//! 右上状态区域组件

use crate::components::Component;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub struct StatusPanel {
    text: String,
    result: StatusResult,
}

#[derive(Default)]
struct StatusResult {
    title: String,
    body: String,
    color: Color,
}

impl StatusPanel {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            result: StatusResult {
                title: "Operation".to_string(),
                body: "暂无结果".to_string(),
                color: Color::DarkGray,
            },
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn set_running_result(&mut self, body: impl Into<String>) {
        self.result = StatusResult {
            title: "Operation".to_string(),
            body: body.into(),
            color: Color::Cyan,
        };
    }

    pub fn set_success_result(&mut self, body: impl Into<String>) {
        self.result = StatusResult {
            title: "Success".to_string(),
            body: body.into(),
            color: Color::Green,
        };
    }

    pub fn set_failure_result(&mut self, body: impl Into<String>) {
        self.result = StatusResult {
            title: "Failure".to_string(),
            body: body.into(),
            color: Color::Red,
        };
    }

    pub fn clear_result(&mut self) {
        self.result = StatusResult {
            title: "Operation".to_string(),
            body: "暂无结果".to_string(),
            color: Color::DarkGray,
        };
    }
}

impl Component for StatusPanel {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if rect.height == 0 || rect.width == 0 {
            return;
        }

        let result_height = rect.height.min(5);
        let top_height = rect.height.saturating_sub(result_height);
        let chunks = Layout::vertical([
            Constraint::Length(top_height),
            Constraint::Length(result_height),
        ])
        .split(rect);

        let widget = Paragraph::new(self.text.as_str())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        f.render_widget(widget, chunks[0]);

        let result = format!("{}\n{}", self.result.title, self.result.body);
        let result_widget = Paragraph::new(result)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(self.result.color));
        f.render_widget(result_widget, chunks[1]);
    }
}
