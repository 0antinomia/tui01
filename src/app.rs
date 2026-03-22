//! App 结构体 - 默认入口的主应用状态

use crate::components::{ContentBlock, ContentBlueprint, ContentSection};
use crate::showcase::{ShowcaseApp, ShowcaseCopy, ShowcaseScreen};

pub struct App {
    inner: ShowcaseApp,
}

impl App {
    pub fn new() -> Self {
        Self {
            inner: ShowcaseApp::new(
                ShowcaseCopy {
                    title_text: "TUI01 Default App\n\n默认入口现在直接展示统一控件系统。",
                    status_controls: "Controls:\n↑/↓ 或 j/k  当前焦点内移动\nShift+J/K  当前焦点区域翻页\nEnter / l  从菜单进入右下\nEnter  确认文本框/下拉框\nEsc  取消当前编辑或返回左下\n←/→/j/k/l  调整下拉与开关\nq  退出",
                },
                vec![
                    screen(
                        "Workspace",
                        vec![
                            section(
                                "项目信息",
                                vec![
                                    ContentBlock::text_input("项目名", "tui01", "输入项目名")
                                        .with_operation_success(800),
                                    ContentBlock::number_input("端口", "3000", "输入端口")
                                        .with_operation_success(700),
                                    ContentBlock::text_input("工作区", "default", "输入工作区")
                                        .with_operation_success(900),
                                ],
                            ),
                            section(
                                "行为开关",
                                vec![
                                    ContentBlock::toggle("自动保存", true)
                                        .with_operation_success(1000),
                                    ContentBlock::toggle("展示状态栏", true)
                                        .with_operation_failure(1000),
                                ],
                            ),
                            section(
                                "运行操作",
                                vec![
                                    ContentBlock::refresh_button("同步工作区", "刷新")
                                        .with_id("workspace_refresh")
                                        .with_result_target("workspace_log")
                                        .with_operation_success(900),
                                    ContentBlock::action_button("清理临时文件", "执行")
                                        .with_id("workspace_cleanup")
                                        .with_result_target("workspace_log")
                                        .with_operation_failure(900),
                                    ContentBlock::log_output("操作输出", "等待操作结果")
                                        .with_id("workspace_log")
                                        .with_height_units(4),
                                ],
                            ),
                        ],
                    ),
                    screen(
                        "Theme",
                        vec![
                            section(
                                "视觉配置",
                                vec![
                                    ContentBlock::select("配色", ["Cyan", "Mono", "Warm"], 0)
                                        .with_operation_success(900),
                                    ContentBlock::select("边框", ["Rounded", "Plain"], 0)
                                        .with_operation_success(900),
                                ],
                            ),
                            section(
                                "标识文案",
                                vec![ContentBlock::text_input("标题", "TUI01", "输入标题")
                                    .with_operation_failure(1100)],
                            ),
                        ],
                    ),
                    screen(
                        "Controls",
                        vec![
                            section(
                                "输入",
                                vec![
                                    ContentBlock::text_input("名称", "demo", "输入名称")
                                        .with_operation_success(700),
                                    ContentBlock::text_input("路径", "/workspace", "输入路径")
                                        .with_operation_success(1000),
                                ],
                            ),
                            section(
                                "选择",
                                vec![
                                    ContentBlock::select("模式", ["View", "Edit", "Review"], 1)
                                        .with_operation_success(900),
                                    ContentBlock::toggle("启用实验功能", false)
                                        .with_height_units(2)
                                        .with_operation_failure(1200),
                                ],
                            ),
                            section(
                                "展示",
                                vec![
                                    ContentBlock::static_data("配置版本", "2026.03"),
                                    ContentBlock::dynamic_data("最近任务", "2 running / 5 ready")
                                        .with_height_units(2),
                                    ContentBlock::log_output(
                                        "最近日志",
                                        "workspace synced\ncache rebuilt\nready",
                                    )
                                    .with_height_units(4),
                                ],
                            ),
                        ],
                    ),
                    filler_screen("Notes 01"),
                    filler_screen("Notes 02"),
                    filler_screen("Notes 03"),
                    filler_screen("Notes 04"),
                    filler_screen("Notes 05"),
                    filler_screen("Notes 06"),
                ],
            ),
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

fn screen(title: &'static str, sections: Vec<ContentSection>) -> ShowcaseScreen {
    ShowcaseScreen {
        title,
        content: ContentBlueprint::new(title).with_sections(sections),
    }
}

fn section(title: &'static str, blocks: Vec<ContentBlock>) -> ContentSection {
    ContentSection::new(title).with_blocks(blocks)
}

fn filler_screen(title: &'static str) -> ShowcaseScreen {
    screen(
        title,
        vec![
            section(
                "填充项",
                vec![
                    ContentBlock::text_input("标签", title, "输入标签")
                        .with_operation_success(700),
                    ContentBlock::toggle("激活", true)
                        .with_operation_success(900),
                ],
            ),
            section(
                "分页",
                vec![ContentBlock::select("级别", ["1", "2", "3"], 0)
                    .with_operation_failure(800)],
            ),
        ],
    )
}
