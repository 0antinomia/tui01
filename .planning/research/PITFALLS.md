# Domain Pitfalls

**Domain:** Rust TUI framework (brownfield) -- pure architecture refactoring
**Researched:** 2026-03-28
**Overall confidence:** HIGH (findings grounded in direct codebase analysis + Rust ecosystem patterns)

## Critical Pitfalls

Mistakes that cause rewrites or major issues. These are specific to the tui01 refactoring context.

---

### Pitfall 1: The Triple-Enum Consistency Trap

**What goes wrong:** The codebase has three separate enums with nearly identical variants: `ControlSpec` (schema.rs:362), `RuntimeControl` (runtime.rs:317), and `ControlKind` (controls.rs:14). Similarly, `OperationSource` is duplicated in both executor.rs:29 and runtime.rs:358. During refactoring, it is extremely tempting to "just consolidate" these into a single canonical enum. But each enum serves a distinct layer purpose (specification, runtime state, UI rendering), and collapsing them naively removes the layer boundaries that prevent coupling.

**Why it happens:** The duplication looks like DRY violation on the surface. The refactoring goal ("new control type only touches 1-2 files") seems to require eliminating all three.

**Consequences:** If you merge all three into one enum, the `ContentPanel` (which currently only needs `ContentControl`) now depends on the rendering details of `ControlKind`. The `schema.rs` specification layer now imports rendering types. You have traded duplication for coupling, which is worse for extensibility.

**Prevention:** Do not collapse the enums into one. Instead, introduce conversion traits or `From`/`Into` implementations between the layers. Each layer keeps its own type, but the mapping is mechanical and single-location. A new control type adds one variant to each enum plus one `From` impl -- that is 3 files, not 10, and the files are logically decoupled.

**Detection:** If a PR adds an `impl` that imports both `ControlKind` and `ControlSpec` in the same file, the boundary has been crossed.

**Phase:** This must be addressed in the "extension mechanism redesign" phase, not the "module reorganization" phase. Reorganize first, consolidate types second.

---

### Pitfall 2: Import Alias Collision During Module Moves

**What goes wrong:** `content_panel.rs` already has to import `OperationSource as ExecutorOperationSource` (line 6) because the same type name exists in both `executor` and `runtime` modules. When you reorganize modules into subdirectories, the import paths change. If the new module structure places `OperationSource` in a shared location, this alias becomes dead code but may not compile if both old and new paths are attempted.

**Why it happens:** Module moves change `use crate::...` paths. Files that import the same name from two different modules (the alias pattern) are fragile during moves because the alias target must be updated precisely.

**Consequences:** Compilation errors that are noisy and cascade. One wrong path resolution can produce 20+ errors in dependent files because Rust reports errors at every use site, not just the import.

**Prevention:**
1. Before moving any module, extract a canonical `types.rs` or `core.rs` for shared types like `OperationSource`. Move the canonical definition first.
2. Add `pub use` re-exports at the old location so downstream imports continue to work.
3. Only after tests pass, update the downstream imports to the new canonical path.
4. Remove the re-exports last.

**Detection:** Run `cargo check` after every single file move. Do not batch multiple moves before checking.

**Phase:** Module reorganization phase. This is the first operation that should happen -- canonical shared types before moving dependent modules.

---

### Pitfall 3: Breaking the `use crate::` Test Imports

**What goes wrong:** All 87 tests live inside `#[cfg(test)] mod tests` blocks within source files. These tests import from sibling modules using `use crate::other_module::Type` paths. When modules are reorganized into subdirectories (e.g., `src/runtime.rs` becomes `src/runtime/mod.rs` or `src/app/runtime.rs`), every test that imports `crate::runtime::*` must be updated. There are 49+ cross-module `use crate::` imports across the codebase.

**Why it happens:** Rust's `use crate::` paths are absolute from the crate root. Moving `runtime.rs` into a subdirectory changes the path from `crate::runtime::ContentBlueprint` to potentially `crate::core::runtime::ContentBlueprint`. If `lib.rs` module declarations change, all downstream paths break.

