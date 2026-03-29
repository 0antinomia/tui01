# Versioning

## 当前阶段

当前 crate 版本是 `0.3.0`。这代表：

- 已经具备可接入的最小宿主能力
- 已经完成一轮明确的 breaking reset，模块边界和对外接入路径比早期版本清楚得多
- API 仍然可能调整
- 不承诺在 `0.x` 阶段保持强兼容

## 兼容性原则

`0.x` 阶段按这条规则看待：

- 小版本可能带破坏性调整
- patch 版本应优先只修复问题
- 当前 `0.3.0` reset 以一个规范化消费者路径为准，不再承诺为旧入口保留过渡层

## 当前优先级

版本策略现在优先保证：

1. 宿主接入路径清晰
2. 配置层和宿主层边界稳定
3. 动作执行链可预测
4. 自定义控件、主题和布局扩展点可持续演进

这里的“宿主接入路径”特指：

- `tui01::prelude`
- `tui01::field`
- `RuntimeHost`

`0.3.0` 的发布语义是显式 breaking `0.x` reset，不表示已经进入稳定 API 承诺阶段。

暂时不优先保证：

- 宽范围 UI 组件 API 完全稳定
- 所有内部模块路径长期不变
- `spec / runtime / controls / components / host / app / infra` 的细节长期不变
- 历史兼容别名路径长期不变

## 对外建议

如果你准备在真实项目里依赖 `tui01`：

- 固定 crate 版本
- 升级旧集成前先读 [docs/MIGRATION.md](./MIGRATION.md)
- 关注 breaking reset 说明时，再读 [CHANGELOG.md](../CHANGELOG.md)
- 避免依赖未在 README 文档化的内部模块细节
- 优先使用 `tui01::prelude`、`tui01::field`、`RuntimeHost`，不要直接绑定内部目录结构

## 升级策略

- 这次 reset 明确偏向一个 canonical consumer path，而不是维持多条历史入口并行。
- `0.3.0` 仍处于 `0.x` 阶段，因此 minor bump 本身就可能承载 breaking 变更。
- 旧版本接入如果使用过兼容别名、根级事件入口或默认二进制启动方式，升级前应先对照 [docs/MIGRATION.md](./MIGRATION.md) 完成路径替换。
- 版本级 breaking 说明统一记录在 [CHANGELOG.md](../CHANGELOG.md)。
