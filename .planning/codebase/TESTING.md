# Testing Patterns

**Analysis Date:** 2026-03-28

## Test Framework

**Runner:**
- Built-in Rust test harness (`cargo test`)
- No custom test runner configuration in `Cargo.toml`
- Edition: 2021

**Assertion Library:**
- Standard library `assert!`, `assert_eq!`, `assert!(matches!(...))`
- `panic!("expected ...")` for unreachable match arms in tests

**Additional Testing Crates:**
- `rstest = "0.24"` (dev-dependency) -- used for parameterized tests via `#[rstest]` + `#[case(...)]`

**Run Commands:**
```bash
cargo test                    # Run all tests
cargo test --lib              # Run library tests only (excludes binary targets)
cargo test --test demo        # Run specific binary test target
cargo test module_name        # Run tests in a specific module (e.g., `cargo test showcase`)
cargo test test_function_name # Run a specific test by name
cargo test -- --nocapture     # Show println! output
cargo test 2>/dev/null        # Suppress compilation warnings for cleaner output
```

## Test File Organization

**Location:**
- All tests are co-located in the same source files using `#[cfg(test)] mod tests` blocks
- No separate `tests/` directory for integration tests (except binary examples)
- No standalone test files

**Files containing tests (13 of 18 source files):**
```
src/builder.rs                  -- 8 tests
src/event.rs                    -- 1 test
src/field.rs                    -- 3 tests
src/framework_log.rs            -- 2 tests
src/host.rs                     -- 6 tests
src/schema.rs                   -- 1 test
src/showcase.rs                 -- 13 tests
src/runtime.rs                  -- 2 tests
src/executor.rs                 -- 11 tests
src/components/content_panel.rs -- 21 tests
src/components/controls.rs      -- 9 tests
src/components/menu.rs          -- 8 tests
src/components/quadrant.rs      -- 2 tests
```

**Total: 87 test functions across 13 files**

**Files without tests (5):**
- `src/main.rs` -- binary entry point
- `src/lib.rs` -- module declarations only
- `src/prelude.rs` -- re-exports only
- `src/action.rs` -- pure enum definition
- `src/components/title_panel.rs` -- simple render component
- `src/components/status_panel.rs` -- simple render component

**Naming:**
- Test module: always `mod tests` (not `mod test`)
- Test functions: descriptive snake_case describing the behavior being verified
- Examples: `page_spec_materializes_runtime_blueprint`, `text_and_number_id_helpers_apply_ids`
- Convention: `<subject>_<behavior>` or `<subject>_<expected_outcome>`

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::{ItemA, ItemB};   // Import items from parent module
    use crate::other_module::X;   // Cross-module imports
    use std::sync::{Arc, Mutex};  // Standard library utilities

    #[test]
    fn subject_does_expected_behavior() {
        // Arrange
        let mut panel = ContentPanel::new();
        panel.set_blueprint(/* ... */);

        // Act
        panel.select_next_block(13);

        // Assert
        assert_eq!(panel.selected_block(), 1);
    }
}
```

**Patterns:**
- Arrange-Act-Assert structure in most tests
- Helper functions at the top of `mod tests` for creating test fixtures
- Tests use `assert_eq!` for equality checks, `assert!(matches!(...))` for enum variant checks
- `panic!("expected X")` in match arms that should not be reached

## Test Helpers and Fixtures

**Common fixture functions:**
```rust
// In src/showcase.rs tests:
fn screen(title: &'static str, text: &'static str) -> ShowcaseScreen {
    ShowcaseScreen {
        title: title.to_string(),
        content: ContentBlueprint::new(title).with_sections(vec![ContentSection::new("概览")
            .with_blocks(vec![ContentBlock::text_input(text, "", "输入值")])]),
    }
}

