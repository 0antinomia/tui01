//! Theme、RenderContext 和 LayoutStrategy 定义。

use crate::controls::ControlFeedback;
use ratatui::{layout::Rect, style::Color};
use serde::{Deserialize, Serialize};

/// 主题配置，包含 6 个语义化颜色槽位（per D-01, D-02）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Theme {
    pub border: Color,
    pub text: Color,
    pub selected: Color,
    pub active: Color,
    pub error: Color,
    pub success: Color,
}

impl Default for Theme {
    /// 默认主题，匹配当前硬编码颜色值（per D-05）。
    fn default() -> Self {
        Self {
            border: Color::Blue,
            text: Color::White,
            selected: Color::White,
            active: Color::Cyan,
            error: Color::Yellow,
            success: Color::Green,
        }
    }
}

/// 控件渲染上下文，承载主题和交互状态（per D-06, D-07, D-08）。
#[derive(Debug, Clone, Copy)]
pub struct RenderContext {
    pub theme: Theme,
    pub selected: bool,
    pub active: bool,
    pub feedback: ControlFeedback,
}

/// 四象限布局区域（per D-11）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutAreas {
    pub title: Rect,
    pub status: Rect,
    pub menu: Rect,
    pub content: Rect,
}

/// 布局策略 trait，定义区域计算接口（per D-10）。
pub trait LayoutStrategy {
    fn areas(&self, total: Rect) -> LayoutAreas;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_default_matches_hardcoded_colors() {
        let theme = Theme::default();
        assert_eq!(theme.border, Color::Blue);
        assert_eq!(theme.text, Color::White);
        assert_eq!(theme.selected, Color::White);
        assert_eq!(theme.active, Color::Cyan);
        assert_eq!(theme.error, Color::Yellow);
        assert_eq!(theme.success, Color::Green);
    }

    #[test]
    fn theme_serde_round_trip() {
        let original = Theme::default();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(original.border, restored.border);
        assert_eq!(original.text, restored.text);
        assert_eq!(original.selected, restored.selected);
        assert_eq!(original.active, restored.active);
        assert_eq!(original.error, restored.error);
        assert_eq!(original.success, restored.success);
    }

    #[test]
    fn render_context_is_copy() {
        let ctx = RenderContext {
            theme: Theme::default(),
            selected: true,
            active: false,
            feedback: ControlFeedback::Idle,
        };
        let copy = ctx;
        assert_eq!(copy.theme.border, ctx.theme.border);
        assert_eq!(copy.selected, ctx.selected);
        assert_eq!(copy.active, ctx.active);
        assert_eq!(copy.feedback, ctx.feedback);
    }

    #[test]
    fn layout_areas_fields() {
        let areas = LayoutAreas {
            title: Rect::new(0, 0, 10, 5),
            status: Rect::new(10, 0, 70, 5),
            menu: Rect::new(0, 5, 10, 19),
            content: Rect::new(10, 5, 70, 19),
        };
        assert_eq!(areas.title, Rect::new(0, 0, 10, 5));
        assert_eq!(areas.status, Rect::new(10, 0, 70, 5));
        assert_eq!(areas.menu, Rect::new(0, 5, 10, 19));
        assert_eq!(areas.content, Rect::new(10, 5, 70, 19));
    }
}
