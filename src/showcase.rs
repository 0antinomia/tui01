//! 通用的四分区展示应用壳层。

use crate::action::Action;
use crate::components::{
    Component, ContentBlueprint, ContentPanel, MenuComponent, MenuItem, QuadrantConfig,
    QuadrantLayout, StatusPanel, TitlePanel,
};
use crate::event::{Event, Key};
use crate::tui::{self, MAX_ASPECT_RATIO, MIN_ASPECT_RATIO, MIN_HEIGHT, MIN_WIDTH};
use ratatui::{layout::Rect, style::{Color, Style}, widgets::Paragraph};

pub struct ShowcaseApp {
    pub running: bool,
    size_error: Option<SizeError>,
    quadrant_layout: QuadrantLayout,
    title_panel: TitlePanel,
    status_panel: StatusPanel,
    menu: MenuComponent,
    content_panel: ContentPanel,
    screens: Vec<ShowcaseScreen>,
    active_screen: usize,
    loaded_screen: Option<usize>,
    copy: ShowcaseCopy,
    focus: FocusTarget,
}

enum SizeError {
    TooSmall,
    TooNarrow,
    TooWide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusTarget {
    Menu,
    Content,
}

pub struct ShowcaseScreen {
    pub title: &'static str,
    pub content: ContentBlueprint,
}

pub struct ShowcaseCopy {
    pub title_text: &'static str,
    pub status_controls: &'static str,
}

impl ShowcaseApp {
    pub fn new(copy: ShowcaseCopy, screens: Vec<ShowcaseScreen>) -> Self {
        let items = screens.iter().map(|screen| MenuItem::new(screen.title)).collect();
        let mut menu = MenuComponent::new(items);
        menu.focus();

        let mut app = Self {
            running: true,
            size_error: None,
            quadrant_layout: QuadrantLayout::new(QuadrantConfig::default()),
            title_panel: TitlePanel::new(copy.title_text),
            status_panel: StatusPanel::new(""),
            menu,
            content_panel: ContentPanel::new(),
            screens,
            active_screen: 0,
            loaded_screen: None,
            copy,
            focus: FocusTarget::Menu,
        };
        app.content_panel.blur();
        app.sync_panels();
        app
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Resize(w, h) => self.apply_action(Action::Resize(w, h)),
            Event::Quit => self.apply_action(Action::Quit),
            Event::Key(key) => self.handle_key(key),
        }
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        if let Some(ref error) = self.size_error {
            self.render_size_error(frame, error);
            return;
        }

        let area = frame.area();
        self.quadrant_layout.render(frame, area);
        self.sync_panels();

        let (top_left, top_right, bottom_left, bottom_right) =
            self.quadrant_layout.calculate_quadrants(area);

        self.title_panel.render(frame, top_left);
        self.status_panel.render(frame, top_right);
        self.menu.render(frame, bottom_left);
        self.content_panel.render(frame, bottom_right);
    }

    pub fn active_screen(&self) -> usize {
        self.active_screen
    }

    pub fn selected_index(&self) -> usize {
        self.menu.selected_index()
    }

    pub fn has_size_error(&self) -> bool {
        self.size_error.is_some()
    }

    pub fn content_page(&self) -> usize {
        self.content_panel.current_page()
    }

    pub fn content_selected_block(&self) -> usize {
        self.content_panel.selected_block()
    }

    fn handle_key(&mut self, key: Key) {
        if key == Key::Char('q') {
            self.apply_action(Action::Quit);
            return;
        }

        match self.focus {
            FocusTarget::Menu => self.handle_menu_key(key),
            FocusTarget::Content => self.handle_content_key(key),
        }
    }

