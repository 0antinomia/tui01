# Project Research Summary

**Project:** tui01 (Rust TUI framework -- brownfield refactoring)
**Domain:** Ratatui-based declarative form framework extensibility
**Researched:** 2026-03-28
**Confidence:** HIGH

## Executive Summary

tui01 is a ratatui-based Rust TUI framework that provides a declarative builder API (AppSpec / FieldSpec) for constructing terminal forms with operations. The framework is functionally complete with 87 passing tests, but suffers from a critical extensibility flaw: adding a new control type requires modifying 10+ files because five separate enums mirror the same control variants across layers, and six match-arm routing functions must be updated in lockstep. The research consensus across stack, feature, architecture, and pitfall analysis is clear -- introduce a `ControlTrait` that encapsulates control behaviors (render, handle_key, value, preferred_width), keep an exhaustive `BuiltinControl` enum for zero-cost dispatch on framework-defined controls, and reserve `Box<dyn ControlTrait>` for future plugin extensibility.

The recommended approach is a phased refactoring that introduces the trait abstraction first (no behavior change), then splits large files, then restructures the module hierarchy. Dependencies are locked (ratatui 0.30, crossterm 0.29, tokio 1.50) -- this is purely an internal architecture change. The refactoring must preserve all 87 existing tests at every phase boundary. Key table-stakes features for v1 are a centralized Theme struct with semantic slots, a LayoutStrategy trait to replace the hardcoded four-quadrant layout, and a ControlRegistry for named control lookup. These create the extension seams without implementing full feature sets.

The primary risk is the dual-state synchronization contract in ContentPanel, where blueprint indices must correspond exactly to runtime field_state indices. Splitting this across modules without making the invariant explicit will cause silent data corruption that passes existing tests. The secondary risk is cascading import breakage during module reorganization, mitigated by a strict one-move-at-a-time discipline with `cargo test` between every move.

## Key Findings

### Recommended Stack

Dependencies are fixed. The "stack" research is about architectural patterns, not technology choices. The recommended patterns align with ratatui's own architecture and the Rust API Guidelines.

**Core patterns:**
- **Sealed Component trait**: Prevents downstream implementation, protects against accidental semver breaks when adding methods. Consistent with Rust API Guidelines C-SEALED.
- **Enum dispatch for closed control set**: `BuiltinControl` enum provides zero-cost, exhaustive dispatch. Compiler catches missed match arms. ratatui itself uses this pattern extensively (Constraint, Modifier, Color).
- **Trait objects only for extension point**: `ContentControl::Custom(Box<dyn ControlTrait>)` reserved for host-app-defined controls. Avoids vtable overhead for built-in types.
- **Nested module hierarchy with mod.rs re-exports**: Domain-aligned directories (spec/, runtime/, controls/, components/, host/, infra/) matching ratatui's own organization.
- **Builder pattern preserved**: The existing declarative AppSpec/FieldSpec builder API is the framework's primary strength and must not be disrupted.

### Expected Features

**Must have (table stakes for v1 refactoring):**
- Control trait abstraction -- the single highest-impact change; reduces new-control cost from 10 files to 2-3
- Theme struct with semantic slots -- centralized color/style definitions replacing hardcoded values
- LayoutStrategy trait -- replaces hardcoded QuadrantLayout, enables future sidebar/tabs/single-panel layouts
- Control registration by name -- ControlRegistry maps string names to trait-object factories

**Should have (v0.3 feature release):**
- Built-in theme presets (dark, light, monochrome, high-contrast)
- Theme loading from TOML (Serde-derived Theme struct)
- Validation hooks per control (validate() method on ControlTrait)
- Configurable keyboard bindings

**Defer (v2+):**
- Derive macro for ControlTrait (requires proc-macro crate, wait for API stability)
- Subscription-based inter-component events (complex, wait for real use cases)
- Effects/animations via tachyonfx (integration point, not dependency)
- Layout composition via nesting (ratatui already supports it, add declarative tree later)

### Architecture Approach

The target architecture replaces the five-mirrored-enums pattern with a layered system: Specification Layer (spec/) produces Runtime Layer types (runtime/) via a single-direction materialization pipeline; the Runtime Layer owns data while the Control Layer (controls/) owns behavior through ControlTrait; the Component Layer (components/) orchestrates layout and rendering by calling trait methods, never matching on control variants.

**Major components:**
1. **ControlTrait** (src/controls/trait.rs) -- render, handle_key, value, preferred_width, is_editable, triggers_on_activate
2. **BuiltinControl enum** (src/controls/mod.rs) -- exhaustive wrapper for 8 built-in control types, zero-cost dispatch
3. **ContentControl enum** (src/runtime/block.rs) -- Builtin variant + Custom variant (Box<dyn ControlTrait>)
4. **ContentPanel** (src/components/content_panel/) -- split from 1526-line monolith into layout, render, interaction, operation_flow sub-modules
5. **spec/materialize.rs** -- single-location mapping from ControlSpec to RuntimeControl, the only place new control variants add a pipeline stage

