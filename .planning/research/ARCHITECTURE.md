# Architecture Research

**Domain:** Rust TUI framework (library crate) -- refactoring for extensibility
**Researched:** 2026-03-28
**Confidence:** HIGH (based on direct source analysis + ratatui ecosystem patterns)

## Problem Statement

tui01 is a ratatui-based TUI framework with a critical extensibility flaw: adding a new control type requires modifying 10+ files because five separate enums (`ControlKind`, `ContentControl`, `RuntimeControl`, `ControlSpec`, `SelectedControlKind`) each mirror the same set of control variants, and six match-arm routing functions must be updated in lockstep.

This research recommends a target architecture that reduces new-control addition to 1-2 file changes while preserving Rust's type safety and performance characteristics.

## Recommended Architecture

### System Overview

```
+------------------------------------------------------------------+
|                     Public API Layer (prelude)                    |
|  AppSpec, PageSpec, SectionSpec, FieldSpec, field::*, RuntimeHost |
+------------------------------------------------------------------+
         |                              |
         v                              v
+---------------------+    +---------------------------+
| Specification Layer |    | Host Integration Layer    |
| ControlSpec         |    | RuntimeHost, ShellPolicy  |
| FieldSpec           |    | ActionRegistry            |
| Materialization     |    | OperationExecutor         |
+---------------------+    +---------------------------+
         |                              |
         v                              v
+------------------------------------------------------+
|                    Runtime Layer                       |
|  ContentBlueprint, ContentSection, ContentBlock       |
|  ContentControl (single unified enum)                 |
|  OperationSpec, ContentRuntimeState                   |
+------------------------------------------------------+
         |
         v
+------------------------------------------------------+
|                  Control Layer                         |
|  Control trait (render + handle_key + value + width)  |
|  Built-in implementations: text_input.rs, select.rs,  |
|    toggle.rs, action_button.rs, data_display.rs,      |
|    log_output.rs, number_input.rs                      |
|  BuiltinControl enum (exhaustive dispatch)            |
+------------------------------------------------------+
         |
         v
+------------------------------------------------------+
|                  Component Layer                       |
|  Component trait, QuadrantLayout, TitlePanel,         |
|  StatusPanel, MenuComponent, ContentPanel             |
|  (ContentPanel delegates to Control trait, not enum)  |
+------------------------------------------------------+
         |
         v
+------------------------------------------------------+
|               Infrastructure Layer                     |
|  Tui (terminal lifecycle), EventHandler, Event, Key   |
|  FrameworkLogger                                      |
+------------------------------------------------------+
```

### Key Architectural Change: Control Trait

The central innovation is replacing the five-mirrored-enums pattern with a **Control trait** that each control type implements, combined with a single `ContentControl` enum that wraps `Box<dyn ControlTrait>` for user-defined controls or uses a `BuiltinControl` enum for zero-cost dispatch on built-in types.

### Component Responsibilities

| Component | Responsibility | Current Location | Proposed Location |
|-----------|----------------|------------------|-------------------|
| `ControlTrait` | Render, key handling, value extraction, width hint for any control | (does not exist) | `src/controls/trait.rs` |
| `BuiltinControl` | Exhaustive enum for built-in controls, zero-cost dispatch | `ControlKind` in controls.rs + `ContentControl` in runtime.rs | `src/controls/builtin.rs` |
| Individual controls | Per-type state, render logic, key handling | All in controls.rs (1008 lines) | `src/controls/text_input.rs`, `src/controls/select.rs`, etc. |
| `ContentBlock` | Label + control + operation binding | runtime.rs | `src/runtime/block.rs` |
| `ContentPanel` | Form layout, pagination, operation lifecycle | content_panel.rs (1526 lines) | `src/components/content_panel/` (split into sub-modules) |
| `OperationExecutor` | Shell commands, registered actions, async execution | executor.rs (870 lines) | `src/host/executor.rs` |
| `ShowcaseApp` | TEA core: event/action/render orchestration | showcase.rs (813 lines) | `src/app/showcase.rs` |

## Recommended Project Structure

