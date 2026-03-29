//! 左下菜单组件。
//!
//! 提供固定的单选菜单，支持方向键和 vim 风格导航。

use crate::action::Action;
use crate::components::Component;
use crate::event::{Event, Key};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};

/// 单个菜单项，只包含展示标签。
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// 菜单显示标签。
    pub label: String,
}

impl MenuItem {
    /// 使用给定标签创建菜单项。
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into() }
    }
}

/// 菜单状态，负责选择和分页。
#[derive(Debug, Clone)]
pub struct MenuState {
    items: Vec<MenuItem>,
    selected: usize,
    current_page: usize,
    items_per_page: usize,
}

impl MenuState {
    /// 使用给定菜单项创建状态。
    ///
    /// 默认选中第一项。
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            selected: 0,
            current_page: 0,
            items_per_page: 10, // 默认值，实际会在渲染时重算
        }
    }

    /// 获取当前选中项索引。
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// 获取当前选中项。
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected)
    }

    /// 获取当前页码，从 0 开始。
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// 获取每页项数。
    pub fn items_per_page(&self) -> usize {
        self.items_per_page
    }

    /// 根据每页项数计算总页数。
    pub fn total_pages(&self) -> usize {
        if self.items_per_page == 0 || self.items.is_empty() {
            return 1;
        }
        (self.items.len().saturating_sub(1) / self.items_per_page) + 1
    }

    /// 获取当前页所有标签文本。
    pub fn current_page_labels(&self) -> Vec<String> {
        if self.items.is_empty() {
            return Vec::new();
        }
        let start = self.current_page * self.items_per_page;
        let end = std::cmp::min(start + self.items_per_page, self.items.len());
        self.items[start..end].iter().map(|item| item.label.clone()).collect()
    }

    /// 切到上一页。
    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.clamp_selection_to_page();
        }
    }

    /// 切到下一页。
    pub fn next_page(&mut self) {
        if self.current_page < self.total_pages().saturating_sub(1) {
            self.current_page += 1;
            self.clamp_selection_to_page();
        }
    }

    /// 根据可用高度更新每页项数。
    pub fn update_items_per_page(&mut self, available_height: usize) {
        // 顶部预留 1 行分页线，下方再留 1 行空白。
        // 每个菜单项占 2 行，保证有足够留白。
        self.items_per_page = (available_height.saturating_sub(2) / 2).max(1);
    }

    /// 保证当前页覆盖选中项。
    fn update_page_for_selection(&mut self) {
        if self.items_per_page == 0 {
            return;
        }
        self.current_page = self.selected / self.items_per_page;
    }

    /// 将选中项限制在当前页范围内。
    fn clamp_selection_to_page(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let page_start = self.current_page * self.items_per_page;
        let page_end = std::cmp::min(page_start + self.items_per_page, self.items.len());
        if self.selected < page_start || self.selected >= page_end {
            self.selected = page_start;
        }
    }

    /// 选择上一项。
    ///
    /// 如果已经在第一项，则保持不变。
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.update_page_for_selection();
        }
    }

    /// 选择下一项。
    ///
    /// 如果已经在最后一项，则保持不变。
    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            self.update_page_for_selection();
        }
    }

    /// 获取菜单项数量。
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 判断菜单是否为空。
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 生成分页线字符序列。
    ///
    /// 当前页使用更亮的线段，其余页保持可见但稍微压暗。
    pub fn pagination_bar(&self, width: usize) -> Vec<(char, Color)> {
        if width == 0 {
            return Vec::new();
        }

        let total_pages = self.total_pages();
        let mut bar = Vec::with_capacity(width);

        for idx in 0..width {
            let page = idx * total_pages / width;
            let (ch, color) = if page == self.current_page.min(total_pages.saturating_sub(1)) {
                ('━', Color::White)
            } else {
                ('─', Color::Rgb(170, 170, 170))
            };
            bar.push((ch, color));
        }

        bar
    }
}

/// 菜单组件。
///
/// 负责焦点状态管理，并在按下回车时产生 `MenuSelect` 动作。
pub struct MenuComponent {
    state: MenuState,
    focused: bool,
}

impl MenuComponent {
    /// 使用给定菜单项创建菜单组件。
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self { state: MenuState::new(items), focused: false }
    }

    /// 获取当前选中索引。
    pub fn selected_index(&self) -> usize {
        self.state.selected_index()
    }

    /// 获取当前选中项。
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.state.selected_item()
    }

    /// 获取菜单状态的只读引用。
    pub fn state(&self) -> &MenuState {
        &self.state
    }

    /// 获取菜单状态的可变引用。
    pub fn state_mut(&mut self) -> &mut MenuState {
        &mut self.state
    }

    fn item_style(&self, idx: usize, local_selected: usize) -> Style {
        if idx == local_selected {
            Style::default().fg(Color::White).bg(Color::Rgb(52, 86, 112))
        } else if self.focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }
}

