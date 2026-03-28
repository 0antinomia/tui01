//! `Action` 枚举，用于 TEA（The Elm Architecture）模式。

/// `Action` 表示用户意图，由更新函数处理。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// 无操作。
    Noop,
    /// 退出应用。
    Quit,
    /// 终端尺寸变化事件。
    Resize(u16, u16),
    /// 菜单项选中，携带选中项索引。
    MenuSelect(usize),
}
