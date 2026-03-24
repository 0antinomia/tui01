use tui01::field;
use tui01::prelude::{page, screen, section, AppSpec};

pub fn build_app_spec() -> AppSpec {
    AppSpec::new()
        .title_text("Your App\n\nHost template layout.")
        .status_controls(
            "Controls:\n↑/↓ 或 j/k 当前焦点内移动\nShift+J/K 当前焦点区域翻页\nEnter / l 进入或确认\nEsc / h 返回\nq 退出",
        )
        .screen(screen(
            "Workspace",
            page("Workspace")
                .section(
                    section("基础配置")
                        .field(
                            field::text_id("项目名", "demo", "输入项目名", "project_name"),
                        )
                        .field(
                            field::number_id("端口", "3000", "输入端口", "server_port"),
                        ),
                )
                .section(
                    section("操作")
                        .field(
                            field::refresh_registered_to_log(
                                "同步工作区",
                                "同步",
                                "sync_action",
                                "sync_workspace",
                                "workspace_log",
                            ),
                        )
                        .field(
                            field::log_id("输出", "等待执行结果", "workspace_log")
                                .with_height_units(4),
                        ),
                ),
        ))
}
