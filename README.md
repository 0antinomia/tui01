# tui01

`tui01` 是一个基于 `ratatui` 的四分区 TUI 框架。

当前 crate 版本：`0.1.0`

当前结构分成四层：

- `builder`：面向使用的顶层入口
- `schema`：声明式页面/字段定义
- `runtime`：运行时模型与状态
- `components`：纯 UI 渲染与交互

## 当前推荐入口

优先使用 [src/builder.rs](/Users/bcsy/Desktop/myproject/tui01/src/builder.rs) 提供的 `AppSpec`、`page(...)`、`section(...)`、`screen(...)`。

配套文档：

- [docs/GETTING_STARTED.md](/Users/bcsy/Desktop/myproject/tui01/docs/GETTING_STARTED.md)
- [docs/VERSIONING.md](/Users/bcsy/Desktop/myproject/tui01/docs/VERSIONING.md)
- [docs/RELEASE_SCOPE.md](/Users/bcsy/Desktop/myproject/tui01/docs/RELEASE_SCOPE.md)
- [CHANGELOG.md](/Users/bcsy/Desktop/myproject/tui01/CHANGELOG.md)

最小示例：

```rust
use tui01::builder::{page, screen, section, AppSpec};
use tui01::schema::FieldSpec;

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
                    .field(FieldSpec::text_input("项目名", "demo", "输入项目名"))
                    .field(FieldSpec::number_input("端口", "3000", "输入端口"))
                    .field(FieldSpec::toggle("启用缓存", true)),
            )
            .section(
                section("操作")
                    .field(
                        FieldSpec::refresh_button("刷新工作区", "刷新")
                            .with_id("refresh_workspace")
                            .with_result_target("workspace_log")
                            .with_shell_command("printf 'workspace refreshed\\n'"),
                    )
                    .field(
                        FieldSpec::log_output("输出", "等待执行结果")
                            .with_id("workspace_log")
                            .with_height_units(4),
                    ),
            ),
    ))
    .into_showcase_app();
```

## 运行 demo

```bash
cargo run --bin demo
```

默认入口：

```bash
cargo run
```

## 字段类型

当前已经支持：

- 文本输入：`FieldSpec::text_input(...)`
- 数值输入：`FieldSpec::number_input(...)`
- 下拉选择：`FieldSpec::select(...)`
- 开关：`FieldSpec::toggle(...)`
- 动作按钮：`FieldSpec::action_button(...)`
- 刷新按钮：`FieldSpec::refresh_button(...)`
- 静态展示：`FieldSpec::static_data(...)`
- 动态展示：`FieldSpec::dynamic_data(...)`
- 日志输出：`FieldSpec::log_output(...)`

## 操作绑定

字段可以绑定三类操作方式：

- 模拟成功：`with_operation_success(...)`
- 模拟失败：`with_operation_failure(...)`
- 真实 shell 命令：`with_shell_command(...)`

命令输出可以写入某个日志字段：

```rust
FieldSpec::action_button("同步", "执行")
    .with_id("sync_action")
    .with_result_target("sync_log")
    .with_shell_command("printf 'sync ok\\n'")
```

除了 shell 模板，也可以通过独立宿主层 `RuntimeHost` 注册 Rust handler：

```rust
use tui01::executor::ActionOutcome;
use tui01::host::{HostEvent, HostLogLevel, RuntimeHost, ShellPolicy};

let mut host = RuntimeHost::new();
host.register_action_handler("sync_workspace", |context| async move {
    let project = context.params.get("project_name").cloned().unwrap_or_default();
    ActionOutcome::success(format!("synced {project}"))
});
host = host
    .set_context("project_root", "/workspace/demo")
    .set_working_dir("/workspace/demo")
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
                FieldSpec::action_button("同步", "执行")
                    .with_registered_action("sync_workspace"),
            ),
        ),
    ))
    .into_showcase_app_with_host(host);
app.validate_registered_actions()?;
```

这条接口更适合真实项目，因为它不要求把所有执行逻辑都降级成 shell。`RuntimeHost` 现在是推荐的宿主接入点，后续环境信息、日志桥接和外部服务句柄也应该挂在这里。它当前已经能统一承接：

- 动作注册
- 宿主上下文 `context`
- shell 工作目录
- shell 环境变量
- shell 执行策略
- 允许的工作目录白名单
- 允许的环境变量白名单
- 宿主日志钩子
- 宿主事件钩子

`ShowcaseApp` 上仍然保留了 `register_*` 方法，但它们只是兼容入口。

## 外部配置格式

现在已经有一个稳定的外部配置数据形状，位置在 [src/config.rs](/Users/bcsy/Desktop/myproject/tui01/src/config.rs)。

当前支持：

- `AppConfig::from_json_str(...)`
- `AppConfig::from_yaml_str(...)`
- `AppConfig::from_lua_str(...)`
- `AppConfig::from_json_file(...)`
- `AppConfig::from_yaml_file(...)`
- `AppConfig::from_lua_file(...)`
- `AppConfig::from_file(...)`

YAML 示例：

