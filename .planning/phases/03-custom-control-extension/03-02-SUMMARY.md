---
phase: 03-custom-control-extension
plan: 02
status: complete
started: "2026-03-28"
completed: "2026-03-28"
---

## Objective

Migrate pipeline from BuiltinControl to AnyControl, add custom control declaration API,  
wire registry through materialization.

## Changes Made

### Task 1: Migrate runtime and content_panel from BuiltinControl to AnyControl

- **src/runtime.rs**: ContentBlock.control, RuntimeFieldState.control,  
  OperationStatus::Running fields changed from BuiltinControl to AnyControl.
  All factory methods wrap in AnyControl::Builtin(). From impls updated.
- **src/components/content_panel.rs**: render_control, handle_control_key.  
  control_value changed to use AnyControl dispatch. SelectedControlKind::Custom added.
- **src/builder.rs**: is_log check updated to AnyControl::Builtin(BuiltinControl::LogOutput(_)).
- **src/showcase.rs**: Test match arms updated for AnyControl.

### Task 2: Custom control declaration API and registry threading
- **src/runtime.rs**: RuntimeControl::Custom variant. from_runtime_field(value.  
  registry) method. from_runtime_section and from_runtime_page registry-aware methods.  
  From impls delegate to from_runtime_* methods.
- **src/schema.rs**: ControlSpec::Custom variant. FieldSpec::custom() constructor.  
  materialize handles Custom.
- **src/field.rs**: custom() factory function.
- **src/showcase.rs**: ShowcaseScreen::from_page_with_registry for custom controls.
- **src/builder.rs**: AppSpec::pending_pages field for deferred materialization.  
  page() stores (title, pagespec) for later materialization with registry.
- **src/prelude.rs**: ControlTrait and ControlRegistry exported.
- **src/field.rs**: custom() factory function added.

## Key Decisions
- From<RuntimeField> preserved for backward compat. delegates to from_runtime_field(value, None).
- RuntimeControl::Custom panics at conversion time if no registry provided - intentional fail-fast.
- AppSpec stores pending pages as (title, PageSpec) tuples and materializes lazily with registry access.
- SelectedControlKind::Custom branch uses trait method queries for editability/triggers.

## Test Results
- 100 tests passing (95 original + 3 from Plan 01 + 2 from Plan 02)
- custom_control_panics_without_registry confirms fail-fast behavior
- custom_factory_creates_field_with_control_name confirms factory output

## key-files
### Modified
- src/runtime.rs
- src/components/content_panel.rs
- src/builder.rs
- src/showcase.rs
- src/schema.rs
- src/field.rs
- src/prelude.rs