fn make_app() -> ShowcaseApp {
    ShowcaseApp::new(
        ShowcaseCopy { title_text: "Title".to_string(), status_controls: "Controls".to_string() },
        vec![screen("One", "First"), screen("Two", "Second")],
    )
}
```

```rust
// In src/components/content_panel.rs tests:
fn panel_with_sections() -> ContentPanel {
    let mut panel = ContentPanel::new();
    panel.set_blueprint(ContentBlueprint::new("Root").with_sections(vec![
        ContentSection::new("概览").with_blocks(vec![
            ContentBlock::text_input("用户名", "demo", "输入用户名"),
            ContentBlock::toggle("开启高级模式", true),
        ]),
        // ... more sections
    ]));
    panel
}
```

```rust
// In src/executor.rs tests:
fn test_framework_logger() -> FrameworkLogger {
    FrameworkLogger::new(std::env::temp_dir()).unwrap()
}
```

**Fixture location:**
- All fixtures are defined inside the `mod tests` block of the file that uses them
- No shared fixture modules or test utility files
- Fixtures use `std::env::temp_dir()` for file-based tests (with cleanup)

## Mocking

**Framework:** No mocking framework. Uses real implementations and `Arc<Mutex<Vec<_>>>` for capturing side effects.

**Pattern -- Event capture:**
```rust
let events = Arc::new(Mutex::new(Vec::<HostEvent>::new()));
let capture = events.clone();
let hook = Arc::new(move |event| {
    capture.lock().unwrap().push(event);
});
```
Used in `src/executor.rs` and `src/host.rs` tests to capture callback invocations.

**Pattern -- Logger capture:**
```rust
let logs = Arc::new(Mutex::new(Vec::<HostLogRecord>::new()));
let capture = logs.clone();
let logger = Arc::new(move |record| {
    capture.lock().unwrap().push(record);
});
```

**What to "Mock":**
- Event hooks: Replace with `Arc<Mutex<Vec<_>>>` capture closures
- Logger hooks: Replace with capture closures
- File system: Use `std::env::temp_dir()` with `std::process::id()` for unique paths, clean up after test

**What NOT to Mock:**
- Real components (MenuComponent, ContentPanel) -- construct and test directly
- Blueprint/schema types -- use real constructors
- Event/Key types -- use real enum variants

## Parameterized Tests

**Framework:** `rstest` crate

**Pattern:**
```rust
use rstest::rstest;

#[rstest]
#[case(CrosstermKeyCode::Char('x'), Key::Char('x'))]
#[case(CrosstermKeyCode::Up, Key::Up)]
#[case(CrosstermKeyCode::Down, Key::Down)]
#[case(CrosstermKeyCode::F(1), Key::Unknown)]
fn converts_crossterm_keycodes(#[case] input: CrosstermKeyCode, #[case] expected: Key) {
    assert_eq!(Key::from(input), expected);
}
```

**Where used:** `src/event.rs` -- for testing KeyCode-to-Key conversion (9 cases in a single function)

**When to use:**
- When testing the same logic with multiple input/output pairs
- When the test body is identical except for input and expected output
- Use `#[rstest]` + `#[case(...)]` attributes
- Use `#[case]` attribute on parameters

## Async Testing

**Pattern:**
```rust
#[tokio::test]
async fn registered_handler_action_returns_custom_result() {
    let mut registry = ActionRegistry::new();
    registry.register_action_handler("echo_params", |context| async move {
        ActionOutcome::success(format!(
            "project={}",
            context.params.get("project_name").cloned().unwrap_or_default()
        ))
    });
    let mut executor = OperationExecutor::with_registry(registry);

    executor.submit(OperationRequest {
        operation_id: 1,
        // ... fields
    });

    // Poll for result with timeout
    let mut result = None;
    for _ in 0..20 {
        if let Some(value) = executor.try_recv() {
            result = Some(value);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let result = result.expect("expected async action result");
    assert!(result.success);
    assert_eq!(result.stdout, "project=tui01");
}
```

**Async testing conventions:**
- Use `#[tokio::test]` for async tests
- Poll with a retry loop (20 iterations, 10ms sleep) for background task results
- Use `.expect("expected ...")` on Option results rather than `.unwrap()`
- Total async test timeout: ~200ms maximum (20 * 10ms)

**Where used:** `src/executor.rs` -- 7 of 11 tests are async

## Error Testing

**Pattern -- Validation error matching:**
```rust
let err = AppSpec::new()
    .title_text("Demo")
    // ... setup
    .validate()
    .unwrap_err();

assert!(matches!(
    err,
    AppValidationError::UnknownRegisteredAction { .. }
));
```

**Pattern -- Debug string inspection (for opaque types):**
```rust
let text = text_id("项目名", "demo", "输入项目名", "project_name");
let debug_text = format!("{text:?}");
assert!(debug_text.contains("project_name"));
```
Used when struct fields are private and cannot be directly inspected.

**Pattern -- Control state verification:**
```rust
match &panel.blueprint().sections[0].blocks[0].control {
    ContentControl::TextInput(control) => assert_eq!(control.value, "demox"),
    _ => panic!("expected text input"),
}
```

