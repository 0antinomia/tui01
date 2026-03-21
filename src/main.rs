//! tui01 TUI 框架入口点

use tui01::app::App;
use tui01::event::EventHandler;
use tui01::tui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // 1. 首先设置 panic hook，在任何可能 panic 的代码之前
    tui::init_panic_hook();

    // 2. 初始化 color_eyre 错误处理
    color_eyre::install()?;

    // 3. 检查终端尺寸
    if let Err(msg) = tui::check_minimum_size() {
        eprintln!("{}", msg);
        return Ok(());
    }

    // 4. 创建 TUI 实例
    let mut tui = tui::Tui::new()?;

    // 5. 创建事件处理器
    let mut event_handler = EventHandler::new();

    // 6. 创建 App 实例
    let mut app = App::new();

    // 7. 主循环
    while app.running() {
        tui.draw(|f| app.render(f))?;

        if let Some(event) = event_handler.next().await {
            app.handle_event(event);
        }
    }

    // 8. 清理退出
    tui.exit()?;
    Ok(())
}