**Consequences:** A single module move can break 10-20 test files simultaneously. The error messages point at the test files, not at the moved module, making diagnosis confusing. Tests may also fail at the `use` line with "unresolved import" before reaching any assertion, giving no signal about what actually broke functionally.

**Prevention:**
1. Never change `lib.rs` public module structure until internal reorganization is complete and tests pass.
2. Use `pub mod` re-exports in `lib.rs` to maintain the old top-level module names even after internal restructuring. For example, if `runtime.rs` becomes `src/core/runtime.rs`, add `pub mod runtime; pub use core::runtime;` in `lib.rs`.
3. Migrate test imports incrementally: move one module, fix its tests, run `cargo test`, commit, then move the next.
4. Create a migration checklist: for each file to move, list all files that import from it (the grep output shows this clearly).

**Detection:** `cargo test 2>&1 | grep "unresolved import" | sort | uniq -c | sort -rn` -- if this produces output, too many things moved at once.

**Phase:** Module reorganization phase. The incrementality rule (one module at a time) is the single most important discipline for this entire refactoring.

---

### Pitfall 4: Destroying the Dual State Synchronization Contract

**What goes wrong:** `ContentPanel` stores both `blueprint` (immutable template with control defaults) and `runtime.field_states` (mutable runtime values). The `blueprint()` method reconstructs the blueprint from runtime state by iterating sections/blocks and replacing controls (content_panel.rs:135-181). This dual-write pattern is fragile but currently works because the index spaces are kept in sync. If the refactoring splits the blueprint management from the runtime state management into separate modules, the implicit contract that "index N in blueprint corresponds to index N in field_states" can silently break.

**Why it happens:** The current code co-locates both states in the same struct, so the synchronization is maintained by proximity. Splitting into separate modules means the synchronization must become explicit, and any mismatch causes silent data corruption (wrong control values shown for wrong fields) rather than a compile error.

**Consequences:** Tests that check specific control values (e.g., `toggle_changes_persist_while_syncing_panels`) may still pass because they test the happy path. The corruption only manifests when the index arithmetic is wrong for non-trivial section/block combinations. This is exactly the kind of bug that passes all 87 tests but breaks in production.

**Prevention:**
1. Before splitting, add explicit assertions or debug assertions that validate `blueprint.sections.iter().map(|s| s.blocks.len()).sum::<usize>() == field_states.len()` at every entry point that modifies either.
2. Introduce a typed `FieldIndex` newtype instead of raw `usize` for block indexing, so the compiler can catch mismatches.
3. If splitting the dual state, keep both behind a single facade type that enforces the invariant. Do not expose raw `field_states` and `blueprint` to separate modules.

**Detection:** If a test that calls `panel.blueprint()` and then `panel.field_state_mut()` in sequence starts returning inconsistent data, the contract is broken.

**Phase:** Large file splitting phase (specifically when splitting content_panel.rs). This is the highest-risk single operation in the entire refactoring.

---

### Pitfall 5: The `prelude.rs` Re-export Chain Explosion

**What goes wrong:** `prelude.rs` currently re-exports from `builder`, `event`, `field`, and `host`. During refactoring, it is tempting to add re-exports for all moved types to maintain backward compatibility. But `prelude.rs` is a public API surface: every `pub use` added there becomes part of the crate's public API and cannot be removed without a semver break. Adding "temporary" re-exports for migration creates permanent API surface.

**Why it happens:** The desire to make the refactoring invisible to downstream consumers (in this case, `main.rs` and any future host apps) leads to aggressive `pub use` chains. These chains can also create circular or redundant re-export paths that confuse documentation and IDE autocomplete.

**Consequences:** The public API becomes bloated with implementation details. Users see types at multiple import paths (`tui01::runtime::ContentBlueprint` and `tui01::ContentBlueprint`), which causes confusion. Removing the redundant paths later is itself a breaking change if anyone adopted them.

