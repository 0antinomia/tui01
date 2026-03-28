# Feature Research

**Domain:** TUI framework extensibility (theme/style, layout, plugin/custom controls)
**Researched:** 2026-03-28
**Confidence:** HIGH

## Feature Landscape

Research surveyed mature TUI frameworks across ecosystems: **ratatui** (Rust), **tui-realm** (Rust), **Textual** (Python), **Brick** (Haskell), **AppCUI** (Rust), **Cursive** (Rust), plus emerging crates like **ratatui-themes**, **ratatui-garnish**, **tachyonfx**, and **saorsa-tui**. Features are categorized for the tui01 brownfield refactoring: what extension points a mature TUI framework must expose.

---

### Table Stakes (Users Expect These)

Features framework consumers assume exist. Missing these makes the framework feel incomplete or hostile to extension.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Centralized theme struct** | Every TUI framework consumer wants to change colors without editing every widget. Currently tui01 hardcodes `Color::Cyan`, `Color::DarkGray` etc. in controls.rs. A `Theme` struct with semantic color/style slots (border, selected, active, title, text, error, success) is the minimum. | LOW | Ratatui's `Style` + `Color` types are the building blocks. The pattern: define a `Theme` struct, thread `&Theme` through render calls. Brick's `AttrMap`, Textual's CSS themes, and ratatui-themes crate all implement this. No external dependency needed. |
| **Theme swap at runtime** | Users expect to switch between light/dark or named presets without restarting. | LOW | Store `Theme` behind `Arc<Theme>` or `Rc<Theme>`. Swap the reference. Brick's `Theme` type is designed for this. ratatui-themes provides 50+ presets. |
| **Style inheritance / cascade** | A parent component's style should propagate to children unless overridden. Without this, every widget needs explicit styling. | MEDIUM | Brick's hierarchical `AttrName` system (e.g., `baseAttr <> "highlight"`) is the gold standard. For tui01, a simpler approach: Theme provides defaults per semantic role, individual widgets can override. No need for full CSS cascade. |
| **Widget trait for custom controls** | Framework consumers must be able to define their own control types without modifying framework internals. Currently adding a control requires touching 10 files. | HIGH | Ratatui's `Widget` + `StatefulWidget` traits are the foundation. tui01 should define its own `Control` trait (render, handle_key, value, preferred_width) that maps to the ratatui Widget trait internally. The `ControlKind` enum becomes a registry, not an exhaustive list. |
| **Declarative layout with constraints** | Consumers expect to define layouts declaratively (not by computing pixel/cell positions manually). Currently tui01 hardcodes a four-quadrant layout. | MEDIUM | Ratatui's `Layout` + `Constraint` system already provides this. tui01 should accept a `LayoutSpec` instead of hardcoding `QuadrantLayout::calculate_quadrants`. The `Flex` system (added ratatui 0.27+) and `Constraint::Fill` handle flexible spacing. |
| **Control value get/set API** | Host applications need to read and write control values programmatically, not just through user interaction. | LOW | Already partially exists via `RuntimeFieldState`. Needs to be formalized as a public trait method on controls: `fn value(&self) -> String`, `fn set_value(&mut self, val: &str)`. |
| **Event routing to focused component** | Key events should automatically route to the focused/active control. Consumers should not need to write event dispatch code. | MEDIUM | Already implemented in tui01's TEA loop. The extension point: custom controls should be able to register which keys they handle and receive them automatically via the `Control` trait's `handle_key` method. |
| **Validation hook per control** | Host applications need to validate input before operation submission. Currently NumberInput has no validation. | LOW | Add a `validate(&self) -> Result<(), String>` method to the `Control` trait. Default implementation returns `Ok(())`. Controls like NumberInput override it. |

---

### Differentiators (Competitive Advantage)

