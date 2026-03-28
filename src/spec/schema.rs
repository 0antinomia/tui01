//! 用于构建运行时页面定义的声明式结构。

use crate::runtime::{
    OperationSource, RuntimeControl, RuntimeField, RuntimeOperation, RuntimePage, RuntimeSection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageSpec {
    title: String,
    sections: Vec<SectionSpec>,
}

impl PageSpec {
    /// 创建一个页面，该页面会作为左下菜单中的一个页面项显示。
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
        }
    }

    /// 一次性替换全部分区。
    pub fn with_sections(mut self, sections: Vec<SectionSpec>) -> Self {
        self.sections = sections;
        self
    }

    /// 追加一个分区。
    pub fn section(mut self, section: SectionSpec) -> Self {
        self.sections.push(section);
        self
    }

    /// 将声明式页面转换为运行时数据。
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
    /// 创建一个显示在右下内容区中的分区。
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            fields: Vec::new(),
        }
    }

    /// 一次性替换全部字段。
    pub fn with_fields(mut self, fields: Vec<FieldSpec>) -> Self {
        self.fields = fields;
        self
    }

    /// 追加一个字段。
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
    /// 文本输入字段。
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

    /// 数值输入字段。编辑时只接受半角数字。
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

    /// 单选下拉字段。
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

    /// 开关字段。
    pub fn toggle(label: impl Into<String>, on: bool) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::Toggle { on },
            height_units: 1,
            operation: None,
        }
    }

    /// 动作按钮字段。
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

    /// 刷新按钮字段。
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

    /// 静态只读值展示字段。
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

    /// 动态只读值展示字段。
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

    /// 只读日志输出字段。
    pub fn log_output(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::LogOutput {
                content: content.into(),
                file_source: None,
                tail_lines: None,
            },
            height_units: 4,
            operation: None,
        }
    }

    /// 自定义控件字段，通过控件名称引用宿主应用注册的控件工厂。
    pub fn custom(label: impl Into<String>, control_name: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::Custom {
                control_name: control_name.into(),
            },
            height_units: 1,
            operation: None,
        }
    }

    /// 由磁盘文件驱动的只读日志输出字段。
    pub fn log_output_from_file(
        label: impl Into<String>,
        path: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            id: None,
            label: label.into(),
            control: ControlSpec::LogOutput {
                content: String::new(),
                file_source: Some(path.into()),
                tail_lines: None,
            },
            height_units: 4,
            operation: None,
        }
    }

    /// 将日志输出限制为最近 N 行。
    pub fn with_log_tail_lines(mut self, tail_lines: usize) -> Self {
        if let ControlSpec::LogOutput {
            tail_lines: slot, ..
        } = &mut self.control
        {
            *slot = Some(tail_lines.max(1));
        }
        self
    }

    /// 指定稳定 id，用于结果回写或未来外部绑定。
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 设置高度，单位为 3 行。
    pub fn with_height_units(mut self, height_units: u16) -> Self {
        self.height_units = height_units.max(1);
        self
    }

    /// 绑定一个模拟成功的操作。
    pub fn with_operation_success(mut self, duration_ms: u64) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|binding| binding.result_target.clone());
        self.operation =
            Some(OperationBinding::simulated_success(duration_ms).with_target_opt(target));
        self
    }

    /// 绑定一个模拟失败的操作。
    pub fn with_operation_failure(mut self, duration_ms: u64) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|binding| binding.result_target.clone());
        self.operation =
            Some(OperationBinding::simulated_failure(duration_ms).with_target_opt(target));
        self
    }

    /// 绑定一个真实命令行命令。
    pub fn with_shell_command(mut self, command: impl Into<String>) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|binding| binding.result_target.clone());
        self.operation = Some(OperationBinding::shell(command).with_target_opt(target));
        self
    }

    /// 绑定一个由宿主应用解析的已注册动作名。
    pub fn with_registered_action(mut self, action: impl Into<String>) -> Self {
        let target = self
            .operation
            .as_ref()
            .and_then(|binding| binding.result_target.clone());
        self.operation = Some(OperationBinding::registered(action).with_target_opt(target));
        self
    }

    /// 将命令输出路由到指定 id 的日志字段。
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
                ControlSpec::LogOutput {
                    content,
                    file_source,
                    tail_lines,
                } => RuntimeControl::LogOutput {
                    content: content.clone(),
                    file_source: file_source.clone(),
                    tail_lines: *tail_lines,
                },
                ControlSpec::Custom { control_name } => RuntimeControl::Custom {
                    control_name: control_name.clone(),
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
        file_source: Option<std::path::PathBuf>,
        tail_lines: Option<usize>,
    },
    Custom {
        control_name: String,
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
