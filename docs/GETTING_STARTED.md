# 快速开始

## 先运行官方示例

先从唯一的官方可运行示例开始：

```bash
cargo run --example host_template
```

当前推荐的接入方式，以 [../examples/host_template.rs](../examples/host_template.rs) 为准。

## 推荐的项目结构

真实项目中，建议把接入代码拆成比较清晰的几层：

```text
project root
├── Cargo.toml
└── src
    ├── app.rs
    ├── actions.rs
    ├── host.rs
    └── main.rs
```

- `actions.rs`：动作名称和业务逻辑
- `host.rs`：`RuntimeHost` 构造、策略、hook 和 allowlist
- `app.rs`：`AppSpec`、页面、分区和字段定义
- `main.rs`：把各部分组装起来并启动运行逻辑

## 第一步：先构建 RuntimeHost

建议先把 `RuntimeHost` 搭出来，再去定义会触发副作用的字段，这样执行边界会更清楚。

推荐默认做法：

- 优先使用 `registered_action`，不要直接暴露原始 shell 执行
- 先从 `ShellPolicy::RegisteredOnly` 开始
- 明确设置工作目录
- 真正启用命令前先收紧环境变量白名单
- 在有副作用的场景里尽早挂上 logger 和 event hook

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

## 第二步：用 Prelude 和 Field 定义 AppSpec

应用定义建议始终围绕 `tui01::prelude` 和 `tui01::field` 来写：

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

## 第三步：挂接 Host

当 `RuntimeHost` 和 `AppSpec` 都准备好后，用 `try_into_showcase_app_with_host(host)` 把它们连起来：

```rust
let host = build_host();
let mut app = build_spec().try_into_showcase_app_with_host(host)?;
```

## 第四步：运行前校验注册动作

进入真实运行前，先确认 spec 中声明的 `registered_action` 都已经在 host 上注册：

```rust
app.validate_registered_actions()?;
```

这样可以在用户真正触发动作之前，就提前发现遗漏。

## 第五步：从示例过渡到真实项目

把示例替换成真实业务逻辑时，建议保持这些原则：

- 让 `registered_action` 成为默认执行路径
- 把 cwd 和 env allowlist 收紧到动作真正需要的范围
- 保留宿主日志，便于定位接入阶段的问题
- 在验证行为是否符合预期时，继续保留 event hook
