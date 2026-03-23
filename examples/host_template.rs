use tui01::builder::{page, screen, section, AppSpec};
use tui01::event::EventHandler;
use tui01::executor::ActionOutcome;
use tui01::host::{HostLogLevel, RuntimeHost, ShellPolicy};
use tui01::schema::FieldSpec;
use tui01::tui;

fn build_host() -> RuntimeHost {
    let mut host = RuntimeHost::new();
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

    host.set_working_dir(".")
        .allow_working_dir(".")
        .allow_env_key("APP_ENV")
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tui::init_panic_hook();

    let host = build_host();
    let mut app = AppSpec::new()
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
                            FieldSpec::text_input("项目名", "demo", "输入项目名")
                                .with_id("project_name"),
                        )
                        .field(
                            FieldSpec::number_input("端口", "3000", "输入端口")
                                .with_id("server_port"),
                        ),
                )
                .section(
                    section("操作")
                        .field(
                            FieldSpec::refresh_button("同步工作区", "同步")
                                .with_id("sync_action")
                                .with_registered_action("sync_workspace")
                                .with_result_target("workspace_log"),
                        )
                        .field(
                            FieldSpec::log_output("输出", "等待执行结果")
                                .with_id("workspace_log")
                                .with_height_units(4),
                        ),
                ),
        ))
        .try_into_showcase_app_with_host(host)
        .map_err(|err| color_eyre::eyre::eyre!("invalid app spec: {}", err))?;

    if let Err(msg) = tui::check_minimum_size() {
        eprintln!("{}", msg);
        return Ok(());
    }

    let mut tui = tui::Tui::new()?;
    let mut event_handler = EventHandler::new();

    while app.running {
        tui.draw(|f| app.render(f))?;

        if let Some(event) = event_handler.next().await {
            app.handle_event(event);
        }
    }

    tui.exit()?;
    Ok(())
}
