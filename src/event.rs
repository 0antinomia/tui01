//! 基于 tokio 的异步事件处理

use crossterm::event::{
    Event as CrosstermEvent, EventStream, KeyCode as CrosstermKeyCode, KeyEventKind, KeyModifiers,
};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

/// 项目内部的按键类型，避免在上层直接依赖 crossterm。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Enter,
    Esc,
    Backspace,
    Tab,
    Unknown,
}

impl From<CrosstermKeyCode> for Key {
    fn from(value: CrosstermKeyCode) -> Self {
        match value {
            CrosstermKeyCode::Char(ch) => Self::Char(ch),
            CrosstermKeyCode::Up => Self::Up,
            CrosstermKeyCode::Down => Self::Down,
            CrosstermKeyCode::Left => Self::Left,
            CrosstermKeyCode::Right => Self::Right,
            CrosstermKeyCode::Enter => Self::Enter,
            CrosstermKeyCode::Esc => Self::Esc,
            CrosstermKeyCode::Backspace => Self::Backspace,
            CrosstermKeyCode::Tab => Self::Tab,
            _ => Self::Unknown,
        }
    }
}

/// 内部事件类型
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// 键盘输入事件
    Key(Key),
    /// 终端尺寸变化事件
    Resize(u16, u16),
    /// 退出信号（Ctrl+C）
    Quit,
}

/// 事件处理器，统一转发终端事件和退出信号
pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// 创建新的事件处理器
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let ctrl_c = tokio::signal::ctrl_c();

            tokio::pin!(ctrl_c);

            loop {
                tokio::select! {
                    // 处理终端事件
                    maybe_event = reader.next().fuse() => {
                        if let Some(Ok(evt)) = maybe_event {
                            match evt {
                                CrosstermEvent::Key(key) => {
                                    if key.kind == KeyEventKind::Press {
                                        // 在 raw mode 下，Ctrl+C 作为键盘事件而不是 SIGINT
                                        if key.code == CrosstermKeyCode::Char('c')
                                            && key.modifiers.contains(KeyModifiers::CONTROL)
                                        {
                                            let _ = sender.send(Event::Quit);
                                        } else {
                                            let _ = sender.send(Event::Key(Key::from(key.code)));
                                        }
                                    }
                                }
                                CrosstermEvent::Resize(w, h) => {
                                    let _ = sender.send(Event::Resize(w, h));
                                }
                                _ => {}
                            }
                        }
                    }
                    // 处理 Ctrl+C 信号
                    _ = &mut ctrl_c => {
                        let _ = sender.send(Event::Quit);
                        break;
                    }
                }
            }
        });

        Self { receiver }
    }

    /// 获取下一个事件（异步）
    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::Key;
    use crossterm::event::KeyCode as CrosstermKeyCode;
    use rstest::rstest;

    #[rstest]
    #[case(CrosstermKeyCode::Char('x'), Key::Char('x'))]
    #[case(CrosstermKeyCode::Up, Key::Up)]
    #[case(CrosstermKeyCode::Down, Key::Down)]
    #[case(CrosstermKeyCode::Left, Key::Left)]
    #[case(CrosstermKeyCode::Right, Key::Right)]
    #[case(CrosstermKeyCode::Enter, Key::Enter)]
    #[case(CrosstermKeyCode::Esc, Key::Esc)]
    #[case(CrosstermKeyCode::Backspace, Key::Backspace)]
    #[case(CrosstermKeyCode::Tab, Key::Tab)]
    #[case(CrosstermKeyCode::F(1), Key::Unknown)]
    fn converts_crossterm_keycodes(#[case] input: CrosstermKeyCode, #[case] expected: Key) {
        assert_eq!(Key::from(input), expected);
    }
}
