# Requirements: tui01 架构重构

**Defined:** 2026-03-28
**Core Value:** 新控件扩展只需改动 1-3 个文件，而非当前的 10 个文件

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### 控件扩展机制

- [x] **CTRL-01**: 定义统一 ControlTrait，包含 render、handle_key、value、validate、preferred_width 等方法
- [x] **CTRL-02**: 所有 8 个内置控件实现 ControlTrait
- [ ] **CTRL-03**: 引入 BuiltinControl 枚举替代 5 个镜像枚举（ControlKind、ContentControl、RuntimeControl、ControlSpec、SelectedControlKind）
- [x] **CTRL-04**: 每个控件类型独立文件，内聚 trait 实现和渲染逻辑
- [ ] **CTRL-05**: 新增控件类型只需改 1-3 个文件（控件 impl 文件、BuiltinControl 枚举、materialization 映射）
- [x] **CTRL-06**: ControlRegistry 支持通过字符串名称注册 Box<dyn ControlTrait> 自定义控件
- [x] **CTRL-07**: 所有 87+ 现有测试在重构后继续通过，不改变功能行为

### 模块组织

- [x] **MOD-01**: src/ 重组为领域子模块（spec/、runtime/、controls/、components/、host/、app/、infra/ 等）
- [x] **MOD-02**: content_panel.rs（1526 行）按职责拆分为聚焦的子模块
- [x] **MOD-03**: controls.rs（1008 行）按控件类型拆分为独立文件
- [x] **MOD-04**: executor.rs（870 行）按职责拆分（执行器、注册表、shell 命令、模板渲染）
- [x] **MOD-05**: showcase.rs（813 行）拆分，提取屏幕管理、操作轮询等子模块
- [x] **MOD-06**: lib.rs 通过 pub mod + re-export 保持稳定入口
- [x] **MOD-07**: 公开 API 可重新设计但保持语义等价

### 未来扩展点

- [x] **EXT-01**: Theme struct 定义（类型化、语义槽位如 border/selected/active/error/success、支持 Serde derive）
- [x] **EXT-02**: LayoutStrategy trait 定义，当前四分区布局作为默认实现
- [ ] **EXT-03**: 渲染路径预留 RenderContext 参数位（承载 Theme、Layout 信息）
- [ ] **EXT-04**: AppSpec 构建器支持链式配置 Theme 和 LayoutStrategy

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### 主题系统

- **THEME-01**: 支持 TOML/JSON 文件加载主题
- **THEME-02**: 运行时主题切换
- **THEME-03**: 主题继承与覆盖

### 灵活布局

- **LAYOUT-01**: 多种内置布局策略（侧边栏、标签页、分栏等）
- **LAYOUT-02**: 宿主应用自定义布局策略

### 插件系统

- **PLUGIN-01**: 动态加载外部控件（通过 libloading/FFI）
- **PLUGIN-02**: 控件生命周期管理（init、update、destroy）

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| 新字段类型实现 | 重构后结构要支持轻松添加，但本次不加具体类型 |
| 文档内容更新（docs/、README.md、CHANGELOG.md） | 留给重构完成后的独立阶段 |
| 主题文件加载/解析 | 依赖 Theme struct 但实现复杂度高，属于 v2 |
| CSS-like 主题 DSL | Rust 生态中类型化 Theme 是最佳实践，CSS 不适合 |
| 动态插件加载 | 需要unsafe FFI，复杂度过高，属于 v2+ |
| 性能优化 | 本次纯重构，不改变运行时性能特性 |
| 新增依赖（insta 等） | 保持依赖不变，除非扩展机制明确需要 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CTRL-01 | Phase 1: Control Trait Extraction | Complete |
| CTRL-02 | Phase 1: Control Trait Extraction | Complete |
| CTRL-03 | Phase 2: Control Isolation and Enum Unification | Pending |
| CTRL-04 | Phase 2: Control Isolation and Enum Unification | Complete |
| CTRL-05 | Phase 2: Control Isolation and Enum Unification | Pending |
| CTRL-06 | Phase 3: Custom Control Extension | Complete |
| CTRL-07 | All phases | Complete |
| MOD-01 | Phase 4: Module Hierarchy Restructuring | Complete (P01) |
| MOD-02 | Phase 5: Large File Decomposition | Complete |
| MOD-03 | Phase 2: Control Isolation and Enum Unification | Complete |
| MOD-04 | Phase 5: Large File Decomposition | Complete |
| MOD-05 | Phase 5: Large File Decomposition | Complete |
| MOD-06 | Phase 4: Module Hierarchy Restructuring | Complete (P01) |
| MOD-07 | Phase 6: Extension Points and Public API | Complete |
| EXT-01 | Phase 6: Extension Points and Public API | Complete |
| EXT-02 | Phase 6: Extension Points and Public API | Complete |
| EXT-03 | Phase 6: Extension Points and Public API | Pending |
| EXT-04 | Phase 6: Extension Points and Public API | Pending |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 18
- Unmapped: 0

---
*Requirements defined: 2026-03-28*
*Last updated: 2026-03-28 after roadmap creation*
