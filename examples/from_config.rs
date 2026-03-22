use tui01::config::AppConfig;
use tui01::event::EventHandler;
use tui01::tui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tui::init_panic_hook();

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/demo.yaml".to_string());

    let spec = AppConfig::from_file(&path)
        .map_err(|err| color_eyre::eyre::eyre!("failed to load config from {}: {}", path, err))?
        .into_app_spec();

    let mut app = spec
        .try_into_showcase_app()
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
