use tui01::event::EventHandler;
use tui01::tui;

mod actions;
mod app;
mod host;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tui::init_panic_hook();

    let host = host::build_host();
    let mut app = app::build_app_spec()
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
