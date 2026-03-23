# Host Project Template

这个目录是一套推荐的宿主工程骨架，不参与 Cargo example 编译。

推荐目录结构：

```text
your-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── host.rs
│   └── actions.rs
└── tui/
    └── app.yaml
```

职责划分：

- `src/main.rs`
  负责启动 TUI、加载配置、构建 `RuntimeHost`、运行事件循环
- `src/host.rs`
  负责宿主策略、日志桥、事件桥、工作目录和环境变量白名单
- `src/actions.rs`
  负责注册具体业务动作，不把业务逻辑散落在 `main.rs`
- `tui/app.yaml`
  负责页面结构和宿主需求声明

参考文件：

- [main.rs](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/main.rs)
- [host.rs](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/host.rs)
- [actions.rs](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/actions.rs)
- [app.yaml](/Users/bcsy/Desktop/myproject/tui01/templates/host_project/app.yaml)
