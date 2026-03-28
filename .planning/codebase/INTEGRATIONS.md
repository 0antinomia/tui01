# External Integrations

**Analysis Date:** 2026-03-28

## APIs & External Services

**None.** This project has no external API integrations. It is a self-contained TUI framework that renders in a local terminal.

## Data Storage

**Databases:**
- Not applicable - No database usage

**File Storage:**
- Local filesystem only
  - Framework logs: `.tui01/logs/framework.log` (written by `src/framework_log.rs`)
  - Log output widget: Can read from arbitrary file paths via `field::log_file()` and `field::log_file_tail()` in `src/field.rs`
  - `LogOutputControl` in `src/components/controls.rs` supports `file_source` with `refresh_from_file()` for live file tailing

**Caching:**
- None - No caching layer

## Authentication & Identity

**Auth Provider:**
- Not applicable - Single-user terminal application, no authentication

## Terminal Integration

**crossterm:**
- SDK: crossterm 0.29.0 crate (direct dependency in `Cargo.toml`)
- Raw mode: Enabled/disabled via `enable_raw_mode()` / `disable_raw_mode()` in `src/tui.rs`
- Alternate screen: Entered/left via `EnterAlternateScreen` / `LeaveAlternateScreen` in `src/tui.rs`
- Event stream: `crossterm::event::EventStream` with `event-stream` feature for async keyboard/resize events in `src/event.rs`
- Terminal size query: `crossterm::terminal::size()` for minimum size validation in `src/tui.rs`

**ratatui:**
- SDK: ratatui 0.30.0 crate (direct dependency in `Cargo.toml`)
- Backend: `CrosstermBackend<Stdout>` created in `src/tui.rs`
- Frame rendering: `Terminal::draw()` in the main event loop (`src/main.rs`)

## Shell Command Execution

**Shell Executor:**
- Implementation: `tokio::process::Command` in `src/executor.rs` function `run_shell_command()`
- Shell: `sh -lc <command>` (login shell)
- Policy system: `ShellPolicy` enum with three modes:
  - `AllowAll` (default) - Any inline or registered shell command permitted
  - `RegisteredOnly` - Only pre-registered named actions allowed; inline shell blocked
  - `Disabled` - All shell execution blocked
- Execution policy: `ExecutionPolicy` enforces:
  - Working directory whitelist via `allow_working_dir()` in `src/host.rs`
  - Environment variable whitelist via `allow_env_key()` in `src/host.rs`
- Template rendering: `{{field_id}}` syntax with shell escaping; supports `{{host.key}}` for host context and `{{raw:field_id}}` for unescaped values
- Result routing: `result_target` mechanism routes stdout/stderr to `LogOutput` widgets by field ID

## Monitoring & Observability

**Error Tracking:**
- color-eyre 0.6.5 - Colorized error reports with backtraces in `src/main.rs`
- Panic hook: Custom panic handler in `src/tui.rs::init_panic_hook()` restores terminal state before printing panic info

**Logs:**
- Framework logger: `src/framework_log.rs` writes timestamped log records to `.tui01/logs/framework.log`
  - Format: `{unix_timestamp}.{millis} {LEVEL} {target} {message}`
  - Thread-safe via `Arc<Mutex<File>>`
- Host logger hook: `RuntimeHost::on_log()` in `src/host.rs` allows host applications to receive log records as a callback
- Event hook: `RuntimeHost::on_event()` in `src/host.rs` provides `HostEvent::OperationStarted` and `HostEvent::OperationFinished` callbacks
- tracing/tracing-subscriber: Declared as dependencies but not actively used in the main code path; the framework uses its own `FrameworkLogger` instead

## CI/CD & Deployment

**Hosting:**
- Local terminal only - No deployment target

**CI Pipeline:**
- None detected - No `.github/`, `.gitlab-ci.yml`, or similar CI configuration

## Environment Configuration

**Required env vars:**
- None required at build or runtime

**Optional runtime env vars:**
- Host applications can inject environment variables into shell actions via `RuntimeHost::insert_env(key, value)` in `src/host.rs`
- Environment variables are subject to whitelist enforcement via `ExecutionPolicy::allow_env_key()` in `src/host.rs`

**Secrets location:**
- Not applicable - No secrets management

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Host Application Integration

**Integration surface (for consumers of tui01 as a library):**
- `tui01::prelude` - Recommended public import set (`src/prelude.rs`)
  - Re-exports: `page`, `screen`, `section`, `AppSpec`, `AppValidationError`, `Event`, `EventHandler`, `Key`, `field` module, `HostEvent`, `HostLogLevel`, `HostLogRecord`, `RuntimeHost`, `ShellPolicy`
- `tui01::field` - Field factory functions (`src/field.rs`)
  - Functions: `text`, `text_id`, `number`, `number_id`, `select`, `toggle`, `action`, `action_to_log`, `action_registered_to_log`, `refresh`, `refresh_to_log`, `refresh_registered_to_log`, `static_value`, `dynamic_value`, `log`, `log_id`, `log_file`, `log_file_tail`
- `tui01::host::RuntimeHost` - Host configuration builder (`src/host.rs`)
  - Methods: `register_action_handler`, `register_shell_action`, `set_context`, `set_working_dir`, `insert_env`, `set_shell_policy`, `allow_working_dir`, `allow_env_key`, `on_event`, `on_log`, `set_framework_log_path`, `set_framework_log_enabled`
- `tui01::executor` - Action execution types (`src/executor.rs`)
  - Types: `ActionOutcome`, `ActionContext`, `ActionRegistry`
- `tui01::tui` - Terminal lifecycle (`src/tui.rs`)
  - Functions: `init_panic_hook`, `check_minimum_size`, `terminal_size`
  - Struct: `Tui` (wraps `ratatui::Terminal`)
- `tui01::event::EventHandler` - Async event producer (`src/event.rs`)

**Integration flow (from `examples/host_template.rs`):**
1. Build `RuntimeHost` with registered action handlers
2. Build `AppSpec` with `page`/`screen`/`section`/`field` DSL
3. Call `AppSpec::try_into_showcase_app_with_host(host)` to validate and materialize
4. Create `Tui` and `EventHandler`
5. Run event loop: `tui.draw()` then `event_handler.next().await`

**Template project:**
- `templates/host_project/` - Starter project template for host applications
  - `actions.rs` - Action handler registration
  - `host.rs` - Host configuration and policies
  - `app.rs` - Page/field definitions
  - `main.rs` - Entry point and event loop
- `examples/host_template.rs` - Complete working example

---

*Integration audit: 2026-03-28*
