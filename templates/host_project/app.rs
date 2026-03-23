use tui01::builder::{page, screen, section, AppSpec};
use tui01::schema::FieldSpec;

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
                            FieldSpec::text_input("项目名", "demo", "输入项目名")
                                .with_id("project_name"),
                        )
                        .field(
                            FieldSpec::number_input("端口", "3000", "输入端口")
                                .with_id("server_port"),
                        ),
                )
                .section(
                    section("操作")
                        .field(
                            FieldSpec::refresh_button("同步工作区", "同步")
                                .with_id("sync_action")
                                .with_registered_action("sync_workspace")
                                .with_result_target("workspace_log"),
                        )
                        .field(
                            FieldSpec::log_output("输出", "等待执行结果")
                                .with_id("workspace_log")
                                .with_height_units(4),
                        ),
                ),
        ))
}
