use tui01::field;
use tui01::host::ActionOutcome;
use tui01::prelude::{AppSpec, HostLogLevel, RuntimeHost, ShellPolicy, page, screen, section};

fn build_host() -> RuntimeHost {
    let mut host = RuntimeHost::new();
    host.register_action_handler("sync_workspace", |context| async move {
        let project =
            context.params.get("project_name").cloned().unwrap_or_else(|| "unknown".to_string());
        let port = context.params.get("server_port").cloned().unwrap_or_else(|| "0".to_string());
        let root = context.host.get("project_root").cloned().unwrap_or_else(|| ".".to_string());

        ActionOutcome::success(format!("synced project={project} port={port} root={root}"))
    });

    let mut host = host
        .set_context("project_root", ".")
        .insert_env("APP_ENV", "dev")
        .set_shell_policy(ShellPolicy::RegisteredOnly);
    host.set_logger(|record| {
        let level = match record.level {
            HostLogLevel::Debug => "debug",
            HostLogLevel::Info => "info",
            HostLogLevel::Warn => "warn",
            HostLogLevel::Error => "error",
        };
        eprintln!("[{level}] {}: {}", record.target, record.message);
    });
    host.set_event_hook(|event| {
        eprintln!("[event] {event:?}");
    });

    host.set_working_dir(".").allow_working_dir(".").allow_env_key("APP_ENV")
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let host = build_host();
    let app = AppSpec::new()
        .title_text("Host Template\n\nRust-native host integration example.")
        .status_controls(
            "Controls:\n↑/↓ 或 j/k 当前焦点内移动\nShift+J/K 当前焦点区域翻页\nEnter / l 进入或确认\nEsc / h 返回\nq 退出",
        )
        .screen(screen(
            "Workspace",
            page("Workspace")
                .section(
                    section("基础配置")
                        .field(
                            field::text_id("项目名", "demo", "输入项目名", "project_name"),
                        )
                        .field(
                            field::number_id("端口", "3000", "输入端口", "server_port"),
                        ),
                )
                .section(
                    section("操作")
                        .field(
                            field::refresh_registered_to_log(
                                "同步工作区",
                                "同步",
                                "sync_action",
                                "sync_workspace",
                                "workspace_log",
                            ),
                        )
                        .field(
                            field::log_id("输出", "等待执行结果", "workspace_log")
                                .with_height_units(4),
                        ),
                ),
        ))
        .try_into_showcase_app_with_host(host)
        .map_err(|err| color_eyre::eyre::eyre!("invalid app spec: {}", err))?;

    app.run().await
}
