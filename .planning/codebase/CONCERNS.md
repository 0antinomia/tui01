# Codebase Concerns

**Analysis Date:** 2026-03-28

## Tech Debt

**Duplicated enum hierarchy across three layers:**
- Issue: `ControlSpec` (in `src/schema.rs`), `RuntimeControl` (in `src/runtime.rs`), and `ContentControl` (in `src/runtime.rs`) are three separate enums with nearly identical variants (TextInput, NumberInput, Select, Toggle, etc.). Similarly, `OperationBinding` / `OperationSpec` / `OperationSource` are duplicated across `src/schema.rs`, `src/runtime.rs`, and `src/executor.rs`. Adding a new control type requires changes in all three locations plus the `ControlKind` enum in `src/components/controls.rs`.
- Files: `src/schema.rs` (lines 362-396), `src/runtime.rs` (lines 250-259, 316-350, 358-362), `src/executor.rs` (lines 28-32), `src/components/controls.rs` (lines 14-24)
- Impact: New control types require touching 4+ files with boilerplate mappings. Risk of missing a layer when adding features.
- Fix approach: Consolidate to a single canonical control enum and operation source enum. Use newtypes or wrapper structs at layer boundaries instead of full re-declarations.

**Simulated operations use real shell processes:**
- Issue: `OperationSpec::simulated_success` and `simulated_failure` generate `sleep` + `printf` shell commands. These are executed via `sh -lc` in real subprocess. The simulated operations in the default `App::new()` (`src/app.rs`) use this for every field, spawning real processes for what is effectively a UI demo.
- Files: `src/runtime.rs` (lines 282-292), `src/schema.rs` (lines 419-428), `src/app.rs` (lines 19-44)
- Impact: Unnecessary process overhead in the default demo. On slow systems or constrained environments, sleep-based simulation introduces visible latency unrelated to actual operations.
- Fix approach: Introduce an in-process simulation mode (e.g., tokio::time::sleep + return) that does not spawn a shell, used by `simulated_success`/`simulated_failure` instead of generating shell command strings.

**`feedback_accent` does not differentiate feedback states visually:**
- Issue: The `feedback_accent` function in `src/components/controls.rs` (line 866) matches all `ControlFeedback` variants (Idle, Running, Success, Failure) into the same branch that only checks `active` / `selected`. Success and Failure border colors are never shown.
- Files: `src/components/controls.rs` (lines 866-881)
- Impact: Operation success/failure states have no visual distinction on the control border, despite the infrastructure existing to support it.
- Fix approach: Map `ControlFeedback::Success` to `Color::Green` and `ControlFeedback::Failure` to `Color::Red` in `feedback_accent`.

**Excessive cloning in operation submission path:**
- Issue: `ShowcaseApp::submit_operation` clones `host.context()`, `host.shell().env()`, and `execution_policy().allowed_working_dirs()` on every operation submit. The executor's `submit` method then clones `request.params`, `request.host`, `request.cwd`, `request.env`, and `request.result_target` again into the spawned task.
- Files: `src/showcase.rs` (lines 479-498), `src/executor.rs` (lines 226-236)
- Impact: For operations triggered by user interaction, the overhead is acceptable. However, if operations were batch-triggered or high-frequency, this allocation pattern would become a bottleneck.
- Fix approach: Move the host context into an `Arc` shared between the app and executor, or pass ownership of request data into the spawned task without intermediate clones.

## Security Considerations

**Shell command injection surface via template rendering:**
- Risk: `render_command_template` in `src/executor.rs` substitutes `{{param}}` values into shell commands. While non-raw values go through `shell_escape` (single-quote wrapping), the `raw:` prefix bypasses escaping entirely. A malicious or malformed parameter value could inject arbitrary commands.
- Files: `src/executor.rs` (lines 436-483), `src/executor.rs` (lines 485-492)
- Current mitigation: `shell_escape` wraps values in single quotes and escapes embedded single quotes. `ShellPolicy` can disable inline shell commands. Execution policy validates working directory and env keys.
- Recommendations: Document the `raw:` prefix security implications clearly. Consider adding a warning log when raw interpolation is used. Validate that parameter values do not contain null bytes.

**Framework logger fallback panics on failure:**
- Risk: `FrameworkLogger::fallback()` calls `.unwrap_or_else(|err| panic!(...))` if it cannot create a log directory relative to `std::env::current_dir()`. In read-only filesystems or sandboxed environments, this panics at startup.
- Files: `src/framework_log.rs` (lines 42-47)
- Current mitigation: None. The fallback is used when no explicit path is configured.
- Recommendations: Return `FrameworkLogger::disabled()` instead of panicking. Log a warning to stderr when falling back to disabled mode.