```
src/
+-- lib.rs                          # Public API surface (pub mod declarations)
+-- prelude.rs                      # Recommended imports for host apps
|
+-- spec/                           # Declarative specification layer
|   +-- mod.rs                      # pub use re-exports
|   +-- app.rs                      # AppSpec builder with validation
|   +-- page.rs                     # PageSpec, SectionSpec
|   +-- field.rs                    # FieldSpec, factory functions
|   +-- control_spec.rs             # ControlSpec enum (spec-layer only)
|   +-- materialize.rs              # ControlSpec -> RuntimeControl conversion
|
+-- runtime/                        # Runtime state layer
|   +-- mod.rs                      # pub use re-exports
|   +-- blueprint.rs                # ContentBlueprint, ContentSection
|   +-- block.rs                    # ContentBlock, ContentControl enum
|   +-- operation.rs                # OperationSpec, OperationSource, OperationStatus
|   +-- state.rs                    # ContentRuntimeState, RuntimeFieldState
|   +-- migration.rs                # RuntimePage/Field/Section + From impls
|
+-- controls/                       # Control type implementations
|   +-- mod.rs                      # ControlTrait, BuiltinControl, pub re-exports
|   +-- trait.rs                    # ControlTrait definition + blanket impls
|   +-- text_input.rs               # TextInputControl
|   +-- number_input.rs             # NumberInputControl
|   +-- select.rs                   # SelectControl
|   +-- toggle.rs                   # ToggleControl
|   +-- action_button.rs            # ActionButtonControl
|   +-- data_display.rs             # DataDisplayControl (static + dynamic)
|   +-- log_output.rs               # LogOutputControl
|
+-- components/                     # UI region components
|   +-- mod.rs                      # Component trait + pub re-exports
|   +-- quadrant.rs                 # QuadrantLayout
|   +-- title_panel.rs              # TitlePanel
|   +-- status_panel.rs             # StatusPanel
|   +-- menu.rs                     # MenuComponent
|   +-- content_panel/              # ContentPanel (split from 1526-line monolith)
|       +-- mod.rs                  # ContentPanel struct, public API
|       +-- layout.rs               # Page layout, pagination calculations
|       +-- render.rs               # Frame rendering logic
|       +-- interaction.rs          # Key handling, control activation
|       +-- operation_flow.rs       # Operation request/result management
|
+-- host/                           # Host integration layer
|   +-- mod.rs                      # pub use re-exports
|   +-- runtime_host.rs             # RuntimeHost, ShellPolicy, ExecutionPolicy
|   +-- executor.rs                 # OperationExecutor, ActionRegistry
|   +-- action_registry.rs          # ActionRegistry (extracted from executor)
|   +-- logger.rs                   # FrameworkLogger
|
+-- app/                            # Application layer (TEA core)
|   +-- mod.rs                      # pub use re-exports
|   +-- showcase.rs                 # ShowcaseApp (TEA orchestrator)
|   +-- action.rs                   # Action enum, FocusTarget
|
+-- infra/                          # Infrastructure layer
|   +-- mod.rs                      # pub use re-exports
|   +-- tui.rs                      # Terminal lifecycle (Tui struct)
|   +-- event.rs                    # EventHandler, Event, Key
|
+-- main.rs                         # Demo binary entry point
+-- bin_app.rs                      # Default App wrapper (current app.rs)
```

### Structure Rationale

- **`spec/`**: The declarative builder API is the primary surface for host applications. Grouping it together makes the "how to define a UI" story clear. `materialize.rs` isolates the spec-to-runtime conversion so new control types only add one mapping.

- **`runtime/`**: Runtime state is separate from rendering. `migration.rs` holds the `RuntimeControl` enum and its `From<RuntimeControl> for ContentBlock` conversion, which is the only place new variants need adding on the runtime side.

- **`controls/`**: Each control type is its own file. The `ControlTrait` defines the contract. `BuiltinControl` enum provides exhaustive matching for the framework's internal use. Adding a new built-in control means: (1) creating a new file, (2) adding a variant to `BuiltinControl`, (3) adding a mapping in `materialize.rs`. Three files, all local to the control's concern.

- **`components/content_panel/`**: The 1526-line file splits into four focused sub-modules. `interaction.rs` is the key file that previously needed enum-match updates -- with the `ControlTrait`, it calls `control.handle_key(key)` instead of matching on `ContentControl` variants.

