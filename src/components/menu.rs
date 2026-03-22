//! Menu component for navigable option list
//!
//! Provides a static single-select menu with arrow/vim navigation
//! and `>` prefix marker for selected item.

use crate::action::Action;
use crate::components::Component;
use crate::event::{Event, Key};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

/// A single menu item with label and optional component association ID
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Display label for the menu item
    pub label: String,
    /// Optional ID for component association at App level
    pub id: Option<String>,
}

impl MenuItem {
    /// Create a new MenuItem with the given label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: None,
        }
    }

    /// Set an ID for component association
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Menu state for managing selection and pagination
#[derive(Debug, Clone)]
pub struct MenuState {
    items: Vec<MenuItem>,
    selected: usize,
    current_page: usize,
    items_per_page: usize,
}

impl MenuState {
    /// Create a new MenuState with the given items
    ///
    /// First item is selected by default
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            selected: 0,
            current_page: 0,
            items_per_page: 10, // Default, recalculated on render
        }
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Get the currently selected item, if any
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected)
    }

    /// Get the current page (0-indexed)
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Get items per page
    pub fn items_per_page(&self) -> usize {
        self.items_per_page
    }

    /// Calculate total pages based on items_per_page
    pub fn total_pages(&self) -> usize {
        if self.items_per_page == 0 || self.items.is_empty() {
            return 1;
        }
        (self.items.len().saturating_sub(1) / self.items_per_page) + 1
    }

    /// Get labels for current page as strings
    pub fn current_page_labels(&self) -> Vec<String> {
        if self.items.is_empty() {
            return Vec::new();
        }
        let start = self.current_page * self.items_per_page;
        let end = std::cmp::min(start + self.items_per_page, self.items.len());
        self.items[start..end].iter().map(|item| item.label.clone()).collect()
    }

    /// Go to previous page (per D-11)
    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.clamp_selection_to_page();
        }
    }

    /// Go to next page (per D-11)
    pub fn next_page(&mut self) {
        if self.current_page < self.total_pages().saturating_sub(1) {
            self.current_page += 1;
            self.clamp_selection_to_page();
        }
    }

    /// Update items per page based on available height (per D-10)
    pub fn update_items_per_page(&mut self, available_height: usize) {
        // Reserve 1 line for the pagination bar at the top and 1 blank spacer line below it.
        // Each item uses 2 terminal rows to create breathing room.
        self.items_per_page = (available_height.saturating_sub(2) / 2).max(1);
    }

    /// Ensure current page contains the selected item
    fn update_page_for_selection(&mut self) {
        if self.items_per_page == 0 {
            return;
        }
        self.current_page = self.selected / self.items_per_page;
    }

    /// Clamp selection to be within current page
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

    /// Select the previous item (up)
    ///
    /// Does nothing if already at first item
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.update_page_for_selection();
        }
    }

    /// Select the next item (down)
    ///
    /// Does nothing if already at last item
    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            self.update_page_for_selection();
        }
    }

    /// Get the number of items
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if menu is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Generate a progress bar glyph sequence for pagination.
    ///
    /// Current-page segments are brighter and thicker. Inactive segments stay visible
    /// but are slightly dimmed so the active page reads more clearly.
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

/// Menu component implementing the Component trait
///
/// Manages focus state and produces MenuSelect action when Enter pressed.
pub struct MenuComponent {
    state: MenuState,
    focused: bool,
}

impl MenuComponent {
    /// Create a new MenuComponent with the given items
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            state: MenuState::new(items),
            focused: false,
        }
    }

    /// Get the current selection index
    pub fn selected_index(&self) -> usize {
        self.state.selected_index()
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.state.selected_item()
    }

    /// Get a reference to the menu state
    pub fn state(&self) -> &MenuState {
        &self.state
    }

    /// Get a mutable reference to the menu state
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
        // Only respond to keys when focused
        if !self.focused {
            return Action::Noop;
        }

        if let Some(Event::Key(key)) = event {
            match key {
                Key::Char('K') => {
                    self.state.previous_page();
                    return Action::Noop;
                }
                Key::Char('J') => {
                    self.state.next_page();
                    return Action::Noop;
                }
                Key::Up | Key::Char('k') => {
                    self.state.select_previous();
                    return Action::Noop;
                }
                Key::Down | Key::Char('j') => {
                    self.state.select_next();
                    return Action::Noop;
                }
                Key::Enter => {
                    return Action::MenuSelect(self.state.selected_index());
                }
                _ => {}
            }
        }

        Action::Noop
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        // Update items per page based on available height (per D-10)
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

        let list_rect = Rect::new(
            rect.x,
            rect.y.saturating_add(2),
            rect.width,
            rect.height.saturating_sub(2),
        );

        let page_start = self.state.current_page * self.state.items_per_page;
        let local_selected = self.state.selected.saturating_sub(page_start);
        let labels = self.state.current_page_labels();

        for (idx, label) in labels.iter().enumerate() {
            let y = list_rect.y + (idx as u16 * 2);
            if y >= list_rect.y + list_rect.height {
                break;
            }

            let style = self.item_style(idx, local_selected);

            let para = Paragraph::new(label.as_str())
                .alignment(Alignment::Center)
                .style(style);
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
        (0..count)
            .map(|idx| MenuItem::new(format!("Item {idx}")))
            .collect()
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
        assert!(bar
            .iter()
            .any(|(ch, color)| *ch == '─' && *color == Color::Rgb(170, 170, 170)));
    }
}