impl Component for MenuComponent {
    fn can_focus(&self) -> bool {
        true
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn blur(&mut self) {
        self.focused = false;
    }

    fn handle_events(&mut self, event: Option<Event>) -> Action {
        // 只有获得焦点时才响应按键。
        if !self.focused {
            return Action::Noop;
        }

        if let Some(Event::Key(key)) = event {
            match key {
                Key::Char('K') => {
                    self.state.previous_page();
                    return Action::Noop;
                },
                Key::Char('J') => {
                    self.state.next_page();
                    return Action::Noop;
                },
                Key::Up | Key::Char('k') => {
                    self.state.select_previous();
                    return Action::Noop;
                },
                Key::Down | Key::Char('j') => {
                    self.state.select_next();
                    return Action::Noop;
                },
                Key::Enter => {
                    return Action::MenuSelect(self.state.selected_index());
                },
                _ => {},
            }
        }

        Action::Noop
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        // 根据当前可用高度重算每页容量。
        self.state.update_items_per_page(rect.height as usize);

        let bar = self.state.pagination_bar(rect.width.saturating_sub(4) as usize);
        let left_padding = rect.width.saturating_sub(bar.len() as u16) / 2;
        for (idx, (ch, color)) in bar.into_iter().enumerate() {
            let x = rect.x + left_padding + idx as u16;
            if x >= rect.x + rect.width {
                break;
            }
            f.buffer_mut()[(x, rect.y)].set_char(ch).set_fg(color);
        }

        let list_rect =
            Rect::new(rect.x, rect.y.saturating_add(2), rect.width, rect.height.saturating_sub(2));

        let page_start = self.state.current_page * self.state.items_per_page;
        let local_selected = self.state.selected.saturating_sub(page_start);
        let labels = self.state.current_page_labels();

        for (idx, label) in labels.iter().enumerate() {
            let y = list_rect.y + (idx as u16 * 2);
            if y >= list_rect.y + list_rect.height {
                break;
            }

            let style = self.item_style(idx, local_selected);

            let para = Paragraph::new(label.as_str()).alignment(Alignment::Center).style(style);
            f.render_widget(para, Rect::new(list_rect.x, y, list_rect.width, 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MenuComponent, MenuItem, MenuState};
    use crate::action::Action;
    use crate::components::Component;
    use crate::event::{Event, Key};
    use ratatui::style::Color;

    fn make_items(count: usize) -> Vec<MenuItem> {
        (0..count).map(|idx| MenuItem::new(format!("Item {idx}"))).collect()
    }

    #[test]
    fn menu_state_updates_page_when_selection_moves_past_page_boundary() {
        let mut state = MenuState::new(make_items(6));
        state.update_items_per_page(5);

        for _ in 0..3 {
            state.select_next();
        }

        assert_eq!(state.selected_index(), 3);
        assert_eq!(state.current_page(), 3);
    }

    #[test]
    fn menu_state_previous_page_clamps_selection_into_new_page() {
        let mut state = MenuState::new(make_items(8));
        state.update_items_per_page(5);

        for _ in 0..6 {
            state.select_next();
        }
        assert_eq!(state.selected_index(), 6);
        assert_eq!(state.current_page(), 6);

        state.previous_page();

        assert_eq!(state.current_page(), 5);
        assert_eq!(state.selected_index(), 5);
    }

    #[test]
    fn focused_menu_component_emits_menu_select_on_enter() {
        let mut menu = MenuComponent::new(make_items(3));
        menu.focus();
        menu.handle_events(Some(Event::Key(Key::Down)));

        let action = menu.handle_events(Some(Event::Key(Key::Enter)));

        assert_eq!(action, Action::MenuSelect(1));
    }

    #[test]
    fn unfocused_menu_component_ignores_navigation() {
        let mut menu = MenuComponent::new(make_items(3));

        let action = menu.handle_events(Some(Event::Key(Key::Down)));

        assert_eq!(action, Action::Noop);
        assert_eq!(menu.selected_index(), 0);
    }

    #[test]
    fn unfocused_menu_dims_non_selected_items() {
        let menu = MenuComponent::new(make_items(3));

        let selected = menu.item_style(0, 0);
        let other = menu.item_style(1, 0);

        assert_eq!(selected.fg, Some(Color::White));
        assert_eq!(other.fg, Some(Color::DarkGray));
    }

    #[test]
    fn focused_menu_component_uses_uppercase_jk_for_pagination() {
        let mut menu = MenuComponent::new(make_items(8));
        menu.focus();
        menu.state_mut().update_items_per_page(5);

        menu.handle_events(Some(Event::Key(Key::Char('J'))));
        assert_eq!(menu.state().current_page(), 1);

        menu.handle_events(Some(Event::Key(Key::Char('K'))));
        assert_eq!(menu.state().current_page(), 0);
    }

    #[test]
    fn pagination_bar_is_visible_for_single_page() {
        let state = MenuState::new(make_items(3));
        let bar = state.pagination_bar(8);

        assert_eq!(bar.len(), 8);
        assert!(bar.iter().all(|(ch, color)| *ch == '━' && *color == Color::White));
    }

    #[test]
    fn pagination_bar_dims_inactive_segments() {
        let mut state = MenuState::new(make_items(12));
        state.update_items_per_page(6);
        state.next_page();

        let bar = state.pagination_bar(8);

        assert!(bar.iter().any(|(ch, color)| *ch == '━' && *color == Color::White));
        assert!(bar.iter().any(|(ch, color)| *ch == '─' && *color == Color::Rgb(170, 170, 170)));
    }
}
