# Getting Started

## Run The Canonical Example

Start with the one official runnable example:

```bash
cargo run --example host_template
```

Use [../examples/host_template.rs](../examples/host_template.rs) as the source of truth for the reset-era integration pattern.

## Recommended Downstream Layout

For a real application, keep the wiring compact and explicit:

```text
project root
├── Cargo.toml
└── src
    ├── app.rs
    ├── actions.rs
    ├── host.rs
    └── main.rs
```

- `actions.rs`: action names and business logic
- `host.rs`: `RuntimeHost` construction, policy, hooks, and allowlists
- `app.rs`: `AppSpec` screens, pages, sections, and fields
- `main.rs`: assemble the app and start your runtime

## Step 1: Build RuntimeHost First

Create `RuntimeHost` before authoring side-effectful fields so the allowed execution surface is clear.

Recommended defaults:

- prefer `registered_action` over raw shell commands
- start from `ShellPolicy::RegisteredOnly`
- set a working directory explicitly
- add env allowlists before enabling real commands
- attach logger and event hooks before shipping side effects

```rust
use tui01::host::ActionOutcome;
use tui01::prelude::{HostLogLevel, RuntimeHost, ShellPolicy};

fn build_host() -> RuntimeHost {
    let mut host = RuntimeHost::new();
    host.register_action_handler("sync_workspace", |context| async move {
        let project = context
            .params
            .get("project_name")
            .cloned()
            .unwrap_or_else(|| "demo".to_string());
        ActionOutcome::success(format!("synced {project}"))
    });

    let mut host = host
        .set_context("project_root", ".")
        .set_working_dir(".")
        .allow_working_dir(".")
        .insert_env("APP_ENV", "dev")
        .allow_env_key("APP_ENV")
        .set_shell_policy(ShellPolicy::RegisteredOnly);

    host.set_logger(|record| {
        let level = match record.level {
            HostLogLevel::Debug => "debug",
            HostLogLevel::Info => "info",
            HostLogLevel::Warn => "warn",
            HostLogLevel::Error => "error",
        };
        eprintln!("[{level}] {}", record.message);
    });
    host.set_event_hook(|event| eprintln!("{event:?}"));

    host
}
```

## Step 2: Define AppSpec With Prelude + Field

Keep the app definition on the canonical consumer surface: `tui01::prelude` and `tui01::field`.

```rust
use tui01::field;
use tui01::prelude::{AppSpec, page, screen, section};

fn build_spec() -> AppSpec {
    AppSpec::new().screen(
        screen(
            "Workspace",
            page("Workspace")
                .section(
                    section("Config")
                        .field(field::text_id("Project", "demo", "Project name", "project_name")),
                )
                .section(
                    section("Actions").field(
                        field::refresh_registered_to_log(
                            "Sync workspace",
                            "Sync",
                            "sync_action",
                            "sync_workspace",
                            "workspace_log",
                        ),
                    ),
                )
                .section(
                    section("Output").field(
                        field::log_id("Output", "Waiting for results", "workspace_log"),
                    ),
                ),
        ),
    )
}
```

## Step 3: Attach The Host

Once your `RuntimeHost` and `AppSpec` are ready, attach them with `try_into_showcase_app_with_host(host)`.

```rust
let host = build_host();
let mut app = build_spec().try_into_showcase_app_with_host(host)?;
```

## Step 4: Validate Registered Actions Before Running

Before entering your runtime loop, verify that every `registered_action` used in the spec exists on the host:

```rust
app.validate_registered_actions()?;
```

This catches missing host registrations before users trigger real behavior.

## Step 5: Move From Example To Production

As you replace the example logic with real work:

- keep `registered_action` as the primary execution path
- narrow cwd and env allowlists to what the action needs
- log host activity so failures are visible during integration
- keep event hooks wired while you validate operational behavior

## Upgrading Older Integrations

If you are migrating older `tui01` code, leave the newcomer path here and switch to [docs/MIGRATION.md](MIGRATION.md) instead of mixing upgrade work into this flow.
