//! 右下内容区域组件

use crate::components::{Component, ControlFeedback, ControlKind};
use crate::event::Key;
use crate::executor::{
    OperationRequest, OperationResult, OperationSource as ExecutorOperationSource,
};
use crate::runtime::{
    ContentBlock, ContentBlueprint, ContentControl, ContentRuntimeState, OperationSource,
    OperationStatus,
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};
use std::collections::HashMap;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn render_control(
    control: &ContentControl,
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    selected: bool,
    active: bool,
    feedback: ControlFeedback,
) {
    match control {
        ContentControl::TextInput(control) => {
            ControlKind::TextInput(control.clone()).render(area, buf, selected, active, feedback)
        }
        ContentControl::NumberInput(control) => {
            ControlKind::NumberInput(control.clone()).render(area, buf, selected, active, feedback)
        }
        ContentControl::Select(control) => {
            ControlKind::Select(control.clone()).render(area, buf, selected, active, feedback)
        }
        ContentControl::Toggle(control) => {
            ControlKind::Toggle(control.clone()).render(area, buf, selected, active, feedback)
        }
        ContentControl::ActionButton(control) => {
            ControlKind::ActionButton(control.clone()).render(area, buf, selected, active, feedback)
        }
        ContentControl::StaticData(control) => {
            ControlKind::StaticData(control.clone()).render(area, buf, selected, active, feedback)
        }
        ContentControl::DynamicData(control) => {
            ControlKind::DynamicData(control.clone()).render(area, buf, selected, active, feedback)
        }
        ContentControl::LogOutput(control) => {
            ControlKind::LogOutput(control.clone()).render(area, buf, selected, active, feedback)
        }
    }
}

fn handle_control_key(control: &mut ContentControl, key: Key) -> bool {
    match control {
        ContentControl::TextInput(control) => control.handle_key(key),
        ContentControl::NumberInput(control) => control.handle_key(key),
        ContentControl::Select(control) => control.handle_key(key),
        ContentControl::Toggle(control) => control.handle_key(key),
        ContentControl::ActionButton(control) => control.handle_key(key),
        ContentControl::StaticData(_)
        | ContentControl::DynamicData(_)
        | ContentControl::LogOutput(_) => false,
    }
}

