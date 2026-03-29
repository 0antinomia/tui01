//! 右下内容区域组件

mod interaction;
mod layout;
mod operations;
mod render;

use crate::components::Component;
use crate::controls::{AnyControl, BuiltinControl};
use crate::host::executor::{OperationRequest, OperationResult};
use crate::infra::event::Key;
use crate::runtime::{
    ContentBlock, ContentBlueprint, ContentRuntimeState, OperationStatus, RuntimeFieldState,
};
use crate::theme::Theme;
use ratatui::{Frame, layout::Rect};

#[cfg(test)]
use ratatui::style::Color;

fn scope_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_sep = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            slug.push('_');
            last_was_sep = true;
        }
    }

    slug.trim_matches('_').to_string()
}

pub struct ContentPanel {
    blueprint: ContentBlueprint,
    runtime: ContentRuntimeState,
    focused: bool,
    control_active: bool,
    pub(super) theme: Theme,
}

impl ContentPanel {
    const PAGE_SLOT_HEIGHT: u16 = 3;
    const PAGE_LABEL_MAX_CHARS: usize = 10;
    const GUTTER_WIDTH: u16 = 12;
    const COLUMN_GAP: u16 = 2;
    const TOP_PADDING: u16 = 1;

    pub fn new() -> Self {
        Self {
            blueprint: ContentBlueprint::new(""),
            runtime: ContentRuntimeState::default(),
            focused: false,
            control_active: false,
            theme: Theme::default(),
        }
    }

    /// 设置主题（由 ShowcaseApp 同步调用）。
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_blueprint(&mut self, blueprint: ContentBlueprint) {
        operations::set_blueprint_impl(self, blueprint);
        operations::assert_dual_state_consistency(self);
    }

    pub fn blueprint(&self) -> ContentBlueprint {
        let mut snapshot = self.blueprint.clone();
        let mut index = 0usize;
        for section in &mut snapshot.sections {
            for block in &mut section.blocks {
                if let Some(control) = self.block_control(index) {
                    block.control = control.clone();
                }
                index += 1;
            }
        }
        snapshot
    }

    pub fn next_page(&mut self, width: u16, height: u16) {
        let total_pages = self.total_pages(width, height);
        if self.runtime.current_page + 1 < total_pages {
            self.clear_all_statuses();
            self.runtime.current_page += 1;
            self.control_active = false;
            layout::clamp_selection_to_page(self, height);
        }
    }

    pub fn previous_page(&mut self) {
        if self.runtime.current_page > 0 {
            self.clear_all_statuses();
            self.runtime.current_page -= 1;
            self.control_active = false;
        }
    }

    pub fn previous_page_with_height(&mut self, height: u16) {
        self.previous_page();
        layout::clamp_selection_to_page(self, height);
    }

    pub fn current_page(&self) -> usize {
        self.runtime.current_page
    }

    pub fn selected_block(&self) -> usize {
        self.runtime.selected_block
    }

    pub fn has_selectable_blocks(&self, height: u16) -> bool {
        !layout::current_page_block_indices(self, height).is_empty()
    }

    pub fn tick(&mut self) {
        self.runtime.spinner_tick = self.runtime.spinner_tick.wrapping_add(1);
        self.refresh_file_logs();
    }

    fn refresh_file_logs(&mut self) {
        for state in &mut self.runtime.field_states {
            if let AnyControl::Builtin(BuiltinControl::LogOutput(control)) = &mut state.control
                && control.file_source().is_some()
            {
                control.refresh_from_file();
            }
        }
    }

    fn clear_all_statuses(&mut self) {
        self.runtime.clear_statuses();
    }

    pub fn ensure_visible_selection(&mut self, height: u16) {
        layout::clamp_selection_to_page(self, height);
    }

    pub fn handle_control_key(&mut self, key: Key) -> bool {
        interaction::handle_panel_control_key(self, key)
    }

    pub fn activate_selected_control(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        interaction::activate_panel_selected_control(self, operation_id, screen_index)
    }

    pub fn confirm_control(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        interaction::confirm_panel_control(self, operation_id, screen_index)
    }

    pub fn cancel_control(&mut self) {
        interaction::cancel_panel_control(self)
    }

