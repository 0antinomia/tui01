//! 推荐公开接入面使用的简洁字段工厂。

use crate::schema::FieldSpec;
use std::path::PathBuf;

pub fn text(
    label: impl Into<String>,
    value: impl Into<String>,
    placeholder: impl Into<String>,
) -> FieldSpec {
    FieldSpec::text_input(label, value, placeholder)
}

pub fn text_id(
    label: impl Into<String>,
    value: impl Into<String>,
    placeholder: impl Into<String>,
    id: impl Into<String>,
) -> FieldSpec {
    text(label, value, placeholder).with_id(id)
}

pub fn number(
    label: impl Into<String>,
    value: impl Into<String>,
    placeholder: impl Into<String>,
) -> FieldSpec {
    FieldSpec::number_input(label, value, placeholder)
}

pub fn number_id(
    label: impl Into<String>,
    value: impl Into<String>,
    placeholder: impl Into<String>,
    id: impl Into<String>,
) -> FieldSpec {
    number(label, value, placeholder).with_id(id)
}

pub fn select(
    label: impl Into<String>,
    options: impl IntoIterator<Item = impl Into<String>>,
    selected: usize,
) -> FieldSpec {
    FieldSpec::select(label, options, selected)
}

pub fn toggle(label: impl Into<String>, on: bool) -> FieldSpec {
    FieldSpec::toggle(label, on)
}

pub fn action(label: impl Into<String>, button_label: impl Into<String>) -> FieldSpec {
    FieldSpec::action_button(label, button_label)
}

pub fn action_to_log(
    label: impl Into<String>,
    button_label: impl Into<String>,
    id: impl Into<String>,
    log_id: impl Into<String>,
) -> FieldSpec {
    action(label, button_label)
        .with_id(id)
        .with_result_target(log_id)
}

pub fn action_registered_to_log(
    label: impl Into<String>,
    button_label: impl Into<String>,
    id: impl Into<String>,
    action_name: impl Into<String>,
    log_id: impl Into<String>,
) -> FieldSpec {
    action_to_log(label, button_label, id, log_id).with_registered_action(action_name)
}

pub fn refresh(label: impl Into<String>, button_label: impl Into<String>) -> FieldSpec {
    FieldSpec::refresh_button(label, button_label)
}

pub fn refresh_to_log(
    label: impl Into<String>,
    button_label: impl Into<String>,
    id: impl Into<String>,
    log_id: impl Into<String>,
) -> FieldSpec {
    refresh(label, button_label)
        .with_id(id)
        .with_result_target(log_id)
}

pub fn refresh_registered_to_log(
    label: impl Into<String>,
    button_label: impl Into<String>,
    id: impl Into<String>,
    action_name: impl Into<String>,
    log_id: impl Into<String>,
) -> FieldSpec {
    refresh_to_log(label, button_label, id, log_id).with_registered_action(action_name)
}

pub fn static_value(label: impl Into<String>, value: impl Into<String>) -> FieldSpec {
    FieldSpec::static_data(label, value)
}

pub fn dynamic_value(label: impl Into<String>, value: impl Into<String>) -> FieldSpec {
    FieldSpec::dynamic_data(label, value)
}

pub fn log(label: impl Into<String>, content: impl Into<String>) -> FieldSpec {
    FieldSpec::log_output(label, content)
}

pub fn log_id(
    label: impl Into<String>,
    content: impl Into<String>,
    id: impl Into<String>,
) -> FieldSpec {
    log(label, content).with_id(id)
}

pub fn log_file(label: impl Into<String>, path: impl Into<PathBuf>) -> FieldSpec {
    FieldSpec::log_output_from_file(label, path)
}

pub fn log_file_tail(
    label: impl Into<String>,
    path: impl Into<PathBuf>,
    tail_lines: usize,
) -> FieldSpec {
    log_file(label, path).with_log_tail_lines(tail_lines)
}

#[cfg(test)]
mod tests {
    use super::{action_registered_to_log, log_file_tail, number_id, text_id};

    #[test]
    fn text_and_number_id_helpers_apply_ids() {
        let text = text_id("项目名", "demo", "输入项目名", "project_name");
        let number = number_id("端口", "3000", "输入端口", "server_port");

        let debug_text = format!("{text:?}");
        let debug_number = format!("{number:?}");
        assert!(debug_text.contains("project_name"));
        assert!(debug_number.contains("server_port"));
    }

    #[test]
    fn action_registered_to_log_helper_applies_binding() {
        let field =
            action_registered_to_log("同步", "执行", "sync_action", "sync_workspace", "sync_log");
        let debug = format!("{field:?}");

        assert!(debug.contains("sync_action"));
        assert!(debug.contains("sync_workspace"));
        assert!(debug.contains("sync_log"));
    }

    #[test]
    fn log_file_tail_helper_applies_tail_limit() {
        let field = log_file_tail("框架日志", ".tui01/logs/framework.log", 20);
        let debug = format!("{field:?}");

        assert!(debug.contains("framework.log"));
        assert!(debug.contains("tail_lines: Some(20)"));
    }
}