- **`host/`**: Host integration concerns are grouped together. `executor.rs` (870 lines) is split, with action registry in its own file.

- **`app/`**: The TEA core is isolated. `showcase.rs` stays large but no longer directly matches on control variants.

## Architectural Patterns

### Pattern 1: Control Trait with Builtin Enum Wrapper

**What:** Define a trait that captures all control behaviors, then wrap built-in implementations in an enum for zero-cost dispatch while allowing `Box<dyn ControlTrait>` for extensibility.

**When to use:** This is the primary pattern for the control system.

**Trade-offs:**
- Pro: New controls only implement the trait -- no enum variant needed for custom controls
- Pro: Built-in controls get exhaustive matching (compiler catches missed cases) via `BuiltinControl` enum
- Pro: `ContentPanel` calls trait methods instead of matching on variants
- Con: Slight indirection for the enum-wrapper approach (one extra method call)
- Con: Trait objects (`dyn ControlTrait`) have vtable overhead -- but we avoid them for built-ins

**Example:**

```rust
// src/controls/trait.rs

/// Core behaviors every control must implement.
pub trait ControlTrait {
    /// Render the control into a buffer.
    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        active: bool,
        feedback: ControlFeedback,
    );

    /// Handle a key press. Returns true if the control consumed the key.
    fn handle_key(&mut self, key: Key) -> bool;

    /// Extract the current value as a string (for template parameterization).
    fn value(&self) -> String;

    /// Preferred render width in terminal columns.
    fn preferred_width(&self) -> u16;

    /// Whether this control accepts user editing (Enter activates edit mode).
    fn is_editable(&self) -> bool {
        false
    }

    /// Whether this control triggers an operation on activation (buttons, toggles).
    fn triggers_on_activate(&self) -> bool {
        false
    }

    /// Clone into a boxed trait object (for snapshot storage).
    fn clone_boxed(&self) -> Box<dyn ControlTrait>;
}
```

```rust
// src/controls/mod.rs

/// Built-in control enum for exhaustive, zero-cost dispatch.
/// Adding a new built-in control: add variant here + implement ControlTrait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinControl {
    TextInput(TextInputControl),
    NumberInput(NumberInputControl),
    Select(SelectControl),
    Toggle(ToggleControl),
    ActionButton(ActionButtonControl),
    StaticData(DataDisplayControl),
    DynamicData(DataDisplayControl),
    LogOutput(LogOutputControl),
}

/// Unified control type used in ContentBlock.
#[derive(Debug, Clone)]
pub enum ContentControl {
    Builtin(BuiltinControl),
    Custom(Box<dyn ControlTrait>),
}
```

### Pattern 2: Single-Direction Materialization Pipeline

**What:** Control types flow in one direction: `ControlSpec` -> `RuntimeControl` -> `ContentControl`. Each step is a pure data transformation in a dedicated module.

**When to use:** Always -- this replaces the current scattered conversion logic.

**Trade-offs:**
- Pro: New control type adds one mapping per pipeline stage (3 files total: spec, migration, control)
- Pro: Each stage is independently testable
- Con: More files to navigate, but each file is smaller and focused

**Example pipeline:**

```
Host defines:  FieldSpec::text_input("Label", "value", "hint")
                     |
                     v  (in spec/materialize.rs)
              RuntimeControl::TextInput { value, placeholder }
                     |
                     v  (in runtime/migration.rs, From impl)
              ContentBlock { label, control: ContentControl::Builtin(BuiltinControl::TextInput(...)) }
```

### Pattern 3: Module-as-Boundary Visibility

**What:** Use Rust's `pub(crate)` and `pub` visibility to enforce architectural boundaries. The `controls/` module exposes only `ControlTrait`, `ContentControl`, and individual control types. Internal implementation details (render helpers, width constants) stay `pub(crate)`.

**When to use:** Throughout the refactoring.

**Trade-offs:**
- Pro: Compiler enforces that `components/` cannot reach into control internals
- Pro: Public API surface is explicit and small
- Con: Requires careful `pub(crate)` annotation during refactoring
- Con: Some test files may need `#[cfg(test)]` overrides or test-module-level re-exports

