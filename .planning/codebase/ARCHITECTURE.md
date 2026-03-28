# Architecture

**Analysis Date:** 2026-03-28

## Pattern Overview

**Overall:** The Elm Architecture (TEA) with a declarative builder layer

The application follows a unidirectional data flow: Events are dispatched to the application state, which produces actions that update state and trigger re-renders. On top of this, a declarative builder API (`AppSpec` / `PageSpec` / `SectionSpec` / `FieldSpec`) allows host applications to define their UI structure without touching rendering code.

**Key Characteristics:**
- TEA-inspired event/action cycle: `Event -> handle_event -> Action -> apply_action -> render`
- Declarative specification layer that materializes into runtime blueprints
- Four-quadrant ("田"字形) fixed layout: title (top-left), status (top-right), menu (bottom-left), content (bottom-right)
- Async operation execution via tokio channels with shell policy enforcement
- Library-first design: `lib.rs` exposes a public API; `main.rs` is a demo/development entry point
- Host integration pattern: external Rust applications consume the library via `RuntimeHost` and `AppSpec`

## Layers

### Specification Layer (Declarative API)
- Purpose: Declarative UI definition used by host applications to describe screens, sections, and fields
- Location: `src/builder.rs`, `src/schema.rs`, `src/field.rs`
- Contains: `AppSpec`, `PageSpec`, `SectionSpec`, `FieldSpec`, convenience factory functions in `field`
- Depends on: `src/runtime.rs` (materialization targets)
- Used by: Host applications via `src/prelude.rs`, `src/app.rs`

### Runtime Layer (Mutable State)
- Purpose: Runtime representation of content as blueprints, blocks, controls, and operation statuses
- Location: `src/runtime.rs`
- Contains: `ContentBlueprint`, `ContentSection`, `ContentBlock`, `ContentControl`, `OperationSpec`, `ContentRuntimeState`, `RuntimeFieldState`
- Depends on: `src/components/` (control types like `TextInputControl`, `ToggleControl`, etc.)
- Used by: `src/showcase.rs`, `src/components/content_panel.rs`, `src/schema.rs` (materialization)

### Application Layer (TEA Core)
- Purpose: Orchestrates the event/action/render cycle, manages focus, screens, and operation lifecycle
- Location: `src/showcase.rs`, `src/action.rs`
- Contains: `ShowcaseApp`, `Action`, `FocusTarget`, `ShowcaseScreen`, `ShowcaseCopy`
- Depends on: All other layers
- Used by: `src/app.rs` (default demo), host applications

### Component Layer (Rendering)
- Purpose: Self-contained UI regions implementing the `Component` trait with independent render logic
- Location: `src/components/`
- Contains: `Component` trait, `QuadrantLayout`, `TitlePanel`, `StatusPanel`, `MenuComponent`, `ContentPanel`, individual controls
- Depends on: ratatui, crossterm
- Used by: `src/showcase.rs`

### Host Integration Layer
- Purpose: Public API surface for embedding tui01 as a library in external Rust applications
- Location: `src/host.rs`, `src/prelude.rs`, `src/executor.rs`, `src/framework_log.rs`
- Contains: `RuntimeHost`, `ShellPolicy`, `ExecutionPolicy`, `ActionRegistry`, `OperationExecutor`, `FrameworkLogger`
- Depends on: tokio (async execution), std::process (shell commands)
- Used by: External Rust applications, `src/showcase.rs`

### Infrastructure Layer
- Purpose: Terminal lifecycle, event capture, and error handling
- Location: `src/tui.rs`, `src/event.rs`
- Contains: `Tui` (terminal wrapper), `EventHandler` (async event stream), `Event`, `Key`
- Depends on: crossterm, tokio, futures
- Used by: `src/main.rs`, host applications

## Data Flow

### Main Event Loop

1. `EventHandler` spawns a tokio task that merges crossterm events, tick timer (120ms), and Ctrl+C into a single `mpsc::UnboundedReceiver<Event>`
2. Main loop calls `event_handler.next().await` to get the next event
3. Event is dispatched to `ShowcaseApp::handle_event(event)`
4. Depending on focus target (Menu or Content), the event is routed to `handle_menu_key` or `handle_content_key`
5. Key presses produce `Action` variants (Quit, MenuSelect, Noop, Resize) or mutate control state directly
6. After state mutation, `tui.draw(|f| app.render(f))` renders the frame

### Operation Execution Flow

1. User activates a control (Enter on button, confirm text input, toggle)
2. `ContentPanel` creates an `OperationRequest` with operation source, params (field values), and optional `result_target`
3. `ShowcaseApp::submit_operation` enriches the request with host context (cwd, env, execution policy)
4. `OperationExecutor::submit` spawns a tokio task that:
   - Validates execution policy (allowed working dirs, env keys)
   - Checks shell policy (AllowAll / RegisteredOnly / Disabled)
   - Resolves registered actions (shell template or async handler)
   - Runs shell command via `tokio::process::Command` or calls async handler
   - Sends `OperationResult` back through `mpsc::UnboundedSender`
