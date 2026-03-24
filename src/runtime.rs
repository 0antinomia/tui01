//! 与界面渲染解耦的运行时页面状态定义。

use crate::components::{
    ActionButtonControl, DataDisplayControl, LogOutputControl, NumberInputControl, SelectControl,
    TextInputControl, ToggleControl,
};
use std::time::Instant;

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
}

impl ContentBlock {
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
        }
    }

    pub fn toggle(label: impl Into<String>, on: bool) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::Toggle(ToggleControl::new(on)),
            height_units: 1,
            operation: None,
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
        }
    }

    pub fn action_button(label: impl Into<String>, button_label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::ActionButton(ActionButtonControl::new(button_label)),
            height_units: 1,
            operation: None,
        }
    }

    pub fn refresh_button(label: impl Into<String>, button_label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::ActionButton(ActionButtonControl::refresh(button_label)),
            height_units: 1,
            operation: None,
        }
    }

    pub fn static_data(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::StaticData(DataDisplayControl::new(value)),
            height_units: 1,
            operation: None,
        }
    }

    pub fn dynamic_data(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::DynamicData(DataDisplayControl::new(value)),
            height_units: 1,
            operation: None,
        }
    }

    pub fn log_output(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::LogOutput(LogOutputControl::new(content)),
            height_units: 4,
            operation: None,
        }
    }

    pub fn log_output_from_file(
        label: impl Into<String>,
        path: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ContentControl::LogOutput(LogOutputControl::from_file(path)),
            height_units: 4,
            operation: None,
        }
    }

    pub fn with_log_tail_lines(mut self, tail_lines: usize) -> Self {
        if let ContentControl::LogOutput(control) = &mut self.control {
            control.set_tail_lines(tail_lines);
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
        let target = self
            .operation
            .as_ref()
            .and_then(|spec| spec.result_target.clone());
        let mut spec = OperationSpec::simulated_success(duration_ms);
        spec.result_target = target;
        self.operation = Some(spec);
        self
    }

    pub fn with_operation_failure(mut self, duration_ms: u64) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|spec| spec.result_target.clone());
        let mut spec = OperationSpec::simulated_failure(duration_ms);
        spec.result_target = target;
        self.operation = Some(spec);
        self
    }

    pub fn with_shell_command(mut self, command: impl Into<String>) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|spec| spec.result_target.clone());
        let mut spec = OperationSpec::shell(command);
        spec.result_target = target;
        self.operation = Some(spec);
        self
    }

    pub fn with_registered_action(mut self, action: impl Into<String>) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|spec| spec.result_target.clone());
        let mut spec = OperationSpec::registered(action);
        spec.result_target = target;
        self.operation = Some(spec);
        self
    }

    pub fn with_result_target(mut self, target_id: impl Into<String>) -> Self {
        let target_id = target_id.into();
        let spec = self
            .operation
            .get_or_insert_with(|| OperationSpec::shell("true"));
        spec.result_target = Some(target_id);
        self
    }

    pub fn row_height(&self) -> usize {
        self.height_units.max(1) as usize * 3
    }
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationSpec {
    pub source: OperationSource,
    pub result_target: Option<String>,
}

impl OperationSpec {
    pub fn shell(command: impl Into<String>) -> Self {
        Self {
            source: OperationSource::ShellCommand(command.into()),
            result_target: None,
        }
    }

    pub fn registered(action: impl Into<String>) -> Self {
        Self {
            source: OperationSource::RegisteredAction(action.into()),
            result_target: None,
        }
    }

    pub fn simulated_success(duration_ms: u64) -> Self {
        let seconds = duration_ms as f64 / 1000.0;
        Self::shell(format!("sleep {seconds:.3}; printf '操作成功\\n'; exit 0"))
    }

