# Changelog

## 0.3.0

Breaking reset release for downstream users adopting the new public contract. This affects teams upgrading older integrations that depended on compatibility aliases, root event/tui entry paths, or the bare `cargo run` flow. The supported consumer surface is now centered on `tui01::prelude`, `tui01::field`, and `RuntimeHost`. Use [docs/MIGRATION.md](docs/MIGRATION.md) before upgrading existing integrations.

### Breaking

- Removed compatibility aliases such as `tui01::builder` from the public guidance path. Import canonical builder helpers through `tui01::prelude` instead.
- Removed legacy root-level public entry paths such as `tui01::event` and `tui01::tui` from the supported consumer contract.
- Removed bare `cargo run` as the recommended entry path. Run `cargo run --example host_template` for the canonical reset-era example.
- Older integrations should migrate to `tui01::prelude`, `tui01::field`, and `RuntimeHost` with the old-to-new mappings in [docs/MIGRATION.md](docs/MIGRATION.md).

### Changed

- Public onboarding and versioning docs now route upgrade work through [docs/MIGRATION.md](docs/MIGRATION.md) and keep the canonical example as the only official runnable entrypoint.
- Release-facing guidance now documents the `0.3.0` reset as a breaking `0.x` contract update instead of a transitional compatibility story or a `1.0.0` stabilization promise.

## 0.1.0

首个可接入版本，当前范围包括：

- 四分区 TUI 壳层
- 菜单、内容区、日志区等基础组件
- `AppSpec` Rust 原生入口
- `RuntimeHost` 宿主接入层
- 已注册动作、shell 策略、cwd/env 白名单
- 宿主 logger / event hook
- 宿主模板工程与可运行 example
