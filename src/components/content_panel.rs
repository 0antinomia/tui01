//! 右下内容区域组件

use crate::components::{
    ActionButtonControl, Component, ControlFeedback, ControlKind, DataDisplayControl,
    LogOutputControl, NumberInputControl, SelectControl, TextInputControl, ToggleControl,
};
use crate::executor::{OperationRequest, OperationResult};
use crate::event::Key;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};
use std::time::Instant;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentBlueprint {
    pub title: String,
    pub sections: Vec<ContentSection>,
}

impl ContentBlueprint {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
        }
    }

    pub fn with_sections(mut self, sections: Vec<ContentSection>) -> Self {
        self.sections = sections;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentSection {
    pub subtitle: String,
    pub blocks: Vec<ContentBlock>,
}

impl ContentSection {
    pub fn new(subtitle: impl Into<String>) -> Self {
        Self {
            subtitle: subtitle.into(),
            blocks: Vec::new(),
        }
    }

    pub fn with_blocks(mut self, blocks: Vec<ContentBlock>) -> Self {
        self.blocks = blocks;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentBlock {
    pub id: Option<String>,
    pub label: String,
    pub control: ContentControl,
    pub height_units: u16,
    pub operation: Option<OperationSpec>,
    pub status: OperationStatus,
}

impl ContentBlock {
    pub fn description(text: impl Into<String>) -> Self {
        Self {
            id: None,
            label: text.into(),
            control: ContentControl::StaticText(String::new()),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn text_input(
        label: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::TextInput(TextInputControl::new(value, placeholder)),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn select(
        label: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<String>>,
        selected: usize,
    ) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::Select(SelectControl::new(options, selected)),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn toggle(label: impl Into<String>, on: bool) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::Toggle(ToggleControl::new(on)),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn number_input(
        label: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::NumberInput(NumberInputControl::new(value, placeholder)),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn action_button(label: impl Into<String>, button_label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::ActionButton(ActionButtonControl::new(button_label)),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn refresh_button(label: impl Into<String>, button_label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::ActionButton(ActionButtonControl::refresh(button_label)),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn static_data(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::StaticData(DataDisplayControl::new(value)),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn dynamic_data(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::DynamicData(DataDisplayControl::new(value)),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn log_output(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::LogOutput(LogOutputControl::new(content)),
            height_units: 4,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn static_text(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::StaticText(value.into()),
            height_units: 1,
            operation: None,
            status: OperationStatus::Idle,
        }
    }

    pub fn with_toggle_labels(
        mut self,
        on_label: impl Into<String>,
        off_label: impl Into<String>,
    ) -> Self {
        if let ContentControl::Toggle(toggle) = &mut self.control {
            *toggle = toggle.clone().labels(on_label, off_label);
        }
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_height_units(mut self, height_units: u16) -> Self {
        self.height_units = height_units.max(1);
        self
    }

    pub fn with_operation_success(mut self, duration_ms: u64) -> Self {
        let target = self.operation.as_ref().and_then(|spec| spec.result_target.clone());
        let mut spec = OperationSpec::simulated_success(duration_ms);
        spec.result_target = target;
        self.operation = Some(spec);
        self
    }

    pub fn with_operation_failure(mut self, duration_ms: u64) -> Self {
        let target = self.operation.as_ref().and_then(|spec| spec.result_target.clone());
        let mut spec = OperationSpec::simulated_failure(duration_ms);
        spec.result_target = target;
        self.operation = Some(spec);
        self
    }

    pub fn with_shell_command(mut self, command: impl Into<String>) -> Self {
        let target = self.operation.as_ref().and_then(|spec| spec.result_target.clone());
        let mut spec = OperationSpec::shell(command);
        spec.result_target = target;
        self.operation = Some(spec);
        self
    }

    pub fn with_result_target(mut self, target_id: impl Into<String>) -> Self {
        let target_id = target_id.into();
        let spec = self.operation.get_or_insert_with(|| OperationSpec::shell("true"));
        spec.result_target = Some(target_id);
        self
    }

    fn row_height(&self) -> usize {
        self.height_units.max(1) as usize * 3
    }

    fn start_operation(
        &mut self,
        operation_id: u64,
        screen_index: usize,
        block_index: usize,
    ) -> Option<OperationRequest> {
        let original_control = self.control.clone();
        let pending_control = self.control.clone();
        self.start_operation_with_controls(
            operation_id,
            screen_index,
            block_index,
            original_control,
            pending_control,
        )
    }

    fn start_operation_with_controls(
        &mut self,
        operation_id: u64,
        screen_index: usize,
        block_index: usize,
        original_control: ContentControl,
        pending_control: ContentControl,
    ) -> Option<OperationRequest> {
        let Some(spec) = self.operation.clone() else {
            return None;
        };

        self.status = OperationStatus::Running {
            operation_id,
            started_at: Instant::now(),
            original_control,
            pending_control: pending_control.clone(),
        };
        Some(OperationRequest {
            operation_id,
            screen_index,
            block_index,
            command: spec.command,
            result_target: spec.result_target,
        })
    }

    fn is_running(&self) -> bool {
        matches!(self.status, OperationStatus::Running { .. })
    }

    fn apply_operation_result(&mut self, result: &OperationResult) {
        let OperationStatus::Running {
            operation_id,
            ref original_control,
            ref pending_control,
            ..
        } = self.status
        else {
            return;
        };

        if operation_id != result.operation_id {
            return;
        }

        if result.success {
            self.control = pending_control.clone();
            self.status = OperationStatus::Success;
        } else {
            self.control = original_control.clone();
            self.status = OperationStatus::Failure;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationSpec {
    pub command: String,
    pub result_target: Option<String>,
}

impl OperationSpec {
    pub fn shell(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            result_target: None,
        }
    }

    pub fn simulated_success(duration_ms: u64) -> Self {
        let seconds = duration_ms as f64 / 1000.0;
        Self::shell(format!(
            "sleep {seconds:.3}; printf '操作成功\\n'; exit 0"
        ))
    }

    pub fn simulated_failure(duration_ms: u64) -> Self {
        let seconds = duration_ms as f64 / 1000.0;
        Self::shell(format!(
            "sleep {seconds:.3}; printf '操作失败\\n' >&2; exit 1"
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationStatus {
    Idle,
    Running {
        operation_id: u64,
        started_at: Instant,
        original_control: ContentControl,
        pending_control: ContentControl,
    },
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentControl {
    TextInput(TextInputControl),
    NumberInput(NumberInputControl),
    Select(SelectControl),
    Toggle(ToggleControl),
    ActionButton(ActionButtonControl),
    StaticData(DataDisplayControl),
    DynamicData(DataDisplayControl),
    LogOutput(LogOutputControl),
    StaticText(String),
}

impl ContentControl {
    fn render(
        &self,
        area: Rect,
        buf: &mut ratatui::buffer::Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    ) {
        match self {
            Self::TextInput(control) => {
                ControlKind::TextInput(control.clone()).render(area, buf, selected, active, feedback)
            }
            Self::NumberInput(control) => ControlKind::NumberInput(control.clone()).render(
                area, buf, selected, active, feedback,
            ),
            Self::Select(control) => {
                ControlKind::Select(control.clone()).render(area, buf, selected, active, feedback)
            }
            Self::Toggle(control) => {
                ControlKind::Toggle(control.clone()).render(area, buf, selected, active, feedback)
            }
            Self::ActionButton(control) => ControlKind::ActionButton(control.clone()).render(
                area, buf, selected, active, feedback,
            ),
            Self::StaticData(control) => {
                ControlKind::StaticData(control.clone()).render(area, buf, selected, active, feedback)
            }
            Self::DynamicData(control) => ControlKind::DynamicData(control.clone()).render(
                area, buf, selected, active, feedback,
            ),
            Self::LogOutput(control) => {
                ControlKind::LogOutput(control.clone()).render(area, buf, selected, active, feedback)
            }
            Self::StaticText(text) => {
                ControlKind::StaticText(text.clone()).render(area, buf, selected, active, feedback)
            }
        }
    }

    fn handle_key(&mut self, key: Key) -> bool {
        match self {
            Self::TextInput(control) => control.handle_key(key),
            Self::NumberInput(control) => control.handle_key(key),
            Self::Select(control) => control.handle_key(key),
            Self::Toggle(control) => control.handle_key(key),
            Self::ActionButton(control) => control.handle_key(key),
            Self::StaticData(_) | Self::DynamicData(_) | Self::LogOutput(_) => false,
            Self::StaticText(_) => false,
        }
    }
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
    StaticText,
}

impl VisibleBlock {
    fn row_height(&self) -> usize {
        self.block.row_height()
    }
}

pub struct ContentPanel {
    blueprint: ContentBlueprint,
    current_page: usize,
    selected_block: usize,
    focused: bool,
    control_active: bool,
    control_snapshot: Option<ContentControl>,
    spinner_tick: usize,
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
            current_page: 0,
            selected_block: 0,
            focused: false,
            control_active: false,
            control_snapshot: None,
            spinner_tick: 0,
        }
    }

    pub fn set_blueprint(&mut self, blueprint: ContentBlueprint) {
        let content_changed = self.blueprint != blueprint;
        self.blueprint = blueprint;

        if content_changed {
            self.clear_all_statuses();
            self.current_page = 0;
            self.selected_block = 0;
            self.control_active = false;
            self.control_snapshot = None;
            self.spinner_tick = 0;
        }
    }

    pub fn blueprint(&self) -> &ContentBlueprint {
        &self.blueprint
    }

    pub fn set_content(
        &mut self,
        title: impl Into<String>,
        summary: impl Into<String>,
        notes: impl IntoIterator<Item = impl Into<String>>,
    ) {
        let title = title.into();
        let summary = summary.into();
        let notes: Vec<String> = notes.into_iter().map(Into::into).collect();

        let mut sections = Vec::new();
        if !summary.trim().is_empty() {
            sections.push(
                ContentSection::new("概览")
                    .with_blocks(vec![ContentBlock::description(summary.clone())]),
            );
        }

        if !notes.is_empty() {
            sections.push(ContentSection::new("细项").with_blocks(
                notes.into_iter().map(ContentBlock::description).collect(),
            ));
        }

        if sections.is_empty() {
            sections.push(
                ContentSection::new("概览")
                    .with_blocks(vec![ContentBlock::description("当前没有可展示内容")]),
            );
        }

        self.set_blueprint(ContentBlueprint::new(title).with_sections(sections));
    }

    pub fn next_page(&mut self, width: u16, height: u16) {
        let total_pages = self.total_pages(width, height);
        if self.current_page + 1 < total_pages {
            self.clear_all_statuses();
            self.current_page += 1;
            self.control_active = false;
            self.control_snapshot = None;
            self.clamp_selection_to_page(height);
        }
    }

    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.clear_all_statuses();
            self.current_page -= 1;
            self.control_active = false;
            self.control_snapshot = None;
        }
    }

    pub fn previous_page_with_height(&mut self, height: u16) {
        self.previous_page();
        self.clamp_selection_to_page(height);
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn selected_block(&self) -> usize {
        self.selected_block
    }

    pub fn has_selectable_blocks(&self, height: u16) -> bool {
        !self.current_page_block_indices(height).is_empty()
    }

    pub fn tick(&mut self) {
        self.spinner_tick = self.spinner_tick.wrapping_add(1);
    }

    fn clear_all_statuses(&mut self) {
        for section in &mut self.blueprint.sections {
            for block in &mut section.blocks {
                block.status = OperationStatus::Idle;
            }
        }
    }

    pub fn ensure_visible_selection(&mut self, height: u16) {
        self.clamp_selection_to_page(height);
    }

    pub fn handle_control_key(&mut self, key: Key) -> bool {
        if self
            .selected_block_ref()
            .map(|block| block.is_running())
            .unwrap_or(false)
        {
            return false;
        }

        self.selected_block_mut()
            .map(|block| block.control.handle_key(key))
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
                self.control_snapshot = self.selected_block_ref().map(|block| block.control.clone());
                self.control_active = true;
                None
            }
            Some(SelectedControlKind::Toggle) => {
                self.control_active = false;
                self.control_snapshot = None;
                self.toggle_selected_control(operation_id, screen_index)
            }
            Some(SelectedControlKind::ActionButton) => {
                self.control_active = false;
                self.control_snapshot = None;
                self.start_selected_action(operation_id, screen_index)
            }
            Some(
                SelectedControlKind::StaticData
                | SelectedControlKind::DynamicData
                | SelectedControlKind::StaticText,
            )
            | None => None,
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
        if let Some(snapshot) = self.control_snapshot.take() {
            if let Some(block) = self.selected_block_mut() {
                block.control = snapshot;
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
        let snapshot = self.control_snapshot.take();
        let block_index = self.selected_block;
        let Some(block) = self.selected_block_mut() else {
            return None;
        };

        if block.is_running() {
            return None;
        }

        if block.operation.is_some() {
            if let Some(original) = snapshot {
                let pending = block.control.clone();
                return block.start_operation_with_controls(
                    operation_id,
                    screen_index,
                    block_index,
                    original,
                    pending,
                );
            }
            return block.start_operation(operation_id, screen_index, block_index);
        }

        None
    }

    fn toggle_selected_control(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        let block_index = self.selected_block;
        let Some(block) = self.selected_block_mut() else {
            return None;
        };

        if block.is_running() {
            return None;
        }

        let mut pending = block.control.clone();
        let changed = pending.handle_key(Key::Enter);
        if !changed {
            return None;
        }

        if block.operation.is_some() {
            let original = block.control.clone();
            block.start_operation_with_controls(
                operation_id,
                screen_index,
                block_index,
                original,
                pending,
            )
        } else {
            block.control = pending;
            None
        }
    }

    fn start_selected_action(
        &mut self,
        operation_id: u64,
        screen_index: usize,
    ) -> Option<OperationRequest> {
        let block_index = self.selected_block;
        let Some(block) = self.selected_block_mut() else {
            return None;
        };

        if block.is_running() {
            return None;
        }

        block.start_operation(operation_id, screen_index, block_index)
    }

    pub fn select_next_block(&mut self, height: u16) {
        let visible = self.current_page_block_indices(height);
        if visible.is_empty() {
            return;
        }

        if let Some(pos) = visible.iter().position(|index| *index == self.selected_block) {
            if pos + 1 < visible.len() {
                self.selected_block = visible[pos + 1];
                self.control_active = false;
                self.control_snapshot = None;
            } else {
                let current_page = self.current_page;
                self.next_page(0, height);
                if self.current_page != current_page {
                    let next_visible = self.current_page_block_indices(height);
                    if let Some(first) = next_visible.first() {
                        self.selected_block = *first;
                        self.control_active = false;
                        self.control_snapshot = None;
                    }
                }
            }
        } else {
            self.selected_block = visible[0];
            self.control_active = false;
            self.control_snapshot = None;
        }
    }

    pub fn select_previous_block(&mut self, height: u16) {
        let visible = self.current_page_block_indices(height);
        if visible.is_empty() {
            return;
        }

        if let Some(pos) = visible.iter().position(|index| *index == self.selected_block) {
            if pos > 0 {
                self.selected_block = visible[pos - 1];
                self.control_active = false;
                self.control_snapshot = None;
            } else if self.current_page > 0 {
                self.previous_page_with_height(height);
                let previous_visible = self.current_page_block_indices(height);
                if let Some(last) = previous_visible.last() {
                    self.selected_block = *last;
                    self.control_active = false;
                    self.control_snapshot = None;
                }
            }
        } else {
            self.selected_block = visible[0];
            self.control_active = false;
            self.control_snapshot = None;
        }
    }

    pub fn has_multiple_pages(&self, width: u16, height: u16) -> bool {
        self.total_pages(width, height) > 1
    }

    pub fn total_pages(&self, _width: u16, height: u16) -> usize {
        self.layout_pages(self.effective_height(height)).len().max(1)
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
        let page_index = self.current_page.min(pages.len().saturating_sub(1));
        pages.get(page_index).cloned()
    }

    fn current_page_block_indices(&self, height: u16) -> Vec<usize> {
        self.current_page_body(0, height)
            .map(|page| {
                page.blocks
                    .into_iter()
                    .map(|block| block.index)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn clamp_selection_to_page(&mut self, height: u16) {
        let visible = self.current_page_block_indices(height);
        if visible.is_empty() {
            self.selected_block = 0;
            return;
        }

        if !visible.contains(&self.selected_block) {
            self.selected_block = visible[0];
            self.control_active = false;
            self.control_snapshot = None;
        }
    }

    fn selected_block_ref(&self) -> Option<&ContentBlock> {
        let mut index = 0usize;
        for section in &self.blueprint.sections {
            for block in &section.blocks {
                if index == self.selected_block {
                    return Some(block);
                }
                index += 1;
            }
        }
        None
    }

    fn selected_block_mut(&mut self) -> Option<&mut ContentBlock> {
        let mut index = 0usize;
        for section in &mut self.blueprint.sections {
            for block in &mut section.blocks {
                if index == self.selected_block {
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

    fn block_mut_by_id(&mut self, target_id: &str) -> Option<&mut ContentBlock> {
        for section in &mut self.blueprint.sections {
            for block in &mut section.blocks {
                if block.id.as_deref() == Some(target_id) {
                    return Some(block);
                }
            }
        }
        None
    }

    pub fn apply_operation_result(&mut self, result: &OperationResult) {
        if let Some(block) = self.block_mut_by_index(result.block_index) {
            block.apply_operation_result(result);
        }

        if let Some(target_id) = result.result_target.as_deref() {
            if let Some(block) = self.block_mut_by_id(target_id) {
                if let ContentControl::LogOutput(log) = &mut block.control {
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
        self.blueprint
            .sections
            .iter()
            .flat_map(|section| section.blocks.iter())
            .nth(self.selected_block)
            .map(|block| match block.control {
                ContentControl::TextInput(_) => SelectedControlKind::TextInput,
                ContentControl::NumberInput(_) => SelectedControlKind::NumberInput,
                ContentControl::Select(_) => SelectedControlKind::Select,
                ContentControl::Toggle(_) => SelectedControlKind::Toggle,
                ContentControl::ActionButton(_) => SelectedControlKind::ActionButton,
                ContentControl::StaticData(_) => SelectedControlKind::StaticData,
                ContentControl::DynamicData(_) => SelectedControlKind::DynamicData,
                ContentControl::LogOutput(_) => SelectedControlKind::LogOutput,
                ContentControl::StaticText(_) => SelectedControlKind::StaticText,
            })
    }

    fn gutter_width(&self, width: u16) -> u16 {
        Self::GUTTER_WIDTH.min(width.saturating_sub(1).max(1))
    }

    fn page_label(&self, height: u16, page: usize) -> String {
        let pages = self.layout_pages(self.effective_height(height));
        pages.get(page)
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
        let current_page = self.current_page.min(total_pages.saturating_sub(1));
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
                text_width,
                control_width,
                block.index == self.selected_block,
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
            .set_char(if selected && self.focused {
                '>'
            } else {
                ' '
            })
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

        let control_rect = Rect::new(control_x, content_rect.y, control_width, content_rect.height);
        let feedback = self.feedback_for(&block.status);
        block.control.render(
            control_rect,
            f.buffer_mut(),
            selected && self.focused,
            selected && self.focused && self.control_active,
            feedback,
        );
    }

    fn feedback_for(&self, status: &OperationStatus) -> ControlFeedback {
        match status {
            OperationStatus::Idle => ControlFeedback::Idle,
            OperationStatus::Running { .. } => ControlFeedback::Running(self.spinner_tick),
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

            f.buffer_mut()[(cursor_x, y)]
                .set_char(ch)
                .set_style(style);

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
        self.control_snapshot = None;
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
    use super::{
        ContentBlock, ContentBlueprint, ContentPanel, ContentSection,
    };
    use crate::executor::OperationResult;
    use ratatui::style::Color;

    fn panel_with_sections() -> ContentPanel {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(
            ContentBlueprint::new("Root").with_sections(vec![
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
            ]),
        );
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
            ContentBlueprint::new("Root").with_sections(vec![ContentSection::new(
                "一个超过十个字符的子标题名称",
            )
            .with_blocks(vec![ContentBlock::description("描述")])]),
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
                .with_blocks(vec![ContentBlock::description("One")])]),
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
    fn legacy_set_content_builds_default_sections() {
        let mut panel = ContentPanel::new();
        panel.set_content("Title", "Summary", ["One", "Two"]);

        assert_eq!(panel.blueprint.sections.len(), 2);
        assert_eq!(panel.blueprint.sections[0].subtitle, "概览");
        assert_eq!(panel.blueprint.sections[1].subtitle, "细项");
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

        match &panel.blueprint.sections[0].blocks[0].control {
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

        match &panel.blueprint.sections[0].blocks[1].control {
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

        match &panel.blueprint.sections[1].blocks[1].control {
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

        match &panel.blueprint.sections[1].blocks[0].control {
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
        panel.set_blueprint(
            ContentBlueprint::new("Root").with_sections(vec![ContentSection::new("动作").with_blocks(vec![
                ContentBlock::action_button("执行同步", "执行")
                    .with_id("run_sync")
                    .with_result_target("sync_log")
                    .with_shell_command("printf 'done\\n'"),
                ContentBlock::log_output("输出", "等待操作").with_id("sync_log"),
            ])]),
        );

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

        match &panel.blueprint.sections[0].blocks[1].control {
            super::ContentControl::LogOutput(log) => assert!(log.content.contains("done")),
            _ => panic!("expected log output"),
        }
    }

    #[test]
    fn operation_result_appends_to_existing_log_output() {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(
            ContentBlueprint::new("Root").with_sections(vec![ContentSection::new("动作").with_blocks(vec![
                ContentBlock::action_button("执行同步", "执行")
                    .with_id("run_sync")
                    .with_result_target("sync_log")
                    .with_shell_command("printf 'done\\n'"),
                ContentBlock::log_output("输出", "previous line").with_id("sync_log"),
            ])]),
        );

        panel.apply_operation_result(&OperationResult {
            operation_id: 12,
            screen_index: 0,
            block_index: 0,
            result_target: Some("sync_log".to_string()),
            success: true,
            stdout: "done".to_string(),
            stderr: String::new(),
        });

        match &panel.blueprint.sections[0].blocks[1].control {
            super::ContentControl::LogOutput(log) => {
                assert!(log.content.contains("previous line"));
                assert!(log.content.contains("[success]"));
                assert!(log.content.contains("done"));
            }
            _ => panic!("expected log output"),
        }
    }
}
