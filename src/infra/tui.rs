//! 终端生命周期管理，支持发生异常时安全清理。

use std::io::{self, Stdout};
use std::panic;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// 最小终端宽度
pub const MIN_WIDTH: u16 = 80;
/// 最小终端高度
pub const MIN_HEIGHT: u16 = 24;
/// 最小宽高比（宽度/高度），防止终端过窄
pub const MIN_ASPECT_RATIO: f64 = 0.5;
/// 最大宽高比（宽度/高度），防止终端过宽
pub const MAX_ASPECT_RATIO: f64 = 4.0;

/// `Tui` 结构体，封装 `ratatui` 的终端实例。
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// 创建新的 `Tui` 实例，进入原始输入模式和备用屏幕。
    pub fn new() -> color_eyre::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// 使用提供的闭包绘制一帧
    pub fn draw<F>(&mut self, f: F) -> color_eyre::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }

    /// 退出终端，恢复原始状态
    pub fn exit(&mut self) -> color_eyre::Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }
}

/// 安装异常钩子，在程序异常时恢复终端状态。
/// 必须在任何可能触发异常的代码之前调用。
pub fn init_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // 在打印异常信息前先恢复终端状态。
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

/// 获取当前终端尺寸。
pub fn terminal_size() -> io::Result<(u16, u16)> {
    crossterm::terminal::size()
}
