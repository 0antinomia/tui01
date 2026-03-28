# Coding Conventions

**Analysis Date:** 2026-03-28

## Language and Edition

**Primary Language:** Rust 2021 Edition

**Key Language Patterns:**
- Extensive use of builder pattern (all `with_*` / `set_*` methods return `Self`)
- TEA (The Elm Architecture) pattern for UI components
- Async runtime via tokio throughout
- All documentation comments and user-facing strings in Chinese (Simplified)
- Error types implement `std::fmt::Display` and `std::error::Error`

## Naming Patterns

**Files:**
- `snake_case.rs` -- every file follows snake_case
- Module directories use `snake_case/` with a `mod.rs` inside (e.g., `src/components/mod.rs`)

**Types (structs, enums):**
- `PascalCase` for all type names
- Public structs expose fields via methods, not direct field access (exceptions: `ShowcaseApp.running`, `ShowcaseScreen.title`, `ShowcaseScreen.content`, control structs like `TextInputControl.value`)
- Example structs: `ContentBlueprint`, `OperationRequest`, `ActionOutcome`, `AppSpec`
- Example enums: `Action`, `Event`, `Key`, `ContentControl`, `OperationSource`, `ShellPolicy`

**Functions and methods:**
- `snake_case` for all functions and methods
- Factory functions are `snake_case` free functions in module scope (e.g., `page()`, `screen()`, `section()` in `src/builder.rs`; `text()`, `toggle()`, `select()` in `src/field.rs`)
- Constructor convention: `new()` for primary constructors, `from_page()`, `from_file()`, `from_blueprint()` for alternate constructors
- Builder methods prefixed with `with_` for consuming-self chaining, `set_` for mutating-self

**Variables:**
- `snake_case` everywhere
- Short names for local indices: `idx`, `ch`, `w`, `h`
- Descriptive names for domain concepts: `blueprint`, `operation_id`, `selected_block`

**Constants:**
- `SCREAMING_SNAKE_CASE` for associated constants (e.g., `MIN_WIDTH`, `MAX_LINES`, `PAGE_SLOT_HEIGHT`)
- Regular constants at module level use `SCREAMING_SNAKE_CASE` (e.g., `MIN_WIDTH`, `MIN_HEIGHT` in `src/tui.rs`)

## Code Style

**Formatting:**
- No explicit rustfmt.toml or .rustfmt.toml -- uses Rust standard formatting defaults
- No explicit clippy.toml
- Indentation: 4 spaces (Rust default)

**Linting:**
- `#[allow(dead_code)]` used sparingly (e.g., `MenuState::len()`, `MenuState::is_empty()` in `src/components/menu.rs`)
- `#[allow(clippy::too_many_arguments)]` used for render methods with many parameters in `src/components/content_panel.rs`

## Import Organization

**Order:**
1. `crate` internal imports (`use crate::...`)
2. External crate imports (`use ratatui::...`, `use std::...`, etc.)

**Pattern (from `src/runtime.rs`):**
```rust
use crate::components::{
    ActionButtonControl, DataDisplayControl, LogOutputControl, NumberInputControl, SelectControl,
    TextInputControl, ToggleControl,
};
use std::time::Instant;
```

**Pattern (from `src/showcase.rs`):**
```rust
use crate::action::Action;
use crate::builder::AppValidationError;
use crate::components::{ ... };
use crate::event::{Event, Key};
use crate::executor::{ ... };
use ratatui::{ layout::Rect, style::{Color, Style}, widgets::Paragraph };
```

**No wildcards** -- all imports are explicit, named imports. No `use crate::*` patterns.

## Error Handling

**Strategy:** Typed error enums with `Result<T, E>` return types; `color_eyre` for top-level error reporting in `main()`.

**Validation errors:**
- Use dedicated error enums implementing `std::fmt::Display` and `std::error::Error`
- Example: `AppValidationError` in `src/builder.rs` with variants:
  - `DuplicateFieldId(String)`
  - `MissingResultTarget { source_field, target_id }`
  - `InvalidResultTarget { source_field, target_id }`
  - `UnknownRegisteredAction { source_field, action }`

**Top-level errors:**
- `color_eyre::Result<()>` used in `main()` and terminal lifecycle (`src/tui.rs`)
- `color_eyre::install()?` called at program start

**Async error suppression:**
- `let _ = sender.send(...)` -- errors from channel sends are explicitly ignored in event handler
- No `.unwrap()` on fallible operations in production code paths (only in tests and fallback initializers)

**Result routing:**
- `try_into_showcase_app()` / `try_into_showcase_app_with_host()` return `Result<ShowcaseApp, AppValidationError>` for fallible construction
- `validate()` / `validate_with_registry()` return `Result<(), AppValidationError>`

## Logging

**Framework:** `tracing` crate for structured logging, `FrameworkLogger` for framework-internal file logging

**Framework Logger** (`src/framework_log.rs`):
- Writes to `.tui01/logs/framework.log` under the working directory
- Format: `{timestamp}.{millis} {LEVEL} {target} {message}`
- Uses `Arc<Mutex<File>>` for thread-safe writes
- Can be disabled via `FrameworkLogger::disabled()`

**Host Logger** (`src/host.rs`):
- Optional callback-based logger: `Fn(HostLogRecord) + Send + Sync`
- Log levels: `Debug`, `Info`, `Warn`, `Error`
- `HostLogRecord` carries `level`, `target`, `message`