    fn handle_menu_key(&mut self, key: Key) {
        match key {
            Key::Enter | Key::Char('l') => {
                self.sync_active_to_menu_selection();
                self.focus_content();
            }
            Key::Char('K') => {
                self.content_panel.previous_page_with_height(self.current_content_rect().height);
                let action = self.menu.handle_events(Some(Event::Key(key)));
                self.apply_action(action);
                self.sync_active_to_menu_selection();
            }
            Key::Char('J') => {
                let content_rect = self.current_content_rect();
                self.content_panel
                    .next_page(content_rect.width, content_rect.height);
                let action = self.menu.handle_events(Some(Event::Key(key)));
                self.apply_action(action);
                self.sync_active_to_menu_selection();
            }
            _ => {
                let action = self.menu.handle_events(Some(Event::Key(key)));
                self.apply_action(action);
                self.sync_active_to_menu_selection();
            }
        }
    }

    fn handle_content_key(&mut self, key: Key) {
        let content_rect = self.current_content_rect();
        if self.content_panel.is_control_active() {
            match key {
                Key::Char('h') => {
                    self.content_panel.cancel_control();
                    self.focus_menu();
                }
                Key::Esc => self.content_panel.cancel_control(),
                Key::Enter => self.content_panel.confirm_control(),
                Key::Left => {
                    let _ = self.content_panel.handle_control_key(key);
                }
                Key::Right | Key::Char('l') => {
                    let _ = self.content_panel.handle_control_key(key);
                }
                _ => {
                    let _ = self.content_panel.handle_control_key(key);
                }
            }
            return;
        }

        match key {
            Key::Up | Key::Char('k') => {
                self.content_panel.select_previous_block(content_rect.height);
            }
            Key::Down | Key::Char('j') => {
                self.content_panel.select_next_block(content_rect.height);
            }
            Key::Char('K') => {
                self.content_panel.previous_page_with_height(content_rect.height);
            }
            Key::Char('J') => {
                self.content_panel
                    .next_page(content_rect.width, content_rect.height);
            }
            Key::Char('l') | Key::Enter => {
                let _ = self.content_panel.activate_selected_control();
            }
            Key::Char('h') | Key::Esc => {
                self.focus_menu();
            }
            _ => {}
        }
    }

    fn sync_active_to_menu_selection(&mut self) {
        let index = self.menu.selected_index();
        if index < self.screens.len() {
            if index != self.active_screen {
                self.persist_active_screen_content();
            }
            self.active_screen = index;
            self.load_active_screen_content();
        }
        self.sync_panels();
    }

    fn focus_menu(&mut self) {
        self.focus = FocusTarget::Menu;
        self.content_panel.blur();
        self.menu.focus();
    }

    fn focus_content(&mut self) {
        self.focus = FocusTarget::Content;
        self.menu.blur();
        self.content_panel.focus();
        let height = self.current_content_rect().height;
        if self.content_panel.has_selectable_blocks(height) {
            self.content_panel.ensure_visible_selection(height);
        }
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::Resize(w, h) => {
                self.size_error = Self::check_size(w, h);
            }
            Action::MenuSelect(index) => {
                if index < self.screens.len() {
                    if index != self.active_screen {
                        self.persist_active_screen_content();
                    }
                    self.active_screen = index;
                    self.load_active_screen_content();
                }
            }
            Action::Noop => {}
        }

