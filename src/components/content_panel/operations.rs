//! 内容面板操作：提交操作、应用结果和双状态同步校验。

use super::ContentPanel;
use crate::controls::{AnyControl, BuiltinControl};
use crate::host::executor::{
    OperationRequest, OperationResult, OperationSource as ExecutorOperationSource,
};
use crate::runtime::{ContentBlock, OperationSource, OperationStatus};
use std::collections::HashMap;

/// 校验蓝图区块数与运行时字段状态数一致（双状态同步安全检查）。
///
/// 在 debug 构建中，当 blueprint 的区块总数与 runtime.field_states 长度不匹配时 panic。
/// release 构建中此函数为空操作。
pub(super) fn assert_dual_state_consistency(panel: &ContentPanel) {
    let mut total_blocks = 0usize;
    for section in &panel.blueprint.sections {
        total_blocks += section.blocks.len();
    }
    debug_assert_eq!(
        panel.runtime.field_states.len(),
        total_blocks,
        "Dual-state mismatch: blueprint has {} blocks but runtime has {} field_states",
        total_blocks,
        panel.runtime.field_states.len()
    );
}

/// set_blueprint 的实现逻辑，从 facade 委托调用。
pub(super) fn set_blueprint_impl(
    panel: &mut ContentPanel,
    blueprint: crate::runtime::ContentBlueprint,
) {
    let content_changed = panel.blueprint != blueprint;
    panel.blueprint = blueprint;

    if content_changed {
        panel.runtime = crate::runtime::ContentRuntimeState::from_blueprint(&panel.blueprint);
        panel.refresh_file_logs();
        panel.control_active = false;
    }
}

pub(super) fn start_operation(
    panel: &mut ContentPanel,
    operation_id: u64,
    screen_index: usize,
    block_index: usize,
    original_control: AnyControl,
    pending_control: AnyControl,
) -> Option<OperationRequest> {
    let command = block_mut_by_index(panel, block_index)?.operation.clone()?;
    let state = panel.block_state_mut(block_index)?;
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
            },
            OperationSource::RegisteredAction(action) => {
                ExecutorOperationSource::RegisteredAction(action)
            },
        },
        params: operation_params(panel),
        host: HashMap::new(),
        cwd: None,
        env: HashMap::new(),
        allowed_working_dirs: vec![],
        allowed_env_keys: None,
        result_target: command.result_target,
    })
}

fn operation_params(panel: &ContentPanel) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let page_scope = super::scope_slug(&panel.blueprint.title);
    let mut index = 0usize;

    for section in &panel.blueprint.sections {
        for block in &section.blocks {
            if let Some(id) = &block.id
                && let Some(control) = panel.block_control(index)
            {
                let value = control.value();
                params.insert(id.clone(), value.clone());
                params.insert(format!("screen.{id}"), value.clone());
                if !page_scope.is_empty() {
                    params.insert(format!("{page_scope}.{id}"), value);
                }
            }
            index += 1;
        }
    }

    params
}

pub(super) fn apply_operation_result(panel: &mut ContentPanel, result: &OperationResult) {
    assert_dual_state_consistency(panel);

    apply_operation_result_to_block(panel, result.block_index, result);

    if let Some(target_id) = result.result_target.as_deref()
        && let Some(AnyControl::Builtin(BuiltinControl::LogOutput(log))) =
            panel.block_control_mut_by_id(target_id)
    {
        log.append_entry(format_result_output(result));
    }

    assert_dual_state_consistency(panel);
}

fn apply_operation_result_to_block(
    panel: &mut ContentPanel,
    target_index: usize,
    result: &OperationResult,
) {
    let Some(state) = panel.block_state(target_index).cloned() else {
        return;
    };
    let OperationStatus::Running { operation_id, original_control, pending_control, .. } =
        state.status
    else {
        return;
    };

    if operation_id != result.operation_id {
        return;
    }

    if let Some(state) = panel.block_state_mut(target_index) {
        state.control = if result.success { pending_control } else { original_control };
        state.snapshot = None;
        state.status =
            if result.success { OperationStatus::Success } else { OperationStatus::Failure };
    }
}

pub(super) fn format_result_output(result: &OperationResult) -> String {
    let mut parts = Vec::new();
    parts.push(if result.success { "[success]".to_string() } else { "[failure]".to_string() });

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

fn block_mut_by_index(panel: &mut ContentPanel, target_index: usize) -> Option<&mut ContentBlock> {
    let mut index = 0usize;
    for section in &mut panel.blueprint.sections {
        for block in &mut section.blocks {
            if index == target_index {
                return Some(block);
            }
            index += 1;
        }
    }
    None
}
