//! 通用的四分区展示应用壳层。

use crate::action::Action;
use crate::builder::AppValidationError;
use crate::components::{
    Component, ContentPanel, MenuComponent, MenuItem, QuadrantConfig, QuadrantLayout, StatusPanel,
    TitlePanel,
};
use crate::event::{Event, Key};
use crate::executor::{ActionRegistry, OperationExecutor, OperationRequest, OperationResult};
use crate::framework_log::FrameworkLogger;
use crate::host::RuntimeHost;
use crate::runtime::ContentBlueprint;
use crate::schema::PageSpec;
use crate::tui::{self, MAX_ASPECT_RATIO, MIN_ASPECT_RATIO, MIN_HEIGHT, MIN_WIDTH};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

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
    executor: OperationExecutor,
    host: RuntimeHost,
    next_operation_id: u64,
    copy: ShowcaseCopy,
    focus: FocusTarget,
}

enum SizeError {
    Small,
    Narrow,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusTarget {
    Menu,
    Content,
}

pub struct ShowcaseScreen {
    pub title: String,
    pub content: ContentBlueprint,
}

impl ShowcaseScreen {
    pub fn from_page(title: impl Into<String>, page: PageSpec) -> Self {
        Self {
            title: title.into(),
            content: page.materialize().into(),
        }
    }

    /// 从页面规格和控件注册表创建 ShowcaseScreen，支持自定义控件。
    pub fn from_page_with_registry(
        title: impl Into<String>,
        page: PageSpec,
        registry: Option<&crate::host::ControlRegistry>,
    ) -> Self {
        Self {
            title: title.into(),
            content: ContentBlueprint::from_runtime_page(page.materialize(), registry),
        }
    }
}

pub struct ShowcaseCopy {
    pub title_text: String,
    pub status_controls: String,
}

impl ShowcaseApp {
    pub fn new(copy: ShowcaseCopy, screens: Vec<ShowcaseScreen>) -> Self {
        Self::with_host(copy, screens, RuntimeHost::new())
    }

    pub fn with_host(copy: ShowcaseCopy, screens: Vec<ShowcaseScreen>, host: RuntimeHost) -> Self {
        Self::with_registry_and_host(copy, screens, host.action_registry(), host)
    }

    pub fn with_registry(
        copy: ShowcaseCopy,
        screens: Vec<ShowcaseScreen>,
        registry: ActionRegistry,
    ) -> Self {
        Self::with_registry_and_host(
            copy,
            screens,
            registry.clone(),
            RuntimeHost::with_registry(registry),
        )
    }