    pub fn is_control_active(&self) -> bool {
        self.control_active
    }

    pub fn active_control_uses_l_as_confirm(&self) -> bool {
        matches!(
            self.selected_control_kind(),
            Some(layout::SelectedControlKind::Select | layout::SelectedControlKind::LogOutput)
        )
    }

    pub fn active_control_uses_h_as_cancel(&self) -> bool {
        !matches!(
            self.selected_control_kind(),
            Some(layout::SelectedControlKind::TextInput | layout::SelectedControlKind::NumberInput)
        )
    }

    pub fn select_next_block(&mut self, height: u16) {
        interaction::select_next_block(self, height)
    }

    pub fn select_previous_block(&mut self, height: u16) {
        interaction::select_previous_block(self, height)
    }

    pub fn has_multiple_pages(&self, width: u16, height: u16) -> bool {
        self.total_pages(width, height) > 1
    }

    pub fn total_pages(&self, _width: u16, height: u16) -> usize {
        layout::layout_pages(self, layout::effective_height(self, height)).len().max(1)
    }

    pub fn apply_operation_result(&mut self, result: &OperationResult) {
        operations::apply_operation_result(self, result);
    }

    fn selected_control_kind(&self) -> Option<layout::SelectedControlKind> {
        self.block_control(self.runtime.selected_block).map(|control| match control {
            AnyControl::Builtin(BuiltinControl::TextInput(_)) => {
                layout::SelectedControlKind::TextInput
            },
            AnyControl::Builtin(BuiltinControl::NumberInput(_)) => {
                layout::SelectedControlKind::NumberInput
            },
            AnyControl::Builtin(BuiltinControl::Select(_)) => layout::SelectedControlKind::Select,
            AnyControl::Builtin(BuiltinControl::Toggle(_)) => layout::SelectedControlKind::Toggle,
            AnyControl::Builtin(BuiltinControl::ActionButton(_)) => {
                layout::SelectedControlKind::ActionButton
            },
            AnyControl::Builtin(BuiltinControl::StaticData(_)) => {
                layout::SelectedControlKind::StaticData
            },
            AnyControl::Builtin(BuiltinControl::DynamicData(_)) => {
                layout::SelectedControlKind::DynamicData
            },
            AnyControl::Builtin(BuiltinControl::LogOutput(_)) => {
                layout::SelectedControlKind::LogOutput
            },
            AnyControl::Custom(_) => layout::SelectedControlKind::Custom,
        })
    }

    // Test-only delegates for layout/render methods used by tests
    #[cfg(test)]
    fn page_label(&self, height: u16, page: usize) -> String {
        layout::page_label(self, height, page)
    }

    #[cfg(test)]
    fn truncated_page_label(&self, height: u16, page: usize, max_width: usize) -> String {
        layout::truncated_page_label(self, height, page, max_width)
    }

    #[cfg(test)]
    fn pagination_rows(&self, height: u16) -> Vec<(char, Color, Option<String>)> {
        layout::pagination_rows(self, height)
    }

    pub(super) fn selected_block_state(&self) -> Option<&RuntimeFieldState> {
        self.runtime.field_state(self.runtime.selected_block)
    }

    pub(super) fn selected_block_state_mut(&mut self) -> Option<&mut RuntimeFieldState> {
        self.runtime.field_state_mut(self.runtime.selected_block)
    }

    pub(super) fn clear_selected_snapshot(&mut self) {
        if let Some(state) = self.selected_block_state_mut() {
            state.snapshot = None;
        }
    }

    pub(super) fn block_state(&self, index: usize) -> Option<&RuntimeFieldState> {
        self.runtime.field_state(index)
    }

    pub(super) fn block_state_mut(&mut self, index: usize) -> Option<&mut RuntimeFieldState> {
        self.runtime.field_state_mut(index)
    }

    pub(super) fn block_is_running(&self, index: usize) -> bool {
        self.block_state(index)
            .map(|state| matches!(state.status, OperationStatus::Running { .. }))
            .unwrap_or(false)
    }

    pub(super) fn block_control(&self, index: usize) -> Option<&AnyControl> {
        self.block_state(index).map(|state| &state.control)
    }