**Operation logging** (`src/executor.rs`):
- Every operation logs start and finish to both the framework logger and the optional host logger
- Log target: `"tui01.operation"`
- Start message: `"started op={id} screen={si} block={bi} source={desc}"`
- Finish message: `"finished op={id} screen={si} block={bi} source={desc} success={bool}"`

**When to log:**
- Log at `Info` level for operation lifecycle events (start, finish)
- Log at `Warn` level for policy-blocked operations
- Log at `Error` level for failed operations (in finish records)

## Comments

**When to Comment:**
- Module-level doc comments (`//!`) on every file, describing the module's purpose in Chinese
- Public item doc comments (`///`) on all public structs, enums, functions, and methods
- Inline comments for non-obvious logic (e.g., boundary calculations, pixel offsets)

**Doc Comment Language:**
- All doc comments are written in Chinese
- Example from `src/event.rs`:
  ```rust
  /// 键盘输入事件。
  Key(Key),
  ```

**No code-level comments for obvious code** -- the codebase avoids noise comments

## Function Design

**Size:** Functions range from 1-liners to ~100 lines. The largest functions are render methods in component files.

**Parameters:**
- Use `impl Into<String>` for string parameters to accept `&str`, `String`, or any string-like type
- Use `impl Into<PathBuf>` for path parameters
- Use `impl IntoIterator<Item = impl Into<String>>` for collection parameters (e.g., select options)

**Return Values:**
- Builders return `Self` for chaining
- Factory functions return constructed types directly
- Fallible operations return `Result<T, E>`
- Optional results use `Option<T>` (e.g., `block_control()` returns `Option<&ContentControl>`)
- Boolean methods prefixed with `is_`, `has_`, `can_` (e.g., `is_control_active()`, `has_action()`, `can_focus()`)

**Async pattern:**
- `async fn` for async functions
- `tokio::spawn()` for fire-and-forget concurrent tasks
- `mpsc::unbounded_channel()` for inter-task communication

## Module Design

**Exports:**
- Each module has a clear public API exported via `pub` items
- `src/prelude.rs` re-exports the recommended public API surface for host applications
- `src/components/mod.rs` re-exports all component types with `pub use`
- No barrel-file anti-pattern -- re-exports are selective and purposeful

**Visibility:**
- Internal helper functions are module-private (no `pub`)
- Fields on structs are generally private unless they need cross-module access
- `pub(crate)` is not used -- modules use `pub` for public API or no visibility modifier for private

**Trait Design:**
- `Component` trait in `src/components/mod.rs` defines the UI component contract:
  ```rust
  pub trait Component {
      fn init(&mut self) -> color_eyre::Result<()> { Ok(()) }
      fn can_focus(&self) -> bool { false }
      fn is_focused(&self) -> bool { false }
      fn focus(&mut self) {}
      fn blur(&mut self) {}
      fn handle_events(&mut self, _event: Option<Event>) -> Action { Action::Noop }
      fn update(&mut self, _action: Action) -> Option<Action> { None }
      fn render(&mut self, f: &mut Frame, rect: Rect);
  }
  ```
- Default implementations provided for optional methods

## Builder Pattern Convention

The codebase uses a consistent builder pattern throughout:

**Consuming builder** (returns `Self`, consumes `self`):
```rust
pub fn title_text(mut self, title_text: impl Into<String>) -> Self {
    self.title_text = title_text.into();
    self
}
```

**Mutating builder** (returns `()`, takes `&mut self`):
```rust
pub fn insert_context(&mut self, key: impl Into<String>, value: impl Into<String>) {
    self.context.insert(key.into(), value.into());
}
```

**When to use which:**
- Use consuming builder (`mut self -> Self`) for initial configuration chains
- Use mutating builder (`&mut self`) for runtime mutations

Files that follow this pattern:
- `src/builder.rs`: `AppSpec` (consuming)
- `src/host.rs`: `RuntimeHost`, `ShellRuntime`, `ExecutionPolicy` (mix of both)
- `src/schema.rs`: `PageSpec`, `SectionSpec`, `FieldSpec` (consuming)
- `src/runtime.rs`: `ContentBlueprint`, `ContentSection`, `ContentBlock` (consuming)

## Conversion Pattern

**`From` / `Into` implementations:**
- Use `From<RuntimePage> for ContentBlueprint` for type conversions between layers
- `materialize()` methods convert schema types to runtime types
- Example in `src/schema.rs`:
  ```rust
  pub fn materialize(&self) -> RuntimePage { ... }
  ```

**Direction:** Schema -> Runtime -> Content (blueprint). Each layer has its own types, with explicit conversion methods.

## Struct Visibility Conventions

**Public struct fields (accessible directly):**
- `ShowcaseApp.running` -- checked in main loop
- `ShowcaseApp.active_screen()`, `.selected_index()`, `.has_size_error()` etc. -- accessed via methods
- Control structs (`TextInputControl.value`, `SelectControl.selected`, `ToggleControl.on`) -- public fields for direct access in tests and rendering

**Private struct fields (accessed via methods):**
- Most struct fields are private with getter/setter methods
- Example: `ContentPanel.blueprint` accessed via `blueprint()` method

---

*Convention analysis: 2026-03-28*