Features that set the framework apart from raw ratatui usage and other Rust TUI frameworks. Not required for v1, but valuable for adoption.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Named theme presets** | Ship with 5-10 built-in themes (dark, light, monochrome, solarized, dracula, nord). Host apps select by name. | LOW | ratatui-themes crate provides 50+ presets. For tui01, embed a small curated set. The `ratatui::style::palette::tailwind` module (v0.26+) provides color palettes. HIGH confidence. |
| **Control trait with derive macro** | `#[derive(Control)]` auto-generates boilerplate for the `Control` trait (preferred_width default, render dispatch, value serialization). Analogous to tui-realm's `#[derive(MockComponent)]`. | HIGH | Requires proc-macro crate. tui-realm's `tuirealm_derive` proves the pattern works. For tui01, this is a v2 feature -- start with manual trait implementation, add derive later. |
| **Decorator pattern for widgets** | Wrap any control with borders, padding, title, shadow without changing the control's code. ratatui-garnish implements this as a "flat decorator" pattern. | MEDIUM | ratatui-garnish provides this. For tui01's extension points, the `Block` wrapper pattern (already used) is the simpler built-in approach. Decorator pattern is a natural v1.x enhancement. |
| **Subscription-based inter-component events** | Components can subscribe to events from other components without tight coupling. tui-realm's `Sub` ruleset is the reference implementation. | HIGH | tui-realm implements this via declarative subscription rules. For tui01, this is a v2+ feature. The current `Action` enum + operation result channel provides basic inter-component communication. |
| **Effects/animations integration point** | Controls can trigger visual effects (fade, blink, glow) on state changes. tachyonfx provides this for ratatui. | MEDIUM | tachyonfx is a ratatui ecosystem crate. For tui01, the extension point is: controls emit "effect requests" (e.g., on success/failure), the framework applies them if tachyonfx is linked. This is a nice-to-have, not a dependency. |
| **Layout composition via nesting** | Define arbitrarily nested layouts (sidebar + tabs + split panes) via a declarative tree, not just fixed quadrants. | MEDIUM | Ratatui's `Layout::split()` already supports nesting. The tui01 extension point: accept a `LayoutTree` or `LayoutSpec` enum that describes the desired layout structure, then materialize it into nested `Layout::split()` calls. This replaces the hardcoded `QuadrantLayout`. |
| **Control plugin via trait object registration** | Host apps register custom control types by name at app construction, then use them in FieldSpec builders by that name. No framework code changes needed. | HIGH | The pattern: `app.register_control("my_widget", MyWidgetFactory)`. Internally stores as `Box<dyn ControlFactory>`. FieldSpec references `"my_widget"` by name. Materialization looks up the factory. Analogous to how tui-realm registers components by ID. |
| **Configurable keyboard bindings** | Host applications can override default keybindings (e.g., change Enter to Space for activation, add custom shortcuts). | MEDIUM | Store a `KeyBindings` struct in the app. Controls consult it via the `Control` trait. The ratatui component template uses this pattern. |
| **Async operation progress reporting** | Long-running operations report progress (0-100%) back to the framework, displayed as a gauge or spinner variant. | MEDIUM | The current `OperationStatus::Running` tracks state but not progress. Add an optional `progress: Option<u8>` field. Controls can render this as a progress bar variant. |
| **Serde-serializable theme** | Themes load from TOML/JSON files, enabling user-customizable appearance without code changes. | LOW | The `Theme` struct derives `Serialize`/`Deserialize`. ratatui-themes does this. Useful for CLI tools that read config from `~/.config/app/theme.toml`. |

---

### Anti-Features (Commonly Requested, Often Problematic)

