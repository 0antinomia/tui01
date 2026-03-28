# Phase 2: Control Isolation and Enum Unification - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 02-control-isolation-and-enum-unification
**Areas discussed:** 文件组织方式, RefreshButton 统一策略, 枚举替换边界, BuiltinControl 数据承载方式

---

## 文件组织方式

| Option | Description | Selected |
|--------|-------------|----------|
| src/components/controls/ 子目录 | 创建子目录，包含 mod.rs + 8 个控件文件 + control_trait.rs，与 quadrant.rs、menu.rs 平级但隔离 | ✓ |
| 平铺在 src/components/ 下 | 所有控件文件直接放在 components/ 下，路径简单但目录膨胀 | |

**User's choice:** src/components/controls/ 子目录

### 子目录结构

| Option | Description | Selected |
|--------|-------------|----------|
| 一对一文件 | 每个控件类型一个独立文件，包含 struct + ControlTrait impl + 渲染辅助 | ✓ |
| 合并相关控件 | 如 action_button.rs 包含 ActionButton + Refresh，文件数更少 | |

**User's choice:** 一对一文件

### 辅助枚举放置

| Option | Description | Selected |
|--------|-------------|----------|
| 放在 controls/ 内部 | ControlFeedback 和 ActionButtonKind 都放 controls/ 子目录内 | |
| 各归其主文件 | ActionButtonKind → action_button.rs，ControlFeedback → control_trait.rs | ✓ |

**User's choice:** Claude's discretion — 各归其主文件

**Notes:** User deferred to Claude's recommendation.

---

## RefreshButton 统一策略

| Option | Description | Selected |
|--------|-------------|----------|
| 吸收为 ActionButton 子类 | BuiltinControl 只有 8 个变体，RefreshButton 在 materialize 时映射为 ActionButton(Refresh) | ✓ |
| 保持为独立变体 | BuiltinControl 有 9 个变体，与 RuntimeControl/ControlSpec 完全对齐 | |

**User's choice:** 吸收为 ActionButton 子类
**Notes:** 更简洁的枚举设计，消除 RefreshButton 作为独立变体的必要性。

---

## 枚举替换边界

| Option | Description | Selected |
|--------|-------------|----------|
| 4 个替换，1 个保留 | 替换 ControlKind + ContentControl + RuntimeControl + SelectedControlKind，保留 ControlSpec | ✓ |
| 3 个替换，2 个保留 | 只替换行为层 3 个，保留 RuntimeControl 作为数据层 | |

**User's choice:** 4 个替换，1 个保留
**Notes:** ControlSpec 保留在规范层不动。RuntimeControl 消除后，materialize 直接映射到 BuiltinControl。

---

## BuiltinControl 数据承载方式

| Option | Description | Selected |
|--------|-------------|----------|
| 完整结构体 | BuiltinControl 承载完整控件结构体，行为和数据统一访问 | ✓ |
| kind-only + 分离数据 | BuiltinControl 只承载 kind 标识，数据通过其他机制承载（需泛型，与 Phase 1 冲突） | |

**User's choice:** 完整结构体
**Notes:** 与 Phase 1 ControlTrait 完全对齐，每个变体包装的就是已实现 trait 的结构体。

---

## Claude's Discretion

- controls/ 子目录内各文件的具体实现细节
- mod.rs 中 re-exports 的具体组织方式
- materialize 逻辑的调整顺序
- helpers.rs vs mod.rs 中放工具函数的选择
- 替换 4 个枚举时的迁移策略

## Deferred Ideas

None — discussion stayed within phase scope