    pub fn simulated_failure(duration_ms: u64) -> Self {
        let seconds = duration_ms as f64 / 1000.0;
        Self::shell(format!(
            "sleep {seconds:.3}; printf '操作失败\\n' >&2; exit 1"
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePage {
    pub title: String,
    pub sections: Vec<RuntimeSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSection {
    pub title: String,
    pub fields: Vec<RuntimeField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeField {
    pub id: Option<String>,
    pub label: String,
    pub control: RuntimeControl,
    pub height_units: u16,
    pub operation: Option<RuntimeOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeControl {
    TextInput {
        value: String,
        placeholder: String,
    },
    NumberInput {
        value: String,
        placeholder: String,
    },
    Select {
        options: Vec<String>,
        selected: usize,
    },
    Toggle {
        on: bool,
    },
    ActionButton {
        label: String,
    },
    RefreshButton {
        label: String,
    },
    StaticData {
        value: String,
    },
    DynamicData {
        value: String,
    },
    LogOutput {
        content: String,
        file_source: Option<std::path::PathBuf>,
        tail_lines: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOperation {
    pub source: OperationSource,
    pub result_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationSource {
    ShellCommand(String),
    RegisteredAction(String),
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
pub struct RuntimeFieldState {
    pub control: ContentControl,
    pub status: OperationStatus,
    pub snapshot: Option<ContentControl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentRuntimeState {
    pub current_page: usize,
    pub selected_block: usize,
    pub spinner_tick: usize,
    pub field_states: Vec<RuntimeFieldState>,
}

impl ContentRuntimeState {
    pub fn from_blueprint(blueprint: &ContentBlueprint) -> Self {
        let field_count = blueprint
            .sections
            .iter()
            .map(|section| section.blocks.len())
            .sum::<usize>();
        let mut field_states = Vec::with_capacity(field_count);
        for section in &blueprint.sections {
            for block in &section.blocks {
                field_states.push(RuntimeFieldState {
                    control: block.control.clone(),
                    status: OperationStatus::Idle,
                    snapshot: None,
                });
            }
        }

        Self {
            field_states,
            ..Self::default()
        }
    }

    pub fn clear_statuses(&mut self) {
        for state in &mut self.field_states {
            state.status = OperationStatus::Idle;
            state.snapshot = None;
        }
    }

    pub fn field_state(&self, index: usize) -> Option<&RuntimeFieldState> {
        self.field_states.get(index)
    }

    pub fn field_state_mut(&mut self, index: usize) -> Option<&mut RuntimeFieldState> {
        self.field_states.get_mut(index)
    }
}

impl From<RuntimePage> for ContentBlueprint {
    fn from(value: RuntimePage) -> Self {
        ContentBlueprint::new(value.title).with_sections(
            value
                .sections
                .into_iter()
                .map(ContentSection::from)
                .collect(),
        )
    }
}

impl From<RuntimeSection> for ContentSection {
    fn from(value: RuntimeSection) -> Self {
        ContentSection::new(value.title)
            .with_blocks(value.fields.into_iter().map(ContentBlock::from).collect())
    }
}

impl From<RuntimeField> for ContentBlock {
    fn from(value: RuntimeField) -> Self {
        let mut block = match value.control {
            RuntimeControl::TextInput {
                value: field_value,
                placeholder,
            } => ContentBlock::text_input(value.label, field_value, placeholder),
            RuntimeControl::NumberInput {
                value: field_value,
                placeholder,
            } => ContentBlock::number_input(value.label, field_value, placeholder),
            RuntimeControl::Select { options, selected } => {
                ContentBlock::select(value.label, options, selected)
            }
            RuntimeControl::Toggle { on } => ContentBlock::toggle(value.label, on),
            RuntimeControl::ActionButton { label } => {
                ContentBlock::action_button(value.label, label)
            }
            RuntimeControl::RefreshButton { label } => {
                ContentBlock::refresh_button(value.label, label)
            }
            RuntimeControl::StaticData { value: field_value } => {
                ContentBlock::static_data(value.label, field_value)
            }
            RuntimeControl::DynamicData { value: field_value } => {
                ContentBlock::dynamic_data(value.label, field_value)
            }
            RuntimeControl::LogOutput {
                content,
                file_source,
                tail_lines,
            } => {
                let mut block = ContentBlock::log_output(value.label, content);
                if let Some(path) = file_source {
                    block.control = ContentControl::LogOutput(LogOutputControl::from_file(path));
                }
                if let ContentControl::LogOutput(control) = &mut block.control {
                    if let Some(limit) = tail_lines {
                        control.set_tail_lines(limit);
                        if control.file_source().is_some() {
                            control.refresh_from_file();
                        }
                    }
                }
                block
            }
        };

        if let Some(id) = value.id {
            block = block.with_id(id);
        }

        block = block.with_height_units(value.height_units);

        if let Some(operation) = value.operation {
            let mut spec = OperationSpec {
                source: operation.source,
                result_target: None,
            };
            spec.result_target = operation.result_target;
            block.operation = Some(spec);
        }

        block
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ContentBlock, ContentBlueprint, ContentControl, ContentRuntimeState, ContentSection,
        OperationSource, OperationStatus, RuntimeControl, RuntimeField, RuntimeOperation,
        RuntimePage, RuntimeSection,
    };

    #[test]
    fn runtime_page_converts_to_content_blueprint() {
        let runtime = RuntimePage {
            title: "Workspace".to_string(),
            sections: vec![RuntimeSection {
                title: "Main".to_string(),
                fields: vec![RuntimeField {
                    id: Some("project_name".to_string()),
                    label: "项目名".to_string(),
                    control: RuntimeControl::TextInput {
                        value: "tui01".to_string(),
                        placeholder: "输入项目名".to_string(),
                    },
                    height_units: 1,
                    operation: Some(RuntimeOperation {
                        source: OperationSource::ShellCommand("true".to_string()),
                        result_target: Some("log".to_string()),
                    }),
                }],
            }],
        };

        let blueprint: ContentBlueprint = runtime.into();
        assert_eq!(blueprint.sections.len(), 1);
        assert_eq!(
            blueprint.sections[0].blocks[0].id.as_deref(),
            Some("project_name")
        );
        match &blueprint.sections[0].blocks[0].control {
            ContentControl::TextInput(control) => assert_eq!(control.value, "tui01"),
            _ => panic!("expected text input"),
        }
        assert!(blueprint.sections[0].blocks[0].operation.is_some());
    }

    #[test]
    fn content_runtime_state_tracks_block_count() {
        let blueprint = ContentBlueprint::new("Root").with_sections(vec![ContentSection::new("A")
            .with_blocks(vec![
                ContentBlock::toggle("one", true),
                ContentBlock::toggle("two", false),
            ])]);

        let state = ContentRuntimeState::from_blueprint(&blueprint);
        assert_eq!(state.field_states.len(), 2);
        assert!(matches!(
            state.field_states[0].status,
            OperationStatus::Idle
        ));
        match &state.field_states[0].control {
            ContentControl::Toggle(control) => assert!(control.on),
            _ => panic!("expected toggle"),
        }
    }
}