**Prevention:**
1. The prelude should only contain items intended for long-term public API. Do not add migration re-exports to the prelude.
2. Use `pub(crate)` re-exports in `lib.rs` for internal migration, not `pub` re-exports.
3. Decide on the final public API surface before starting the refactoring, and only put those items in `prelude.rs`.
4. The `pub use crate::field;` line in prelude.rs (line 5) re-exports an entire module. This is unusual -- consider whether individual functions or a submodule is more appropriate.

**Detection:** If `prelude.rs` grows beyond 10 lines of re-exports during the refactoring, the trap has been sprung.

**Phase:** Public API redesign phase. Must be designed before module reorganization begins, because module structure determines what the prelude exposes.

---

### Pitfall 6: Orphan Rule Blocking Trait Consolidation

**What goes wrong:** The `Component` trait (components/mod.rs:27) is defined in the `components` module. If the refactoring wants to move it to a shared `core` module while keeping its implementations in `components`, Rust's orphan rule requires that either the trait or the implementing type must be in the same crate as the `impl` block. This is fine for a single-crate project like tui01, but if individual control types (TextInputControl, etc.) are split into their own files and the trait is in a different module, the `impl Component for X` blocks must be in the module that defines either the trait or the type.

**Why it happens:** When splitting large files, developers often try to put the trait definition in one module and the implementations in another module's file. In Rust, `impl Trait for Type` must be in the crate that defines either `Trait` or `Type`. Within a single crate, this is always satisfied, but the module structure can make it unclear where implementations belong.

**Consequences:** Compiles fine (no orphan rule violation within a crate), but creates confusion about where to add new `impl` blocks. This defeats the "1-2 files to add a new control" goal if the developer must hunt through the module tree to find where the trait impl goes.

**Prevention:**
1. Co-locate each control type's struct definition and its `impl` blocks (including trait impls) in the same file.
2. The `Component` trait itself can live in a shared location, but each control file should contain its own `impl Component for TextInputControl` etc.
3. Document a clear convention: "New controls go in `src/controls/text_input.rs`. Each file contains the struct, the `ControlVariant` impl, the `Component` impl, and the render function."

**Detection:** If a control type's struct is defined in one file but its `impl Component` is in a different file, the architecture needs correction.

**Phase:** Extension mechanism redesign phase.

---

## Moderate Pitfalls

### Pitfall 7: Re-Export Visibility Trap When Moving to Subdirectories

**What goes wrong:** When `src/runtime.rs` becomes `src/runtime/mod.rs`, any item that was `pub` in the old file remains `pub`. But if `runtime/mod.rs` then declares submodules (e.g., `mod types;`), items in `types.rs` are private to the `runtime` module unless explicitly made `pub` or `pub(crate)`. Items that were visible as `crate::runtime::Type` may become invisible if they move to `runtime::types::Type` without a re-export.

**Prevention:** After every file-to-directory conversion, check that all previously `pub` items are still accessible at their old paths. Add `pub use` in the `mod.rs` for each submodule's public items. Use `cargo doc --document-private-items` to detect items that lost visibility.

**Phase:** Module reorganization.

---

### Pitfall 8: Test Fixture Coupling to Module Structure

**What goes wrong:** Test fixtures like `panel_with_sections()` in content_panel.rs or `make_app()` in showcase.rs construct types using `ContentBlock::text_input(...)`, `ContentBlueprint::new(...)`, etc. These constructors may move to different modules during refactoring. If the fixture helper functions are updated but the tests that call them are in a different file (not currently the case, but may happen if tests are extracted to separate files), the fixtures and tests can drift apart.

**Prevention:** Keep test fixtures in the same `mod tests` block as the tests that use them. Do not extract test fixtures to shared modules during this refactoring. The current pattern (fixtures co-located with tests) is correct and should be preserved.

**Phase:** Large file splitting phase. When splitting content_panel.rs (1526 lines, 21 tests), the test block is large. Resist the urge to extract it. Instead, split the production code and keep the test block intact.

---

### Pitfall 9: `rstest` Parameterized Tests and Enum Variant Changes

**What goes wrong:** The `event.rs` test uses `rstest` with `#[case]` attributes mapping `CrosstermKeyCode` variants to `Key` variants (9 cases). If the refactoring moves the `Key` enum to a different module or changes its visibility, the `#[case]` attribute paths break silently -- they compile but the test function signature changes, which can confuse `cargo test` filtering by name.

