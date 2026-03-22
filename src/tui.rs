//! 终端生命周期管理，支持 panic 安全清理

use std::io::{self, Stdout};
use std::panic;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// 最小终端宽度
pub const MIN_WIDTH: u16 = 80;
/// 最小终端高度
pub const MIN_HEIGHT: u16 = 24;
/// 最小宽高比（宽度/高度），防止终端过窄
pub const MIN_ASPECT_RATIO: f64 = 0.5;
/// 最大宽高比（宽度/高度），防止终端过宽
pub const MAX_ASPECT_RATIO: f64 = 4.0;

/// Tui 结构体，封装 ratatui Terminal
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// 创建新的 Tui 实例，进入 raw 模式和备用屏幕
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

/// 安装 panic hook，在 panic 时恢复终端状态
/// 必须在任何可能 panic 的代码之前调用
pub fn init_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // 在打印 panic 信息前恢复终端状态
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

/// 检查终端是否满足最小尺寸要求
/// 如果太小或宽高比异常，返回 Err 并附带错误信息
pub fn check_minimum_size() -> Result<(), String> {
    let (width, height) = terminal_size().map_err(|e| format!("无法获取终端尺寸: {}", e))?;

    if width < MIN_WIDTH || height < MIN_HEIGHT {
        Err(format!(
            "终端太小（最小需要 {}x{}，当前 {}x{}）",
            MIN_WIDTH, MIN_HEIGHT, width, height
        ))
    } else {
        let aspect_ratio = width as f64 / height as f64;
        if aspect_ratio < MIN_ASPECT_RATIO {
            Err(format!(
                "终端过窄（宽高比 {:.2}，最小需要 {:.2}，当前 {}x{}）",
                aspect_ratio, MIN_ASPECT_RATIO, width, height
            ))
        } else if aspect_ratio > MAX_ASPECT_RATIO {
            Err(format!(
                "终端过宽（宽高比 {:.2}，最大允许 {:.2}，当前 {}x{}）",
                aspect_ratio, MAX_ASPECT_RATIO, width, height
            ))
        } else {
            Ok(())
        }
    }
}

/// 获取当前终端尺寸。
pub fn terminal_size() -> io::Result<(u16, u16)> {
    crossterm::terminal::size()
}
