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
                    status_controls: "Controls:\n↑/↓ 或 j/k  当前焦点内移动\nShift+J/K  当前焦点区域翻页\nEnter / l  从菜单进入右下\nEsc  返回左下菜单\n右下直接输入 / Backspace 编辑文本框\n←/→/h/l/Enter 操作下拉与开关\nq  退出",
                },
                vec![
                    screen(
                        "Workspace",
                        vec![
                            section(
                                "项目信息",
                                vec![
                                    ContentBlock::text_input("项目名", "tui01", "输入项目名"),
                                    ContentBlock::text_input("工作区", "default", "输入工作区"),
                                ],
                            ),
                            section(
                                "行为开关",
                                vec![
                                    ContentBlock::toggle("自动保存", true),
                                    ContentBlock::toggle("展示状态栏", true),
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
                                    ContentBlock::select("配色", ["Cyan", "Mono", "Warm"], 0),
                                    ContentBlock::select("边框", ["Rounded", "Plain"], 0),
                                ],
                            ),
                            section(
                                "标识文案",
                                vec![ContentBlock::text_input("标题", "TUI01", "输入标题")],
                            ),
                        ],
                    ),
                    screen(
                        "Controls",
                        vec![
                            section(
                                "输入",
                                vec![
                                    ContentBlock::text_input("名称", "demo", "输入名称"),
                                    ContentBlock::text_input("路径", "/workspace", "输入路径"),
                                ],
                            ),
                            section(
                                "选择",
                                vec![
                                    ContentBlock::select("模式", ["View", "Edit", "Review"], 1),
                                    ContentBlock::toggle("启用实验功能", false).with_height_units(2),
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
                    ContentBlock::text_input("标签", title, "输入标签"),
                    ContentBlock::toggle("激活", true),
                ],
            ),
            section(
                "分页",
                vec![ContentBlock::select("级别", ["1", "2", "3"], 0)],
            ),
        ],
    )
}