**Prevention:** Do not move the `Key` enum or `Event` enum during the initial module reorganization. These are leaf types with minimal coupling. Move them only after all structural changes are stable.

**Phase:** Module reorganization (deferred action, not early action).

---

### Pitfall 10: The `showcase.rs` Integration Hub Fragility

**What goes wrong:** `showcase.rs` (813 lines) imports from 10 different modules: `action`, `builder`, `components`, `event`, `executor`, `framework_log`, `host`, `runtime`, `schema`, and `tui`. It is the central integration hub. Any module move that changes any of these import paths will break showcase.rs. With 13 tests in showcase.rs, this is the single file most likely to accumulate compilation errors during module reorganization.

**Prevention:** Treat showcase.rs as the canary in the coal mine. After every module move, run `cargo test showcase` first. If it passes, the move is safe. If it fails, the import chain is broken.

**Phase:** Module reorganization. This file should be the first test target after every move.

---

### Pitfall 11: Async Test Timing Sensitivity During Executor Refactoring

**What goes wrong:** The executor tests use a poll loop pattern (20 iterations, 10ms sleep each, ~200ms total timeout). If the executor module is split and the async task spawning logic changes even slightly (e.g., different channel capacity, different task structure), these timing-sensitive tests may become flaky. A test that passes at 10ms polling interval may fail at 15ms, or pass locally but fail under CI load.

**Prevention:** Do not change the executor's internal async task spawning mechanism during the file splitting phase. Split the file structure but keep the actual tokio::spawn + mpsc channel logic identical. The async mechanism should only change in a dedicated phase with explicit test stability verification.

**Phase:** Large file splitting (executor.rs: 870 lines, 11 tests, 7 async). This file should be split structurally but not behaviorally.

---

### Pitfall 12: `lib.rs` Module Declaration Order Matters

**What goes wrong:** `lib.rs` declares modules in a specific order: `action, app, builder, components, event, executor, field, framework_log, host, prelude, runtime, schema, showcase, tui`. Rust processes module declarations top-to-bottom. If module A imports from module B, and B is declared after A, this works in Rust 2021 edition (uniform paths resolve from crate root). But if the refactoring introduces re-exports that depend on declaration order, or if a module is moved to a subdirectory and its `mod.rs` has ordering dependencies, cryptic "cannot find type" errors can appear.

**Prevention:** Maintain the current declaration order in `lib.rs`. Only add new module declarations at the end. If moving a module to a subdirectory, the `pub mod` declaration in `lib.rs` stays in the same position.

**Phase:** Module reorganization.

---

## Minor Pitfalls

### Pitfall 13: `main.rs` Uses External Crate Name

**What goes wrong:** `main.rs` imports `use tui01::app::App` using the crate name `tui01`. This is correct for a binary target in a Rust crate. But if the crate is renamed or if the module structure changes `app::App` to a different path, `main.rs` must be updated separately. It is outside the `use crate::` system.

**Prevention:** After refactoring, verify `cargo build --bin tui01` succeeds in addition to `cargo test`. The binary target is easy to overlook when focused on test results.

**Phase:** Final verification phase.

---

### Pitfall 14: `Cargo.toml` Unused Dependency Confusion

**What goes wrong:** The codebase has `tracing` and `tracing-subscriber` as dependencies but never uses them (noted in CONCERNS.md). During refactoring, someone might try to "clean up" by removing them. This changes `Cargo.toml` during a pure refactoring, which is out of scope and can cause surprising `cargo build` cache invalidation.

**Prevention:** Do not modify `Cargo.toml` during this refactoring. Dependency cleanup is a separate task.

**Phase:** Out of scope. Flag for post-refactoring cleanup.

---

### Pitfall 15: `node_modules` in a Rust Project

**What goes wrong:** A `node_modules/` directory exists in the project root (untracked). If the refactoring uses any tool that traverses the directory tree (e.g., file watchers, certain IDE refactorings), this directory can cause slowdowns or false positives.

