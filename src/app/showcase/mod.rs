//! 通用的四分区展示应用壳层。

mod operation_poll;
mod screen_manager;
mod tea_core;

use crate::builder::AppValidationError;
use crate::components::{
    Component, ContentPanel, MenuComponent, MenuItem, QuadrantConfig, QuadrantLayout, StatusPanel,
    TitlePanel,
};
use crate::event::Event;
use crate::host::FrameworkLogger;
use crate::host::{ActionRegistry, OperationExecutor, RuntimeHost};
use crate::runtime::ContentBlueprint;
use crate::schema::PageSpec;
use crate::theme::{LayoutStrategy, Theme};
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
    theme: Theme,
    layout_strategy: Box<dyn LayoutStrategy>,
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
        Self { title: title.into(), content: page.materialize().into() }
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
        let items = screens.iter().map(|screen| MenuItem::new(screen.title.clone())).collect();
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
            theme: Theme::default(),
            layout_strategy: Box::new(QuadrantLayout::new(QuadrantConfig::default())),
        };
        app.content_panel.blur();
        screen_manager::sync_panels(&mut app);
        app
    }

    pub fn handle_event(&mut self, event: Event) {
        tea_core::handle_event(self, event)
    }

    pub async fn run(mut self) -> color_eyre::Result<()> {
        tui::init_panic_hook();
        let mut terminal = tui::Tui::new()?;
        let mut events = crate::event::EventHandler::new();

        let run_result: color_eyre::Result<()> = async {
            while self.running {
                terminal.draw(|frame| self.render(frame))?;
                let Some(event) = events.next().await else {
                    break;
                };
                self.handle_event(event);
            }

            Ok(())
        }
        .await;

        let exit_result = terminal.exit();
        run_result?;
        exit_result
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        if let Some(ref error) = self.size_error {
            self.render_size_error(frame, error);
            return;
        }

        let area = frame.area();
        self.quadrant_layout.render(frame, area);
        screen_manager::sync_panels(self);

        let areas = self.layout_strategy.areas(area);

        self.title_panel.render(frame, areas.title);
        self.status_panel.render(frame, areas.status);
        self.menu.render(frame, areas.menu);
        self.content_panel.render(frame, areas.content);
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

    /// 设置自定义主题。
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// 设置自定义布局策略。
    pub fn set_layout_strategy(&mut self, strategy: Box<dyn LayoutStrategy>) {
        self.layout_strategy = strategy;
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

                    if let Some(operation) = &block.operation
                        && let crate::runtime::OperationSource::RegisteredAction(action) =
                            &operation.source
                        && !self.host.has_action(action)
                    {
                        return Err(AppValidationError::UnknownRegisteredAction {
                            source_field,
                            action: action.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(test)]
    fn sync_panels(&mut self) {
        screen_manager::sync_panels(self);
    }

    fn current_content_rect(&self) -> Rect {
        let (width, height) = tui::terminal_size().unwrap_or((MIN_WIDTH, MIN_HEIGHT));
        let area = Rect::new(0, 0, width, height);
        let areas = self.layout_strategy.areas(area);
        areas.content
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

    fn render_size_error(&self, frame: &mut ratatui::Frame, error: &SizeError) {
        use ratatui::layout::Alignment;

        let error_msg = match error {
            SizeError::Small => {
                format!("终端太小（最小需要 {}x{}）", MIN_WIDTH, MIN_HEIGHT)
            },
            SizeError::Narrow => {
                format!(
                    "终端过窄（宽高比需 >= {:.1}，当前: {:.2}）",
                    MIN_ASPECT_RATIO,
                    Self::current_aspect_ratio()
                )
            },
            SizeError::Wide => {
                format!(
                    "终端过宽（宽高比需 <= {:.1}，当前: {:.2}）",
                    MAX_ASPECT_RATIO,
                    Self::current_aspect_ratio()
                )
            },
        };

        let widget = Paragraph::new(error_msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        frame.render_widget(widget, frame.area());
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
    use crate::controls::{AnyControl, BuiltinControl};
    use crate::event::{Event, Key};
    use crate::runtime::{ContentBlock, ContentBlueprint, ContentSection};
    fn screen(title: &'static str, text: &'static str) -> ShowcaseScreen {
        ShowcaseScreen {
            title: title.to_string(),
            content: ContentBlueprint::new(title).with_sections(vec![
                ContentSection::new("概览").with_blocks(vec![ContentBlock::text_input(
                    text,
                    "",
                    "输入值",
                )]),
            ]),
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
                content: ContentBlueprint::new("One").with_sections(vec![
                    ContentSection::new("概览").with_blocks(vec![
                        ContentBlock::toggle("A", true),
                        ContentBlock::select("B", ["One", "Two"], 0),
                    ]),
                ]),
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
                content: ContentBlueprint::new("One").with_sections(vec![
                    ContentSection::new("概览").with_blocks(vec![ContentBlock::toggle("A", false)]),
                ]),
            }],
        );

        app.handle_event(Event::Key(Key::Char('l')));
        app.handle_event(Event::Key(Key::Enter));
        app.sync_panels();

        match &app.content_panel.blueprint().sections[0].blocks[0].control {
            AnyControl::Builtin(BuiltinControl::Toggle(control)) => {
                assert!(control.on)
            },
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
                content: ContentBlueprint::new("One").with_sections(vec![
                    ContentSection::new("概览").with_blocks(vec![ContentBlock::select(
                        "B",
                        ["One", "Two"],
                        0,
                    )]),
                ]),
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
                content: ContentBlueprint::new("One").with_sections(vec![
                    ContentSection::new("概览").with_blocks(vec![ContentBlock::select(
                        "B",
                        ["One", "Two"],
                        0,
                    )]),
                ]),
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
                content: ContentBlueprint::new("One").with_sections(vec![
                    ContentSection::new("概览")
                        .with_blocks(vec![ContentBlock::text_input("Name", "", "输入")]),
                ]),
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
            },
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
