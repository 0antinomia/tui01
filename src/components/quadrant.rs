//! 四区"田"字形布局组件

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

use super::Component;
use crate::theme::{LayoutAreas, LayoutStrategy};
use ratatui::widgets::BorderType;

/// 四区布局分割比例配置
///
/// 默认为 20/80 非对称分割：
/// - 顶部行占 20% 高度，底部行占 80%
/// - 左侧列占 20% 宽度，右侧列占 80%
/// - 结果：左上 4%，右上 16%，左下 16%，右下 64%
#[derive(Debug, Clone, Copy)]
pub struct QuadrantConfig {
    /// 左侧列的水平分割百分比（默认: 20）
    pub horizontal_split: u16,
    /// 顶部行的垂直分割百分比（默认: 20）
    pub vertical_split: u16,
}

impl Default for QuadrantConfig {
    fn default() -> Self {
        Self { horizontal_split: 20, vertical_split: 20 }
    }
}

/// 四区布局组件
///
/// 渲染"田"字形布局，四个区域都有边框
pub struct QuadrantLayout {
    config: QuadrantConfig,
}

impl QuadrantLayout {
    /// 使用指定配置创建 QuadrantLayout
    pub fn new(config: QuadrantConfig) -> Self {
        Self { config }
    }

    /// 根据实际绘制出的边框与分割线，计算四个象限的内容区域
    pub fn calculate_quadrants(&self, area: Rect) -> (Rect, Rect, Rect, Rect) {
        let actual_height = area.height.min(area.width);
        let layout_rect = Rect::new(area.x, area.y, area.width, actual_height);

        let inner_x = layout_rect.x + 1;
        let inner_y = layout_rect.y + 1;
        let inner_width = layout_rect.width.saturating_sub(2);
        let inner_height = layout_rect.height.saturating_sub(2);

        if inner_width == 0 || inner_height == 0 {
            let zero = Rect::new(inner_x, inner_y, 0, 0);
            return (zero, zero, zero, zero);
        }

        let vertical_divider_x = inner_x + (inner_width * self.config.horizontal_split / 100);
        let horizontal_divider_y = inner_y + (inner_height * self.config.vertical_split / 100);
        let inner_right = inner_x + inner_width;
        let inner_bottom = inner_y + inner_height;

        let left_width = vertical_divider_x.saturating_sub(inner_x);
        let right_x = vertical_divider_x.saturating_add(1);
        let right_width = inner_right.saturating_sub(right_x);
        let top_height = horizontal_divider_y.saturating_sub(inner_y);
        let bottom_y = horizontal_divider_y.saturating_add(1);
        let bottom_height = inner_bottom.saturating_sub(bottom_y);

        (
            Rect::new(inner_x, inner_y, left_width, top_height),
            Rect::new(right_x, inner_y, right_width, top_height),
            Rect::new(inner_x, bottom_y, left_width, bottom_height),
            Rect::new(right_x, bottom_y, right_width, bottom_height),
        )
    }
}

impl LayoutStrategy for QuadrantLayout {
    fn areas(&self, total: Rect) -> LayoutAreas {
        let (title, status, menu, content) = self.calculate_quadrants(total);
        LayoutAreas { title, status, menu, content }
    }
}

impl Component for QuadrantLayout {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        // 全宽，高度不超过宽度，顶部对齐
        let actual_height = rect.height.min(rect.width);

        let layout_rect = Rect::new(rect.x, rect.y, rect.width, actual_height);

        // 绘制外边框（圆角）
        let outer_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Blue));
        f.render_widget(outer_block, layout_rect);

        // 计算分割线位置
        // 垂直分割线在 horizontal_split 百分比处
        // 考虑外边框（每边 1 个字符）
        let inner_x = layout_rect.x + 1;
        let inner_y = layout_rect.y + 1;
        let inner_width = layout_rect.width.saturating_sub(2);
        let inner_height = layout_rect.height.saturating_sub(2);

        if inner_width == 0 || inner_height == 0 {
            return;
        }

        let vertical_divider_x = inner_x + (inner_width * self.config.horizontal_split / 100);
        let horizontal_divider_y = inner_y + (inner_height * self.config.vertical_split / 100);

        // 绘制垂直分割线（│）
        for y in inner_y..inner_y + inner_height {
            f.buffer_mut()[(vertical_divider_x, y)].set_char('│').set_fg(Color::Blue);
        }

        // 绘制水平分割线（─）
        for x in inner_x..inner_x + inner_width {
            f.buffer_mut()[(x, horizontal_divider_y)].set_char('─').set_fg(Color::Blue);
        }

        // 绘制中心交叉点（┼）
        f.buffer_mut()[(vertical_divider_x, horizontal_divider_y)]
            .set_char('┼')
            .set_fg(Color::Blue);

        // 修复与外边框的交叉点
        // 顶部交叉点（┬）
        if inner_y > layout_rect.y {
            f.buffer_mut()[(vertical_divider_x, inner_y - 1)].set_char('┬').set_fg(Color::Blue);
        }
        // 底部交叉点（┴）
        if inner_y + inner_height < layout_rect.y + layout_rect.height {
            f.buffer_mut()[(vertical_divider_x, inner_y + inner_height)]
                .set_char('┴')
                .set_fg(Color::Blue);
        }
        // 左侧交叉点（├）
        if inner_x > layout_rect.x {
            f.buffer_mut()[(inner_x - 1, horizontal_divider_y)].set_char('├').set_fg(Color::Blue);
        }
        // 右侧交叉点（┤）
        if inner_x + inner_width < layout_rect.x + layout_rect.width {
            f.buffer_mut()[(inner_x + inner_width, horizontal_divider_y)]
                .set_char('┤')
                .set_fg(Color::Blue);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{QuadrantConfig, QuadrantLayout};
    use crate::theme::LayoutStrategy;
    use ratatui::layout::Rect;

    #[test]
    fn calculate_quadrants_align_with_rendered_boundaries() {
        let layout =
            QuadrantLayout::new(QuadrantConfig { horizontal_split: 25, vertical_split: 40 });

        let (top_left, top_right, bottom_left, bottom_right) =
            layout.calculate_quadrants(Rect::new(0, 0, 100, 50));

        assert_eq!(top_left, Rect::new(1, 1, 24, 19));
        assert_eq!(top_right, Rect::new(26, 1, 73, 19));
        assert_eq!(bottom_left, Rect::new(1, 21, 24, 28));
        assert_eq!(bottom_right, Rect::new(26, 21, 73, 28));
    }

    #[test]
    fn default_config_matches_documented_ratios() {
        let config = QuadrantConfig::default();

        assert_eq!(config.horizontal_split, 20);
        assert_eq!(config.vertical_split, 20);
    }

    #[test]
    fn layout_strategy_returns_same_areas_as_calculate_quadrants() {
        let layout = QuadrantLayout::new(QuadrantConfig::default());
        let area = Rect::new(0, 0, 80, 24);
        let (tl, tr, bl, br) = layout.calculate_quadrants(area);
        let areas = LayoutStrategy::areas(&layout, area);
        assert_eq!(areas.title, tl);
        assert_eq!(areas.status, tr);
        assert_eq!(areas.menu, bl);
        assert_eq!(areas.content, br);
    }
}
