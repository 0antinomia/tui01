# tui01 架构重构

## What This Is

tui01 是一个基于 ratatui 的 Rust TUI 框架（当前 v0.1.0），提供四分区布局、声明式字段定义、宿主动作注册和 shell 命令执行。本次工作是对其进行纯架构重构——不改变任何功能行为，但重新组织代码结构，使其更易于扩展和维护。

## Core Value

新控件扩展只需改动 1-2 个文件，而非当前的 10 个文件。

## Requirements

### Validated

- ✓ 四分区布局渲染（title/status/menu/content） — existing
- ✓ 声明式 AppSpec/PageSpec/SectionSpec/FieldSpec 构建器 — existing
- ✓ 文本/数值/下拉/开关/动作/刷新/日志等字段类型 — existing
- ✓ RuntimeHost 宿主集成（动作注册、shell 策略、执行策略） — existing
- ✓ 异步操作执行（tokio channel，shell 命令，注册动作） — existing
- ✓ 字段值参数化（{{field_id}}、{{host.key}}） — existing
- ✓ 操作结果回写到日志控件 — existing
- ✓ AppSpec 层统一校验（ID 唯一性、result_target、registered_action） — existing
- ✓ 框架日志系统（FrameworkLogger） — existing
- ✓ ControlTrait 统一接口（7 methods, 8 controls） — Phase 1
- ✓ 控件 trait dispatch 替代 clone-then-wrap 模式 — Phase 1
- ✓ 大文件拆分：executor.rs(870行)、content_panel.rs(1502行)、showcase.rs(825行) 按职责拆分为目录模块 — Phase 5
- ✓ src/ 模块分层重组：建立清晰的子模块层级（host/、app/、components/） — Phase 4

### Active

- [ ] 控件扩展机制简化：新增控件类型只需改 1-2 个文件（ControlTrait foundation laid, BuiltinControl enum next）
- [ ] 公开 API 重新设计：保持语义等价但结构更合理
- [ ] 为主题/风格系统预留扩展点
- [ ] 为灵活布局系统（非仅四分区）预留扩展点
- [ ] 为插件/自定义控件预留扩展点

### Out of Scope

- 不改变任何已有功能的行为 — 这是纯重构
- 不添加新的字段类型 — 重构后结构要支持轻松添加，但本次不加
- 不实现主题系统 — 只预留扩展点
- 不实现灵活布局 — 只预留扩展点
- 不实现插件系统 — 只预留扩展点
- 不修改文档内容（docs/、README.md、CHANGELOG.md） — 文档更新留给后续

## Context

### 技术栈

- Rust 2021 edition
- ratatui 0.30 + crossterm 0.29（终端渲染）
- tokio（异步运行时）
- color-eyre（错误处理）
- 87 个测试分布在 13 个文件中

### 当前架构痛点

1. **控件扩展成本极高**：添加一个新控件需要修改 controls.rs、runtime.rs、schema.rs、field.rs、content_panel.rs 等 10 个文件
2. **大文件过多**：content_panel.rs 1526 行（表单布局+分页+操作生命周期）、controls.rs 1008 行（所有控件类型+渲染）、executor.rs 870 行、showcase.rs 813 行
3. **结构扁平**：src/ 下 16 个文件平铺，只有 components/ 一个子目录，缺乏层级感
4. **职责混杂**：content_panel.rs 同时负责布局、分页、控件渲染分发、操作状态管理

### 代码库规模

- src/ 下 23 个 .rs 文件（16 个顶层 + 7 个 components/）
- 总代码约 6000+ 行
- 测试覆盖率：87 个测试

## Constraints

- **Tech Stack**: Rust 2021, ratatui, crossterm, tokio — 不改变
- **Functional**: 所有现有功能和测试必须在重构后继续通过
- **License**: MIT OR Apache-2.0 双证书不变

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 可重新设计公开 API | 宿主应用集成示例可同步更新，且当前还在早期阶段(v0.1.0) | — Pending |
| 为主题/布局/插件预留扩展点 | 用户路线图中明确规划了这三个方向 | — Pending |
| ControlTrait 7 methods no generics | Per D-07/D-09, 7 specific methods, no generics or associated types | Phase 1 — Implemented |
| DataDisplayControl.dynamic field | Per D-02, replaces render parameter with struct field | Phase 1 — Implemented |
| Immediate trait migration (D-10) | Dispatch sites use trait methods from Phase 1, no intermediate clone pattern | Phase 1 — Implemented |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-28 after Phase 6 completion — all phases done*