```yaml
title_text: Demo
status_controls: |
  Controls:
  ↑/↓ 或 j/k 移动
  Enter / l 确认
  Esc / h 返回
  q 退出
screens:
  - title: Workspace
    page:
      title: Workspace
      sections:
        - title: Main
          fields:
            - label: 项目名
              control:
                type: text_input
                value: tui01
                placeholder: 输入项目名
            - label: 启用缓存
              control:
                type: toggle
                on: true
        - title: Actions
          fields:
            - label: 刷新
              control:
                type: refresh_button
                button_label: 刷新
              operation:
                kind: shell
                command: printf 'workspace refreshed\n'
                result_target: workspace_log
            - label: 输出
              id: workspace_log
              height_units: 4
              control:
                type: log_output
                content: 等待结果
host_requirements:
  required_context_keys:
    - project_root
  required_registered_actions: []
  required_env_keys:
    - APP_ENV
  requires_working_dir: true
```

直接从文件运行示例：

```bash
cargo run --example from_config
```

指定配置文件：

```bash
cargo run --example from_config -- examples/demo.yaml
```

Lua 示例文件：

```bash
cargo run --example from_config -- examples/demo.lua
```

宿主集成模板：

```bash
cargo run --example host_template
```

对应文件：

- [examples/host_template.rs](/Users/bcsy/Desktop/myproject/tui01/examples/host_template.rs)
- [examples/host_app.yaml](/Users/bcsy/Desktop/myproject/tui01/examples/host_app.yaml)

推荐工程骨架：

- [templates/host_project/README.md](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/README.md)
- [templates/host_project/main.rs](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/main.rs)
- [templates/host_project/host.rs](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/host.rs)
- [templates/host_project/actions.rs](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/actions.rs)
- [templates/host_project/app.yaml](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/app.yaml)

建议的接入顺序是：

1. Rust 原生：`AppSpec`
2. YAML / JSON：`AppConfig`
3. Lua：返回一个与 `AppConfig` 同形状的 table

## 配置校验

现在会在 `AppSpec` 层做统一校验，主要检查：

- 重复字段 `id`
- `result_target` 是否存在
- `result_target` 是否真的指向日志控件
- `registered_action` 是否已经在宿主应用注册

`examples/from_config.rs` 默认会在运行前执行这一步校验。

如果你使用的是运行期注册的 Rust handler，而不是 `AppSpec::shell_action(...)`，则应在 host 或 app 装配完成后调用 `ShowcaseApp::validate_registered_actions()`。更推荐直接使用 `AppSpec::try_into_showcase_app_with_host(...)`。

## 配置与宿主闭环

`AppConfig` 现在支持声明最小宿主需求：

- `required_context_keys`
- `required_registered_actions`
- `required_env_keys`
- `requires_working_dir`

加载顺序建议固定成：

1. 读取 `AppConfig`
2. 构建 `RuntimeHost`
3. 调用 `AppConfig::validate_against_host(&host)`
4. 再调用 `try_into_showcase_app_with_host(...)`

这样配置层只能声明“它需要什么”，最终是否接受仍由宿主决定。

## 当前边界

这个项目现在已经适合直接用 Rust 代码配置页面。

Lua 现在已经有最小接入，但它只是配置层，不直接操作 UI 组件。更合理的顺序仍然是：

1. 先稳定 Rust 侧 `AppSpec/PageSpec/FieldSpec`
2. 再稳定 `AppConfig` 的 JSON/YAML 形状
3. 再让 Lua 映射到 `AppConfig`

## 参数化动作

注册动作现在支持引用当前字段值。做法是：

1. 给字段设置稳定 `id`
2. 在宿主应用里注册动作模板
3. 在模板里使用 `{{field_id}}`、`{{screen.field_id}}` 或 `{{page_slug.field_id}}`

例如：

```rust
FieldSpec::text_input("项目名", "tui01", "输入项目名").with_id("project_name");
FieldSpec::number_input("端口", "3000", "输入端口").with_id("server_port");

AppSpec::new()
    .shell_action(
        "refresh_workspace",
        "printf 'workspace=%s port=%s\\n' {{project_name}} {{server_port}}",
    );
```

提交操作时，框架会把当前字段值和宿主上下文一起替换进模板。

- `{{field_id}}`：默认做 shell 安全转义
- `{{screen.field_id}}`：显式引用当前页面作用域下的字段
- `{{page_slug.field_id}}`：显式引用当前页面的 slug 作用域，例如 `Workspace` 对应 `workspace`
- `{{host.key}}`：引用 `RuntimeHost` 中注入的宿主上下文，例如 `{{host.project_root}}`
- `{{raw:field_id}}`：原样插入，不做转义

默认应优先使用 `{{field_id}}`。只有在模板本身已经明确需要原始 shell 片段，例如固定 flag 组合或你自己完成了转义时，才使用 `{{raw:...}}`。

当操作走 shell 执行时，`RuntimeHost` 里的工作目录和环境变量会自动注入到底层命令执行环境；当操作走 Rust handler 时，这些值也会通过 `ActionContext.cwd` 和 `ActionContext.env` 一起传入。

如果你要把框架放进真实应用环境，建议默认把 shell 策略设成 `ShellPolicy::RegisteredOnly`。这样页面里不能直接跑裸 shell，只能走宿主明确注册过的动作名。

进一步收紧时，建议同时配置：

- `allow_working_dir(...)`
- `allow_env_key(...)`

这样即使动作本身被允许执行，底层 shell 也只能在你明确批准的目录和环境变量范围内运行。

如果你的应用已经有自己的日志系统，优先接 `RuntimeHost::on_log(...)`。它比事件钩子更适合直接落日志；事件钩子更适合做监控、通知或额外联动。