Patterns that seem appealing but create complexity, maintenance burden, or architectural problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **CSS-like styling DSL** | Web developers want familiar CSS syntax for terminal styling. Textual (Python) does this successfully. | In Rust, parsing CSS-like syntax adds a heavy dependency (requires a full parser). Runtime string-based style resolution defeats Rust's type safety. Performance overhead on every render frame. Textual can afford this because Python is already dynamically typed. | Use a typed `Theme` struct with semantic fields. If file-based configuration is needed, use TOML/JSON with Serde, not a custom DSL. |
| **Full plugin system with dynamic loading** | Load third-party `.so`/`.dylib` plugins at runtime for new control types. | `libloading` + FFI is fragile, unsafe, platform-specific, and version-sensitive. ABI stability in Rust is not guaranteed. Plugin crashes crash the host. The complexity is enormous for a TUI framework. | Use trait-object-based registration. Host apps statically link custom controls and register them at construction time. This is how tui-realm works and it is sufficient. |
| **Flexbox/Grid layout engine** | Web developers want CSS Flexbox or Grid in the terminal. saorsa-tui and reactive_tui attempt this. | Terminal cells are integer-sized, fixed-width. Sub-pixel rounding, baseline alignment, and all the complexities of CSS layout are meaningless at terminal resolution. Adds 2-3x code complexity for marginal benefit. | Ratatui's `Layout` + `Constraint` + `Flex` system already handles 95% of terminal layout needs. Add `Constraint::Fill` for flexible space. Do not build a CSS engine. |
| **Immediate-mode widget state** | Store widget state in the widget itself (like React component state), mutating during render. | Ratatui's architecture is immediate-mode rendering: widgets are consumed on render. Storing mutable state in widgets conflicts with `&self` render signatures. Leads to `RefCell`/`Mutex` overhead and borrowing nightmares. | Use the `StatefulWidget` pattern: separate immutable widget config from mutable `State` struct passed by reference. tui01 already does this partially via `RuntimeFieldState`. |
| **Global mutable theme** | A global `static` or `lazy_static` theme that any widget can access. | Defeats testability (tests cannot isolate themes). Prevents multiple themed apps in the same process. Encourages implicit coupling between distant code. | Pass `&Theme` as a parameter through the render call chain. Explicit, testable, composable. |
| **Macro-based control definition** | A single macro that generates the entire control (struct, trait impl, render, key handling, serialization) from a DSL. | Macros hide complexity but make debugging extremely difficult. Error messages are inscrutable. New Rust contributors cannot understand the codebase. tui-realm's `MockComponent` derive works but has this problem. | Define the `Control` trait with default method implementations. Provide concrete examples. Let implementors write normal Rust code. Add derive macro later as a convenience, not the primary mechanism. |
| **Widget tree with diffing** | Build a virtual widget tree and diff it each frame, only updating changed cells. Like React's virtual DOM. | Terminal buffers are tiny (typically under 10K cells). Full re-render takes microseconds. Diffing adds complexity with no measurable performance gain. Ratatui already optimizes by only flushing changed cells to the terminal. | Direct rendering via ratatui's buffer. Cache layout computations (tui01 already needs this per CONCERNS.md), but do not cache/diff widget trees. |

---

## Feature Dependencies

```
[Centralized Theme Struct]
    +--required-by--> [Named Theme Presets]
    +--required-by--> [Serde-serializable Theme]
    +--required-by--> [Theme Swap at Runtime]

[Widget Trait for Custom Controls]
    +--required-by--> [Control Plugin via Trait Object Registration]
    +--required-by--> [Control Trait with Derive Macro]
    +--enhances-----> [Validation Hook per Control]
    +--enhances-----> [Configurable Keyboard Bindings]

[Declarative Layout with Constraints]
    +--required-by--> [Layout Composition via Nesting]

[Style Inheritance / Cascade]
    +--requires-----> [Centralized Theme Struct]

[Effects/Animations Integration Point]
    +--requires-----> [Widget Trait for Custom Controls]
    +--enhances-----> [Centralized Theme Struct]

[Async Operation Progress Reporting]
    +--enhances-----> [Widget Trait for Custom Controls]
    +--conflicts----> [Immediate-Mode Widget State] (conflicting state models)

[Subscription-based Inter-Component Events]
    +--requires-----> [Widget Trait for Custom Controls]
    +--requires-----> [Event Routing to Focused Component]
```

### Dependency Notes

- **Theme Struct requires nothing**: It is the foundation. Pure data struct with Style fields. Can be built first.
- **Widget Trait requires Theme**: Custom controls need to know what styles to apply. The `render` method signature should accept `&Theme`.
- **Layout System is orthogonal**: Layout is independent of styling and control type. Can be developed in parallel with theme and widget trait work.
- **Plugin Registration requires Widget Trait**: You cannot register custom controls until there is a trait to implement.
- **Derive Macro requires Widget Trait**: The macro generates trait implementations. Must come after the trait API is stable.
- **Effects requires Widget Trait**: Controls need a way to emit effect requests. The trait provides the hook.
- **Subscriptions require both Widget Trait and Event Routing**: Inter-component communication needs a component model and an event dispatch mechanism.
- **Style Inheritance requires Theme Struct**: Inheritance/cascade is layered on top of the base theme. Start simple (override-by-widget), add hierarchy later.

---

## MVP Definition

### Phase 1: Refactoring Extension Points (v0.2.0 -- current milestone)

These are the extension points to create during the architectural refactoring. They do not implement features; they create the seams.

