# Getting Started

## 先跑起来

先运行推荐样板：

```bash
cargo run --example host_template
```

仓库根目录下的：

```bash
cargo run
```

只用于框架仓库自身的默认应用，不作为推荐宿主接入样板。

## 再接入自己的项目

推荐顺序：

1. 复制 [templates/host_project](/Users/bcsy/Desktop/myproject/tui01/templates/host_project)
2. 在 `src/host.rs` 里收紧宿主策略
3. 在 `src/actions.rs` 里注册动作
4. 在 `src/app.rs` 里定义页面
5. 在 `src/main.rs` 里完成 `try_into_showcase_app_with_host(host)` 装配

推荐目录：

```text
your-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── host.rs
│   ├── actions.rs
│   └── app.rs
```

## 推荐写法

宿主代码优先只使用：

- `tui01::prelude`
- `tui01::field`
- `RuntimeHost`

推荐启动顺序：

1. 构建 `RuntimeHost`
2. 通过 `prelude + field` 构建 `AppSpec`
3. `try_into_showcase_app_with_host(host)`
4. 进入事件循环

## 默认建议

- 动作优先用 `registered_action`
- 裸 shell 只用于本地快速原型
- 默认从 `ShellPolicy::RegisteredOnly` 开始
- host 侧至少配置：
  - `project_root` 这类基础 context
  - working dir
  - env 白名单
  - logger
  - event hook

## 接入检查清单

- 页面里所有 `result_target` 都存在且指向日志控件
- 所有 `registered_action` 都已在宿主注册
- host policy 不允许未注册 shell
- host working dir 在白名单内
- host env key 在白名单内
- `AppSpec` 和宿主注册动作已经过 `validate()` / `try_into_showcase_app_with_host(...)` 校验

## 适用场景

适合：

- 内部工具面板
- 配置驱动的运维/开发 TUI
- 需要宿主动作注册和执行约束的命令行工具

当前不建议直接用于：

- 需要复杂表格/树视图的大型终端应用
- 需要稳定公开插件生态的长期平台
- 需要完整国际化和主题系统的产品型 TUI
