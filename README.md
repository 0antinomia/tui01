# tui01

`tui01` 是一个 Rust TUI 框架，用来构建固定四分区布局、声明式页面结构、以及由宿主控制副作用执行的工具型应用。

当前建议直接围绕下面这三个公开入口使用：

- `tui01::prelude`
- `tui01::field`
- `RuntimeHost`

## 从这里开始

先运行官方示例：

```bash
cargo run --example host_template
```

这个示例就是当前最直接、也最推荐参考的接入路径：
[examples/host_template.rs](examples/host_template.rs)

## 适合什么场景

`tui01` 适合以下类型的项目：

- 内部运维工具或开发者工具
- 希望用声明式页面结构组织 TUI，而不是手写大量 widget 拼装逻辑
- 需要由宿主统一注册动作、限制 shell 执行边界的应用

## 最小应用结构

页面定义建议通过 `tui01::prelude` 完成，字段辅助函数放在 `tui01::field`：

```rust
use tui01::field;
use tui01::prelude::{AppSpec, page, screen, section};

let app = AppSpec::new().screen(
    screen(
        "Workspace",
        page("Workspace").section(
            section("Config")
                .field(field::text("Project", "demo", "项目名称"))
                .field(field::toggle("启用同步", true)),
        ),
    ),
);
```

## 宿主接入方式

真实应用里，建议把所有副作用都放到 `RuntimeHost` 注册动作后面：

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

完整可运行版本可直接看：
[examples/host_template.rs](examples/host_template.rs)

## 推荐接入顺序

1. 先运行 `cargo run --example host_template`
2. 按示例中的结构复制接入方式
3. 用 `tui01::prelude` 和 `tui01::field` 定义页面与字段
4. 构建 `RuntimeHost`，注册动作，再通过 `try_into_showcase_app_with_host(host)` 挂接
5. 在进入真实运行逻辑前执行 `validate_registered_actions()`

## 继续阅读

- 更完整的接入说明：[docs/GETTING_STARTED.md](docs/GETTING_STARTED.md)
- 当前版本策略说明：[docs/VERSIONING.md](docs/VERSIONING.md)