### Critical Pitfalls

1. **Dual-state synchronization breakage** -- ContentPanel's blueprint and field_states must stay index-aligned. Add debug assertions and typed FieldIndex newtype before splitting. Keep both behind a single facade.
2. **Triple-enum naive consolidation** -- Do NOT collapse ControlSpec, RuntimeControl, and ControlKind into one enum. They serve distinct layers. Use conversion traits (From/Into) between them.
3. **Import cascade during module moves** -- Move one file, run cargo test, commit. showcase.rs is the canary (imports from 10 modules). Extract shared types (OperationSource) to canonical locations before moving dependents.
4. **Prelude re-export explosion** -- Only put long-term public API items in prelude.rs. Use pub(crate) for internal migration paths. Define target API surface before reorganizing.
5. **Async test flakiness from executor changes** -- Split executor.rs structurally but keep the tokio::spawn + mpsc logic identical. Do not change async mechanism during file splitting.

## Implications for Roadmap

Based on the combined research, the following phase structure is recommended. The ordering is driven by dependency analysis: traits before enums, enums before file splits, file splits before directory moves.

### Phase 1: Control Trait Extraction
**Rationale:** Foundation for all subsequent work. Introduces ControlTrait without changing any existing behavior or removing any enums.
**Delivers:** ControlTrait definition in controls/trait.rs, implementations for all 8 existing control types
**Addresses:** Control trait abstraction (P1 feature), Widget trait for custom controls (table stakes)
**Avoids:** Refactoring everything at once (Pitfall anti-pattern 4)

### Phase 2: Control File Splitting
**Rationale:** Each control type gets its own file. Depends on Phase 1 trait existing so implementations have a home.
**Delivers:** Individual files (text_input.rs, select.rs, toggle.rs, etc.), co-located trait impls
**Addresses:** Reducing new-control touch points
**Avoids:** Co-locate trait impls with type definitions to prevent orphan confusion (Pitfall 6)

### Phase 3: BuiltinControl Enum Unification
**Rationale:** Replaces the scattered ControlKind/ContentControl/RuntimeControl with a single BuiltinControl enum plus ContentControl wrapper. Depends on Phase 2 (individual types in own files).
**Delivers:** BuiltinControl enum with exhaustive dispatch, ContentControl enum with Custom variant for future plugins
**Addresses:** Eliminating enum mirroring (architecture anti-pattern 1)
**Avoids:** Triple-enum consolidation trap -- use conversion traits, not merged enum (Pitfall 1)

### Phase 4: Module Hierarchy Restructuring
**Rationale:** Move files into domain-aligned directories (spec/, runtime/, controls/, components/, host/, app/, infra/). Depends on Phases 1-3 (file contents are stable).
**Delivers:** Clean module hierarchy, updated lib.rs and prelude.rs
**Addresses:** Public API clarity, module-per-domain organization
**Avoids:** Import cascade -- move one module at a time, run cargo test after each (Pitfall 3)

### Phase 5: Large File Splitting
**Rationale:** Break content_panel.rs (1526 lines) into sub-modules, executor.rs (870 lines) into executor + action_registry. Depends on Phase 4 (directory structure exists).
**Delivers:** No file over ~300 lines, focused single-responsibility modules
**Addresses:** ContentPanel complexity (scaling priority 2), God File anti-pattern
**Avoids:** Dual-state synchronization breakage -- add assertions before splitting content_panel (Pitfall 4), executor async flakiness (Pitfall 11)

### Phase 6: Extension Points and Theme Foundation
**Rationale:** With clean architecture in place, add the theme struct and layout strategy trait as extension seams. These are the table-stakes features identified in feature research.
**Delivers:** Theme struct with semantic slots, LayoutStrategy trait, ControlRegistry for named lookup, RenderContext struct reserved for theme parameterization
**Addresses:** Theme struct (P1), Layout trait (P1), Control registration (P1)
**Avoids:** Global mutable theme (anti-feature), CSS-like DSL (anti-feature)

### Phase Ordering Rationale