## Data Flow

### Request Flow (User Interaction)

```
[User Key Press]
    |
    v
EventHandler (tokio task) --> Event::Key(key)
    |
    v
ShowcaseApp::handle_event(event)
    |
    +-- FocusTarget::Menu --> MenuComponent::handle_events --> Action::MenuSelect
    |
    +-- FocusTarget::Content --> ContentPanel::handle_events
                                        |
                                        v
                              ContentPanel::handle_control_key(key)
                                        |
                                        v
                              control.handle_key(key)   // trait method, no match
                                        |
                                        v  (on Enter/activate)
                              ContentPanel::activate_selected_control()
                                        |
                                        v
                              operation_request assembled from control.value()
                                        |
                                        v
                              ShowcaseApp::submit_operation()
                                        |
                                        v
                              OperationExecutor::submit() --> tokio task
```

### State Management Flow

```
[ContentPanel]
    |
    +-- blueprint: ContentBlueprint (static definition)
    +-- runtime: ContentRuntimeState (mutable field states)
    |
    +-- RuntimeFieldState {
    |       control: ContentControl,     // current control state
    |       status: OperationStatus,     // idle/running/success/failure
    |       snapshot: Option<ContentControl>,  // pre-edit snapshot
    |   }
    |
    +-- On screen switch: persist_active_screen_content()
    |   Copies runtime.control back into blueprint for persistence
    |
    +-- On operation result: update field_state.status + append to log target
```

### Key Data Flows

1. **Specification materialization:** `AppSpec` -> `PageSpec::materialize()` -> `RuntimePage` -> `From impl` -> `ContentBlueprint` -> loaded into `ContentPanel`. This is a one-way transformation with no back-reference.

2. **Operation lifecycle:** Control activation produces `OperationRequest` -> `OperationExecutor` spawns tokio task -> result returns via `mpsc` channel -> `ContentPanel` updates `OperationStatus` -> log targets receive output. This is async and decoupled by channels.

3. **Control value extraction:** `control.value()` (trait method) used for template parameterization in `OperationRequest`. No enum matching needed.

## Build Order Implications

The refactoring must proceed in a specific order due to dependencies. Here is the recommended phase structure with explicit dependency reasoning:

### Phase 1: Extract Control Trait (no behavior change)

**What:** Define `ControlTrait` in a new `src/controls/trait.rs`. Implement it for all existing control structs via a blanket `impl` or per-type impl blocks. Do NOT remove any enums yet.

**Why first:** This is the foundation everything else depends on. It introduces the abstraction without breaking anything.

**Files changed:** New files only. `src/controls/trait.rs`, `src/controls/mod.rs` (new). Existing `controls.rs` gains `impl ControlTrait for ...` blocks.

**Tests:** All 87 existing tests continue to pass unchanged.

### Phase 2: Split controls.rs into Individual Files

**What:** Move each control struct from `src/components/controls.rs` into `src/controls/text_input.rs`, `select.rs`, etc. Update `mod.rs` re-exports. No behavioral changes.

**Why second:** Depends on Phase 1 (trait definition exists). After this, each control is independently editable.

**Files changed:** File moves only. `src/controls/text_input.rs`, etc. are new. `src/components/controls.rs` becomes thin re-export or is deleted.

**Tests:** Tests move with their respective control files. All pass.

### Phase 3: Introduce BuiltinControl Enum

**What:** Create `BuiltinControl` enum in `src/controls/mod.rs` that wraps the individual control types. Implement `ControlTrait` for `BuiltinControl` by delegating to the inner type. Replace `ControlKind` (in current `controls.rs`) with `BuiltinControl`.

**Why third:** Depends on Phase 2 (individual control files exist). This unifies the render-dispatch enum with the runtime enum, eliminating one of the five duplicates.

**Files changed:** `src/controls/mod.rs`, `src/components/content_panel.rs` (switch `render_control` to use `BuiltinControl::render`).

**Tests:** Update import paths. Behavior unchanged.

### Phase 4: Unify ContentControl with BuiltinControl

