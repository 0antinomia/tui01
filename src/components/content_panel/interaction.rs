//! 内容面板交互：键盘处理、导航和控件激活。

use super::ContentPanel;
use super::layout::{SelectedControlKind, current_page_block_indices};
use super::operations;
use crate::controls::AnyControl;
use crate::host::executor::OperationRequest;
use crate::infra::event::Key;

fn handle_control_key(control: &mut AnyControl, key: Key) -> bool {
    control.handle_key(key)
}

pub(super) fn handle_panel_control_key(panel: &mut ContentPanel, key: Key) -> bool {
    if panel
        .selected_block_state()
        .map(|state| matches!(state.status, crate::runtime::OperationStatus::Running { .. }))
        .unwrap_or(false)
    {
        return false;
    }

    panel
        .selected_block_control_mut()
        .map(|control| handle_control_key(control, key))
        .unwrap_or(false)
}

pub(super) fn activate_panel_selected_control(
    panel: &mut ContentPanel,
    operation_id: u64,
    screen_index: usize,
) -> Option<OperationRequest> {
    match selected_control_kind(panel) {
        Some(
            SelectedControlKind::TextInput
            | SelectedControlKind::NumberInput
            | SelectedControlKind::Select
            | SelectedControlKind::LogOutput,
        ) => {
            if let Some(state) = panel.selected_block_state_mut() {
                state.snapshot = Some(state.control.clone());
            }
            panel.control_active = true;
            None
        }
        Some(SelectedControlKind::Toggle) => {
            panel.control_active = false;
            panel.clear_selected_snapshot();
            toggle_selected_control(panel, operation_id, screen_index)
        }
        Some(SelectedControlKind::ActionButton) => {
            panel.control_active = false;
            panel.clear_selected_snapshot();
            start_selected_action(panel, operation_id, screen_index)
        }
        Some(SelectedControlKind::StaticData | SelectedControlKind::DynamicData) | None => None,
        Some(SelectedControlKind::Custom) => {
            let is_editable =
                panel.selected_block_control_mut().map(|c| c.is_editable()).unwrap_or(false);
            let triggers = panel
                .selected_block_control_mut()
                .map(|c| c.triggers_on_activate())
                .unwrap_or(false);

            if triggers {
                panel.control_active = false;
                panel.clear_selected_snapshot();
                start_selected_action(panel, operation_id, screen_index)
            } else if is_editable {
                if let Some(state) = panel.selected_block_state_mut() {
                    state.snapshot = Some(state.control.clone());
                }
                panel.control_active = true;
                None
            } else {
                None
            }
        }
    }
}

pub(super) fn confirm_panel_control(
    panel: &mut ContentPanel,
    operation_id: u64,
    screen_index: usize,
) -> Option<OperationRequest> {
    panel.control_active = false;
    confirm_selected_control(panel, operation_id, screen_index)
}

pub(super) fn cancel_panel_control(panel: &mut ContentPanel) {
    if let Some(state) = panel.selected_block_state_mut()
        && let Some(snapshot) = state.snapshot.take()
    {
        state.control = snapshot;
    }
    panel.control_active = false;
}

pub(super) fn select_next_block(panel: &mut ContentPanel, height: u16) {
    let visible = current_page_block_indices(panel, height);
    if visible.is_empty() {
        return;
    }

    if let Some(pos) = visible.iter().position(|index| *index == panel.runtime.selected_block) {
        if pos + 1 < visible.len() {
            panel.runtime.selected_block = visible[pos + 1];
            panel.control_active = false;
            panel.clear_selected_snapshot();
        } else {
            let current_page = panel.runtime.current_page;
            panel.next_page(0, height);
            if panel.runtime.current_page != current_page {
                let next_visible = current_page_block_indices(panel, height);
                if let Some(first) = next_visible.first() {
                    panel.runtime.selected_block = *first;
                    panel.control_active = false;
                    panel.clear_selected_snapshot();
                }
            }
        }
    } else {
        panel.runtime.selected_block = visible[0];
        panel.control_active = false;
        panel.clear_selected_snapshot();
    }
}

