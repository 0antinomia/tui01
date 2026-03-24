# Getting Started

## 适用场景

`tui01` 当前适合这类场景：

- 内部工具面板
- 配置驱动的运维/开发 TUI
- 需要宿主动作注册和执行约束的命令行工具

当前不建议直接用于这类场景：

- 需要复杂表格/树视图的大型终端应用
- 需要稳定公开插件生态的长期平台
- 需要完整国际化和主题系统的产品型 TUI

## 最短接入路径

推荐按这个顺序接入：

1. 跑通现成样板  
   使用 [examples/host_template.rs](/Users/bcsy/Desktop/myproject/tui01/examples/host_template.rs)

仓库根目录下的 `cargo run` 只用于框架自身的默认应用，不作为推荐宿主接入样板。

2. 复制宿主工程骨架  
   参考 [templates/host_project/README.md](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/README.md)

3. 先注册动作，再写页面  
   不要先把页面写满，再回头补宿主动作

4. 默认收紧宿主策略  
   建议从 `ShellPolicy::RegisteredOnly` 开始

5. 只用推荐入口  
   宿主代码优先使用 `tui01::prelude` 和 `tui01::field`

## 推荐目录

```text
your-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── host.rs
│   ├── actions.rs
│   └── app.rs
```

## 推荐启动顺序

1. 构建 `RuntimeHost`
2. 通过 `prelude + field` 构建 `AppSpec`
3. `try_into_showcase_app_with_host(host)`
4. 进入事件循环

## 默认建议

- 动作优先用 `registered_action`
- 裸 shell 只用于本地快速原型
- host 侧必须配置：
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