**What:** Replace `ContentControl` enum in `runtime.rs` with a new version that wraps `BuiltinControl`. Replace `SelectedControlKind` with trait-based queries (`is_editable()`, `triggers_on_activate()`). Remove `ControlKind` entirely.

**Why fourth:** Depends on Phase 3. This is the largest change -- it removes the routing functions (`render_control`, `handle_control_key`, `control_value`, `selected_control_kind`) from content_panel.rs, replacing them with trait method calls.

**Files changed:** `src/runtime.rs`, `src/components/content_panel.rs`, `src/builder.rs` (is_log check).

**Tests:** This is the highest-risk phase. Run all 87 tests after every sub-step.

### Phase 5: Restructure Module Hierarchy

**What:** Create the directory structure (`spec/`, `runtime/`, `controls/`, `host/`, `app/`, `infra/`). Move files to new locations. Update `lib.rs`, `prelude.rs`, all `use` statements.

**Why fifth:** Depends on Phases 1-4 completing (the file contents are stable). This is a pure reorganization.

**Files changed:** All files (import paths change). No behavioral changes.

**Tests:** All pass after import fixes.

### Phase 6: Split Large Files

**What:** Break `content_panel` into sub-modules (`layout.rs`, `render.rs`, `interaction.rs`, `operation_flow.rs`). Break `executor.rs` into `executor.rs` + `action_registry.rs`. Break `showcase.rs` into focused sub-modules if warranted.

**Why sixth:** Depends on Phase 5 (new directory structure exists). This reduces file sizes without changing behavior.

**Files changed:** `src/components/content_panel/`, `src/host/`, `src/app/`.

**Tests:** All pass.

### Dependency Graph

```
Phase 1 (Control Trait)
    |
    v
Phase 2 (Split controls.rs)
    |
    v
Phase 3 (BuiltinControl enum)
    |
    v
Phase 4 (Unify ContentControl)  <-- highest risk, most test scrutiny
    |
    v
Phase 5 (Module hierarchy)      <-- most files touched
    |
    v
Phase 6 (Split large files)     <-- lowest risk, pure file moves
```

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 8-10 built-in controls (current) | `BuiltinControl` enum + `ControlTrait` is optimal. Enum gives exhaustive matching. |
| 15-20 built-in controls | Still fine with enum. Each control is isolated in its own file, so adding one is still 3-file change (control impl, enum variant, materialization mapping). |
| Custom controls from host apps | `ContentControl::Custom(Box<dyn ControlTrait>)` allows host apps to define their own controls without framework changes. This is the extension point for the plugin system (noted as future work). |
| Theme/style system | `ControlTrait::render()` accepts a style parameter (or `RenderContext` struct). Theme system provides `RenderContext`. Controls that don't customize use the theme defaults. |
| Flexible layout | `QuadrantLayout` becomes one implementation of a `Layout` trait. `ContentPanel` adapts to any layout. This is a separate concern from controls. |

### Scaling Priorities

1. **First bottleneck:** Control type addition cost -- solved by `ControlTrait` pattern.
2. **Second bottleneck:** ContentPanel complexity -- solved by splitting into sub-modules.
3. **Third bottleneck:** Public API surface clarity -- solved by module hierarchy.

## Anti-Patterns

### Anti-Pattern 1: Enum Mirroring (the current problem)

**What people do:** Create separate enums in each layer that mirror the same set of variants, then write conversion functions between them.

**Why it's wrong:** Adding a variant requires updating N enums + M match expressions. The compiler helps (exhaustive matching), but the labor is O(N*M) per addition.

**Do this instead:** Single enum (`BuiltinControl`) in the controls layer. Other layers use `ContentControl` which wraps it. Trait methods replace most match-based dispatch.

### Anti-Pattern 2: God File (content_panel.rs at 1526 lines)

**What people do:** Let a single file accumulate all related logic -- layout, rendering, interaction, operation management.

**Why it's wrong:** Changes to pagination logic risk breaking operation lifecycle. Merge conflicts increase. Navigation takes longer than the actual edit.

**Do this instead:** Sub-module directory (`content_panel/`) with focused files. `mod.rs` defines the public struct. Private sub-modules handle distinct concerns.

### Anti-Pattern 3: Premature Trait Objecting