**Log file content from untrusted file paths:**
- Risk: `LogOutputControl::refresh_from_file` reads arbitrary file paths into memory. The `file_source` path is set during blueprint construction. If the path is user-controlled or derived from user input, it could read sensitive files.
- Files: `src/components/controls.rs` (lines 570-586), `src/runtime.rs` (lines 161-172)
- Current mitigation: Paths come from the host application's blueprint definition, not directly from user input.
- Recommendations: Consider restricting file sources to paths under the working directory or an explicit allowlist.

## Performance Bottlenecks

**`layout_pages` recomputed on every render:**
- Problem: `ContentPanel::layout_pages` iterates through all blueprint sections and blocks, building a `Vec<ContentPage>` on every call. It is called by `total_pages`, `current_page_body`, and `pagination_rows`, all of which are called during `render`. A single render may call `layout_pages` 3+ times.
- Files: `src/components/content_panel.rs` (lines 483-556)
- Cause: No caching of the pagination layout. The layout is recomputed from scratch on every render frame (~8fps at 120ms tick rate).
- Improvement path: Cache the `Vec<ContentPage>` result and invalidate only when `blueprint` or effective height changes. Track a dirty flag on `set_blueprint` and height changes.

**`refresh_file_logs` called on every tick:**
- Problem: `ContentPanel::tick` is called on every `Event::Tick` (every 120ms). It calls `refresh_file_logs`, which iterates all field states and calls `fs::read_to_string` for every `LogOutputControl` that has a `file_source`.
- Files: `src/components/content_panel.rs` (lines 218-231), `src/components/controls.rs` (lines 570-586)
- Cause: No throttling or file modification time checking. Every tick reads all log files from disk.
- Improvement path: Add a tick counter and only refresh file logs every N ticks (e.g., every 500ms instead of 120ms). Alternatively, check file modification time before reading.

**`apply_operation_result` creates a throwaway `ContentPanel`:**
- Problem: `ShowcaseApp::apply_operation_result` constructs a fresh `ContentPanel`, loads a blueprint, applies the result, then extracts the blueprint back. This is done for non-active screens to persist state.
- Files: `src/showcase.rs` (lines 500-511)
- Cause: `ContentPanel` has no way to apply a result to a blueprint without full panel state.
- Improvement path: Add a standalone function that applies an operation result directly to a `ContentBlueprint` without requiring a full `ContentPanel` instance.

## Fragile Areas

**`selected_block_ref` and `block_mut_by_index` use manual index iteration:**
- Files: `src/components/content_panel.rs` (lines 719-743)
- Why fragile: These methods manually iterate through `sections -> blocks` with a mutable `index` counter to find a block by its global index. If the block layout ever changes (e.g., nested sections, dynamic block removal), these linear scans will silently return wrong blocks or `None`.
- Safe modification: Extract a flattened block index at `set_blueprint` time and store it as a `Vec<&ContentBlock>` or `Vec<usize>` section/block pair for O(1) lookup.
- Test coverage: Covered indirectly via `activate_selected_control` tests, but no direct unit tests for `block_mut_by_index` at boundary indices.

**Dual representation of control state (blueprint vs runtime):**
- Files: `src/components/content_panel.rs` (lines 135-140, 158-181), `src/showcase.rs` (lines 454-470)
- Why fragile: The `ContentPanel` stores both `blueprint` (which holds `ContentBlock` definitions including control defaults) and `runtime.field_states` (which holds the actual control values). The `blueprint()` method reconstructs the blueprint from `runtime` state by iterating all sections/blocks and replacing controls. Any mismatch between the two index spaces causes silent data corruption.
- Safe modification: Unify state into a single source of truth. Either always read from `field_states`, or store controls only in `field_states` and keep the blueprint as an immutable template.
- Test coverage: Tested via `activate_selected_control_enters_text_input_mode` and `toggle_changes_persist_while_syncing_panels`, but the dual-write pattern is error-prone for future changes.

**`sync_panels` called excessively:**
- Files: `src/showcase.rs` (lines 426-452)
- Why fragile: `sync_panels` is called from `apply_action`, `sync_active_to_menu_selection`, and the constructor. It calls `load_active_screen_content` on every invocation, which is guarded by `loaded_screen` check. However, `sync_panels` also re-creates `title_panel` text and `status_panel` text on every call, even when nothing changed.
- Safe modification: Track dirty flags for each panel and only update when values actually change.
- Test coverage: Covered by `toggle_changes_persist_while_syncing_panels`, but the frequency of calls in production is not tested.

## Scaling Limits

**In-memory only state with no persistence:**
- Current capacity: All screen content, control values, and operation results are held in memory as `Vec<ShowcaseScreen>` and `Vec<RuntimeFieldState>`.
- Limit: Application state is lost on quit. There is no save/load mechanism. For the default demo this is acceptable, but host applications using this as a framework would need their own persistence layer.
- Scaling path: Add serialization for `ContentBlueprint` and optional file-backed state. The type already derives `Serialize`/`Deserialize` candidates.

