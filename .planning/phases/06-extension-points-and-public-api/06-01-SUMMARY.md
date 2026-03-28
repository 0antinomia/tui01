---
phase: 06-extension-points-and-public-api
plan: 01
subsystem: framework-core
tags: [theme, serde, layout-strategy, render-context, extension-points, ratatui]

# Dependency graph
requires:
  - phase: 05-large-file-decomposition
    provides: decomposed module structure with controls/ and components/ directories
provides:
  - Theme struct with 6 semantic Color slots and Serde derive
  - RenderContext struct bundling theme + selected + active + feedback (Copy)
  - LayoutStrategy trait with areas() returning LayoutAreas
  - LayoutAreas struct with named Rect fields for title/status/menu/content
  - QuadrantLayout implementing LayoutStrategy
  - Theme types exported via tui01::prelude
affects: [06-extension-points-and-public-api/02]

# Tech tracking
tech-stack:
  added: [serde 1.x with derive feature, ratatui serde feature, serde_json dev-dependency]
  patterns: [semantic theme slots, layout strategy trait, render context bundling]

key-files:
  created:
    - src/theme.rs
  modified:
    - Cargo.toml
    - src/lib.rs
    - src/prelude.rs
    - src/components/quadrant.rs

key-decisions:
  - "lib.rs module registration moved to Task 1 (required for compilation/testing)"
  - "Theme uses manual Default impl rather than derive (Color has no Default)"
  - "serde_json added as dev-dependency for round-trip test"

patterns-established:
  - "Theme with 6 semantic slots: border/text/selected/active/error/success"
  - "LayoutStrategy trait with single areas() method for layout abstraction"
  - "RenderContext as Copy struct bundling render parameters for ControlTrait"

requirements-completed: [EXT-01, EXT-02, MOD-07]

# Metrics
duration: 5min
completed: 2026-03-28
---

# Phase 06 Plan 01: Extension Point Types Summary

**Theme struct with 6 semantic Color slots and Serde derive, LayoutStrategy trait implemented by QuadrantLayout, RenderContext bundling render parameters, all exported via prelude**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-28T14:41:54Z
- **Completed:** 2026-03-28T14:46:51Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Created src/theme.rs with Theme (6 Color fields, Serialize/Deserialize, manual Default matching hardcoded colors), RenderContext (Copy), LayoutAreas, and LayoutStrategy trait
- QuadrantLayout implements LayoutStrategy by wrapping existing calculate_quadrants into LayoutAreas
- All types exported via tui01::prelude for host application access
- Added serde + ratatui serde feature to Cargo.toml; all 105 tests pass (100 existing + 5 new)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create src/theme.rs with Theme, RenderContext, LayoutAreas, LayoutStrategy** - `038e2f4` (feat)
2. **Task 2: Implement LayoutStrategy for QuadrantLayout, register theme module, export via prelude** - `9deecc7` (feat)

## Files Created/Modified
- `src/theme.rs` - Theme, RenderContext, LayoutAreas, LayoutStrategy definitions with 4 unit tests
- `Cargo.toml` - Added serde dependency with derive feature, ratatui serde feature, serde_json dev-dependency
- `src/lib.rs` - Added `pub mod theme;` registration
- `src/prelude.rs` - Added Theme, LayoutStrategy, RenderContext, LayoutAreas re-exports
- `src/components/quadrant.rs` - LayoutStrategy impl for QuadrantLayout + layout_strategy test

## Decisions Made
- Moved lib.rs module registration from Task 2 to Task 1 (module must be registered before tests can compile/run)
- Theme uses manual `impl Default` rather than `#[derive(Default)]` because `ratatui::style::Color` does not implement Default
- Added serde_json as dev-dependency (needed for theme_serde_round_trip test)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Moved lib.rs registration to Task 1**
- **Found during:** Task 1 (theme module creation)
- **Issue:** Plan specified lib.rs update in Task 2, but the theme module cannot compile or be tested without `pub mod theme;` in lib.rs
- **Fix:** Registered the module in lib.rs as part of Task 1 commit
- **Files modified:** src/lib.rs
- **Verification:** `cargo test theme --lib` passes all 4 tests
- **Committed in:** 038e2f4 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minimal -- lib.rs registration was moved earlier but content is identical to what Task 2 specified. Task 2 only needed quadrant.rs and prelude.rs changes.

## Issues Encountered
None - straightforward module creation following established patterns.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Theme, RenderContext, LayoutStrategy, and LayoutAreas are defined and tested
- Plan 02 will integrate these into ControlTrait::render signature, ShowcaseApp, and AppSpec builder
- All 105 tests passing, no regressions

---
*Phase: 06-extension-points-and-public-api*
*Completed: 2026-03-28*

## Self-Check: PASSED

- FOUND: src/theme.rs
- FOUND: src/prelude.rs
- FOUND: src/components/quadrant.rs
- FOUND: src/lib.rs
- FOUND: Cargo.toml
- FOUND: 038e2f4 (Task 1 commit)
- FOUND: 9deecc7 (Task 2 commit)