fn control_value(control: &ContentControl) -> String {
    match control {
        ContentControl::TextInput(control) => control.value.clone(),
        ContentControl::NumberInput(control) => control.value.clone(),
        ContentControl::Select(control) => control
            .options
            .get(control.selected)
            .cloned()
            .unwrap_or_default(),
        ContentControl::Toggle(control) => control.on.to_string(),
        ContentControl::ActionButton(control) => control.label.clone(),
        ContentControl::StaticData(control) | ContentControl::DynamicData(control) => {
            control.value.clone()
        }
        ContentControl::LogOutput(control) => control.content.clone(),
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContentPage {
    subtitle: String,
    blocks: Vec<VisibleBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VisibleBlock {
    index: usize,
    block: ContentBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedControlKind {
    TextInput,
    NumberInput,
    Select,
    Toggle,
    ActionButton,
    StaticData,
    DynamicData,
    LogOutput,
}

impl VisibleBlock {
    fn row_height(&self) -> usize {
        self.block.row_height()
    }
}

pub struct ContentPanel {
    blueprint: ContentBlueprint,
    runtime: ContentRuntimeState,
    focused: bool,
    control_active: bool,
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
        }
    }

    pub fn set_blueprint(&mut self, blueprint: ContentBlueprint) {
        let content_changed = self.blueprint != blueprint;
        self.blueprint = blueprint;

        if content_changed {
            self.runtime = ContentRuntimeState::from_blueprint(&self.blueprint);
            self.control_active = false;
        }
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
            self.clamp_selection_to_page(height);
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
        self.clamp_selection_to_page(height);
    }

    pub fn current_page(&self) -> usize {
        self.runtime.current_page
    }

    pub fn selected_block(&self) -> usize {
        self.runtime.selected_block
    }

    pub fn has_selectable_blocks(&self, height: u16) -> bool {
        !self.current_page_block_indices(height).is_empty()
    }

    pub fn tick(&mut self) {
        self.runtime.spinner_tick = self.runtime.spinner_tick.wrapping_add(1);
    }

    fn clear_all_statuses(&mut self) {
        self.runtime.clear_statuses();
    }

    pub fn ensure_visible_selection(&mut self, height: u16) {
        self.clamp_selection_to_page(height);
    }

    pub fn handle_control_key(&mut self, key: Key) -> bool {
        if self
            .selected_block_state()
            .map(|state| matches!(state.status, OperationStatus::Running { .. }))
            .unwrap_or(false)
        {
            return false;
        }

        self.selected_block_control_mut()
            .map(|control| handle_control_key(control, key))
            .unwrap_or(false)
    }

    pub fn activate_selected_control(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        match self.selected_control_kind() {
            Some(
                SelectedControlKind::TextInput
                | SelectedControlKind::NumberInput
                | SelectedControlKind::Select
                | SelectedControlKind::LogOutput,
            ) => {
                if let Some(state) = self.selected_block_state_mut() {
                    state.snapshot = Some(state.control.clone());
                }
                self.control_active = true;
                None
            }
            Some(SelectedControlKind::Toggle) => {
                self.control_active = false;
                self.clear_selected_snapshot();
                self.toggle_selected_control(operation_id, screen_index)
            }
            Some(SelectedControlKind::ActionButton) => {
                self.control_active = false;
                self.clear_selected_snapshot();
                self.start_selected_action(operation_id, screen_index)
            }
            Some(SelectedControlKind::StaticData | SelectedControlKind::DynamicData) | None => None,
        }
    }

    pub fn confirm_control(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        self.control_active = false;
        self.confirm_selected_control(operation_id, screen_index)
    }

    pub fn cancel_control(&mut self) {
        if let Some(state) = self.selected_block_state_mut() {
            if let Some(snapshot) = state.snapshot.take() {
                state.control = snapshot;
            }
        }
        self.control_active = false;
    }

    pub fn is_control_active(&self) -> bool {
        self.control_active
    }

    pub fn active_control_uses_l_as_confirm(&self) -> bool {
        matches!(
            self.selected_control_kind(),
            Some(SelectedControlKind::Select | SelectedControlKind::LogOutput)
        )
    }

    pub fn active_control_uses_h_as_cancel(&self) -> bool {
        !matches!(
            self.selected_control_kind(),
            Some(SelectedControlKind::TextInput | SelectedControlKind::NumberInput)
        )
    }

    fn confirm_selected_control(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        let snapshot = self
            .selected_block_state_mut()
            .and_then(|state| state.snapshot.take());
        let block_index = self.runtime.selected_block;
        if self.block_is_running(block_index) {
            return None;
        }
        let Some(block) = self.selected_block_ref() else {
            return None;
        };

        if block.operation.is_some() {
            if let Some(original) = snapshot {
                let pending = self.block_control(block_index)?.clone();
                return self.start_operation(
                    operation_id,
                    screen_index,
                    block_index,
                    original,
                    pending,
                );
            }
            let control = self.block_control(block_index)?.clone();
            return self.start_operation(
                operation_id,
                screen_index,
                block_index,
                control.clone(),
                control,
            );
        }

        None
    }

    fn toggle_selected_control(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        let block_index = self.runtime.selected_block;
        if self.block_is_running(block_index) {
            return None;
        }
        let Some(block) = self.selected_block_ref() else {
            return None;
        };

        let mut pending = self.block_control(block_index)?.clone();
        let changed = handle_control_key(&mut pending, Key::Enter);
        if !changed {
            return None;
        }

        if block.operation.is_some() {
            let original = self.block_control(block_index)?.clone();
            self.start_operation(operation_id, screen_index, block_index, original, pending)
        } else {
            if let Some(control) = self.block_control_mut(block_index) {
                *control = pending;
            }
            None
        }
    }

    fn start_selected_action(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        let block_index = self.runtime.selected_block;
        if self.block_is_running(block_index) {
            return None;
        }
        let Some(_block) = self.selected_block_ref() else {
            return None;
        };

        let control = self.block_control(block_index)?.clone();
        self.start_operation(
            operation_id,
            screen_index,
            block_index,
            control.clone(),
            control,
        )
    }

    pub fn select_next_block(&mut self, height: u16) {
        let visible = self.current_page_block_indices(height);
        if visible.is_empty() {
            return;
        }

        if let Some(pos) = visible
            .iter()
            .position(|index| *index == self.runtime.selected_block)
        {
            if pos + 1 < visible.len() {
                self.runtime.selected_block = visible[pos + 1];
                self.control_active = false;
                self.clear_selected_snapshot();
            } else {
                let current_page = self.runtime.current_page;
                self.next_page(0, height);
                if self.runtime.current_page != current_page {
                    let next_visible = self.current_page_block_indices(height);
                    if let Some(first) = next_visible.first() {
                        self.runtime.selected_block = *first;
                        self.control_active = false;
                        self.clear_selected_snapshot();
                    }
                }
            }
        } else {
            self.runtime.selected_block = visible[0];
            self.control_active = false;
            self.clear_selected_snapshot();
        }
    }

    pub fn select_previous_block(&mut self, height: u16) {
        let visible = self.current_page_block_indices(height);
        if visible.is_empty() {
            return;
        }

        if let Some(pos) = visible
            .iter()
            .position(|index| *index == self.runtime.selected_block)
        {
            if pos > 0 {
                self.runtime.selected_block = visible[pos - 1];
                self.control_active = false;
                self.clear_selected_snapshot();
            } else if self.runtime.current_page > 0 {
                self.previous_page_with_height(height);
                let previous_visible = self.current_page_block_indices(height);
                if let Some(last) = previous_visible.last() {
                    self.runtime.selected_block = *last;
                    self.control_active = false;
                    self.clear_selected_snapshot();
                }
            }
        } else {
            self.runtime.selected_block = visible[0];
            self.control_active = false;
            self.clear_selected_snapshot();
        }
    }

    pub fn has_multiple_pages(&self, width: u16, height: u16) -> bool {
        self.total_pages(width, height) > 1
    }

    pub fn total_pages(&self, _width: u16, height: u16) -> usize {
        self.layout_pages(self.effective_height(height))
            .len()
            .max(1)
    }

    fn layout_pages(&self, height: u16) -> Vec<ContentPage> {
        let page_height = height.max(1) as usize;
        let mut pages = Vec::new();

        if self.blueprint.sections.is_empty() {
            return vec![ContentPage {
                subtitle: self.blueprint.title.clone(),
                blocks: Vec::new(),
            }];
        }
        let mut global_index = 0usize;
        let available_height = page_height.max(1);

        for section in &self.blueprint.sections {
            let subtitle = if section.subtitle.trim().is_empty() {
                self.blueprint.title.clone()
            } else {
                section.subtitle.clone()
            };

            if section.blocks.is_empty() {
                pages.push(ContentPage {
                    subtitle,
                    blocks: Vec::new(),
                });
                continue;
            }

            let mut current_page = ContentPage {
                subtitle: subtitle.clone(),
                blocks: Vec::new(),
            };
            let mut remaining = available_height.max(1);

            for block in &section.blocks {
                let block_height = block.row_height();
                let fits_current = block_height <= remaining;
                let has_blocks = !current_page.blocks.is_empty();

                if !fits_current && has_blocks {
                    pages.push(current_page);
                    current_page = ContentPage {
                        subtitle: subtitle.clone(),
                        blocks: Vec::new(),
                    };
                    remaining = available_height.max(1);
                }

                current_page.blocks.push(VisibleBlock {
                    index: global_index,
                    block: block.clone(),
                });
                global_index += 1;

                let consumed = block_height.min(remaining.max(1));
                remaining = remaining.saturating_sub(consumed);

                if remaining == 0 {
                    pages.push(current_page);
                    current_page = ContentPage {
                        subtitle: subtitle.clone(),
                        blocks: Vec::new(),
                    };
                    remaining = available_height.max(1);
                }
            }

            if !current_page.blocks.is_empty() {
                pages.push(current_page);
            }
        }

        pages
    }

    fn current_page_body(&self, _width: u16, height: u16) -> Option<ContentPage> {
        let pages = self.layout_pages(self.effective_height(height));
        let page_index = self.runtime.current_page.min(pages.len().saturating_sub(1));
        pages.get(page_index).cloned()
    }

    fn current_page_block_indices(&self, height: u16) -> Vec<usize> {
        self.current_page_body(0, height)
            .map(|page| page.blocks.into_iter().map(|block| block.index).collect())
            .unwrap_or_default()
    }

    fn clamp_selection_to_page(&mut self, height: u16) {
        let visible = self.current_page_block_indices(height);
        if visible.is_empty() {
            self.runtime.selected_block = 0;
            return;
        }

        if !visible.contains(&self.runtime.selected_block) {
            self.runtime.selected_block = visible[0];
            self.control_active = false;
            self.clear_selected_snapshot();
        }
    }

    fn selected_block_state(&self) -> Option<&crate::runtime::RuntimeFieldState> {
        self.runtime.field_state(self.runtime.selected_block)
    }

    fn selected_block_state_mut(&mut self) -> Option<&mut crate::runtime::RuntimeFieldState> {
        self.runtime.field_state_mut(self.runtime.selected_block)
    }

    fn clear_selected_snapshot(&mut self) {
        if let Some(state) = self.selected_block_state_mut() {
            state.snapshot = None;
        }
    }

    fn block_state(&self, index: usize) -> Option<&crate::runtime::RuntimeFieldState> {
        self.runtime.field_state(index)
    }

    fn block_state_mut(&mut self, index: usize) -> Option<&mut crate::runtime::RuntimeFieldState> {
        self.runtime.field_state_mut(index)
    }

    fn block_is_running(&self, index: usize) -> bool {
        self.block_state(index)
            .map(|state| matches!(state.status, OperationStatus::Running { .. }))
            .unwrap_or(false)
    }

    fn block_control(&self, index: usize) -> Option<&ContentControl> {
        self.block_state(index).map(|state| &state.control)
    }

    fn block_control_mut(&mut self, index: usize) -> Option<&mut ContentControl> {
        self.block_state_mut(index).map(|state| &mut state.control)
    }

    fn selected_block_control_mut(&mut self) -> Option<&mut ContentControl> {
        self.block_control_mut(self.runtime.selected_block)
    }

    fn start_operation(
        &mut self,
        operation_id: u64,
        screen_index: usize,
        block_index: usize,
        original_control: ContentControl,
        pending_control: ContentControl,
    ) -> Option<OperationRequest> {
        let command = self.block_mut_by_index(block_index)?.operation.clone()?;
        let state = self.block_state_mut(block_index)?;
        state.status = OperationStatus::Running {
            operation_id,
            started_at: std::time::Instant::now(),
            original_control,
            pending_control,
        };
        Some(OperationRequest {
            operation_id,
            screen_index,
            block_index,
            source: match command.source {
                OperationSource::ShellCommand(command) => {
                    ExecutorOperationSource::ShellCommand(command)
                }
                OperationSource::RegisteredAction(action) => {
                    ExecutorOperationSource::RegisteredAction(action)
                }
            },
            params: self.operation_params(),
            host: HashMap::new(),
            cwd: None,
            env: HashMap::new(),
            result_target: command.result_target,
        })
    }

    fn operation_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let page_scope = scope_slug(&self.blueprint.title);
        let mut index = 0usize;

        for section in &self.blueprint.sections {
            for block in &section.blocks {
                if let Some(id) = &block.id {
                    if let Some(control) = self.block_control(index) {
                        let value = control_value(control);
                        params.insert(id.clone(), value.clone());
                        params.insert(format!("screen.{id}"), value.clone());
                        if !page_scope.is_empty() {
                            params.insert(format!("{page_scope}.{id}"), value);
                        }
                    }
                }
                index += 1;
            }
        }

        params
    }

    fn apply_operation_result_to_block(&mut self, target_index: usize, result: &OperationResult) {
        let Some(state) = self.block_state(target_index).cloned() else {
            return;
        };
        let OperationStatus::Running {
            operation_id,
            original_control,
            pending_control,
            ..
        } = state.status
        else {
            return;
        };

        if operation_id != result.operation_id {
            return;
        }

        if let Some(state) = self.block_state_mut(target_index) {
            state.control = if result.success {
                pending_control
            } else {
                original_control
            };
            state.snapshot = None;
            state.status = if result.success {
                OperationStatus::Success
            } else {
                OperationStatus::Failure
            };
        }
    }

    fn selected_block_ref(&self) -> Option<&ContentBlock> {
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

    fn block_mut_by_index(&mut self, target_index: usize) -> Option<&mut ContentBlock> {
        let mut index = 0usize;
        for section in &mut self.blueprint.sections {
            for block in &mut section.blocks {
                if index == target_index {
                    return Some(block);
                }
                index += 1;
            }
        }
        None
    }

    fn block_index_by_id(&self, target_id: &str) -> Option<usize> {
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

    fn block_control_mut_by_id(&mut self, target_id: &str) -> Option<&mut ContentControl> {
        let index = self.block_index_by_id(target_id)?;
        self.block_control_mut(index)
    }

    pub fn apply_operation_result(&mut self, result: &OperationResult) {
        self.apply_operation_result_to_block(result.block_index, result);

        if let Some(target_id) = result.result_target.as_deref() {
            if let Some(control) = self.block_control_mut_by_id(target_id) {
                if let ContentControl::LogOutput(log) = control {
                    log.append_entry(Self::format_result_output(result));
                }
            }
        }
    }

    fn format_result_output(result: &OperationResult) -> String {
        let mut parts = Vec::new();
        parts.push(if result.success {
            "[success]".to_string()
        } else {
            "[failure]".to_string()
        });

        if !result.stdout.is_empty() {
            parts.push(result.stdout.clone());
        }

        if !result.stderr.is_empty() {
            parts.push(result.stderr.clone());
        }

        if parts.len() == 1 {
            parts.push(if result.success {
                "操作成功".to_string()
            } else {
                "操作失败".to_string()
            });
        }

        parts.join("\n")
    }

    fn selected_control_kind(&self) -> Option<SelectedControlKind> {
        self.block_control(self.runtime.selected_block)
            .map(|control| match control {
                ContentControl::TextInput(_) => SelectedControlKind::TextInput,
                ContentControl::NumberInput(_) => SelectedControlKind::NumberInput,
                ContentControl::Select(_) => SelectedControlKind::Select,
                ContentControl::Toggle(_) => SelectedControlKind::Toggle,
                ContentControl::ActionButton(_) => SelectedControlKind::ActionButton,
                ContentControl::StaticData(_) => SelectedControlKind::StaticData,
                ContentControl::DynamicData(_) => SelectedControlKind::DynamicData,
                ContentControl::LogOutput(_) => SelectedControlKind::LogOutput,
            })
    }

    fn gutter_width(&self, width: u16) -> u16 {
        Self::GUTTER_WIDTH.min(width.saturating_sub(1).max(1))
    }

    fn page_label(&self, height: u16, page: usize) -> String {
        let pages = self.layout_pages(self.effective_height(height));
        pages
            .get(page)
            .map(|page| page.subtitle.clone())
            .unwrap_or_else(|| self.blueprint.title.clone())
    }

    fn truncated_page_label(&self, height: u16, page: usize, max_width: usize) -> String {
        let label = self.page_label(height, page);
        let max_width = max_width.min(Self::PAGE_LABEL_MAX_CHARS);
        Self::truncate_chars(&label, max_width)
    }

    fn pagination_rows(&self, height: u16) -> Vec<(char, Color, Option<String>)> {
        let height = self.effective_height(height);
        if height == 0 {
            return Vec::new();
        }

        let total_pages = self.total_pages(0, height);
        let visible_pages = (height / Self::PAGE_SLOT_HEIGHT).max(1) as usize;
        let total_visible = total_pages.min(visible_pages);
        let current_page = self.runtime.current_page.min(total_pages.saturating_sub(1));
        let start_page = current_page.saturating_add(1).saturating_sub(total_visible);
        let end_page = (start_page + total_visible).min(total_pages);
        let slots = (total_visible as u16 * Self::PAGE_SLOT_HEIGHT) as usize;

        let inactive = ('│', Color::Rgb(170, 170, 170), None);
        let active = ('┃', Color::White, None);

        if total_pages <= 1 {
            let mut rows = vec![active.clone(); slots];
            if rows.len() > 1 {
                rows[1] = (
                    '┃',
                    Color::White,
                    Some(self.truncated_page_label(
                        height,
                        0,
                        Self::GUTTER_WIDTH.saturating_sub(3) as usize,
                    )),
                );
            }
            return rows;
        }

        let mut rows = vec![inactive.clone(); slots];
        for page in start_page..end_page {
            let local_page = page - start_page;
            let row_start = local_page * Self::PAGE_SLOT_HEIGHT as usize;
            let row_mid = row_start + 1;
            let row_end = row_start + Self::PAGE_SLOT_HEIGHT as usize;
            let glyph = if page == current_page {
                active.clone()
            } else {
                inactive.clone()
            };

            for row in rows.iter_mut().take(row_end.min(slots)).skip(row_start) {
                *row = glyph.clone();
            }

            if page == current_page && row_mid < rows.len() {
                rows[row_mid] = (
                    '┃',
                    Color::White,
                    Some(self.truncated_page_label(
                        height,
                        page,
                        Self::GUTTER_WIDTH.saturating_sub(3) as usize,
                    )),
                );
            }
        }

        rows
    }

    fn effective_height(&self, height: u16) -> u16 {
        height.saturating_sub(Self::TOP_PADDING)
    }

    fn render_page(&self, f: &mut Frame, rect: Rect, page: &ContentPage) {
        let content_width = rect.width as usize;
        if content_width == 0 || rect.height == 0 {
            return;
        }

        let blocks_rect = rect;
        if blocks_rect.height == 0 {
            return;
        }

        let content_width = blocks_rect.width.saturating_sub(2);
        let distributable = content_width.saturating_sub(Self::COLUMN_GAP);
        let text_width = (distributable * 3 / 10).max(1);
        let control_width = distributable.saturating_sub(text_width);

        let mut cursor_y = blocks_rect.y;
        for block in &page.blocks {
            if cursor_y >= blocks_rect.y + blocks_rect.height {
                break;
            }

            let intended_height = block.row_height() as u16;
            let block_height = intended_height.min(blocks_rect.y + blocks_rect.height - cursor_y);
            let block_rect = Rect::new(blocks_rect.x, cursor_y, blocks_rect.width, block_height);
            self.render_content_block(
                f,
                block_rect,
                &block.block,
                block.index,
                text_width,
                control_width,
                block.index == self.runtime.selected_block,
            );
            cursor_y = cursor_y.saturating_add(intended_height);
        }

        while cursor_y < blocks_rect.y + blocks_rect.height {
            self.write_text(
                f,
                blocks_rect.x,
                cursor_y,
                "",
                content_width as usize,
                Color::White,
                None,
            );
            cursor_y += 1;
        }
    }

    fn render_content_block(
        &self,
        f: &mut Frame,
        rect: Rect,
        block: &ContentBlock,
        block_index: usize,
        text_width: u16,
        control_width: u16,
        selected: bool,
    ) {
        if rect.height == 0 || rect.width == 0 {
            return;
        }

        let content_rect = rect;
        let baseline_y = content_rect
            .y
            .saturating_add(1)
            .min(content_rect.y + content_rect.height.saturating_sub(1));
        f.buffer_mut()[(content_rect.x, baseline_y)]
            .set_char(if selected && self.focused { '>' } else { ' ' })
            .set_fg(Color::Yellow);

        let text_rect = Rect::new(
            content_rect.x + 2,
            content_rect.y,
            text_width,
            content_rect.height,
        );
        let control_x = content_rect
            .x
            .saturating_add(2)
            .saturating_add(text_width)
            .saturating_add(Self::COLUMN_GAP);
        let label_width = text_rect.width as usize;
        let truncated = Self::truncate_to_width(block.label.as_str(), label_width.max(1));
        self.write_text(
            f,
            text_rect.x,
            baseline_y,
            &truncated,
            label_width.max(1),
            Color::Cyan,
            if selected && self.focused {
                Some(Style::default().add_modifier(Modifier::BOLD))
            } else {
                None
            },
        );

        let control_rect = Rect::new(
            control_x,
            content_rect.y,
            control_width,
            content_rect.height,
        );
        let feedback = self.feedback_for(block_index);
        render_control(
            self.block_control(block_index).unwrap_or(&block.control),
            control_rect,
            f.buffer_mut(),
            selected && self.focused,
            selected && self.focused && self.control_active,
            feedback,
        );
    }

    fn feedback_for(&self, block_index: usize) -> ControlFeedback {
        match self
            .block_state(block_index)
            .map(|state| &state.status)
            .unwrap_or(&OperationStatus::Idle)
        {
            OperationStatus::Idle => ControlFeedback::Idle,
            OperationStatus::Running { .. } => ControlFeedback::Running(self.runtime.spinner_tick),
            OperationStatus::Success => ControlFeedback::Success,
            OperationStatus::Failure => ControlFeedback::Failure,
        }
    }

    fn truncate_chars(text: &str, max_width: usize) -> String {
        if max_width == 0 {
            return String::new();
        }

        let char_count = text.chars().count();
        if char_count <= max_width {
            return text.to_string();
        }

        if max_width == 1 {
            return "…".to_string();
        }

        text.chars().take(max_width - 1).collect::<String>() + "…"
    }

    fn truncate_to_width(text: &str, max_width: usize) -> String {
        if max_width == 0 {
            return String::new();
        }

        if UnicodeWidthStr::width(text) <= max_width {
            return text.to_string();
        }

        if max_width == 1 {
            return "…".to_string();
        }

        let target = max_width.saturating_sub(1);
        let mut out = String::new();
        let mut width = 0usize;

        for ch in text.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if width + ch_width > target {
                break;
            }
            out.push(ch);
            width += ch_width;
        }

        out.push('…');
        out
    }

    fn write_text(
        &self,
        f: &mut Frame,
        x: u16,
        y: u16,
        text: &str,
        max_width: usize,
        fg: Color,
        style: Option<Style>,
    ) {
        let style = style.unwrap_or_default().fg(fg);
        let mut cursor_x = x;
        let mut used_width = 0usize;

        for ch in text.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if ch_width == 0 {
                continue;
            }
            if used_width + ch_width > max_width {
                break;
            }

            f.buffer_mut()[(cursor_x, y)].set_char(ch).set_style(style);

            cursor_x = cursor_x.saturating_add(ch_width as u16);
            used_width += ch_width;
        }
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
        let gutter_width = self.gutter_width(rect.width);
        let rows = self.pagination_rows(rect.height);
        for (idx, (glyph, color, label)) in rows.into_iter().enumerate() {
            let y = rect.y + idx as u16 + Self::TOP_PADDING;
            if y >= rect.y + rect.height {
                break;
            }
            f.buffer_mut()[(rect.x, y)].set_char(glyph).set_fg(color);

            if let Some(label) = label {
                let label_x = rect.x.saturating_add(2);
                let available = gutter_width.saturating_sub(2) as usize;
                self.write_text(f, label_x, y, &label, available, Color::White, None);
            }
        }

        let content_rect = Rect::new(
            rect.x + gutter_width.min(rect.width),
            rect.y.saturating_add(Self::TOP_PADDING),
            rect.width.saturating_sub(gutter_width),
            self.effective_height(rect.height),
        );

        if let Some(page) = self.current_page_body(rect.width, rect.height) {
            self.render_page(f, content_rect, &page);
        } else {
            let widget = Paragraph::new("No content").style(Style::default().fg(Color::White));
            f.render_widget(widget, content_rect);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContentPanel;
    use crate::executor::OperationResult;
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
                ContentBlock::static_data("版本", "v0.1.0"),
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
        panel.set_blueprint(
            ContentBlueprint::new("Root")
                .with_sections(vec![ContentSection::new("一个超过十个字符的子标题名称")
                    .with_blocks(vec![ContentBlock::static_data("描述", "value")])]),
        );

        let label = panel.truncated_page_label(6, 0, 32);
        assert_eq!(label.chars().count(), 10);
        assert!(label.ends_with('…'));
    }

    #[test]
    fn content_panel_shows_pagination_gutter_for_single_page() {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(
            ContentBlueprint::new("Root").with_sections(vec![ContentSection::new("概览")
                .with_blocks(vec![ContentBlock::static_data("One", "value")])]),
        );

        let glyphs: Vec<char> = panel
            .pagination_rows(8)
            .into_iter()
            .map(|row| row.0)
            .collect();

        assert_eq!(glyphs, vec!['┃', '┃', '┃']);
    }

    #[test]
    fn content_panel_dims_inactive_pagination_segments() {
        let panel = panel_with_sections();

        let cells = panel.pagination_rows(7);

        assert!(cells
            .iter()
            .any(|(glyph, color, _)| *glyph == '┃' && *color == Color::White));
        assert!(cells
            .iter()
            .any(|(glyph, color, _)| *glyph == '│' && *color == Color::Rgb(170, 170, 170)));
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
            super::ContentControl::TextInput(control) => assert_eq!(control.value, "demox"),
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
            super::ContentControl::Toggle(control) => assert!(!control.on),
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
            super::ContentControl::NumberInput(control) => assert_eq!(control.value, "80809"),
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
            super::ContentControl::Select(control) => assert_eq!(control.selected, 0),
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
            super::ContentControl::LogOutput(log) => assert!(log.content.contains("done")),
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
            super::ContentControl::LogOutput(log) => {
                assert!(log.content.contains("previous line"));
                assert!(log.content.contains("[success]"));
                assert!(log.content.contains("done"));
            }
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
        assert_eq!(
            request.params.get("project_name").map(String::as_str),
            Some("tui01")
        );
        assert_eq!(
            request.params.get("server_port").map(String::as_str),
            Some("3000")
        );
        assert_eq!(
            request
                .params
                .get("screen.project_name")
                .map(String::as_str),
            Some("tui01")
        );
        assert_eq!(
            request.params.get("root.server_port").map(String::as_str),
            Some("3000")
        );
    }

    #[test]
    fn page_scope_slug_normalizes_blueprint_title() {
        assert_eq!(super::scope_slug("Workspace"), "workspace");
        assert_eq!(super::scope_slug("Theme Settings"), "theme_settings");
        assert_eq!(super::scope_slug("中文 / Mixed-Page"), "mixed_page");
    }
}