**Unbounded operation submission via `mpsc::unbounded_channel`:**
- Current capacity: Operations are submitted through `mpsc::unbounded_channel`. There is no backpressure mechanism.
- Limit: Rapid-fire button clicks could queue many concurrent `tokio::spawn` tasks, each running a shell command. On resource-constrained systems, this could exhaust process file descriptors or memory.
- Scaling path: Replace with `mpsc::channel` with a bounded capacity (e.g., 16). Return a `TryRecvError::Full` equivalent or disable the button when the queue is full.

## Dependencies at Risk

**`color-eyre` with `tracing` overlap:**
- Risk: `color-eyre` is used for error handling and panic hooks. `tracing` and `tracing-subscriber` are declared as dependencies but are never actually used in the source code -- no `tracing::info!`, `tracing::warn!`, or subscriber initialization is found anywhere.
- Impact: Unnecessary dependency weight. `tracing-subscriber` pulls in additional crates (e.g., `regex`, `thread_local`, `nu-ansi-term`). This inflates compile time and binary size for unused functionality.
- Migration plan: Remove `tracing` and `tracing-subscriber` from `Cargo.toml` dependencies. The project already uses a custom `FrameworkLogger` for structured logging and `color-eyre` for error reporting.

**`node_modules` present in a Rust project:**
- Risk: `node_modules/` exists in the project root and is untracked. Its presence in a Rust-only project is suspicious and suggests either a leftover from a JS tool or an accidental inclusion.
- Impact: Directory bloat. Should be excluded via `.gitignore`.
- Migration plan: Add `node_modules` to `.gitignore`. Investigate and remove if not needed.

## Missing Critical Features

**No timeout on shell operations:**
- Problem: `run_shell_command` spawns a child process with no timeout. A hanging command (e.g., `sleep infinity`) will leave the operation in `Running` state forever, with the spinner animating indefinitely.
- Files: `src/executor.rs` (lines 412-434)
- Blocks: Host applications cannot cancel or timeout long-running operations.

**No cancel/abort operation support:**
- Problem: There is no mechanism to cancel a running operation. Once submitted, the operation runs to completion. The UI shows a spinner but provides no abort keybinding.
- Files: `src/executor.rs`, `src/showcase.rs`
- Blocks: Users cannot recover from accidentally triggered long operations without quitting the app.

**TextInput has no cursor movement:**
- Problem: `TextInputControl::handle_key` only supports appending characters and backspace (delete last). There is no cursor positioning, selection, Home/End, or Delete (forward delete) support.
- Files: `src/components/controls.rs` (lines 152-161)
- Blocks: Users cannot edit the middle of a text value. They must backspace and retype.

**NumberInput has no validation:**
- Problem: `NumberInputControl::handle_key` accepts any digit character but performs no validation (e.g., range checks, leading zeros, decimal points). The value is stored as a string with no numeric interpretation.
- Files: `src/components/controls.rs` (lines 99-108)
- Blocks: Host applications cannot enforce numeric constraints at the UI level.

## Test Coverage Gaps

**`src/app.rs` default application has no tests:**
- What's not tested: The `App` struct, its `filler_screen` helper, and the default screen configuration. No test verifies that `App::new()` produces a valid, renderable app.
- Files: `src/app.rs`
- Risk: Changes to `field::*` helpers or the default app structure could produce runtime panics without test coverage.
- Priority: Medium

**No render tests for any component:**
- What's not tested: Visual rendering output of `ContentPanel`, `MenuComponent`, `QuadrantLayout`, `TitlePanel`, `StatusPanel`. Ratatui supports buffer-based testing, but no tests verify rendered output.
- Files: All files in `src/components/`
- Risk: Layout regressions, text truncation bugs, or color changes could go undetected.
- Priority: Low (visual regressions are typically caught manually in TUI apps)

**`QuadrantLayout::calculate_quadrants` edge cases:**
- What's not tested: Very small terminal sizes (e.g., 80x24 minimum), non-square terminals where `height > width`, and the boundary condition where `height.min(width)` produces unusual splits.
- Files: `src/components/quadrant.rs` (lines 50-82)
- Risk: Layout could produce zero-width or zero-height quadrants at the supported minimum terminal size.
- Priority: High (minimum size is explicitly checked but edge behavior is not verified)

**`EventHandler` has no integration tests:**
- What's not tested: The async event loop, tick timing, Ctrl+C handling, and resize events. Only `Key` conversion is unit-tested.
- Files: `src/event.rs`
- Risk: Event ordering issues, missed events, or tick rate regressions would go undetected.
- Priority: Low (requires terminal for proper testing)

**`LogOutputControl::append_entry` trim behavior:**
- What's not tested: The filtering of empty lines and the interaction between `MAX_LINES` (24) and `tail_lines` when both are active. The `append_entry` method filters empty lines from existing content, which could surprise host applications that intentionally include blank lines for formatting.
- Files: `src/components/controls.rs` (lines 519-541)
- Risk: Log output may not match expected formatting when blank lines are significant.
- Priority: Medium

---

*Concerns audit: 2026-03-28*
