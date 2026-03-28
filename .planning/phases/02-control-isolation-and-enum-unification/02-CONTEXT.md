# Phase 2: Control Isolation and Enum Unification - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

将 controls.rs（1140 行）按控件类型拆分为独立文件，每个文件内聚该控件的结构体定义、ControlTrait 实现和渲染辅助逻辑。引入 BuiltinControl 枚举替代 4 个镜像枚举（ControlKind、ContentControl、RuntimeControl、SelectedControlKind），使新增内置控件只需改动 3 个文件。ControlSpec 保留在规范层不动。所有 87+ 现有测试必须继续通过。

</domain>

<decisions>
## Implementation Decisions

### 文件组织方式
- **D-01:** 在 `src/components/` 下创建 `controls/` 子目录，包含 mod.rs（re-exports + BuiltinControl 定义）、control_trait.rs、8 个控件文件（一对一）和 helpers.rs（工具函数）
- **D-02:** 每个控件类型一个独立文件：text_input.rs、number_input.rs、select.rs、toggle.rs、action_button.rs、data_display.rs、log_output.rs，各包含 struct + ControlTrait impl + 渲染辅助
- **D-03:** ActionButtonKind 放在 action_button.rs 中（只被 ActionButton 使用）
- **D-04:** ControlFeedback 放在 control_trait.rs 中（trait 签名的一部分）
- **D-05:** 工具函数（truncate_to_chars、wrap_text_lines 等）放在 controls/ 子目录内（mod.rs 或 helpers.rs）

### RefreshButton 统一策略
- **D-06:** RefreshButton 被吸收为 ActionButton 的 kind（ActionButtonKind::Refresh），BuiltinControl 只有 8 个变体
- **D-07:** RuntimeControl 和 ControlSpec 中的 RefreshButton 在 materialize 时映射为 ActionButton(ActionButtonKind::Refresh)，消除 RefreshButton 独立变体

### 枚举替换边界
- **D-08:** BuiltinControl 替换 4 个枚举：ControlKind（controls.rs）、ContentControl（runtime.rs）、RuntimeControl（runtime.rs）、SelectedControlKind（content_panel.rs）
- **D-09:** ControlSpec（schema.rs）保留不动——它服务规范层，保持声明式到运行时的映射源角色
- **D-10:** ActionButtonKind 保留（随 ActionButton 归入 action_button.rs），不合并到 BuiltinControl 中

### BuiltinControl 数据承载方式
- **D-11:** BuiltinControl 统一承载完整控件结构体（如 `BuiltinControl::TextInput(TextInputControl)`），不分离 kind 和 data
- **D-12:** RuntimeControl 的 inline 字段（如 `TextInput { value, placeholder }`）在 materialize 时直接构造对应完整结构体，消除 RuntimeControl 作为独立类型
- **D-13:** BuiltinControl 变体可直接通过 trait 调用行为方法，也可通过结构体字段访问数据——无需额外映射层

### Claude's Discretion
- controls/ 子目录内各文件的具体实现细节
- mod.rs 中 re-exports 的具体组织方式
- materialize 逻辑的调整顺序（先拆文件还是先统一枚举）
- helpers.rs vs mod.rs 中放工具函数的选择
- 替换 4 个枚举时的迁移策略（逐个替换还是一次性替换）

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Control Type Definitions (splitting targets)
- `src/components/controls.rs` — ControlKind 枚举、ControlFeedback 枚举、ActionButtonKind 枚举、所有 7 个控件结构体及其 ControlTrait impl、工具函数
- `src/components/control_trait.rs` — ControlTrait 定义（7 个方法），Phase 1 创建

### Runtime Layer (enum replacement targets)
- `src/runtime.rs` — ContentControl 枚举（line ~250）、RuntimeControl 枚举（line ~317）、OperationStatus（引用 ContentControl）
- `src/components/content_panel.rs` — SelectedControlKind 枚举（line ~97）、selected_control_kind() 方法

### Specification Layer (read-only, not replaced)
- `src/schema.rs` — ControlSpec 枚举（line ~363，私有）、FieldSpec::materialize() 转换逻辑
- `src/field.rs` — 工厂函数（理解控件创建 API）

### Module Structure
- `src/components/mod.rs` — Component trait 定义、当前的 pub use re-exports（需要添加 controls/ 子模块）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- 每个控件结构体已有独立的 `impl ControlTrait` 块（Phase 1），迁移到单独文件时直接移动即可
- `ControlTrait` 定义已在 `control_trait.rs` 中独立存在，拆分时只需调整 import 路径
- `ControlKind` 的 match 分发逻辑已在 Phase 1 迁移为 trait 调用，拆分后直接删除 ControlKind 即可

### Established Patterns
- `src/components/mod.rs` 的 pub use re-export 模式——controls/ 子目录可遵循相同模式
- `Component` trait 在 mod.rs 中定义——ControlTrait 在 control_trait.rs 中定义，拆分后模式统一
- Builder 模式用于所有配置——控件构造不需要改变

### Integration Points
- `content_panel.rs` 中的 `render_control()`、`handle_control_key()`、`control_value()` — 已改为 trait 调用，枚举替换后改用 BuiltinControl
- `runtime.rs` 中的 `ContentControl` — 被 OperationStatus::Running 引用，需要更新为 BuiltinControl
- `schema.rs` 中的 `materialize()` — 从 ControlSpec 映射到 RuntimeControl，消除 RuntimeControl 后直接映射到 BuiltinControl
- `showcase.rs` — 可能引用 ContentControl 和 RuntimeControl 的地方需要更新

### Key Differences to Reconcile
- RuntimeControl 有 `RefreshButton` 独立变体，BuiltinControl 将其吸收为 ActionButton 的 kind
- SelectedControlKind 是 field-less Copy 枚举——替换后需要从 BuiltinControl 提取 kind 信息，可能用 `std::mem::discriminant` 或新的 helper 方法
- OperationStatus::Running 内嵌 ContentControl——替换为 BuiltinControl 后引用路径改变

</code_context>

<specifics>
## Specific Ideas

- 拆分后每个控件文件预计 100-180 行（struct 定义 + ControlTrait impl + 渲染辅助），符合 ROADMAP 的 <200 行目标
- BuiltinControl 的 8 个变体：TextInput、NumberInput、Select、Toggle、ActionButton、StaticData、DynamicData、LogOutput
- mod.rs 负责 pub mod 声明、pub use re-exports、BuiltinControl 枚举定义和通用 impl（如批量 trait dispatch）

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-control-isolation-and-enum-unification*
*Context gathered: 2026-03-28*