**Prevention:** Add `node_modules` to `.gitignore` before starting the refactoring. Verify it is excluded from any search/index operations.

**Phase:** Pre-refactoring preparation.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Module reorganization (moving runtime.rs to subdirectory) | Import alias collision (Pitfall 2) | Extract shared types first, add re-exports at old locations |
| Module reorganization (moving multiple files) | Test import breakage cascade (Pitfall 3) | Move one module at a time, run `cargo test` between each |
| Large file splitting: content_panel.rs (1526 lines, 21 tests) | Dual state synchronization breakage (Pitfall 4) | Add debug assertions before splitting, keep both states behind one facade |
| Large file splitting: controls.rs (1008 lines, 9 tests) | Component trait orphan confusion (Pitfall 6) | Co-locate trait impls with type definitions |
| Large file splitting: executor.rs (870 lines, 11 tests) | Async test flakiness (Pitfall 11) | Split structure only, do not change async mechanism |
| Large file splitting: showcase.rs (813 lines, 13 tests) | Import breakage from 10+ dependencies (Pitfall 10) | Test this file first after every change |
| Extension mechanism redesign | Triple-enum naive consolidation (Pitfall 1) | Use conversion traits, not single merged enum |
| Public API redesign | Prelude chain explosion (Pitfall 5) | Define target API before refactoring, use `pub(crate)` for internal paths |
| Key/Event enum relocation | rstest parameterized test breakage (Pitfall 9) | Defer leaf type moves until structure is stable |

## Golden Rules for This Refactoring

1. **One move at a time.** Move one file, run `cargo test`, commit. Never batch module moves.
2. **Re-export before removing.** Add `pub use old_path::Type;` before deleting the old definition.
3. **Canary test: `cargo test showcase`.** After every change, this catches the most import breakage.
4. **Do not change behavior.** The executor's async mechanism, the ContentPanel's index arithmetic, and the sync_panels logic must remain identical. Split structure, not semantics.
5. **Test count must not change.** The refactoring must maintain exactly 87 passing tests. Any change in test count (added, removed, or disabled) is a scope violation.

## Sources

- [Rust Reference: Visibility and Privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html) -- HIGH confidence, official documentation
- [The Rust Book: Refactoring to Improve Modularity](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html) -- HIGH confidence, official documentation
- [Moving and re-exporting a Rust type can be a major breaking change](https://predr.ag/blog/moving-and-reexporting-rust-type-can-be-major-breaking-change/) -- HIGH confidence, detailed analysis of `pub use` vs `pub type` pitfalls
- [Two Ways of Interpreting Visibility in Rust](https://kobzol.github.io/rust/2025/04/23/two-ways-of-interpreting-visibility-in-rust.html) -- HIGH confidence, visibility modifier nuances
- [Rust Users Forum: Split Enum into Multiple Modular Repositories](https://users.rust-lang.org/t/split-enum-into-multiple-modular-repositories/37875) -- MEDIUM confidence, community pattern
- [Stack Overflow: Avoiding duplication when comparing two enums' variants](https://stackoverflow.com/questions/54606664/how-to-avoid-code-duplication-when-comparing-two-enums-variants-and-their-value) -- MEDIUM confidence, enum consolidation patterns
- [Rust Users Forum: enum_dispatch for reducing boilerplate](https://users.rust-lang.org/t/is-there-a-way-to-avoid-writing-duplicated-on-trait-enums/131264) -- MEDIUM confidence, community discussion on `enum_dispatch`
- [Ratatui: The Elm Architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/) -- HIGH confidence, official framework documentation
- [Reddit: Rust Modules Best Practices and Pitfalls](https://www.reddit.com/r/rust/comments/alsph9/rusts_modules_and_project_organization_best/) -- MEDIUM confidence, community consensus
- [corrode.dev: Long-term Rust Project Maintenance](https://corrode.dev/blog/long-term-rust-maintenance/) -- MEDIUM confidence, practical maintenance advice
- Direct codebase analysis of tui01 src/ -- HIGH confidence, primary source

---

*Research: 2026-03-28*
