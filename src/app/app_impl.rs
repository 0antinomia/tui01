//! App 结构体 - 默认入口的主应用状态

use super::showcase::ShowcaseApp;
use crate::prelude::{AppSpec, page, screen, section};
use crate::spec::field;

pub struct App {
    inner: ShowcaseApp,
}

impl App {
    pub fn new() -> Self {
        Self {
            inner: AppSpec::new()
                .title_text("TUI01 Default App\n\n默认入口现在直接展示统一控件系统。")
                .status_controls("Controls:\n↑/↓ 或 j/k  当前焦点内移动\nShift+J/K  当前焦点区域翻页\nEnter / l  从菜单进入右下\nEnter  确认文本框/下拉框\nEsc  取消当前编辑或返回左下\n←/→/j/k/l  调整下拉与开关\nq  退出")
                .screen(screen("Workspace", page("Workspace")
                    .section(section("项目信息")
                        .field(field::text_id("项目名", "tui01", "输入项目名", "project_name").with_operation_success(800))
                        .field(field::number_id("端口", "3000", "输入端口", "server_port").with_operation_success(700))
                        .field(field::text("工作区", "default", "输入工作区").with_operation_success(900)))
                    .section(section("行为开关")
                        .field(field::toggle("自动保存", true).with_operation_success(1000))
                        .field(field::toggle("展示状态栏", true).with_operation_failure(1000)))
                    .section(section("运行操作")
                        .field(field::refresh_registered_to_log("同步工作区", "刷新", "workspace_refresh", "refresh_workspace", "workspace_log"))
                        .field(field::action_to_log("清理临时文件", "执行", "workspace_cleanup", "workspace_log").with_operation_failure(900))
                        .field(field::log_id("操作输出", "等待操作结果", "workspace_log").with_height_units(4)))))
                .screen(screen("Theme", page("Theme")
                    .section(section("视觉配置")
                        .field(field::select("配色", ["Cyan", "Mono", "Warm"], 0).with_operation_success(900))
                        .field(field::select("边框", ["Rounded", "Plain"], 0).with_operation_success(900)))
                    .section(section("标识文案")
                        .field(field::text("标题", "TUI01", "输入标题").with_operation_failure(1100)))))
                .screen(screen("Controls", page("Controls")
                    .section(section("输入")
                        .field(field::text("名称", "demo", "输入名称").with_operation_success(700))
                        .field(field::text("路径", "/workspace", "输入路径").with_operation_success(1000)))
                    .section(section("选择")
                        .field(field::select("模式", ["View", "Edit", "Review"], 1).with_operation_success(900))
                        .field(field::toggle("启用实验功能", false).with_height_units(2).with_operation_failure(1200)))
                    .section(section("展示")
                        .field(field::static_value("配置版本", "2026.03"))
                        .field(field::dynamic_value("最近任务", "2 running / 5 ready").with_height_units(2))
                        .field(field::log("最近日志", "workspace synced\ncache rebuilt\nready").with_height_units(4)))))
                .screen(filler_screen("Notes 01"))
                .screen(filler_screen("Notes 02"))
                .screen(filler_screen("Notes 03"))
                .screen(filler_screen("Notes 04"))
                .screen(filler_screen("Notes 05"))
                .screen(filler_screen("Notes 06"))
                .shell_action(
                    "refresh_workspace",
                    "printf 'workspace=%s port=%s\\n' {{project_name}} {{server_port}}",
                )
                .into_showcase_app(),
        }
    }

    pub fn running(&self) -> bool {
        self.inner.running
    }

    pub fn handle_event(&mut self, event: crate::event::Event) {
        self.inner.handle_event(event);
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        self.inner.render(frame);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn filler_screen(title: &'static str) -> crate::showcase::ShowcaseScreen {
    screen(
        title,
        page(title)
            .section(
                section("填充项")
                    .field(field::text("标签", title, "输入标签").with_operation_success(700))
                    .field(field::toggle("激活", true).with_operation_success(900)),
            )
            .section(
                section("分页")
                    .field(field::select("级别", ["1", "2", "3"], 0).with_operation_failure(800)),
            ),
    )
}