    pub(super) fn block_control_mut(&mut self, index: usize) -> Option<&mut AnyControl> {
        self.block_state_mut(index).map(|state| &mut state.control)
    }

    pub(super) fn selected_block_control_mut(&mut self) -> Option<&mut AnyControl> {
        self.block_control_mut(self.runtime.selected_block)
    }

    pub(super) fn selected_block_ref(&self) -> Option<&ContentBlock> {
        let mut index = 0usize;
        for section in &self.blueprint.sections {
            for block in &section.blocks {
                if index == self.runtime.selected_block {
                    return Some(block);
                }
                index += 1;
            }
        }
        None
    }

    pub(super) fn block_index_by_id(&self, target_id: &str) -> Option<usize> {
        let mut index = 0usize;
        for section in &self.blueprint.sections {
            for block in &section.blocks {
                if block.id.as_deref() == Some(target_id) {
                    return Some(index);
                }
                index += 1;
            }
        }
        None
    }

    pub(super) fn block_control_mut_by_id(&mut self, target_id: &str) -> Option<&mut AnyControl> {
        let index = self.block_index_by_id(target_id)?;
        self.block_control_mut(index)
    }
}

impl Default for ContentPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ContentPanel {
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
        self.control_active = false;
        self.clear_selected_snapshot();
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render::render_content_panel(self, f, rect)
    }
}

#[cfg(test)]
mod tests {
    use super::ContentPanel;
    use crate::host::executor::OperationResult;
    use crate::runtime::{ContentBlock, ContentBlueprint, ContentSection};
    use ratatui::style::Color;

    fn panel_with_sections() -> ContentPanel {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(ContentBlueprint::new("Root").with_sections(vec![
            ContentSection::new("概览").with_blocks(vec![
                ContentBlock::text_input("用户名", "demo", "输入用户名"),
                ContentBlock::toggle("开启高级模式", true),
            ]),
            ContentSection::new("行为").with_blocks(vec![
                ContentBlock::select("日志级别", ["info", "debug", "trace"], 0),
                ContentBlock::number_input("端口", "8080", "输入端口"),
                ContentBlock::text_input("保存路径", "/tmp", "输入路径").with_height_units(2),
            ]),
            ContentSection::new("动作").with_blocks(vec![
                ContentBlock::refresh_button("刷新列表", "刷新").with_operation_success(700),
                ContentBlock::static_data("版本", "v0.2.0"),
                ContentBlock::log_output("最近输出", "line one\nline two").with_height_units(2),
            ]),
        ]));
        panel
    }

    #[test]
    fn content_panel_uses_subtitle_as_page_label() {
        let panel = panel_with_sections();

        assert_eq!(panel.page_label(6, 0), "概览");
    }

    #[test]
    fn content_panel_paginates_by_block_height() {
        let panel = panel_with_sections();

        assert_eq!(panel.total_pages(40, 6), 8);
    }

    #[test]
    fn content_panel_truncates_page_label_to_ten_chars() {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(ContentBlueprint::new("Root").with_sections(vec![
                ContentSection::new("一个超过十个字符的子标题名称")
                    .with_blocks(vec![ContentBlock::static_data("描述", "value")]),
            ]));

        let label = panel.truncated_page_label(6, 0, 32);
        assert_eq!(label.chars().count(), 10);
        assert!(label.ends_with('\u{2026}'));
    }

    #[test]
    fn content_panel_shows_pagination_gutter_for_single_page() {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(ContentBlueprint::new("Root").with_sections(vec![
                ContentSection::new("概览")
                    .with_blocks(vec![ContentBlock::static_data("One", "value")]),
            ]));

        let glyphs: Vec<char> = panel.pagination_rows(8).into_iter().map(|row| row.0).collect();

        assert_eq!(glyphs, vec!['\u{2503}', '\u{2503}', '\u{2503}']);
    }

    #[test]
    fn content_panel_dims_inactive_pagination_segments() {
        let panel = panel_with_sections();

        let cells = panel.pagination_rows(7);

        assert!(
            cells.iter().any(|(glyph, color, _)| *glyph == '\u{2503}' && *color == Color::White)
        );
        assert!(
            cells.iter().any(
                |(glyph, color, _)| *glyph == '\u{2502}' && *color == Color::Rgb(170, 170, 170)
            )
        );
    }

