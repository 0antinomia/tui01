# Changelog

## 0.2.0

### Changed

- 完成一轮纯架构重构，源码按域模块拆分为 `spec / runtime / controls / components / host / app / infra`
- 引入 `ControlTrait`、`AnyControl` 和 `ControlRegistry`，支持宿主应用注册自定义控件
- 将控件实现按类型拆分到 `src/controls/`，降低新增控件时的改动范围
- 将主题与布局扩展点正式纳入公开能力，提供 `Theme`、`RenderContext` 和 `LayoutStrategy`
- 将应用壳层、内容区和执行器按职责拆分为子模块，保留原有行为与交互语义
- README 和接入文档已同步更新到重构后的真实结构与推荐入口

## 0.1.0

首个可接入版本，当前范围包括：

- 四分区 TUI 壳层
- 菜单、内容区、日志区等基础组件
- `AppSpec` Rust 原生入口
- `RuntimeHost` 宿主接入层
- 已注册动作、shell 策略、cwd/env 白名单
- 宿主 logger / event hook
- 宿主模板工程与可运行 example
