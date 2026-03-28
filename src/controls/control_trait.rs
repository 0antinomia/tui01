//! 控件统一接口定义和反馈状态枚举。

use crate::event::Key;
use ratatui::{buffer::Buffer, layout::Rect};
use std::any::Any;

/// 控件操作反馈状态，用于在控件旁显示操作进度指示器。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFeedback {
    Idle,
    Running(usize),
    Success,
    Failure,
}

/// 控件统一接口，提供渲染、按键处理、值获取、校验等标准方法。
///
/// 所有内置控件（TextInput、NumberInput、Select、Toggle、ActionButton、DataDisplay、LogOutput）
/// 均实现此 trait，以支持多态分发，替代当前基于 match 的 dispatch。
pub trait ControlTrait {
    /// 将控件渲染到给定区域。
    ///
    /// - `area`: 渲染区域
    /// - `buf`: 目标缓冲区
    /// - `selected`: 是否被选中（高亮边框）
    /// - `active`: 是否处于编辑/交互状态
    /// - `feedback`: 操作反馈状态（运行中、成功、失败等）
    fn render(&self, area: Rect, buf: &mut Buffer, selected: bool, active: bool, feedback: ControlFeedback);

    /// 处理按键输入，返回值是否发生变化。
    fn handle_key(&mut self, key: Key) -> bool;

    /// 获取控件当前值的字符串表示。
    fn value(&self) -> String;

    /// 校验控件当前值，返回 `Ok(())` 表示通过。
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }

    /// 返回控件的首选渲染宽度。
    fn preferred_width(&self) -> u16;

    /// 返回控件是否可编辑（接受键盘输入）。
    fn is_editable(&self) -> bool;

    /// 返回控件在激活（Enter）时是否触发操作（而非进入编辑模式）。
    fn triggers_on_activate(&self) -> bool;

    /// 创建自身的 boxed 克隆。用于 AnyControl 的 Clone 实现。
    fn box_clone(&self) -> Box<dyn ControlTrait>;

    /// 与另一个 trait object 比较相等性。用于 AnyControl 的 PartialEq 实现。
    fn box_eq(&self, other: &dyn ControlTrait) -> bool;

    /// 返回 Any 引用，用于 downcast。用于 box_eq 的具体类型比较。
    fn as_any(&self) -> &dyn Any;
}
