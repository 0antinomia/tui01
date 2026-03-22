//! Declarative page schema for building runtime page definitions.

use crate::runtime::{
    OperationSource, RuntimeControl, RuntimeField, RuntimeOperation, RuntimePage, RuntimeSection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageSpec {
    title: String,
    sections: Vec<SectionSpec>,
}

impl PageSpec {
    /// Create a page that will be shown as one screen in the left-bottom menu.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
        }
    }

    /// Replace all sections at once.
    pub fn with_sections(mut self, sections: Vec<SectionSpec>) -> Self {
        self.sections = sections;
        self
    }

    /// Append one section.
    pub fn section(mut self, section: SectionSpec) -> Self {
        self.sections.push(section);
        self
    }

    /// Convert the declarative page into runtime data.
    pub fn materialize(&self) -> RuntimePage {
        RuntimePage {
            title: self.title.clone(),
            sections: self.sections.iter().map(SectionSpec::materialize).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionSpec {
    title: String,
    fields: Vec<FieldSpec>,
}

impl SectionSpec {
    /// Create a section shown inside the right-bottom content area.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            fields: Vec::new(),
        }
    }

    /// Replace all fields at once.
    pub fn with_fields(mut self, fields: Vec<FieldSpec>) -> Self {
        self.fields = fields;
        self
    }

    /// Append one field.
    pub fn field(mut self, field: FieldSpec) -> Self {
        self.fields.push(field);
        self
    }

    fn materialize(&self) -> RuntimeSection {
        RuntimeSection {
            title: self.title.clone(),
            fields: self.fields.iter().map(FieldSpec::materialize).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    id: Option<String>,
    label: String,
    control: ControlSpec,
    height_units: u16,
    operation: Option<OperationBinding>,
}

impl FieldSpec {
    /// Text input field.
    pub fn text_input(
        label: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::TextInput {
                value: value.into(),
                placeholder: placeholder.into(),
            },
            height_units: 1,
            operation: None,
        }
    }

    /// Numeric input field. Only ASCII digits are accepted at edit time.
    pub fn number_input(
        label: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::NumberInput {
                value: value.into(),
                placeholder: placeholder.into(),
            },
            height_units: 1,
            operation: None,
        }
    }

    /// Single-choice select field.
    pub fn select(
        label: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<String>>,
        selected: usize,
    ) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::Select {
                options: options.into_iter().map(Into::into).collect(),
                selected,
            },
            height_units: 1,
            operation: None,
        }
    }

    /// Toggle field.
    pub fn toggle(label: impl Into<String>, on: bool) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::Toggle { on },
            height_units: 1,
            operation: None,
        }
    }

    /// Action button field.
    pub fn action_button(label: impl Into<String>, button_label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::ActionButton {
                label: button_label.into(),
            },
            height_units: 1,
            operation: None,
        }
    }

    /// Refresh button field.
    pub fn refresh_button(label: impl Into<String>, button_label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::RefreshButton {
                label: button_label.into(),
            },
            height_units: 1,
            operation: None,
        }
    }

    /// Static read-only value display.
    pub fn static_data(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::StaticData {
                value: value.into(),
            },
            height_units: 1,
            operation: None,
        }
    }

    /// Dynamic read-only value display.
    pub fn dynamic_data(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::DynamicData {
                value: value.into(),
            },
            height_units: 1,
            operation: None,
        }
    }

    /// Read-only log output field.
    pub fn log_output(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::LogOutput {
                content: content.into(),
            },
            height_units: 4,
            operation: None,
        }
    }

    /// Assign a stable id for result routing or future external bindings.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set height in 3-row units.
    pub fn with_height_units(mut self, height_units: u16) -> Self {
        self.height_units = height_units.max(1);
        self
    }

    /// Bind a simulated successful operation.
    pub fn with_operation_success(mut self, duration_ms: u64) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|binding| binding.result_target.clone());
        self.operation =
            Some(OperationBinding::simulated_success(duration_ms).with_target_opt(target));
        self
    }

    /// Bind a simulated failed operation.
    pub fn with_operation_failure(mut self, duration_ms: u64) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|binding| binding.result_target.clone());
        self.operation =
            Some(OperationBinding::simulated_failure(duration_ms).with_target_opt(target));
        self
    }

    /// Bind a real shell command.
    pub fn with_shell_command(mut self, command: impl Into<String>) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|binding| binding.result_target.clone());
        self.operation = Some(OperationBinding::shell(command).with_target_opt(target));
        self
    }

    /// Bind a registered action name resolved by the host application.
    pub fn with_registered_action(mut self, action: impl Into<String>) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|binding| binding.result_target.clone());
        self.operation = Some(OperationBinding::registered(action).with_target_opt(target));
        self
    }

    /// Route command output to a target log field by id.
    pub fn with_result_target(mut self, target_id: impl Into<String>) -> Self {
        let target = target_id.into();
        let binding = self
            .operation
            .get_or_insert_with(|| OperationBinding::shell("true"));
        binding.result_target = Some(target);
        self
    }

    fn materialize(&self) -> RuntimeField {
        RuntimeField {
            id: self.id.clone(),
            label: self.label.clone(),
            control: match &self.control {
                ControlSpec::TextInput { value, placeholder } => RuntimeControl::TextInput {
                    value: value.clone(),
                    placeholder: placeholder.clone(),
                },
                ControlSpec::NumberInput { value, placeholder } => RuntimeControl::NumberInput {
                    value: value.clone(),
                    placeholder: placeholder.clone(),
                },
                ControlSpec::Select { options, selected } => RuntimeControl::Select {
                    options: options.clone(),
                    selected: *selected,
                },
                ControlSpec::Toggle { on } => RuntimeControl::Toggle { on: *on },
                ControlSpec::ActionButton { label } => RuntimeControl::ActionButton {
                    label: label.clone(),
                },
                ControlSpec::RefreshButton { label } => RuntimeControl::RefreshButton {
                    label: label.clone(),
                },
                ControlSpec::StaticData { value } => RuntimeControl::StaticData {
                    value: value.clone(),
                },
                ControlSpec::DynamicData { value } => RuntimeControl::DynamicData {
                    value: value.clone(),
                },
                ControlSpec::LogOutput { content } => RuntimeControl::LogOutput {
                    content: content.clone(),
                },
            },
            height_units: self.height_units,
            operation: self.operation.clone().map(|operation| RuntimeOperation {
                source: operation.source,
                result_target: operation.result_target,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ControlSpec {
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
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationBinding {
    source: OperationSource,
    result_target: Option<String>,
}

impl OperationBinding {
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

    fn with_target_opt(mut self, target: Option<String>) -> Self {
        self.result_target = target;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldSpec, PageSpec, SectionSpec};

    #[test]
    fn page_spec_materializes_runtime_blueprint() {
        let page = PageSpec::new("Workspace").with_sections(vec![SectionSpec::new("运行")
            .with_fields(vec![
                FieldSpec::text_input("项目名", "tui01", "输入项目名")
                    .with_id("project_name")
                    .with_operation_success(800),
                FieldSpec::log_output("输出", "等待结果")
                    .with_id("workspace_log")
                    .with_height_units(4),
            ])]);

        let runtime = page.materialize();
        assert_eq!(runtime.title, "Workspace");
        assert_eq!(runtime.sections.len(), 1);
        assert_eq!(runtime.sections[0].fields.len(), 2);
        assert_eq!(
            runtime.sections[0].fields[0].id.as_deref(),
            Some("project_name")
        );
        assert!(runtime.sections[0].fields[0].operation.is_some());
    }
}
