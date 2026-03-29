# Migration Guide

## Who Should Read This

This guide is for teams upgrading older `tui01` integrations to the reset-era public contract. If you are starting fresh, use the newcomer-first README, `docs/GETTING_STARTED.md`, and the canonical runnable example instead.

## Canonical Reset Contract

The supported public onboarding surface is `tui01::prelude`, `tui01::field`, and `RuntimeHost`.

The single official runnable example is `examples/host_template.rs`:

```bash
cargo run --example host_template
```

Use that example and the reset-era docs as the source of truth for new integrations.

## Old To New Mappings

| Old usage | New usage | Notes |
|-----------|-----------|-------|
| `tui01::builder::AppSpec` | `tui01::prelude::AppSpec` | Builder-facing app spec imports now come from the canonical prelude. |
| Direct builder helpers from `tui01::spec` | `tui01::prelude::{page, screen, section}` | Prefer the prelude entrypoint instead of direct builder helper imports from `spec`. |
| `cargo run` | `cargo run --example host_template` | The default binary is no longer the public entrypoint; use the canonical example. |
| README or onboarding references to internal module tours | Newcomer-first docs plus `examples/host_template.rs` | Reset-era guidance teaches the consumer contract first and treats internal module layout as non-onboarding material. |

## Removed Paths And Behaviors

- `tui01::event` was removed from public guidance and has no reset-era replacement as a supported consumer entry path.
- `tui01::tui` was removed from public guidance and is no longer part of the supported consumer contract.
- The default binary is removed as the recommended public entry path; run `cargo run --example host_template` instead.
- Root compatibility aliases such as `tui01::builder` were removed from the public onboarding story; use `tui01::prelude` and `tui01::field`.
