# tui01

`tui01` 是一个基于 `ratatui` 的四分区 TUI 框架，当前版本为 `0.1.0`。

它现在最适合用来做这类程序：

- 内部工具面板
- 配置驱动的运维/开发 TUI
- 需要宿主动作注册和执行约束的命令行工具

## 开箱即用

先直接运行可用样板：

```bash
cargo run --example host_template
```

这个 example 就是当前推荐的接入方式，不是玩具 demo。

仓库根目录下的默认入口也可以运行：

```bash
cargo run
```

但它主要用于框架仓库自身的 smoke test 和界面观察，不作为推荐宿主样板。

## 最短接入路径

推荐按这个顺序接入：

1. 跑通 [examples/host_template.rs](/Users/bcsy/Desktop/myproject/tui01/examples/host_template.rs)
2. 复制 [templates/host_project](/Users/bcsy/Desktop/myproject/tui01/templates/host_project) 到你的项目
3. 在 `src/actions.rs` 里先注册动作
4. 在 `src/app.rs` 里再写页面
5. 用 `try_into_showcase_app_with_host(host)` 完成装配

宿主代码推荐只直接使用：

- `tui01::prelude`
- `tui01::field`
- `tui01::host::RuntimeHost`

## 最小示例

```rust
use tui01::field;
use tui01::prelude::{page, screen, section, AppSpec};

let app = AppSpec::new()
    .title_text("My TUI\n\n最小示例")
    .status_controls(
        "Controls:\n↑/↓ 或 j/k 移动\nEnter / l 进入或确认\nEsc / h 返回\nq 退出",
    )
    .screen(screen(
        "Workspace",
        page("Workspace")
            .section(
                section("基础配置")
                    .field(field::text("项目名", "demo", "输入项目名"))
                    .field(field::number("端口", "3000", "输入端口"))
                    .field(field::toggle("启用缓存", true)),
            )
            .section(
                section("操作")
                    .field(
                        field::refresh_to_log(
                            "刷新工作区",
                            "刷新",
                            "refresh_workspace",
                            "workspace_log",
                        )
                        .with_shell_command("printf 'workspace refreshed\\n'"),
                    )
                    .field(
                        field::log_id("输出", "等待执行结果", "workspace_log")
                            .with_height_units(4),
                    ),
            ),
    ))
    .into_showcase_app();
```

## 宿主接入

真实项目里更推荐通过 `RuntimeHost` 注册动作，而不是把所有行为都写成 shell 字符串：

```rust
use tui01::executor::ActionOutcome;
use tui01::field;
use tui01::prelude::{
    page, screen, section, AppSpec, HostEvent, HostLogLevel, RuntimeHost, ShellPolicy,
};

let mut host = RuntimeHost::new();
host.register_action_handler("sync_workspace", |context| async move {
    let project = context.params.get("project_name").cloned().unwrap_or_default();
    ActionOutcome::success(format!("synced {project}"))
});

host = host
    .set_context("project_root", "/workspace/demo")
    .set_working_dir("/workspace/demo")
    .set_framework_log_enabled(true)
    .set_framework_log_path("/workspace/demo/.tui01/logs/framework.log")
    .allow_working_dir("/workspace/demo")
    .insert_env("APP_ENV", "dev")
    .allow_env_key("APP_ENV")
    .set_shell_policy(ShellPolicy::RegisteredOnly)
    .on_log(|record| match record.level {
        HostLogLevel::Info => eprintln!("[info] {}", record.message),
        HostLogLevel::Warn => eprintln!("[warn] {}", record.message),
        HostLogLevel::Error => eprintln!("[error] {}", record.message),
        HostLogLevel::Debug => eprintln!("[debug] {}", record.message),
    })
    .on_event(|event| match event {
        HostEvent::OperationStarted { source, .. } => eprintln!("started: {source}"),
        HostEvent::OperationFinished { source, success, .. } => {
            eprintln!("finished: {source} success={success}")
        }
    });

let mut app = AppSpec::new()
    .screen(screen(
        "Workspace",
        page("Workspace").section(
            section("操作").field(
                field::action("同步", "执行").with_registered_action("sync_workspace"),
            ),
        ),
    ))
    .try_into_showcase_app_with_host(host)?;

app.validate_registered_actions()?;
```

## 常用字段

当前已经支持：

- 文本输入：`field::text(...)`
- 数值输入：`field::number(...)`
- 下拉选择：`field::select(...)`
- 开关：`field::toggle(...)`
- 动作按钮：`field::action(...)`
- 刷新按钮：`field::refresh(...)`
- 静态展示：`field::static_value(...)`
- 动态展示：`field::dynamic_value(...)`
- 日志输出：`field::log(...)`
- 文件日志输出：`field::log_file(...)`

常用组合助手：

- `field::text_id(...)`
- `field::number_id(...)`
- `field::action_to_log(...)`
- `field::action_registered_to_log(...)`
- `field::refresh_to_log(...)`
- `field::refresh_registered_to_log(...)`
- `field::log_id(...)`
- `field::log_file_tail(...)`

## 执行与日志

字段可以绑定三类操作方式：

- 模拟成功：`with_operation_success(...)`
- 模拟失败：`with_operation_failure(...)`
- 真实 shell 命令：`with_shell_command(...)`

命令输出可以回写到某个日志字段：

```rust
field::action_to_log("同步", "执行", "sync_action", "sync_log")
    .with_shell_command("printf 'sync ok\\n'")
```

框架日志和日志控件是分开的：

- 框架运行日志：由框架自己写文件，可通过 `RuntimeHost` 配置开关和路径
- 日志控件：只是一个 UI 组件，可以显示任意日志内容，也可以直接读取日志文件

## 参数化动作

注册动作支持引用当前字段值。常用写法：

- `{{field_id}}`
- `{{screen.field_id}}`
- `{{page_slug.field_id}}`
- `{{host.key}}`

默认会做 shell 安全转义。只有明确需要原始片段时才使用 `{{raw:field_id}}`。

## 校验与边界

框架在 `AppSpec` 层会统一检查：

- 重复字段 `id`
- `result_target` 是否存在
- `result_target` 是否真的指向日志控件
- `registered_action` 是否已经在宿主应用注册

当前项目明确以 Rust 原生配置为主路径，不提供 YAML / JSON / Lua 配置入口。

## 进一步阅读

- [docs/GETTING_STARTED.md](/Users/bcsy/Desktop/myproject/tui01/docs/GETTING_STARTED.md)
- [templates/host_project/README.md](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/README.md)
- [docs/VERSIONING.md](/Users/bcsy/Desktop/myproject/tui01/docs/VERSIONING.md)
- [docs/RELEASE_SCOPE.md](/Users/bcsy/Desktop/myproject/tui01/docs/RELEASE_SCOPE.md)
- [CHANGELOG.md](/Users/bcsy/Desktop/myproject/tui01/CHANGELOG.md)
