//! 操作轮询：提交异步操作、轮询结果并应用到界面。

use super::ShowcaseApp;
use crate::components::ContentPanel;
use crate::host::executor::OperationRequest;

pub(super) fn next_operation_id(app: &mut ShowcaseApp) -> u64 {
    let id = app.next_operation_id;
    app.next_operation_id = app.next_operation_id.wrapping_add(1);
    id
}

pub(super) fn poll_operation_results(app: &mut ShowcaseApp) {
    while let Some(result) = app.executor.try_recv() {
        apply_operation_result(app, result);
    }
}

pub(super) fn submit_operation(app: &mut ShowcaseApp, request: OperationRequest) {
    app.executor.submit(OperationRequest {
        host: app.host.context().clone(),
        cwd: app.host.working_dir().and_then(|path| path.to_str()).map(str::to_string),
        env: app.host.shell().env().clone(),
        allowed_working_dirs: app
            .host
            .execution_policy()
            .allowed_working_dirs()
            .iter()
            .filter_map(|path| path.to_str().map(str::to_string))
            .collect(),
        allowed_env_keys: app.host.execution_policy().allowed_env_keys().cloned(),
        ..request
    });
}

pub(super) fn apply_operation_result(
    app: &mut ShowcaseApp,
    result: crate::host::executor::OperationResult,
) {
    if let Some(screen) = app.screens.get_mut(result.screen_index) {
        let mut panel = ContentPanel::new();
        panel.set_blueprint(screen.content.clone());
        panel.apply_operation_result(&result);
        screen.content = panel.blueprint();
    }

    if result.screen_index == app.active_screen {
        app.content_panel.apply_operation_result(&result);
    }
}
