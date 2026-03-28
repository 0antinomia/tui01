# Stack Research

**Domain:** Rust TUI library crate architecture (ratatui-based)
**Researched:** 2026-03-28
**Confidence:** HIGH

This research addresses the question: *What architectural patterns and module organization practices do well-structured Rust library crates (especially TUI frameworks) use? What makes a Rust library crate easy to extend with new components?*

The stack here is not about choosing dependencies -- those are fixed (ratatui 0.30, crossterm 0.29, tokio 1.50). This is about choosing *architectural patterns* for code organization, trait design, and extensibility within the existing stack.

## Recommended Architectural Patterns

### Core Pattern: Module Organization

| Pattern | Purpose | Why Recommended | Confidence |
|---------|---------|-----------------|------------|
| Nested module hierarchy with `mod.rs` re-exports | Group related code by domain/feature | Ratatui itself uses this (core/widgets/backends). Rust API Guidelines C-REEXPORT-ROOT. Prevents flat file sprawl. | HIGH |
| Prelude module for public API | Curated `pub use` for downstream consumers | Ratatui ships `ratatui::prelude::*`. Keeps import ergonomics clean while allowing internal reorganization. | HIGH |
| Private fields + public constructors | Hide internal state from downstream | Rust API Guidelines C-STRUCT-PRIVATE. Allows changing internals without semver breaks. | HIGH |
| Builder pattern for spec types | Construct complex declarative objects | Already used in tui01 (AppSpec builders). Ratatui convention for Widget construction. Consumes self, produces immutable result. | HIGH |

### Supporting Pattern: Trait Design for Extensibility

| Pattern | Purpose | When to Apply | Confidence |
|---------|---------|---------------|------------|
| Sealed trait (private supertrait) | Prevent downstream implementation of framework traits | Use for the `Component` trait and any trait where only framework code should implement. Prevents accidental breakage when adding methods. | HIGH |
| Enum dispatch for closed set | Exhaustive match on known variants without dynamic dispatch | Use for control types that the framework defines. Zero-cost, compile-time complete. Tradeoff: adding new variants requires touching match sites. | HIGH |
| Trait objects (`dyn Trait`) for open set | User-extensible polymorphism | Use for plugin/custom widget extension points. Tradeoff: dynamic dispatch, requires `Box`, cannot use `enum` exhaustiveness. | MEDIUM |
| Partially sealed trait | Some methods sealed, some overridable | Use for component lifecycle: seal `render()` (framework calls it), leave `on_key()` open for override. Balances control with extensibility. | MEDIUM |

### Development Pattern: Testing Architecture

| Pattern | Purpose | Notes | Confidence |
|---------|---------|-------|------------|
| Co-located `#[cfg(test)] mod tests` | Keep tests next to implementation | Already in use (87 tests across 13 files). No change needed. | HIGH |
| Snapshot testing for rendering | Verify widget output without manual assertions | Consider `insta` crate for rendering regression tests post-refactor. Protects against visual breakage. | MEDIUM |

## Recommended Module Structure

Based on ratatui's own architecture and Rust API Guidelines, the recommended internal organization:

```
src/
  lib.rs              -- Re-exports from submodules (public API surface)
  prelude.rs          -- Curated pub use for host applications
  app.rs              -- Application shell (ShowcaseApp)

  spec/               -- Declarative specification layer
    mod.rs            -- Re-exports: AppSpec, PageSpec, SectionSpec, FieldSpec
    builder.rs        -- Builder pattern constructors
    schema.rs         -- Spec data structures
    field.rs          -- Field factory functions

  runtime/            -- Runtime state and execution
    mod.rs            -- Re-exports: ContentBlueprint, OperationSpec, etc.
    state.rs          -- ContentRuntimeState and related types
    host.rs           -- RuntimeHost, ShellPolicy, ExecutionPolicy
    executor.rs       -- OperationExecutor, ActionRegistry (split from current)
    action.rs         -- Action enum
    event.rs          -- EventHandler, Event, Key

  render/             -- Rendering infrastructure
    mod.rs            -- Re-exports: Component trait, layout types
    component.rs      -- Component trait definition (sealed)
    layout.rs         -- QuadrantLayout (renamed from quadrant.rs)
    title_panel.rs    -- TitlePanel
    status_panel.rs   -- StatusPanel
    menu.rs           -- MenuComponent
    content/          -- Content panel split by responsibility
      mod.rs          -- ContentPanel orchestrator
      form.rs         -- Form layout and pagination
      controls.rs     -- Control rendering dispatch
      operations.rs   -- Operation status management

  infra/              -- Infrastructure and utilities
    mod.rs
    tui.rs            -- Terminal lifecycle management
    logging.rs        -- FrameworkLogger
```

### Why This Structure

