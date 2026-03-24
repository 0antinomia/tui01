//! tui01 TUI 框架入口点

use tui01::app::App;
use tui01::event::EventHandler;
use tui01::tui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tui::init_panic_hook();

    color_eyre::install()?;

    if let Err(msg) = tui::check_minimum_size() {
        eprintln!("{}", msg);
        return Ok(());
    }

    let mut tui = tui::Tui::new()?;

    let mut event_handler = EventHandler::new();

    let mut app = App::new();

    while app.running() {
        tui.draw(|f| app.render(f))?;

        if let Some(event) = event_handler.next().await {
            app.handle_event(event);
        }
    }

    tui.exit()?;
    Ok(())
}