    #[test]
    fn content_panel_shows_active_page_label_on_middle_row() {
        let panel = panel_with_sections();

        let cells = panel.pagination_rows(6);

        assert_eq!(cells[1].2.as_deref(), Some("概览"));
    }

    #[test]
    fn content_panel_moves_selection_within_current_page() {
        let mut panel = panel_with_sections();

        panel.select_next_block(13);
        assert_eq!(panel.selected_block(), 1);

        panel.select_previous_block(13);
        assert_eq!(panel.selected_block(), 0);
    }

    #[test]
    fn content_panel_advances_to_next_page_when_selection_hits_bottom() {
        let mut panel = panel_with_sections();

        panel.select_next_block(13);
        panel.select_next_block(13);

        assert_eq!(panel.current_page(), 1);
        assert_eq!(panel.selected_block(), 2);
    }

    #[test]
    fn content_panel_returns_to_previous_page_when_selection_hits_top() {
        let mut panel = panel_with_sections();

        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_previous_block(13);

        assert_eq!(panel.current_page(), 0);
        assert_eq!(panel.selected_block(), 1);
    }

    #[test]
    fn content_panel_clamps_selection_when_page_changes() {
        let mut panel = panel_with_sections();
        panel.select_next_block(9);
        panel.next_page(40, 6);

        assert_eq!(panel.current_page(), 1);
        assert_eq!(panel.selected_block(), 1);

        panel.next_page(40, 6);
        assert_eq!(panel.selected_block(), 2);
    }

    #[test]
    fn activate_selected_control_enters_text_input_mode() {
        let mut panel = panel_with_sections();

        assert!(panel.activate_selected_control(1, 0).is_none());
        assert!(panel.is_control_active());
        assert!(panel.handle_control_key(crate::event::Key::Char('x')));

        match &panel.blueprint().sections[0].blocks[0].control {
            super::AnyControl::Builtin(super::BuiltinControl::TextInput(control)) => {
                assert_eq!(control.value, "demox")
            },
            _ => panic!("expected text input"),
        }
    }

    #[test]
    fn activate_selected_control_toggles_switch_without_entering_edit_mode() {
        let mut panel = panel_with_sections();
        panel.select_next_block(13);

        assert!(panel.activate_selected_control(1, 0).is_none());
        assert!(!panel.is_control_active());

        match &panel.blueprint().sections[0].blocks[1].control {
            super::AnyControl::Builtin(super::BuiltinControl::Toggle(control)) => {
                assert!(!control.on)
            },
            _ => panic!("expected toggle"),
        }
    }

    #[test]
    fn number_input_enters_edit_mode_and_accepts_digits() {
        let mut panel = panel_with_sections();
        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_next_block(13);

        assert!(panel.activate_selected_control(1, 0).is_none());
        assert!(panel.handle_control_key(crate::event::Key::Char('9')));
        assert!(!panel.handle_control_key(crate::event::Key::Char('x')));

        match &panel.blueprint().sections[1].blocks[1].control {
            super::AnyControl::Builtin(super::BuiltinControl::NumberInput(control)) => {
                assert_eq!(control.value, "80809")
            },
            _ => panic!("expected number input"),
        }
    }

    #[test]
    fn cancel_control_restores_previous_select_value() {
        let mut panel = panel_with_sections();
        panel.select_next_block(13);
        panel.select_next_block(13);

        assert!(panel.activate_selected_control(1, 0).is_none());
        assert!(panel.handle_control_key(crate::event::Key::Char('j')));
        panel.cancel_control();

        match &panel.blueprint().sections[1].blocks[0].control {
            super::AnyControl::Builtin(super::BuiltinControl::Select(control)) => {
                assert_eq!(control.selected, 0)
            },
            _ => panic!("expected select"),
        }
        assert!(!panel.is_control_active());
    }

    #[test]
    fn activate_action_button_returns_operation_request() {
        let mut panel = panel_with_sections();
        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_next_block(13);

        let request = panel.activate_selected_control(7, 2);
        assert!(request.is_some());
        let request = request.unwrap();
        assert_eq!(request.operation_id, 7);
        assert_eq!(request.screen_index, 2);
        assert_eq!(request.block_index, 5);
    }

