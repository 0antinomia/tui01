# tui01

`tui01` 是一个基于 `ratatui` 的四分区 TUI 框架，当前版本为 `0.2.0`。

tui01 让你能更快地从 0 到 1 搭起一个可运行、可交互、可接入宿主逻辑的 TUI 工具。

它适合用来做这类程序：

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

## 最短接入路径

推荐按这个顺序接入：

1. 跑通 [examples/host_template.rs](examples/host_template.rs)
2. 复制 `examples/host_template.rs` 的装配方式到你的项目
3. 在 `src/actions.rs` 里先注册动作
4. 在 `src/app.rs` 里再写页面
5. 用 `try_into_showcase_app_with_host(host)` 完成装配

宿主代码推荐只直接使用：

- `tui01::prelude`
- `tui01::field`
- `tui01::host::RuntimeHost`

进阶能力再按需引入：

- `Theme`
- `LayoutStrategy`
- `ControlRegistry`

## 项目结构

当前源码按职责拆成几个域模块：

- `spec/`
  页面、分区、字段的声明式定义
- `runtime/`
  页面物化后的运行时数据和状态
- `controls/`
  内置控件、自定义控件抽象和统一控件接口
- `components/`
  四分区中的菜单、内容区、标题区、状态区等组件
- `host/`
  宿主接入、动作注册、执行器、框架日志
- `app/`
  应用壳层和主事件流
- `infra/`
  终端生命周期与事件采集

对宿主项目来说，通常只需要直接使用 `prelude + field + RuntimeHost`。

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

## 自定义控件

框架现在支持宿主应用注册自定义控件。

使用方式：

1. 在 `RuntimeHost` 上注册控件工厂
2. 在页面中用 `field::custom("标签", "控件名")` 引用

这适合：

- 业务专用输入控件
- 自定义展示控件
- 希望在多个页面复用的交互组件

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

## 主题与布局

框架已经提供两类扩展点：

- `Theme`
  用于统一边框、文字、选中态、激活态、成功/失败等语义颜色
- `LayoutStrategy`
  用于替换默认四分区布局的区域计算逻辑

如果只是直接接入业务页面，这两项不是必需项；只有在你需要统一视觉风格或替换整体布局时再引入。

## 参数化动作

注册动作支持引用当前字段值。常用写法：

- `{{field_id}}`
- `{{screen.field_id}}`
- `{{page_slug.field_id}}`
- `{{host.key}}`

默认会做 shell 安全转义。只有明确需要原始片段时才使用 `{{raw:field_id}}`。

## 校验

框架在 `AppSpec` 层会统一检查：

- 重复字段 `id`
- `result_target` 是否存在
- `result_target` 是否真的指向日志控件
- `registered_action` 是否已经在宿主应用注册

## 兼容路径

当前仍然保留了一层旧路径兼容别名，例如：

- `tui01::builder`
- `tui01::schema`
- `tui01::field`
- `tui01::showcase`
- `tui01::event`
- `tui01::tui`

新项目仍建议优先使用 `prelude`、`field` 和 `RuntimeHost`。

## 进一步阅读

- [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md)
- [docs/VERSIONING.md](docs/VERSIONING.md)
- [docs/RELEASE_SCOPE.md](docs/RELEASE_SCOPE.md)
- [CHANGELOG.md](CHANGELOG.md)

## 许可证

本项目采用 `MIT OR Apache-2.0` 双证书。