5. On each tick, `ShowcaseApp::poll_operation_results` receives completed results
6. Results update `OperationStatus` on the originating block and append output to any linked log target

### Specification Materialization Flow

1. Host defines UI via builder: `AppSpec::new().title_text(...).screen(screen("Title", page(...).section(...)))`
2. `PageSpec::materialize()` converts declarative specs into `RuntimePage` -> `ContentBlueprint`
3. `AppSpec::into_showcase_app()` (or `with_host`, `with_registry`) constructs a `ShowcaseApp`
4. Validation checks: duplicate field IDs within a screen, missing/invalid result targets, unknown registered actions

**State Management:**
- `ShowcaseApp` holds a `Vec<ShowcaseScreen>` where each screen has a `ContentBlueprint`
- Active screen content is loaded into `ContentPanel` on demand (lazy load with `loaded_screen` tracking)
- On screen switch, current content is persisted back via `persist_active_screen_content()`
- `ContentRuntimeState` tracks field states (control values, operation status, snapshots) separately from the blueprint

## Key Abstractions

**`Component` trait (`src/components/mod.rs`):**
- Purpose: Uniform interface for UI regions with focus, event handling, and rendering
- Examples: `src/components/menu.rs`, `src/components/content_panel.rs`, `src/components/quadrant.rs`
- Pattern: TEA-style with `handle_events(Event) -> Action`, `update(Action) -> Option<Action>`, `render(Frame, Rect)`

**`ContentControl` enum (`src/runtime.rs`):**
- Purpose: Type-safe representation of all form control variants
- Examples: `TextInput(TextInputControl)`, `Toggle(ToggleControl)`, `Select(SelectControl)`, `LogOutput(LogOutputControl)`, etc.
- Pattern: Newtype enum wrapping component-specific state structs from `src/components/controls.rs`

**`RuntimeHost` struct (`src/host.rs`):**
- Purpose: Configuration hub for host applications embedding the framework
- Examples: Shell policy, execution policy, context values, event hooks, logger hooks
- Pattern: Builder pattern with fluent setters, stores `ActionRegistry` for async action handlers

**`AppSpec` struct (`src/builder.rs`):**
- Purpose: Top-level declarative app definition that validates and builds `ShowcaseApp`
- Examples: Chain `title_text()`, `status_controls()`, `screen()`, `shell_action()` calls
- Pattern: Builder with validation via `validate()` and fallible construction via `try_into_showcase_app_with_host()`

## Entry Points

**Binary entry (`src/main.rs`):**
- Location: `src/main.rs`
- Triggers: `cargo run` or direct binary execution
- Responsibilities: Demo application using `App` (wrapper around `ShowcaseApp`) with inline shell actions

**Library entry (`src/lib.rs`):**
- Location: `src/lib.rs`
- Triggers: `use tui01::...` from external crates
- Responsibilities: Re-exports all public modules

**Host application entry (example pattern):**
- Location: `examples/host_template.rs`
- Triggers: `cargo run --example host_template`
- Responsibilities: Shows how to build a `RuntimeHost` with custom action handlers and integrate with the framework

## Error Handling

**Strategy:** `color_eyre` for top-level error reporting, `Result<_, AppValidationError>` for specification validation

**Patterns:**
- `color_eyre::Result<()>` as the main function return type for rich error reports
- `AppValidationError` enum for specification errors (duplicate IDs, missing targets, unknown actions) in `src/builder.rs`
- Panic hook (`tui::init_panic_hook`) restores terminal state before printing panic info
- Operation errors are captured as `OperationResult { success: false, stderr: ... }` and displayed in the UI
- Shell policy violations return structured error messages to the operation result channel

## Cross-Cutting Concerns

**Logging:** Dual system -- `tracing` crate imported but primarily unused at runtime; `FrameworkLogger` (`src/framework_log.rs`) writes structured logs to `.tui01/logs/framework.log` with timestamp, level, target, and message. Host applications can also register a custom `HostLogRecord` hook.

**Validation:** `AppSpec::validate()` checks screen definitions before construction. Validates field ID uniqueness per screen, result target existence and log type, and registered action resolution.

**Authentication:** Not applicable (terminal UI framework, no auth layer)

**Terminal Safety:** Minimum size check (80x24) with aspect ratio bounds (0.5 to 4.0). Panic hook restores raw mode and alternate screen on crash.

**Security:** Execution policy system in `RuntimeHost` controls which working directories and environment variables shell commands can use. Shell policy (`AllowAll`, `RegisteredOnly`, `Disabled`) gates command execution. Command template parameters are shell-escaped by default; `raw:` prefix available for intentional unescaped values.

---

*Architecture analysis: 2026-03-28*
