# Host Project Template

这个目录是一套推荐的宿主工程骨架，不参与 Cargo example 编译。

## 使用方式

1. 把这个目录里的 `src/` 拷到你的项目
2. 先修改 `src/host.rs`
3. 再修改 `src/actions.rs`
4. 最后调整 `src/app.rs`

建议不要一开始就直接改 `main.rs`，先把宿主和动作接通。

## 目录结构

```text
your-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── host.rs
│   ├── actions.rs
│   └── app.rs
```

## 文件职责

- `src/main.rs`
  负责启动 TUI、构建 `RuntimeHost`、运行事件循环
- `src/host.rs`
  负责宿主策略、日志桥、事件桥、工作目录和环境变量白名单
- `src/actions.rs`
  负责注册具体业务动作，不把业务逻辑散落在 `main.rs`
- `src/app.rs`
  负责页面结构和 `AppSpec` 定义

## 推荐修改顺序

1. `src/host.rs`
2. `src/actions.rs`
3. `src/app.rs`
4. `src/main.rs`

## 参考文件

- [main.rs](main.rs)
- [host.rs](host.rs)
- [actions.rs](actions.rs)
- [app.rs](app.rs)
