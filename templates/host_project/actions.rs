use tui01::executor::ActionOutcome;
use tui01::host::RuntimeHost;

pub fn register_actions(host: &mut RuntimeHost) {
    host.register_action_handler("sync_workspace", |context| async move {
        let project = context
            .params
            .get("project_name")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let port = context
            .params
            .get("server_port")
            .cloned()
            .unwrap_or_else(|| "0".to_string());
        let root = context
            .host
            .get("project_root")
            .cloned()
            .unwrap_or_else(|| ".".to_string());

        ActionOutcome::success(format!("synced project={project} port={port} root={root}"))
    });
}