- [ ] **Control trait abstraction** -- Define a `Control` trait with methods: `render`, `handle_key`, `value`, `set_value`, `preferred_width`, `validate`. Existing `ControlKind` variants become implementors. This is the single most impactful change: it reduces "new control" from 10-file touch to 1-file.
- [ ] **Theme struct with semantic slots** -- Define a `Theme` struct with fields for each semantic style role (border, selected, active, title, text, error, success, feedback states). Pass `&Theme` through render chains. Replace hardcoded colors.
- [ ] **Layout trait replacing QuadrantLayout** -- Define a `LayoutStrategy` trait with a `layout(area: Rect) -> LayoutAreas` method. The current four-quadrant becomes one implementation. Future layouts (sidebar, tabs, single-panel) become alternative implementations.
- [ ] **Control registration by name** -- Add a `ControlRegistry` type that maps `&str` names to `Box<dyn ControlFactory>`. `FieldSpec` builders reference control types by string. Materialization looks up the registry. Enables host apps to register custom controls without touching framework code.

### Phase 2: Feature Implementation (v0.3.0)

Features to add once extension points are validated.

- [ ] **Built-in theme presets** -- Ship 3-5 presets (dark default, light, monochrome, high-contrast). Selectable via `RuntimeHost` builder.
- [ ] **Theme loading from TOML** -- Derive `Serialize`/`Deserialize` on `Theme`. Add `RuntimeHost::theme_from_file(path)` builder method.
- [ ] **Layout presets** -- Implement 2-3 layout strategies: quadrant (current), sidebar+content, single-panel.
- [ ] **Validation hooks** -- Add `validate()` to `Control` trait with default `Ok(())`. Implement for NumberInput (range checks), TextInput (pattern match).

### Future Consideration (v2+)

Features to defer until the framework has real users providing feedback.

- [ ] **Derive macro for Control trait** -- Requires proc-macro crate. Wait until the trait API is stable and has 5+ implementors.
- [ ] **Subscription-based inter-component events** -- Complex. Wait until real use cases emerge.
- [ ] **Effects/animations via tachyonfx** -- Integration point. Not a dependency.
- [ ] **Decorator pattern via ratatui-garnish** -- Useful but not critical. Host apps can already use `Block` wrappers.
- [ ] **Async operation progress reporting** -- Requires executor protocol changes. Defer.

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Control trait abstraction | HIGH | MEDIUM | P1 |
| Theme struct with semantic slots | HIGH | LOW | P1 |
| Layout trait abstraction | HIGH | MEDIUM | P1 |
| Control registration by name | HIGH | MEDIUM | P1 |
| Validation hook per control | MEDIUM | LOW | P2 |
| Control value get/set API | MEDIUM | LOW | P2 |
| Built-in theme presets | MEDIUM | LOW | P2 |
| Theme swap at runtime | LOW | LOW | P2 |
| Configurable keyboard bindings | MEDIUM | MEDIUM | P2 |
| Layout presets (non-quadrant) | MEDIUM | MEDIUM | P2 |
| Theme loading from TOML | LOW | LOW | P2 |
| Serde-serializable theme | LOW | LOW | P3 |
| Control plugin via trait object | HIGH | HIGH | P3 |
| Derive macro for Control trait | MEDIUM | HIGH | P3 |
| Layout composition via nesting | MEDIUM | MEDIUM | P3 |
| Decorator pattern for widgets | LOW | MEDIUM | P3 |
| Effects/animations integration | LOW | MEDIUM | P3 |
| Subscription-based events | MEDIUM | HIGH | P3 |
| Async progress reporting | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must create during refactoring (extension points)
- P2: Should implement in first feature release
- P3: Future consideration

---

## Competitor Feature Analysis

| Feature | tui-realm (Rust/ratatui) | Textual (Python) | Brick (Haskell) | tui01 (Our Approach) |
|---------|--------------------------|------------------|-----------------|---------------------|
| **Theme system** | No built-in theme; per-component Props | Full CSS theming with selectors, inheritance, theme swap | `AttrMap` with hierarchical `AttrName`, `Theme` type for presets | Typed `Theme` struct with semantic slots, Serde for file loading |
| **Custom controls** | `MockComponent` trait + derive macro, Props bag | Widget class inheritance with CSS styling | Custom `Widget n` type with pure functions | `Control` trait with typed methods, enum-based dispatch for built-in, trait-object for plugins |
| **Layout** | Standard ratatui Layout (no abstraction) | CSS-like layout (flex, grid, dock) | Composable widget functions (no separate layout layer) | `LayoutStrategy` trait wrapping ratatui Layout, declarative spec tree |
| **Plugin system** | Static registration via `Application::add()` | Python import + subclassing | Haskell module system (no runtime plugins) | `ControlRegistry` with trait-object factories, static linking |
| **State management** | Props bag (untyped) per component | Reactive attributes with watchers | Pure functional state via `EventM` monad | `RuntimeFieldState` per control, typed access |
| **Event routing** | Subscription ruleset (`Sub`) | Message passing, bubbling | `Brick.Main` event handlers | TEA-style: Event -> Action -> State update, focused-component dispatch |
| **DX for new controls** | Derive macro + Props (medium friction) | Python class + CSS (low friction) | Pure function + AttrName (low friction) | Implement `Control` trait + register (low friction, no macro needed) |