        self.sync_panels();
    }

    fn render_size_error(&self, frame: &mut ratatui::Frame, error: &SizeError) {
        use ratatui::layout::Alignment;

        let error_msg = match error {
            SizeError::TooSmall => {
                format!("终端太小（最小需要 {}x{}）", MIN_WIDTH, MIN_HEIGHT)
            }
            SizeError::TooNarrow => {
                format!(
                    "终端过窄（宽高比需 >= {:.1}，当前: {:.2}）",
                    MIN_ASPECT_RATIO,
                    Self::current_aspect_ratio()
                )
            }
            SizeError::TooWide => {
                format!(
                    "终端过宽（宽高比需 <= {:.1}，当前: {:.2}）",
                    MAX_ASPECT_RATIO,
                    Self::current_aspect_ratio()
                )
            }
        };

        let widget = Paragraph::new(error_msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        frame.render_widget(widget, frame.area());
    }

    fn sync_panels(&mut self) {
        self.title_panel.set_text(self.copy.title_text);

        let selected = self
            .menu
            .selected_item()
            .map(|item| item.label.as_str())
            .unwrap_or("None");

        let active = self
            .screens
            .get(self.active_screen)
            .map(|screen| screen.title)
            .unwrap_or("None");

        let focus_label = match self.focus {
            FocusTarget::Menu => "MenuComponent",
            FocusTarget::Content => "ContentPanel",
        };

        self.status_panel.set_text(format!(
            "Focus: {}\nSelected: {}\nActive: {}\n\n{}",
            focus_label, selected, active, self.copy.status_controls
        ));

        self.load_active_screen_content();
    }

    fn persist_active_screen_content(&mut self) {
        if let Some(screen) = self.screens.get_mut(self.active_screen) {
            screen.content = self.content_panel.blueprint().clone();
        }
    }

    fn load_active_screen_content(&mut self) {
        if self.loaded_screen == Some(self.active_screen) {
            return;
        }

        let Some(screen) = self.screens.get(self.active_screen) else {
            return;
        };

        self.content_panel.set_blueprint(screen.content.clone());
        self.loaded_screen = Some(self.active_screen);
    }

    fn current_content_rect(&self) -> Rect {
        let (width, height) = tui::terminal_size().unwrap_or((MIN_WIDTH, MIN_HEIGHT));
        let area = Rect::new(0, 0, width, height);
        let (_, _, _, bottom_right) = self.quadrant_layout.calculate_quadrants(area);
        bottom_right
    }

    fn check_size(w: u16, h: u16) -> Option<SizeError> {
        if w < MIN_WIDTH || h < MIN_HEIGHT {
            return Some(SizeError::TooSmall);
        }
        let aspect_ratio = w as f64 / h as f64;
        if aspect_ratio < MIN_ASPECT_RATIO {
            return Some(SizeError::TooNarrow);
        }
        if aspect_ratio > MAX_ASPECT_RATIO {
            return Some(SizeError::TooWide);
        }
        None
    }

    fn current_aspect_ratio() -> f64 {
        let (w, h) = tui::terminal_size().unwrap_or((MIN_WIDTH, MIN_HEIGHT));
        w as f64 / h as f64
    }
}

#[cfg(test)]
mod tests {
    use super::{FocusTarget, ShowcaseApp, ShowcaseCopy, ShowcaseScreen};
    use crate::components::{ContentBlock, ContentBlueprint, ContentSection};
    use crate::event::{Event, Key};

    fn screen(title: &'static str, text: &'static str) -> ShowcaseScreen {
        ShowcaseScreen {
            title,
            content: ContentBlueprint::new(title).with_sections(vec![
                ContentSection::new("概览")
                    .with_blocks(vec![ContentBlock::text_input(text, "", "输入值")]),
            ]),
        }
    }

    fn make_app() -> ShowcaseApp {
        ShowcaseApp::new(
            ShowcaseCopy {
                title_text: "Title",
                status_controls: "Controls",
            },
            vec![
                screen("One", "First"),
                screen("Two", "Second"),
            ],
        )
    }

    #[test]
    fn starts_with_first_screen_selected() {
        let app = make_app();
        assert!(app.running);
        assert_eq!(app.active_screen(), 0);
        assert_eq!(app.selected_index(), 0);
    }

    #[test]
    fn enter_on_menu_updates_selection() {
        let mut app = make_app();
        app.handle_event(Event::Key(Key::Down));
        assert_eq!(app.active_screen(), 1);
    }

    #[test]
    fn moving_menu_selection_updates_preview_immediately() {
        let mut app = make_app();
        app.handle_event(Event::Key(Key::Down));

        assert_eq!(app.active_screen(), 1);
    }

    #[test]
    fn l_moves_focus_from_menu_to_content() {
        let mut app = make_app();
        app.handle_event(Event::Key(Key::Char('l')));

        assert_eq!(app.focus, FocusTarget::Content);
    }