    fn with_registry_and_host(
        copy: ShowcaseCopy,
        screens: Vec<ShowcaseScreen>,
        registry: ActionRegistry,
        host: RuntimeHost,
    ) -> Self {
        let items = screens
            .iter()
            .map(|screen| MenuItem::new(screen.title.clone()))
            .collect();
        let mut menu = MenuComponent::new(items);
        menu.focus();

        let mut app = Self {
            running: true,
            size_error: None,
            quadrant_layout: QuadrantLayout::new(QuadrantConfig::default()),
            title_panel: TitlePanel::new(copy.title_text.clone()),
            status_panel: StatusPanel::new(""),
            menu,
            content_panel: ContentPanel::new(),
            screens,
            active_screen: 0,
            loaded_screen: None,
            executor: OperationExecutor::with_runtime(
                registry,
                host.shell_policy(),
                host.event_hook(),
                host.logger(),
                framework_logger_for_host(&host),
            ),
            host,
            next_operation_id: 1,
            copy,
            focus: FocusTarget::Menu,
        };
        app.content_panel.blur();
        app.sync_panels();
        app
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Tick => {
                self.content_panel.tick();
                self.poll_operation_results();
            }
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

    pub fn host(&self) -> &RuntimeHost {
        &self.host
    }

    pub fn validate_registered_actions(&self) -> Result<(), AppValidationError> {
        for screen in &self.screens {
            for section in &screen.content.sections {
                for block in &section.blocks {
                    let source_field = block
                        .id
                        .clone()
                        .unwrap_or_else(|| format!("{}:{}", screen.title, block.label));

                    if let Some(operation) = &block.operation {
                        if let crate::runtime::OperationSource::RegisteredAction(action) =
                            &operation.source
                        {
                            if !self.host.has_action(action) {
                                return Err(AppValidationError::UnknownRegisteredAction {
                                    source_field,
                                    action: action.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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
                self.content_panel
                    .previous_page_with_height(self.current_content_rect().height);
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
                Key::Char('h') if self.content_panel.active_control_uses_h_as_cancel() => {
                    self.content_panel.cancel_control();
                    self.persist_active_screen_content();
                }
                Key::Esc => {
                    self.content_panel.cancel_control();
                    self.persist_active_screen_content();
                }
                Key::Enter => {
                    let operation_id = self.next_operation_id();
                    if let Some(request) = self
                        .content_panel
                        .confirm_control(operation_id, self.active_screen)
                    {
                        self.submit_operation(request);
                    }
                    self.persist_active_screen_content();
                }
                Key::Char('l') if self.content_panel.active_control_uses_l_as_confirm() => {
                    let operation_id = self.next_operation_id();
                    if let Some(request) = self
                        .content_panel
                        .confirm_control(operation_id, self.active_screen)
                    {
                        self.submit_operation(request);
                    }
                    self.persist_active_screen_content();
                }
                Key::Left => {
                    if self.content_panel.handle_control_key(key) {
                        self.persist_active_screen_content();
                    }
                }
                Key::Right | Key::Char('l') => {
                    if self.content_panel.handle_control_key(key) {
                        self.persist_active_screen_content();
                    }
                }
                _ => {
                    if self.content_panel.handle_control_key(key) {
                        self.persist_active_screen_content();
                    }
                }
            }
            return;
        }

        match key {
            Key::Up | Key::Char('k') => {
                self.content_panel
                    .select_previous_block(content_rect.height);
            }
            Key::Down | Key::Char('j') => {
                self.content_panel.select_next_block(content_rect.height);
            }
            Key::Char('K') => {
                self.content_panel
                    .previous_page_with_height(content_rect.height);
            }
            Key::Char('J') => {
                self.content_panel
                    .next_page(content_rect.width, content_rect.height);
            }
            Key::Char('l') | Key::Enter => {
                let operation_id = self.next_operation_id();
                if let Some(request) = self
                    .content_panel
                    .activate_selected_control(operation_id, self.active_screen)
                {
                    self.submit_operation(request);
                }
                self.persist_active_screen_content();
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

    fn next_operation_id(&mut self) -> u64 {
        let id = self.next_operation_id;
        self.next_operation_id = self.next_operation_id.wrapping_add(1);
        id
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
            SizeError::Small => {
                format!("终端太小（最小需要 {}x{}）", MIN_WIDTH, MIN_HEIGHT)
            }
            SizeError::Narrow => {
                format!(
                    "终端过窄（宽高比需 >= {:.1}，当前: {:.2}）",
                    MIN_ASPECT_RATIO,
                    Self::current_aspect_ratio()
                )
            }
            SizeError::Wide => {
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
        self.title_panel.set_text(self.copy.title_text.clone());

        let selected = self
            .menu
            .selected_item()
            .map(|item| item.label.as_str())
            .unwrap_or("None");

        let active = self
            .screens
            .get(self.active_screen)
            .map(|screen| screen.title.as_str())
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
            screen.content = self.content_panel.blueprint();
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

    fn poll_operation_results(&mut self) {
        while let Some(result) = self.executor.try_recv() {
            self.apply_operation_result(result);
        }
    }

    fn submit_operation(&mut self, request: OperationRequest) {
        self.executor.submit(OperationRequest {
            host: self.host.context().clone(),
            cwd: self
                .host
                .working_dir()
                .and_then(|path| path.to_str())
                .map(str::to_string),
            env: self.host.shell().env().clone(),
            allowed_working_dirs: self
                .host
                .execution_policy()
                .allowed_working_dirs()
                .iter()
                .filter_map(|path| path.to_str().map(str::to_string))
                .collect(),
            allowed_env_keys: self.host.execution_policy().allowed_env_keys().cloned(),
            ..request
        });
    }

    fn apply_operation_result(&mut self, result: OperationResult) {
        if let Some(screen) = self.screens.get_mut(result.screen_index) {
            let mut panel = ContentPanel::new();
            panel.set_blueprint(screen.content.clone());
            panel.apply_operation_result(&result);
            screen.content = panel.blueprint();
        }

        if result.screen_index == self.active_screen {
            self.content_panel.apply_operation_result(&result);
        }
    }

    fn current_content_rect(&self) -> Rect {
        let (width, height) = tui::terminal_size().unwrap_or((MIN_WIDTH, MIN_HEIGHT));
        let area = Rect::new(0, 0, width, height);
        let (_, _, _, bottom_right) = self.quadrant_layout.calculate_quadrants(area);
        bottom_right
    }

    fn check_size(w: u16, h: u16) -> Option<SizeError> {
        if w < MIN_WIDTH || h < MIN_HEIGHT {
            return Some(SizeError::Small);
        }
        let aspect_ratio = w as f64 / h as f64;
        if aspect_ratio < MIN_ASPECT_RATIO {
            return Some(SizeError::Narrow);
        }
        if aspect_ratio > MAX_ASPECT_RATIO {
            return Some(SizeError::Wide);
        }
        None
    }

    fn current_aspect_ratio() -> f64 {
        let (w, h) = tui::terminal_size().unwrap_or((MIN_WIDTH, MIN_HEIGHT));
        w as f64 / h as f64
    }
}

fn framework_logger_for_host(host: &RuntimeHost) -> FrameworkLogger {
    if !host.framework_log_enabled() {
        return FrameworkLogger::disabled();
    }

    if let Some(path) = host.framework_log_path() {
        FrameworkLogger::from_path(path).unwrap_or_else(|_| FrameworkLogger::fallback())
    } else if let Some(cwd) = host.working_dir() {
        FrameworkLogger::new(cwd).unwrap_or_else(|_| FrameworkLogger::fallback())
    } else {
        FrameworkLogger::fallback()
    }
}

#[cfg(test)]
mod tests {
    use super::{FocusTarget, ShowcaseApp, ShowcaseCopy, ShowcaseScreen};
    use crate::event::{Event, Key};
    use crate::components::{AnyControl, BuiltinControl};
    use crate::runtime::{ContentBlock, ContentBlueprint, ContentSection};
    fn screen(title: &'static str, text: &'static str) -> ShowcaseScreen {
        ShowcaseScreen {
            title: title.to_string(),
            content: ContentBlueprint::new(title).with_sections(vec![ContentSection::new("概览")
                .with_blocks(vec![ContentBlock::text_input(text, "", "输入值")])]),
        }
    }

    fn make_app() -> ShowcaseApp {
        ShowcaseApp::new(
            ShowcaseCopy {
                title_text: "Title".to_string(),
                status_controls: "Controls".to_string(),
            },
            vec![screen("One", "First"), screen("Two", "Second")],
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
                title_text: "Title".to_string(),
                status_controls: "Controls".to_string(),
            },
            vec![ShowcaseScreen {
                title: "One".to_string(),
                content: ContentBlueprint::new("One").with_sections(vec![ContentSection::new(
                    "概览",
                )
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
                title_text: "Title".to_string(),
                status_controls: "Controls".to_string(),
            },
            vec![ShowcaseScreen {
                title: "One".to_string(),
                content: ContentBlueprint::new("One")
                    .with_sections(vec![ContentSection::new("概览")
                        .with_blocks(vec![ContentBlock::toggle("A", false)])]),
            }],
        );

        app.handle_event(Event::Key(Key::Char('l')));
        app.handle_event(Event::Key(Key::Enter));
        app.sync_panels();

        match &app.content_panel.blueprint().sections[0].blocks[0].control {
            AnyControl::Builtin(BuiltinControl::Toggle(control)) => {
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
    fn h_cancels_active_control_without_returning_to_menu() {
        let mut app = ShowcaseApp::new(
            ShowcaseCopy {
                title_text: "Title".to_string(),
                status_controls: "Controls".to_string(),
            },
            vec![ShowcaseScreen {
                title: "One".to_string(),
                content: ContentBlueprint::new("One")
                    .with_sections(vec![ContentSection::new("概览")
                        .with_blocks(vec![ContentBlock::select("B", ["One", "Two"], 0)])]),
            }],
        );

        app.handle_event(Event::Key(Key::Char('l')));
        app.handle_event(Event::Key(Key::Enter));
        assert!(app.content_panel.is_control_active());

        app.handle_event(Event::Key(Key::Char('h')));
        assert!(!app.content_panel.is_control_active());
        assert_eq!(app.focus, FocusTarget::Content);
    }

    #[test]
    fn l_confirms_active_select_control() {
        let mut app = ShowcaseApp::new(
            ShowcaseCopy {
                title_text: "Title".to_string(),
                status_controls: "Controls".to_string(),
            },
            vec![ShowcaseScreen {
                title: "One".to_string(),
                content: ContentBlueprint::new("One")
                    .with_sections(vec![ContentSection::new("概览")
                        .with_blocks(vec![ContentBlock::select("B", ["One", "Two"], 0)])]),
            }],
        );

        app.handle_event(Event::Key(Key::Char('l')));
        app.handle_event(Event::Key(Key::Enter));
        assert!(app.content_panel.is_control_active());

        app.handle_event(Event::Key(Key::Char('l')));
        assert!(!app.content_panel.is_control_active());
        match &app.content_panel.blueprint().sections[0].blocks[0].control {
            AnyControl::Builtin(BuiltinControl::Select(control)) => assert_eq!(control.selected, 0),
            _ => panic!("expected select"),
        }
    }

    #[test]
    fn h_is_typed_inside_active_text_input() {
        let mut app = ShowcaseApp::new(
            ShowcaseCopy {
                title_text: "Title".to_string(),
                status_controls: "Controls".to_string(),
            },
            vec![ShowcaseScreen {
                title: "One".to_string(),
                content: ContentBlueprint::new("One")
                    .with_sections(vec![ContentSection::new("概览")
                        .with_blocks(vec![ContentBlock::text_input("Name", "", "输入")])]),
            }],
        );

        app.handle_event(Event::Key(Key::Char('l')));
        app.handle_event(Event::Key(Key::Enter));
        assert!(app.content_panel.is_control_active());

        app.handle_event(Event::Key(Key::Char('h')));
        assert!(app.content_panel.is_control_active());
        match &app.content_panel.blueprint().sections[0].blocks[0].control {
            AnyControl::Builtin(BuiltinControl::TextInput(control)) => {
                assert_eq!(control.value, "h")
            }
            _ => panic!("expected text input"),
        }
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
                title_text: "Title".to_string(),
                status_controls: "Controls".to_string(),
            },
            vec![
                ShowcaseScreen {
                    title: "One".to_string(),
                    content: ContentBlueprint::new("One").with_sections(vec![
                        ContentSection::new("Section A").with_blocks(vec![
                            ContentBlock::text_input("First block", "alpha", "输入")
                                .with_height_units(2),
                            ContentBlock::select("Second block", ["A", "B", "C"], 0)
                                .with_height_units(2),
                            ContentBlock::toggle("Third block", true).with_height_units(2),
                        ]),
                        ContentSection::new("Section B").with_blocks(vec![
                            ContentBlock::text_input("Fourth block", "beta", "输入")
                                .with_height_units(2),
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