### Key Differentiator for tui01

tui01's unique position: it is a **declarative form-oriented framework** built on ratatui, not a general-purpose TUI toolkit. The builder API (`AppSpec` / `FieldSpec`) is its strength. The extension points should preserve and enhance this declarative approach:

- **Theme**: Declarative by nature (specify once, apply everywhere). Fits tui01's builder pattern naturally.
- **Layout**: Spec-driven, not imperative. A `LayoutSpec` enum integrates with `AppSpec` without disrupting the builder chain.
- **Plugin controls**: Registered at app construction time, referenced by name in `FieldSpec`. No dynamic loading needed. The host app controls the full pipeline.

---

## Sources

- [Ratatui Official - Widgets Concept](https://ratatui.rs/concepts/widgets/) -- Widget/StatefulWidget/WidgetRef traits (HIGH confidence, official docs)
- [Ratatui Official - Layout Concept](https://ratatui.rs/concepts/layout/) -- Constraint, Flex, Layout system (HIGH confidence, official docs)
- [Ratatui Official - Component Architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/) -- Component trait pattern (HIGH confidence, official docs)
- [Ratatui v0.26 Highlights - Tailwind Palette](https://ratatui.rs/highlights/v026/) -- Built-in color palettes (HIGH confidence, release notes)
- [Ratatui Style Module - Docs.rs](https://docs.rs/ratatui/latest/ratatui/style/index.html) -- Style struct, Stylize trait, Color, Modifier (HIGH confidence, API docs)
- [Ratatui Constraint - Docs.rs](https://docs.rs/ratatui/latest/ratatui/layout/enum.Constraint.html) -- Constraint variants (HIGH confidence, API docs)
- [ratatui-themes - GitHub](https://github.com/ricardodantas/ratatui-themes) -- 50+ theme presets for ratatui (MEDIUM confidence, verified on crates.io)
- [ratatui-garnish - Docs.rs](https://docs.rs/ratatui-garnish) -- Flat decorator pattern for widget styling (MEDIUM confidence, verified on crates.io)
- [tui-realm - GitHub](https://github.com/veeso/tui-realm) -- Component framework for ratatui with MockComponent, Props, Subscriptions (MEDIUM confidence, verified on docs.rs)
- [tachyonfx - GitHub](https://github.com/junkdog/tachyonfx) -- Effects and animation library for ratatui (MEDIUM confidence, verified on crates.io)
- [Brick Themes - Hackage](https://hackage.haskell.org/package/brick/docs/Brick-Themes.html) -- Theme type with default attributes and customizations (MEDIUM confidence, official Hackage docs)
- [Brick AttrMap - Hackage](https://hackage.haskell.org/package/brick/docs/Brick-AttrMap.html) -- Hierarchical attribute name to style mapping (MEDIUM confidence, official Hackage docs)
- [Textual CSS Styling Guide](https://medium.datadriveninvestor.com/using-css-to-style-a-python-tui-with-textual-a-comprehensive-guide-36c392edf40b) -- CSS-like theme system (LOW confidence, blog post)
- [AppCUI-rs - Hacker News](https://news.ycombinator.com/item?id=45515502) -- Extensible Rust TUI framework (LOW confidence, community discussion)
- [Garnish Your Widgets - Blog](https://franklaranja.github.io/articles/garnish-your-widgets/) -- Decorator pattern for ratatui widgets (MEDIUM confidence, crate author's blog)
- [Ratatui Best Practices - GitHub Discussions](https://github.com/ratatui/ratatui/discussions/220) -- Community guidance on organizing ratatui apps (MEDIUM confidence, official repo)
- [Arroyo Blog - Plugin Systems in Rust](https://www.arroyo.dev/blog/rust-plugin-systems/) -- Host/plugin architecture patterns (MEDIUM confidence, technical blog)

---
*Feature research for: TUI framework extensibility (theme, layout, plugin)*
*Researched: 2026-03-28*