    #[test]
    fn content_focus_moves_block_selection_with_jk() {
        let mut app = ShowcaseApp::new(
            ShowcaseCopy {
                title_text: "Title",
                status_controls: "Controls",
            },
            vec![ShowcaseScreen {
                title: "One",
                content: ContentBlueprint::new("One").with_sections(vec![ContentSection::new("概览")
                    .with_blocks(vec![
                        ContentBlock::toggle("A", true),
                        ContentBlock::select("B", ["One", "Two"], 0),
                    ])]),
            }],
        );

        app.handle_event(Event::Key(Key::Char('l')));
        app.handle_event(Event::Key(Key::Char('j')));
        assert_eq!(app.content_selected_block(), 1);

        app.handle_event(Event::Key(Key::Char('k')));
        assert_eq!(app.content_selected_block(), 0);
    }

    #[test]
    fn toggle_changes_persist_while_syncing_panels() {
        let mut app = ShowcaseApp::new(
            ShowcaseCopy {
                title_text: "Title",
                status_controls: "Controls",
            },
            vec![ShowcaseScreen {
                title: "One",
                content: ContentBlueprint::new("One").with_sections(vec![ContentSection::new("概览")
                    .with_blocks(vec![ContentBlock::toggle("A", false)])]),
            }],
        );

        app.handle_event(Event::Key(Key::Char('l')));
        app.handle_event(Event::Key(Key::Enter));
        app.sync_panels();

        match &app.content_panel.blueprint().sections[0].blocks[0].control {
            crate::components::ContentControl::Toggle(control) => {
                assert!(control.on)
            }
            _ => panic!("expected toggle"),
        }
    }

    #[test]
    fn content_focus_returns_to_menu_with_h() {
        let mut app = make_app();
        app.handle_event(Event::Key(Key::Char('l')));
        app.handle_event(Event::Key(Key::Esc));

        assert_eq!(app.focus, FocusTarget::Menu);
    }

    #[test]
    fn q_quits() {
        let mut app = make_app();
        app.handle_event(Event::Key(Key::Char('q')));
        assert!(!app.running);
    }

    #[test]
    fn resize_updates_size_error() {
        let mut app = make_app();
        app.handle_event(Event::Resize(40, 10));
        assert!(app.has_size_error());
        app.handle_event(Event::Resize(100, 40));
        assert!(!app.has_size_error());
    }

    #[test]
    fn uppercase_j_pages_menu_and_content() {
        let mut app = ShowcaseApp::new(
            ShowcaseCopy {
                title_text: "Title",
                status_controls: "Controls",
            },
            vec![
                ShowcaseScreen {
                    title: "One",
                    content: ContentBlueprint::new("One").with_sections(vec![
                        ContentSection::new("Section A").with_blocks(vec![
                            ContentBlock::text_input("First block", "alpha", "输入").with_height_units(2),
                            ContentBlock::select("Second block", ["A", "B", "C"], 0).with_height_units(2),
                            ContentBlock::toggle("Third block", true).with_height_units(2),
                        ]),
                        ContentSection::new("Section B").with_blocks(vec![
                            ContentBlock::text_input("Fourth block", "beta", "输入").with_height_units(2),
                        ]),
                    ]),
                },
                screen("Two", "Second"),
                screen("Three", "Third"),
                screen("Four", "Fourth"),
                screen("Five", "Fifth"),
                screen("Six", "Sixth"),
                screen("Seven", "Seventh"),
                screen("Eight", "Eighth"),
                screen("Nine", "Ninth"),
                screen("Ten", "Tenth"),
                screen("Eleven", "Eleventh"),
                screen("Twelve", "Twelfth"),
            ],
        );

        app.handle_event(Event::Resize(80, 24));
        app.handle_event(Event::Key(Key::Char('J')));

        assert!(app.menu.state().current_page() >= 1);
        assert!(app.selected_index() >= 1);
        assert_eq!(app.active_screen(), app.selected_index());
        assert_eq!(app.content_page(), 0);
    }
}
