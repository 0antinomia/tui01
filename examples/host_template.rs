use tui01::config::AppConfig;
use tui01::event::EventHandler;
use tui01::executor::ActionOutcome;
use tui01::host::{HostLogLevel, RuntimeHost, ShellPolicy};
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

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/host_app.yaml".to_string());

    let config = AppConfig::from_file(&path)
        .map_err(|err| color_eyre::eyre::eyre!("failed to load config from {}: {}", path, err))?;

    let host = build_host();

    config
        .validate_against_host(&host)
        .map_err(|err| color_eyre::eyre::eyre!("host rejected config from {}: {}", path, err))?;

    let spec = config.into_app_spec();
    let mut app = spec
        .try_into_showcase_app_with_host(host)
        .map_err(|err| color_eyre::eyre::eyre!("invalid app spec from {}: {}", path, err))?;

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
