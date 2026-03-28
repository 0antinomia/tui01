# Codebase Structure

**Analysis Date:** 2026-03-28

## Directory Layout

```
tui01/
├── .git/                        # Git repository data
├── .gitignore                   # Ignores /target and .tui01/
├── .planning/                   # GSD planning documents
│   └── codebase/                # Codebase analysis documents
├── .tui01/                      # Runtime framework logs (gitignored)
│   └── logs/
├── Cargo.toml                   # Package manifest (Rust 2021 edition)
├── Cargo.lock                   # Dependency lockfile
├── CHANGELOG.md                 # Release history
├── LICENSE-APACHE               # Apache 2.0 license
├── LICENSE-MIT                  # MIT license
├── README.md                    # Project documentation
├── docs/                        # Design and process documents
│   ├── GETTING_STARTED.md
│   ├── RELEASE_SCOPE.md
│   └── VERSIONING.md
├── examples/                    # Usage examples for host applications
│   ├── host_template.rs         # Full host integration example
│   └── host_project_template/   # Empty directory for project template
├── src/                         # Library and binary source
│   ├── lib.rs                   # Library root (pub mod declarations)
│   ├── main.rs                  # Binary entry point (demo app)
│   ├── app.rs                   # Default App wrapper for main.rs
│   ├── action.rs                # Action enum (TEA pattern)
│   ├── builder.rs               # AppSpec, page(), screen(), section() builders
│   ├── event.rs                 # EventHandler, Event, Key types
│   ├── executor.rs              # OperationExecutor, ActionRegistry, ActionOutcome
│   ├── field.rs                 # Field factory functions (text, select, toggle, etc.)
│   ├── framework_log.rs         # FrameworkLogger (file-based internal logging)
│   ├── host.rs                  # RuntimeHost, ShellPolicy, ExecutionPolicy
│   ├── prelude.rs               # Recommended public imports for host apps
│   ├── runtime.rs               # ContentBlueprint, ContentControl, OperationStatus
│   ├── schema.rs                # PageSpec, SectionSpec, FieldSpec (declarative specs)
│   ├── showcase.rs              # ShowcaseApp (core application shell)
│   ├── tui.rs                   # Tui struct (terminal lifecycle management)
│   └── components/              # UI components implementing Component trait
│       ├── mod.rs               # Component trait definition, pub re-exports
│       ├── content_panel.rs     # ContentPanel (right-bottom content area)
│       ├── controls.rs          # Individual control types and rendering
│       ├── menu.rs              # MenuComponent (left-bottom menu)
│       ├── quadrant.rs          # QuadrantLayout (four-region layout)
│       ├── status_panel.rs      # StatusPanel (top-right)
│       └── title_panel.rs       # TitlePanel (top-left)
├── target/                      # Build artifacts (gitignored)
├── templates/                   # Template files for new host projects
│   └── host_project/
│       ├── actions.rs           # Action handler template
│       ├── app.rs               # App definition template
│       ├── host.rs              # Host configuration template
│       ├── main.rs              # Main entry template
│       └── README.md            # Template README
└── node_modules/                # Unrelated (gitignored, likely from another tool)
```

## Directory Purposes

**`src/`:**
- Purpose: All Rust source code for both the library and the default binary
- Contains: 16 `.rs` files at top level, 7 `.rs` files in `components/`
- Key files: `lib.rs` (public API surface), `showcase.rs` (core application), `builder.rs` (host-facing builder API)

**`src/components/`:**
- Purpose: Self-contained UI components implementing the `Component` trait
- Contains: Layout components (`quadrant.rs`), panel components (`title_panel.rs`, `status_panel.rs`, `content_panel.rs`, `menu.rs`), and control rendering (`controls.rs`)
- Key files: `content_panel.rs` (1526 lines, largest file -- handles form layout, pagination, operation lifecycle), `controls.rs` (1008 lines -- all control types and their rendering)

**`examples/`:**
- Purpose: Working examples showing how to integrate tui01 as a library
- Contains: `host_template.rs` demonstrating `RuntimeHost` with custom action handlers
- Key files: `examples/host_template.rs` (reference integration pattern)

**`templates/host_project/`:**
- Purpose: Starter templates for creating new host applications
- Contains: Skeleton files for actions, app definition, host config, and main entry
- Key files: `templates/host_project/host.rs`, `templates/host_project/main.rs`

**`docs/`:**
- Purpose: Project documentation (not API docs)
- Contains: Getting started guide, release scope, versioning policy

**`.tui01/`:**
- Purpose: Runtime log output directory created by `FrameworkLogger`
- Contains: `logs/framework.log`
- Generated: Yes (created at runtime)
- Committed: No (gitignored)

## Key File Locations

### Entry Points
- `src/main.rs`: Demo binary entry point -- builds default `App`, runs event loop
- `src/lib.rs`: Library root -- declares all public modules

### Configuration
- `Cargo.toml`: Package definition (name "tui01", edition 2021, dual MIT/Apache license)
- `src/prelude.rs`: Recommended import set for host applications