- **Traits before restructuring:** ControlTrait must exist before files can be split, because trait impls need a definition site. Architecture research Phase 1 proves this.
- **Enum consolidation before moves:** The new enum types must be in place before directory reorganization, because moving files while also changing types doubles the import fix surface.
- **Module moves before file splits:** Directory restructuring first establishes the target structure; then large files split within their new homes.
- **Theme/Layout last:** These are additive features that depend on the refactored module structure. Building them on the old architecture would create throwaway work.
- **Feature research dependency graph confirms:** Theme requires nothing (foundation). Widget trait requires theme (render signature). Layout is orthogonal. Plugin registration requires widget trait. Our phase order respects these dependencies.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3 (BuiltinControl Enum Unification):** Complex type-level refactoring across 5 enums. The exact mapping between old and new types needs careful planning. Consider `/gsd:research-phase` for conversion strategy.
- **Phase 5 (Large File Splitting):** ContentPanel has 1526 lines with subtle index arithmetic. The splitting strategy for dual-state synchronization needs detailed analysis. Consider `/gsd:research-phase` for content_panel decomposition.
- **Phase 6 (Theme Foundation):** RenderContext design affects the public ControlTrait API. Needs research into how ratatui themes, ratatui-themes crate, and Brick's AttrMap handle style parameterization.

Phases with standard patterns (skip additional research):
- **Phase 1 (Control Trait Extraction):** Pure trait definition, well-documented in Rust API Guidelines.
- **Phase 2 (Control File Splitting):** Mechanical file moves with co-located impls.
- **Phase 4 (Module Hierarchy):** Standard Rust module reorganization, covered by Rust Reference visibility rules.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Patterns from ratatui's own architecture, Rust API Guidelines (official), and direct codebase analysis. No technology choices to validate. |
| Features | HIGH | Feature landscape sourced from 6 TUI frameworks across ecosystems. Prioritization matrix grounded in concrete complexity estimates. MVP definition directly tied to codebase analysis. |
| Architecture | HIGH | 6-phase build order derived from direct analysis of tui01 source files. Component responsibilities mapped to current file locations and line counts. Anti-patterns identified from existing code. |
| Pitfalls | HIGH | 15 pitfalls grounded in direct codebase analysis (specific file names, line numbers, import paths). Phase-specific warnings tied to actual file sizes and test counts. |

**Overall confidence:** HIGH

### Gaps to Address

- **RenderContext final design:** The ControlTrait render signature will change when themes are added. Phase 6 should prototype RenderContext before committing to it. The current parameter list (area, buf, selected, active, feedback) works but is growing. Consider bundling into a struct earlier if it simplifies the trait signature.
- **Clone requirement on ContentControl:** The `ContentControl::Custom(Box<dyn ControlTrait>)` variant requires `clone_boxed()` for snapshot storage. This is awkward. During Phase 3, evaluate whether controls need Clone or whether snapshots should use a different mechanism (e.g., storing only the value, not the full control state).
- **Test coverage for index synchronization:** Existing 87 tests may not cover all section/block combinations in ContentPanel. Before Phase 5, add property-based tests that validate blueprint/field_state alignment for arbitrary section/block configurations.
- **insta snapshot testing:** STACK.md recommends insta for rendering regression tests. This is a development dependency decision that should be evaluated during Phase 2 when controls are isolated into individual files.

## Sources

### Primary (HIGH confidence)
- Ratatui v0.30.0 ARCHITECTURE.md -- workspace structure, stability tiers, re-export patterns
- Ratatui official docs (ratatui.rs) -- Widget, StatefulWidget, WidgetRef traits; Layout system; Component patterns
- Rust API Guidelines (rust-lang.github.io/api-guidelines/) -- C-SEALED, C-STRUCT-PRIVATE, C-NEWTYPE-HIDE, C-REEXPORT-ROOT
- Rust Reference: Visibility and Privacy -- pub, pub(crate), module boundary rules
- tui01 codebase direct analysis -- STRUCTURE.md, STACK.md, source file analysis (controls.rs, runtime.rs, content_panel.rs, schema.rs, executor.rs, showcase.rs)

### Secondary (MEDIUM confidence)
- Predrag Radovic sealed traits guide -- comprehensive sealing strategies including partial sealing
- tui-realm (GitHub) -- MockComponent trait, Props system, Subscriptions, derive macro patterns
- Brick (Hackage) -- AttrMap hierarchical attribute system, Theme type with presets
- ratatui-themes (GitHub) -- 50+ theme presets, Serde-based theme loading
- ratatui-garnish (docs.rs) -- Flat decorator pattern for widget styling
- tachyonfx (GitHub) -- Effects and animation library for ratatui
- corrode.dev: Long-term Rust Project Maintenance -- practical maintenance advice
- Arroyo Blog: Plugin Systems in Rust -- host/plugin architecture patterns

### Tertiary (LOW confidence)
- Textual CSS Styling Guide (blog post) -- CSS-like theme system, useful as anti-pattern reference only
- AppCUI-rs (Hacker News discussion) -- extensible Rust TUI framework patterns
- Reddit: Rust Modules Best Practices -- community consensus, variable quality

---
*Research completed: 2026-03-28*
*Ready for roadmap: yes*
