# Roadmap: tui01 Architecture Refactoring

## Overview

A 6-phase pure architecture refactoring of a Rust TUI framework. The journey introduces a ControlTrait abstraction (Phase 1), splits controls into individual files and unifies mirrored enums (Phases 2-3), reorganizes the module hierarchy and splits large files (Phases 4-5), and finishes with extension point seams for themes, layout, and plugins (Phase 6). All 87 existing tests must pass at every phase boundary -- no behavior changes, only structural improvement.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Control Trait Extraction** - Define ControlTrait and implement for all 8 built-in controls without changing behavior
- [ ] **Phase 2: Control Isolation and Enum Unification** - Split controls.rs into per-type files, introduce BuiltinControl enum replacing 5 mirrored enums
- [ ] **Phase 3: Custom Control Extension** - Add ControlRegistry and Box<dyn ControlTrait> support for host-app-defined controls
- [ ] **Phase 4: Module Hierarchy Restructuring** - Reorganize src/ into domain-aligned submodules with clean re-exports
- [ ] **Phase 5: Large File Decomposition** - Split content_panel.rs, executor.rs, and showcase.rs into focused sub-modules
- [ ] **Phase 6: Extension Points and Public API** - Add Theme struct, LayoutStrategy trait, RenderContext, and finalize public API surface

## Phase Details

### Phase 1: Control Trait Extraction
**Goal**: Every built-in control has a uniform trait interface, enabling polymorphic dispatch without changing any rendering or interaction behavior
**Depends on**: Nothing (first phase)
**Requirements**: CTRL-01, CTRL-02, CTRL-07
**Success Criteria** (what must be TRUE):
  1. ControlTrait is defined with methods: render, handle_key, value, validate, preferred_width, is_editable, triggers_on_activate
  2. All 8 built-in control types (TextInput, NumberInput, Select, Toggle, Action, Refresh, Log, Display) implement ControlTrait
  3. All 87+ existing tests pass without modification -- no behavior change
  4. Existing dispatch routes through trait methods (immediate migration per D-10)
**Plans**: 2 plans

Plans:
- [x] 01-01-PLAN.md -- Define ControlTrait and implement for all 8 controls
- [x] 01-02-PLAN.md -- Migrate dispatch sites to use trait methods

### Phase 2: Control Isolation and Enum Unification
**Goal**: Each control type lives in its own file with co-located trait logic, and a single BuiltinControl enum replaces the 5 scattered mirrored enums
**Depends on**: Phase 1
**Requirements**: CTRL-03, CTRL-04, CTRL-05, MOD-03
**Success Criteria** (what must be TRUE):
  1. controls.rs (1008 lines) is replaced by individual files per control type (text_input.rs, select.rs, toggle.rs, etc.), each under 200 lines
  2. BuiltinControl enum exists as a single exhaustive enum replacing ControlKind, ContentControl (builtin variants), RuntimeControl, and SelectedControlKind
  3. Adding a new built-in control type requires changes to exactly 3 files: the control's impl file, BuiltinControl enum definition, and the materialization mapping
  4. All 87+ tests pass; rendering and interaction behavior is unchanged
  5. ControlSpec remains separate (it serves the specification layer) -- not merged with BuiltinControl
**Plans**: 2 plans

Plans:
- [x] 02-01-PLAN.md -- Split controls.rs into controls/ subdirectory with per-type files
- [x] 02-02-PLAN.md -- Replace 4 mirrored enums with BuiltinControl, eliminate RuntimeControl

### Phase 3: Custom Control Extension
**Goal**: Host applications can register custom controls that integrate with the framework's rendering and interaction pipeline
**Depends on**: Phase 2
**Requirements**: CTRL-06
**Success Criteria** (what must be TRUE):
  1. ControlRegistry maps string names to Box<dyn ControlTrait> factories
  2. AnyControl enum has Custom(Box<dyn ControlTrait>) variant alongside Builtin(BuiltinControl)
  3. A registered custom control can render and handle key events through the same pipeline as built-in controls
  4. All 95+ tests pass; existing built-in control behavior is unaffected
**Plans**: 2 plans

Plans:
- [x] 03-01-PLAN.md -- Extend ControlTrait with cloneable-trait-object methods, define AnyControl enum and ControlRegistry
- [ ] 03-02-PLAN.md -- Migrate pipeline from BuiltinControl to AnyControl, add custom control declaration API

### Phase 4: Module Hierarchy Restructuring
**Goal**: Source files are organized into domain-aligned submodules with clear boundaries and stable re-export entry points
**Depends on**: Phase 3
**Requirements**: MOD-01, MOD-06
**Success Criteria** (what must be TRUE):
  1. src/ is organized into subdirectories: spec/, runtime/, controls/, components/, host/, app/, infra/ (or equivalent domain-aligned structure)
  2. lib.rs provides stable pub mod + re-export entry points; downstream code imports work without reaching into internal module paths
  3. All 87+ tests pass after reorganization
  4. No file remains at src/ top level except lib.rs (and main.rs if applicable); all domain code lives in submodules
**Plans**: TBD

### Phase 5: Large File Decomposition
**Goal**: No source file exceeds 300 lines; each module has a single, focused responsibility
**Depends on**: Phase 4
**Requirements**: MOD-02, MOD-04, MOD-05
**Success Criteria** (what must be TRUE):
  1. content_panel.rs (1526 lines) is split into focused sub-modules (layout, render, interaction, operation_flow) -- dual-state synchronization is protected by debug assertions and a facade
  2. executor.rs (870 lines) is split into executor core, action registry, shell command, and template rendering modules
  3. showcase.rs (813 lines) is split with screen management and operation polling extracted into separate sub-modules
  4. No file in the codebase exceeds 300 lines
  5. All 87+ tests pass; no async behavior change (tokio spawn + mpsc logic remains identical)
**Plans**: TBD

### Phase 6: Extension Points and Public API
**Goal**: Theme, layout strategy, and render context seams are in place for future extensibility, and the public API is finalized
**Depends on**: Phase 5
**Requirements**: MOD-07, EXT-01, EXT-02, EXT-03, EXT-04
**Success Criteria** (what must be TRUE):
  1. Theme struct exists with semantic slots (border, selected, active, error, success) and Serde derive
  2. LayoutStrategy trait is defined; the current four-quadrant layout implements it as the default strategy
  3. RenderContext struct carries Theme and Layout information through the render path, parameterizing ControlTrait render calls
  4. AppSpec builder supports chain-configured Theme and LayoutStrategy
  5. Public API is semantically equivalent to the original but structurally improved; all 87+ tests pass
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Control Trait Extraction | 2/2 | Complete | 2026-03-28 |
| 2. Control Isolation and Enum Unification | 2/2 | Complete | 2026-03-28 |
| 3. Custom Control Extension | 1/2 | In progress | - |
| 4. Module Hierarchy Restructuring | 0/? | Not started | - |
| 5. Large File Decomposition | 0/? | Not started | - |
| 6. Extension Points and Public API | 0/? | Not started | - |