### Core Logic (TEA cycle)
- `src/showcase.rs`: `ShowcaseApp` -- central orchestrator for events, actions, rendering, operations (813 lines)
- `src/action.rs`: `Action` enum -- Quit, Resize, MenuSelect, Noop
- `src/event.rs`: `EventHandler`, `Event`, `Key` -- async terminal event processing

### Declarative API (host-facing)
- `src/builder.rs`: `AppSpec`, `page()`, `screen()`, `section()` -- builder pattern with validation (465 lines)
- `src/schema.rs`: `PageSpec`, `SectionSpec`, `FieldSpec` -- declarative data structures that materialize into runtime types (463 lines)
- `src/field.rs`: Factory functions (`field::text`, `field::select`, `field::toggle`, etc.) -- simplified field constructors (168 lines)

### Runtime State
- `src/runtime.rs`: `ContentBlueprint`, `ContentBlock`, `ContentControl`, `OperationSpec`, `ContentRuntimeState` (580 lines)

### Host Integration
- `src/host.rs`: `RuntimeHost`, `ShellPolicy`, `ExecutionPolicy`, `HostEvent`, `HostLogRecord` (395 lines)
- `src/executor.rs`: `OperationExecutor`, `ActionRegistry`, shell command execution, template rendering (870 lines)

### Rendering
- `src/components/content_panel.rs`: Form layout, pagination, block selection, operation result handling (1526 lines)
- `src/components/controls.rs`: All control type rendering and key handling (1008 lines)
- `src/components/menu.rs`: Menu navigation, pagination, rendering (453 lines)
- `src/components/quadrant.rs`: Four-region layout with configurable split ratios (189 lines)

### Infrastructure
- `src/tui.rs`: Terminal setup/teardown, panic hook, size validation (97 lines)
- `src/framework_log.rs`: File-based framework logging (131 lines)
- `src/app.rs`: Thin wrapper around `ShowcaseApp` for the demo binary (93 lines)

### Testing
Tests are co-located within each source file in `#[cfg(test)] mod tests` blocks. See TESTING.md for details.

## Naming Conventions

### Files
- **Rust modules:** `snake_case.rs` matching the module name declared in `lib.rs`
- **Component files:** Named after the component they define (e.g., `menu.rs` for `MenuComponent`, `quadrant.rs` for `QuadrantLayout`)
- **Examples:** `snake_case.rs` in `examples/` directory

### Directories
- **Single level:** `src/components/` is the only subdirectory under `src/`
- **Templates:** `templates/host_project/` follows the naming pattern of the project type

## Where to Add New Code

### New Control Type
1. Define the control struct in `src/components/controls.rs` alongside existing controls
2. Add a variant to `ContentControl` enum in `src/runtime.rs`
3. Add a variant to `ControlKind` enum in `src/components/controls.rs`
4. Add a variant to `SelectedControlKind` in `src/components/content_panel.rs`
5. Add a factory function in `src/field.rs`
6. Add a constructor on `ContentBlock` in `src/runtime.rs`
7. Add a constructor on `FieldSpec` in `src/schema.rs`
8. Add a constructor on `ControlSpec` in `src/schema.rs` and map it in `FieldSpec::materialize()`
9. Handle render routing in `render_control()` and `handle_control_key()` in `src/components/content_panel.rs`
10. Write co-located tests in all affected files

### New Screen/Page
- Use the builder API: add `.screen(screen("Title", page("Title").section(...)))` calls in `src/app.rs` or host application code
- No framework code changes needed

### New Component (UI region)
1. Create `src/components/new_component.rs`
2. Implement the `Component` trait (defined in `src/components/mod.rs`)
3. Add `mod new_component;` and `pub use` to `src/components/mod.rs`
4. Integrate into `ShowcaseApp` in `src/showcase.rs`

### New Action Type
1. Add variant to `Action` enum in `src/action.rs`
2. Handle in `ShowcaseApp::apply_action()` in `src/showcase.rs`

### New Event Type
1. Add variant to `Event` enum in `src/event.rs`
2. Handle in `EventHandler::new()` (the spawned tokio task)
3. Route in `ShowcaseApp::handle_event()` in `src/showcase.rs`

### Utility Functions
- Shared helpers: Add as public functions in the relevant module
- Rendering helpers: Add to `src/components/controls.rs` (e.g., `truncate_to_chars`, `wrap_text_lines`)

### Tests
- Add `#[cfg(test)] mod tests` block at the bottom of the relevant source file
- Integration-style tests: Add to `src/showcase.rs` tests or `src/builder.rs` tests

## Special Directories

**`.tui01/logs/`:**
- Purpose: Runtime framework log files
- Generated: Yes (created by `FrameworkLogger::new()` at `.tui01/logs/framework.log`)
- Committed: No (in `.gitignore`)

**`target/`:**
- Purpose: Rust build artifacts
- Generated: Yes (by `cargo build`)
- Committed: No (in `.gitignore`)

**`templates/host_project/`:**
- Purpose: Starter template for new host applications consuming tui01 as a library
- Generated: No
- Committed: Yes

**`examples/`:**
- Purpose: Working runnable examples demonstrating library integration patterns
- Generated: No
- Committed: Yes
- Run with: `cargo run --example host_template`

---

*Structure analysis: 2026-03-28*
