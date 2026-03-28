# Technology Stack

**Analysis Date:** 2026-03-28

## Languages

**Primary:**
- Rust 2021 Edition - All source code in `src/`, `examples/`, and `templates/`

**Secondary:**
- Not applicable - Pure Rust project

## Runtime

**Environment:**
- rustc 1.93.0 (254b59607 2026-01-19)
- cargo 1.93.0 (083ac5135 2025-12-15)
- Rust Edition 2021 (specified in `Cargo.toml`)

**Package Manager:**
- Cargo (Rust native)
- Lockfile: `Cargo.lock` present (version 4 format)

## Frameworks

**Core:**
- ratatui 0.30.0 - Terminal UI rendering framework (The Elm Architecture pattern)
- crossterm 0.29.0 - Cross-platform terminal manipulation (raw mode, alternate screen, events)
  - Feature `event-stream` enabled for async event handling via `futures::Stream`

**Async Runtime:**
- tokio 1.50.0 - Async runtime with `full` feature set
  - Powers the event loop, async shell command execution, and channel-based message passing

**Testing:**
- rstest 0.24.0 - Parameterized test framework (used in `src/event.rs` for `#[case]` attribute tests)

**Build/Dev:**
- Cargo (standard Rust build system)
- No custom build scripts detected

## Key Dependencies

**Critical:**
- ratatui 0.30.0 - Core TUI rendering; provides `Frame`, `Terminal`, layout primitives, and widget abstractions
- crossterm 0.29.0 - Terminal backend for ratatui; handles keyboard/mouse events, raw mode, alternate screen
- tokio 1.50.0 - Async runtime powering the entire event loop and shell command execution (`tokio::process::Command`)
- color-eyre 0.6.5 - Error reporting with colorized backtraces; used as the top-level `Result` type in `main.rs`

**Infrastructure:**
- tracing 0.1.44 - Structured logging facade (declared but primarily used via `FrameworkLogger` which writes to file)
- tracing-subscriber 0.3.23 - Tracing subscriber with `env-filter` feature
- futures 0.3.32 - Async utilities; specifically `StreamExt` and `FutureExt` for crossterm `EventStream` consumption
- unicode-width 0.2.2 - Unicode character width calculation for correct terminal text alignment

**Transitive (notable):**
- ratatui-core, ratatui-crossterm, ratatui-widgets, ratatui-macros - ratatui 0.30 modular sub-crates
- signal-hook, signal-hook-mio - Unix signal handling via crossterm/tokio

## Configuration

**Environment:**
- No `.env` files present
- No environment variables required at build time
- Runtime environment variables can be injected via `RuntimeHost::insert_env()` for shell action execution
- Environment variable whitelist enforced via `ExecutionPolicy::allow_env_key()`

**Build:**
- `Cargo.toml` at project root - single crate configuration
- No `build.rs` build script
- No `rust-toolchain.toml` or `.rustfmt.toml` configuration files
- `.gitignore` excludes `/target` and `.tui01/`

**Logging:**
- Framework log writes to `.tui01/logs/framework.log` (local to CWD)
- Log path configurable via `RuntimeHost::set_framework_log_path()`
- Can be disabled via `RuntimeHost::set_framework_log_enabled(false)`

## Platform Requirements

**Development:**
- Rust toolchain (edition 2021 compatible, tested with 1.93.0)
- Unix-like OS for shell command execution (`sh -lc` used in `src/executor.rs`)
- Terminal with minimum 80x24 size and aspect ratio between 0.5 and 4.0 (enforced in `src/tui.rs`)

**Production:**
- Terminal application target (no web, no GUI)
- Crossterm supports Unix and Windows terminals
- Shell commands executed via `sh -lc` (Unix-specific; Windows compatibility not explicitly handled)
- No network dependencies, no database, no external service calls

## Project Identity

- **Crate name:** `tui01`
- **Version:** 0.1.0
- **License:** MIT OR Apache-2.0 (dual license)
- **Type:** Library + binary (both `lib.rs` and `main.rs` present)

---

*Stack analysis: 2026-03-28*
