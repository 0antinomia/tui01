# tui01

`tui01` is a Rust TUI framework for building fixed four-quadrant tools with declarative screens and host-controlled side effects.

The reset-era consumer contract is intentionally small:

- `tui01::prelude`
- `tui01::field`
- `RuntimeHost`

## Start Here

Run the canonical example first:

```bash
cargo run --example host_template
```

That example is the single official runnable path for downstream users:
[examples/host_template.rs](examples/host_template.rs)

## Who It Fits

`tui01` is a good fit when you want:

- an internal operations or developer tool
- a TUI driven by declarative page specs instead of ad hoc widget wiring
- a host boundary that registers actions and constrains shell execution

## Minimal App Shape

Use `tui01::prelude` for the builder flow and `tui01::field` for field helpers:

```rust
use tui01::field;
use tui01::prelude::{AppSpec, page, screen, section};

let app = AppSpec::new().screen(
    screen(
        "Workspace",
        page("Workspace").section(
            section("Config")
                .field(field::text("Project", "demo", "Project name"))
                .field(field::toggle("Enable sync", true)),
        ),
    ),
);
```

## Host Integration

For real applications, attach a `RuntimeHost` and keep side effects behind registered actions:

```rust
use tui01::field;
use tui01::host::ActionOutcome;
use tui01::prelude::{AppSpec, RuntimeHost, ShellPolicy, page, screen, section};

let mut host = RuntimeHost::new();
host.register_action_handler("sync_workspace", |_| async move {
    ActionOutcome::success("workspace synced")
});

host = host.set_shell_policy(ShellPolicy::RegisteredOnly);

let mut app = AppSpec::new()
    .screen(
        screen(
            "Workspace",
            page("Workspace").section(
                section("Actions")
                    .field(field::action("Sync", "Run").with_registered_action("sync_workspace")),
            ),
        ),
    )
    .try_into_showcase_app_with_host(host)?;

app.validate_registered_actions()?;
```

The full end-to-end version of that flow lives in
[examples/host_template.rs](examples/host_template.rs).

## Recommended Flow

1. Run `cargo run --example host_template`.
2. Copy the wiring pattern from `examples/host_template.rs`.
3. Define your screens and fields through `tui01::prelude` and `tui01::field`.
4. Build a `RuntimeHost`, register actions, and attach it with `try_into_showcase_app_with_host(host)`.
5. Validate registered actions before shipping host-backed behavior.

## Further Reading

- Deeper onboarding: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md)
- Upgrading older integrations: [docs/MIGRATION.md](docs/MIGRATION.md)
- Breaking reset notes: [CHANGELOG.md](CHANGELOG.md)