**What people do:** Use `Box<dyn Trait>` everywhere because "it's extensible."

**Why it's wrong:** Virtual dispatch overhead on every render call (60+ fps). Loss of enum exhaustive matching. Cloning requires custom `clone_boxed()` methods. Debug/PartialEq become manual.

**Do this instead:** Use `BuiltinControl` enum for built-in types (zero-cost). Reserve `Box<dyn ControlTrait>` for the `Custom` variant only. The enum's match-based dispatch is the fast path.

### Anti-Pattern 4: Refactoring Everything at Once

**What people do:** Try to restructure modules, introduce traits, and split files all in one pass.

**Why it's wrong:** When tests break, you cannot isolate which change caused the regression. Rollback is all-or-nothing.

**Do this instead:** Follow the phased build order. Each phase is independently committable and testable. If Phase 4 breaks, you know it's the ContentControl unification, not the file moves.

## Integration Points

### External Dependencies

| Dependency | Integration Pattern | Notes |
|------------|---------------------|-------|
| ratatui 0.30 | `Widget` trait for rendering, `Buffer`/`Rect`/`Frame` types | Controls use `Buffer` directly (lower-level than `Widget`). This is fine -- `Widget` is for ratatui's own composition. |
| crossterm 0.29 | Terminal events via `crossterm::event` | Wrapped in `EventHandler`. No direct dependency from control code. |
| tokio | `mpsc` channels for async operations, spawned tasks | `OperationExecutor` is the sole tokio consumer in the control path. |
| color-eyre | Error handling in entry points | Not used inside control/render code (those are infallible). |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| spec/ <-> runtime/ | `From<RuntimeControl> for ContentBlock` in `runtime/migration.rs` | One-way: spec produces runtime types. Runtime never references spec. |
| runtime/ <-> controls/ | `ContentControl` wraps `BuiltinControl` from controls/ | Runtime owns the data; controls own the behavior. |
| controls/ <-> components/ | `ControlTrait` methods called by `ContentPanel` | ContentPanel never matches on control variants -- uses trait methods only. |
| components/ <-> app/ | `Component` trait, `Action` enum | Components produce Actions; ShowcaseApp handles them. |
| host/ <-> app/ | `OperationRequest`/`OperationResult` via channels | Async boundary. Host layer never touches UI state. |

## Extension Point Design

### Theme/Style System (reserved, not implemented)

The `ControlTrait::render()` signature should eventually accept a `RenderContext`:

```rust
pub struct RenderContext<'a> {
    pub area: Rect,
    pub buf: &'a mut Buffer,
    pub selected: bool,
    pub active: bool,
    pub feedback: ControlFeedback,
    pub theme: &'a dyn Theme,  // future: theme system
}
```

For now, the existing parameter list is fine. The `RenderContext` can be introduced as a non-breaking change later (control impls that don't use the theme still compile).

### Flexible Layout (reserved, not implemented)

`QuadrantLayout` can become one implementation of a `Layout` trait:

```rust
pub trait Layout {
    fn areas(&self, rect: Rect) -> LayoutAreas;
}
```

`ContentPanel` would accept a generic `L: Layout` or `Box<dyn Layout>`. This is a separate concern from controls and can be done independently.

### Plugin/Custom Controls (reserved, not implemented)

`ContentControl::Custom(Box<dyn ControlTrait>)` is the extension point. Host applications can:

```rust
struct MyDatePicker { /* ... */ }
impl ControlTrait for MyDatePicker { /* ... */ }

FieldSpec::custom("date", MyDatePicker::new())
```

This requires `FieldSpec` to accept custom controls, which is a future API addition. The `ControlTrait` design in Phase 1 enables this without further architectural changes.

## Sources

- Direct source analysis: `src/components/controls.rs`, `src/runtime.rs`, `src/schema.rs`, `src/components/content_panel.rs`, `src/builder.rs`, `src/lib.rs`
- ratatui GitHub discussion on Widget vs Component patterns (ratatui-org/ratatui)
- Rust API guidelines on trait design for extensibility
- ratatui `StatefulWidget` trait pattern as reference for stateful rendering

---

*Architecture research for: tui01 Rust TUI framework refactoring*
*Researched: 2026-03-28*