1. **Domain-aligned directories**: `spec/`, `runtime/`, `render/`, `infra/` map directly to the three layers in tui01's architecture (declarative spec -> runtime state -> rendering). This is the same principle ratatui uses (core/widgets/backends).

2. **`content_panel.rs` split into `render/content/`**: The current 1526-line file mixes form layout, pagination, control dispatch, and operation lifecycle. Each becomes its own file under `render/content/`, with `mod.rs` holding the `ContentPanel` struct that composes them.

3. **Controls stay centralized in `render/content/controls.rs`**: All control rendering stays in one file. What changes is the dispatch mechanism (see Extension Mechanism below).

4. **Flat files eliminated**: No file over ~300 lines (vs current 1526, 1008, 870, 813). Each file has a single, clear responsibility.

## Extension Mechanism: Adding a New Control

The core goal: reduce new-control addition from 10 files to 1-2 files.

### Current Problem (10 files)

1. Control struct in `controls.rs`
2. `ContentControl` enum variant in `runtime.rs`
3. `ControlKind` enum variant in `controls.rs`
4. `SelectedControlKind` variant in `content_panel.rs`
5. Factory function in `field.rs`
6. `ContentBlock` constructor in `runtime.rs`
7. `FieldSpec` constructor in `schema.rs`
8. `ControlSpec` mapping in `schema.rs`
9. Render routing in `content_panel.rs`
10. Key handling in `content_panel.rs`

### Recommended Approach: Enum Dispatch + Centralized Registration

Keep enum-based dispatch (not trait objects) for the core control types. The framework defines the closed set of controls. This is the right choice because:

- **Performance**: Zero-cost dispatch, no `Box` allocation on every render frame.
- **Exhaustiveness**: Compiler catches missed match arms during refactoring.
- **Ratatui alignment**: Ratatui itself uses enums for layout (`Constraint`), styles (`Modifier`), and text (`Line`, `Span` variants via the type system).
- **Clarity**: All control variants visible in one enum definition.

But restructure so the "registration" is concentrated:

**File 1: `render/content/controls.rs`** (the only file for rendering logic):
```rust
// Define control data types, render methods, key handlers
// Each control type is a self-contained impl block
pub struct TextInput { /* fields */ }
impl TextInput {
    pub fn render(...) { /* ... */ }
    pub fn handle_key(...) { /* ... */ }
}
```

**File 2: `render/content/controls/mod.rs`** or `render/content/control_dispatch.rs`:
```rust
// Single match dispatch for render + key handling
// Adding a new control: add one match arm here
fn render_control(kind: &ControlKind, ...) {
    match kind {
        ControlKind::Text(c) => c.render(...),
        ControlKind::Select(c) => c.render(...),
        ControlKind::NewType(c) => c.render(...), // <- ONE LINE
    }
}
```

**File 3 (optional): `spec/field.rs`** -- add factory function:
```rust
pub fn new_type(...) -> FieldSpec { /* ... */ }
```

This reduces the touch points from 10 to 2-3 files. The key insight: co-locate the data type, rendering, and key handling for each control in the same module, then have a single dispatch file.

### For Plugin/Custom Controls (Future)

Reserve an extension point using trait objects:

```rust
// In render/component.rs -- the open extension point
pub trait CustomWidget: Send + Sync {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn handle_key(&mut self, key: KeyEvent) -> bool;
}

// In the control enum:
pub enum ControlKind {
    // ... built-in variants (enum dispatch, zero cost)
    Custom(Box<dyn CustomWidget>), // open extension point
}
```

This gives the best of both worlds: zero-cost for built-in controls, extensibility for user-defined ones.

## Key Trait Design Decisions

### Component Trait: Seal It

```rust
mod private { pub trait Sealed {} }

pub trait Component: private::Sealed {
    fn render(&mut self, f: &mut Frame, area: Rect);
    fn handle_key(&mut self, key: KeyEvent) -> Option<Action>;
}
```

**Why seal**: The framework calls `Component::render()` internally. If downstream users implement it, adding a method (e.g., `fn on_focus(&mut self)`) becomes a breaking change. Sealing prevents this. Ratatui's `Widget` trait is effectively sealed through the "consumable builder" pattern (takes `self`, not `&self`).

**Confidence**: HIGH -- Rust API Guidelines C-SEALED explicitly recommends this.

### ControlKind Enum: Keep It

Do not replace `ControlKind` with `Box<dyn ControlTrait>`. The enum is correct for the framework-defined control set. Reserve trait objects for the `Custom` variant only.

**Why**: Compile-time exhaustiveness checking is invaluable during refactoring. Every `match` on `ControlKind` will produce a compiler error if a variant is missed. This safety net is worth the trade-off of touching more files.

**Confidence**: HIGH -- ratatui itself uses enums extensively (Constraint, Modifier, Color).

### Spec Types: Newtype for API Stability

```rust
pub struct FieldSpec { inner: FieldSpecInner }
```

