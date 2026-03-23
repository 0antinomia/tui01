use tui01::config::AppConfig;
use tui01::event::EventHandler;
use tui01::tui;

mod actions;
mod host;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tui::init_panic_hook();

    let config = AppConfig::from_yaml_file("tui/app.yaml")
        .map_err(|err| color_eyre::eyre::eyre!("failed to load config: {err}"))?;
    let host = host::build_host();

    config
        .validate_against_host(&host)
        .map_err(|err| color_eyre::eyre::eyre!("host rejected config: {err}"))?;

    let spec = config.into_app_spec();
    let mut app = spec
        .try_into_showcase_app_with_host(host)
        .map_err(|err| color_eyre::eyre::eyre!("invalid app spec: {err}"))?;

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