**Pattern -- Success/failure checking:**
```rust
assert!(result.success);
assert_eq!(result.stdout, "expected output");
assert!(result.stderr.contains("expected error text"));
```

## Test Isolation

**File system tests:**
```rust
#[test]
fn framework_logger_writes_to_file() {
    let base = std::env::temp_dir().join(format!("tui01-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);  // Clean up from previous runs

    let logger = FrameworkLogger::new(&base).unwrap();
    logger.log(&HostLogRecord { /* ... */ });

    let content = fs::read_to_string(PathBuf::from(logger.path())).unwrap();
    assert!(content.contains("expected text"));

    let _ = fs::remove_dir_all(base.join(".tui01"));  // Clean up after test
}
```

**Isolation conventions:**
- Use `std::process::id()` for unique temp paths to avoid collisions
- Clean up created files/directories after test assertions
- Use `let _ = fs::remove_dir_all(...)` for best-effort cleanup (ignore errors)

## Coverage

**Requirements:** None enforced (no `tarpaulin`, `llvm-cov`, or coverage configuration)

**Estimated coverage by module:**
- `src/builder.rs` -- 8 tests, covers validation and construction paths
- `src/executor.rs` -- 11 tests, covers shell commands, registered actions, policy enforcement, event hooks
- `src/showcase.rs` -- 13 tests, covers keyboard navigation, focus management, screen switching, resize
- `src/components/content_panel.rs` -- 21 tests, covers pagination, selection, control activation, operation results
- `src/components/controls.rs` -- 9 tests, covers individual control behaviors (input, select, toggle, log)
- `src/components/menu.rs` -- 8 tests, covers navigation, pagination, focus behavior
- `src/host.rs` -- 6 tests, covers context, actions, shell policy, event hooks, logger hooks
- `src/field.rs` -- 3 tests, covers helper function composition
- `src/runtime.rs` -- 2 tests, covers type conversion and state tracking
- `src/schema.rs` -- 1 test, covers materialization
- `src/event.rs` -- 1 parameterized test (9 cases), covers all Key conversions
- `src/framework_log.rs` -- 2 tests, covers file writing and disabled mode
- `src/components/quadrant.rs` -- 2 tests, covers layout calculation

**Uncovered areas:**
- `src/main.rs` -- integration-level run loop not tested (requires terminal)
- `src/tui.rs` -- terminal size checking tested indirectly via `src/showcase.rs` resize tests
- `src/components/title_panel.rs` -- pure rendering, no logic
- `src/components/status_panel.rs` -- pure rendering, no logic
- Rendering output -- no visual regression tests (ratatui buffer inspection not used for assertions)

## Test Types

**Unit Tests:**
- Scope: Individual functions, methods, and type conversions
- Approach: Construct types directly, call methods, assert on state
- Location: `#[cfg(test)] mod tests` blocks within source files

**Integration Tests:**
- Scope: Multi-module interactions (e.g., executor + host + framework logger)
- Approach: Wire up real components and verify end-to-end behavior
- Location: Same `mod tests` blocks -- no separate integration test directory

**E2E Tests:**
- Not used. No terminal simulation or visual regression testing.

## Common Patterns Summary

**1. Construct -> Act -> Assert:**
```rust
let mut app = make_app();
app.handle_event(Event::Key(Key::Down));
assert_eq!(app.active_screen(), 1);
```

**2. Capture side effects via Arc<Mutex<Vec>>:**
```rust
let events = Arc::new(Mutex::new(Vec::new()));
let capture = events.clone();
let host = RuntimeHost::new().on_event(move |event| {
    capture.lock().unwrap().push(event);
});
// ... trigger event
assert_eq!(events.lock().unwrap().len(), 1);
```

**3. Match on control variant:**
```rust
match &panel.blueprint().sections[0].blocks[0].control {
    ContentControl::Toggle(control) => assert!(control.on),
    _ => panic!("expected toggle"),
}
```

**4. Async poll loop:**
```rust
let mut result = None;
for _ in 0..20 {
    if let Some(value) = executor.try_recv() {
        result = Some(value);
        break;
    }
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}
let result = result.expect("expected async action result");
```

**5. Parameterized via rstest:**
```rust
#[rstest]
#[case(input_a, expected_a)]
#[case(input_b, expected_b)]
fn test_name(#[case] input: Type, #[case] expected: Type) {
    assert_eq!(function(input), expected);
}
```

---

*Testing analysis: 2026-03-28*