    #[test]
    fn static_data_does_not_enter_edit_mode() {
        let mut panel = panel_with_sections();
        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_next_block(13);
        panel.select_next_block(13);

        assert!(panel.activate_selected_control(9, 2).is_none());
        assert!(!panel.is_control_active());
    }

    #[test]
    fn log_output_enters_view_mode() {
        let mut panel = panel_with_sections();
        for _ in 0..7 {
            panel.select_next_block(13);
        }

        assert!(panel.activate_selected_control(10, 2).is_none());
        assert!(panel.is_control_active());
    }

    #[test]
    fn operation_result_updates_bound_log_output() {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(ContentBlueprint::new("Root").with_sections(vec![
            ContentSection::new("动作").with_blocks(vec![
                ContentBlock::action_button("执行同步", "执行")
                    .with_id("run_sync")
                    .with_result_target("sync_log")
                    .with_shell_command("printf 'done\\n'"),
                ContentBlock::log_output("输出", "等待操作").with_id("sync_log"),
            ]),
        ]));

        let request = panel.activate_selected_control(11, 0).unwrap();
        assert_eq!(request.result_target.as_deref(), Some("sync_log"));

        panel.apply_operation_result(&OperationResult {
            operation_id: 11,
            screen_index: 0,
            block_index: 0,
            result_target: Some("sync_log".to_string()),
            success: true,
            stdout: "done".to_string(),
            stderr: String::new(),
        });

        match &panel.blueprint().sections[0].blocks[1].control {
            super::AnyControl::Builtin(super::BuiltinControl::LogOutput(log)) => {
                assert!(log.content.contains("done"))
            },
            _ => panic!("expected log output"),
        }
    }

    #[test]
    fn operation_result_appends_to_existing_log_output() {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(ContentBlueprint::new("Root").with_sections(vec![
            ContentSection::new("动作").with_blocks(vec![
                ContentBlock::action_button("执行同步", "执行")
                    .with_id("run_sync")
                    .with_result_target("sync_log")
                    .with_shell_command("printf 'done\\n'"),
                ContentBlock::log_output("输出", "previous line").with_id("sync_log"),
            ]),
        ]));

        panel.apply_operation_result(&OperationResult {
            operation_id: 12,
            screen_index: 0,
            block_index: 0,
            result_target: Some("sync_log".to_string()),
            success: true,
            stdout: "done".to_string(),
            stderr: String::new(),
        });

        match &panel.blueprint().sections[0].blocks[1].control {
            super::AnyControl::Builtin(super::BuiltinControl::LogOutput(log)) => {
                assert!(log.content.contains("previous line"));
                assert!(log.content.contains("[success]"));
                assert!(log.content.contains("done"));
            },
            _ => panic!("expected log output"),
        }
    }

    #[test]
    fn registered_action_request_includes_current_field_values() {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(ContentBlueprint::new("Root").with_sections(vec![
            ContentSection::new("动作").with_blocks(vec![
                    ContentBlock::text_input("项目名", "tui01", "输入项目名")
                        .with_id("project_name"),
                    ContentBlock::number_input("端口", "3000", "输入端口").with_id("server_port"),
                    ContentBlock::refresh_button("刷新", "刷新")
                        .with_registered_action("refresh_workspace"),
                ]),
        ]));

        panel.select_next_block(20);
        panel.select_next_block(20);

        let request = panel.activate_selected_control(21, 0).unwrap();
        assert_eq!(request.params.get("project_name").map(String::as_str), Some("tui01"));
        assert_eq!(request.params.get("server_port").map(String::as_str), Some("3000"));
        assert_eq!(request.params.get("screen.project_name").map(String::as_str), Some("tui01"));
        assert_eq!(request.params.get("root.server_port").map(String::as_str), Some("3000"));
    }

    #[test]
    fn page_scope_slug_normalizes_blueprint_title() {
        assert_eq!(super::scope_slug("Workspace"), "workspace");
        assert_eq!(super::scope_slug("Theme Settings"), "theme_settings");
        assert_eq!(super::scope_slug("中文 / Mixed-Page"), "mixed_page");
    }
}