pub(super) fn select_previous_block(panel: &mut ContentPanel, height: u16) {
    let visible = current_page_block_indices(panel, height);
    if visible.is_empty() {
        return;
    }

    if let Some(pos) = visible.iter().position(|index| *index == panel.runtime.selected_block) {
        if pos > 0 {
            panel.runtime.selected_block = visible[pos - 1];
            panel.control_active = false;
            panel.clear_selected_snapshot();
        } else if panel.runtime.current_page > 0 {
            panel.previous_page_with_height(height);
            let previous_visible = current_page_block_indices(panel, height);
            if let Some(last) = previous_visible.last() {
                panel.runtime.selected_block = *last;
                panel.control_active = false;
                panel.clear_selected_snapshot();
            }
        }
    } else {
        panel.runtime.selected_block = visible[0];
        panel.control_active = false;
        panel.clear_selected_snapshot();
    }
}

fn confirm_selected_control(
    panel: &mut ContentPanel,
    operation_id: u64,
    screen_index: usize,
) -> Option<OperationRequest> {
    let snapshot = panel.selected_block_state_mut().and_then(|state| state.snapshot.take());
    let block_index = panel.runtime.selected_block;
    if panel.block_is_running(block_index) {
        return None;
    }
    let block = panel.selected_block_ref()?;

    if block.operation.is_some() {
        if let Some(original) = snapshot {
            let pending = panel.block_control(block_index)?.clone();
            return operations::start_operation(
                panel,
                operation_id,
                screen_index,
                block_index,
                original,
                pending,
            );
        }
        let control = panel.block_control(block_index)?.clone();
        return operations::start_operation(
            panel,
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
    panel: &mut ContentPanel,
    operation_id: u64,
    screen_index: usize,
) -> Option<OperationRequest> {
    let block_index = panel.runtime.selected_block;
    if panel.block_is_running(block_index) {
        return None;
    }
    let block = panel.selected_block_ref()?;

    let mut pending = panel.block_control(block_index)?.clone();
    let changed = handle_control_key(&mut pending, Key::Enter);
    if !changed {
        return None;
    }

    if block.operation.is_some() {
        let original = panel.block_control(block_index)?.clone();
        operations::start_operation(
            panel,
            operation_id,
            screen_index,
            block_index,
            original,
            pending,
        )
    } else {
        if let Some(control) = panel.block_control_mut(block_index) {
            *control = pending;
        }
        None
    }
}

fn start_selected_action(
    panel: &mut ContentPanel,
    operation_id: u64,
    screen_index: usize,
) -> Option<OperationRequest> {
    let block_index = panel.runtime.selected_block;
    if panel.block_is_running(block_index) {
        return None;
    }
    let _block = panel.selected_block_ref()?;

    let control = panel.block_control(block_index)?.clone();
    operations::start_operation(
        panel,
        operation_id,
        screen_index,
        block_index,
        control.clone(),
        control,
    )
}

fn selected_control_kind(panel: &ContentPanel) -> Option<SelectedControlKind> {
    panel.block_control(panel.runtime.selected_block).map(|control| match control {
        AnyControl::Builtin(crate::controls::BuiltinControl::TextInput(_)) => {
            SelectedControlKind::TextInput
        }
        AnyControl::Builtin(crate::controls::BuiltinControl::NumberInput(_)) => {
            SelectedControlKind::NumberInput
        }
        AnyControl::Builtin(crate::controls::BuiltinControl::Select(_)) => {
            SelectedControlKind::Select
        }
        AnyControl::Builtin(crate::controls::BuiltinControl::Toggle(_)) => {
            SelectedControlKind::Toggle
        }
        AnyControl::Builtin(crate::controls::BuiltinControl::ActionButton(_)) => {
            SelectedControlKind::ActionButton
        }
        AnyControl::Builtin(crate::controls::BuiltinControl::StaticData(_)) => {
            SelectedControlKind::StaticData
        }
        AnyControl::Builtin(crate::controls::BuiltinControl::DynamicData(_)) => {
            SelectedControlKind::DynamicData
        }
        AnyControl::Builtin(crate::controls::BuiltinControl::LogOutput(_)) => {
            SelectedControlKind::LogOutput
        }
        AnyControl::Custom(_) => SelectedControlKind::Custom,
    })
}