Hide the internal structure of spec types behind newtypes or private fields. This allows changing the spec representation (e.g., adding validation metadata) without breaking the builder API.

**Confidence**: MEDIUM -- Rust API Guidelines C-NEWTYPE-HIDE recommends this, but the current builder pattern already provides some encapsulation.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Enum dispatch for controls | Pure trait objects (`Box<dyn Control>`) | When user-defined controls are a primary use case (not just future extension point) |
| Sealed Component trait | Open Component trait | When you want users to implement their own top-level components (but this conflicts with framework-controlled rendering) |
| Module-per-domain directories | Flat `src/` with naming conventions | When the project is small (< 2000 lines). tui01 is 6000+ lines -- directories are warranted. |
| Co-located tests | Separate `tests/` integration tests | When you need to test across module boundaries. Keep co-located for unit tests, add integration tests for cross-module behavior. |
| `mod.rs` re-export pattern | `name.rs` + `name/` directory pattern (Rust 2018+) | Either works. `mod.rs` is more conventional and matches ratatui's style. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Macro-based control registration | Macros hide control flow, make debugging harder, and add implicit coupling. The 2-3 file approach is explicit and debuggable. | Explicit match dispatch with co-located control definitions |
| Separate workspace crates for tui01 | tui01 is 6000 lines, not 60,000. Workspace splits add compilation and publishing overhead without benefit at this scale. | Module directories within a single crate |
| `impl Trait` return types in public API | Limits callers to a single concrete type, prevents trait object usage. Use concrete types or `Box<dyn Trait>` explicitly. | Concrete return types with `pub use` re-exports |
| Glob re-exports (`pub use module::*`) in lib.rs | Pollutes namespace, creates import conflicts, makes API surface unclear. | Explicit `pub use` of each named type/trait |

## Stack Patterns by Variant

**If adding theme/styling system later:**
- Define a `Theme` trait with methods for each styled element
- Use `Theme` as a parameter on `Component::render()` rather than a global/static
- Store theme state in the application shell, pass down through render calls
- This is how ratatui's `Style` works -- explicit, composable, no globals

**If adding flexible layout (non-four-quadrant):**
- Define a `LayoutStrategy` trait: `fn layout(&self, area: Rect) -> Vec<Rect>`
- Implement `QuadrantLayout` as one strategy, allow custom implementations
- The `Component` trait's render method already takes `area: Rect` -- layout strategy just changes what areas are passed

**If adding plugin/custom controls:**
- Add `Custom(Box<dyn CustomWidget>)` variant to `ControlKind` enum
- Define `CustomWidget` trait with render + key handling
- Plugin registers custom widgets through a builder method on `FieldSpec`
- Dispatch in the same match -- custom variant delegates to trait object

## Version Compatibility

| Package | Version | Compatible With | Notes |
|---------|---------|-----------------|-------|
| ratatui | 0.30.0 | ratatui-core 0.30, ratatui-widgets 0.30 | Modular sub-crate architecture introduced. Widget trait unchanged from 0.29. |
| ratatui Widget trait | 0.30.0 | Consumable builder pattern | `fn render(self, area, buf)` -- takes ownership. StatefulWidget takes `&mut State`. |
| ratatui WidgetRef | 0.30.0 (unstable) | Behind `unstable-widget-ref` feature | Reference-based rendering for `Box<dyn WidgetRef>`. Not stable yet. |
| crossterm | 0.29.0 | ratatui 0.30 (requires this version) | `event-stream` feature requires `futures` crate. |

## Sources

- Ratatui v0.30.0 ARCHITECTURE.md (raw.githubusercontent.com/ratatui-org/ratatui/main/ARCHITECTURE.md) -- Workspace structure, stability tiers, re-export patterns [HIGH confidence]
- Ratatui official docs: ratatui.rs/concepts/widgets/ -- Widget, StatefulWidget, WidgetRef trait definitions [HIGH confidence]
- Ratatui official docs: ratatui.rs/recipes/widgets/custom/ -- Custom widget implementation patterns [HIGH confidence]
- Rust API Guidelines (github.com/ratatui-org/ratatui/blob/main/API.md) -- C-SEALED, C-STRUCT-PRIVATE, C-NEWTYPE-HIDE, C-REEXPORT-ROOT [HIGH confidence]
- Predrag Radovic sealed traits guide (predr.ag/blog/sealed-traits-in-rust/) -- Comprehensive sealing strategies including partial sealing [MEDIUM confidence]
- Rust API Guidelines official (rust-lang.github.io/api-guidelines/) -- Sealed traits, prelude conventions [HIGH confidence]
- tui01 codebase analysis (.planning/codebase/STRUCTURE.md, .planning/codebase/STACK.md) -- Current architecture and pain points [HIGH confidence]

---
*Stack research for: Rust TUI library architecture refactoring*
*Researched: 2026-03-28*
