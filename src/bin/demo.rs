//! TUI01 框架演示程序

use tui01::components::{ContentBlock, ContentBlueprint, ContentSection};
use tui01::event::EventHandler;
use tui01::showcase::{ShowcaseApp, ShowcaseCopy, ShowcaseScreen};
use tui01::tui;

fn build_demo() -> ShowcaseApp {
    ShowcaseApp::new(
        ShowcaseCopy {
            title_text: "TUI01 Framework Demo\n\n这个 demo 直接展示新的统一控件系统：文本框、下拉选择框、开关。",
            status_controls: "Controls:\n↑/↓ 或 j/k  当前焦点内移动\nShift+J/K  当前焦点区域翻页\nEnter / l  从菜单进入右下\nEsc  返回左下菜单\n右下直接输入 / Backspace 编辑文本框\n←/→/h/l/Enter 操作下拉与开关\nq  退出",
        },
        vec![
            screen(
                "Layout",
                vec![
                    section(
                        "布局参数",
                        vec![
                            ContentBlock::select("布局比例", ["20/80", "30/70", "40/60"], 0),
                            ContentBlock::toggle("显示分割线", true),
                        ],
                    ),
                    section(
                        "标题文案",
                        vec![ContentBlock::text_input("面板标题", "TUI01", "输入标题")],
                    ),
                ],
            ),
            screen(
                "Menu",
                vec![
                    section(
                        "菜单行为",
                        vec![
                            ContentBlock::toggle("允许分页", true),
                            ContentBlock::select("高亮样式", ["加粗", "反色", "下划线"], 0),
                        ],
                    ),
                    section(
                        "菜单文案",
                        vec![ContentBlock::text_input("默认标签", "Item 01", "输入标签")],
                    ),
                ],
            ),
            screen(
                "Content",
                vec![
                    section(
                        "输入控件",
                        vec![
                            ContentBlock::text_input("用户名", "demo-user", "输入用户名"),
                            ContentBlock::text_input("项目名", "tui01", "输入项目名"),
                        ],
                    ),
                    section(
                        "选择控件",
                        vec![
                            ContentBlock::select("语言", ["Rust", "Go", "Zig"], 0),
                            ContentBlock::select("主题", ["Classic", "Ocean", "Mono"], 1),
                        ],
                    ),
                    section(
                        "开关控件",
                        vec![
                            ContentBlock::toggle("启用缓存", true),
                            ContentBlock::toggle("显示日志", false).with_height_units(2),
                        ],
                    ),
                ],
            ),
            screen(
                "Events",
                vec![
                    section(
                        "输入流",
                        vec![
                            ContentBlock::toggle("键盘事件", true),
                            ContentBlock::toggle("尺寸事件", true),
                        ],
                    ),
                    section(
                        "调试配置",
                        vec![
                            ContentBlock::select("日志级别", ["info", "debug", "trace"], 1),
                            ContentBlock::text_input("事件前缀", "evt", "输入前缀"),
                        ],
                    ),
                ],
            ),
            screen(
                "Lifecycle",
                vec![
                    section(
                        "终端行为",
                        vec![
                            ContentBlock::toggle("Raw Mode", true),
                            ContentBlock::toggle("Alt Screen", true),
                        ],
                    ),
                    section(
                        "清理策略",
                        vec![
                            ContentBlock::select("退出恢复", ["完整", "最小", "跳过"], 0),
                            ContentBlock::text_input("最小宽度", "80", "输入宽度"),
                        ],
                    ),
                ],
            ),
            filler_screen("Pagination Demo 01"),
            filler_screen("Pagination Demo 02"),
            filler_screen("Pagination Demo 03"),
            filler_screen("Pagination Demo 04"),
            filler_screen("Pagination Demo 05"),
            filler_screen("Pagination Demo 06"),
            filler_screen("Pagination Demo 07"),
            filler_screen("Pagination Demo 08"),
            filler_screen("Pagination Demo 09"),
            filler_screen("Pagination Demo 10"),
            filler_screen("Pagination Demo 11"),
            filler_screen("Pagination Demo 12"),
            filler_screen("Pagination Demo 13"),
            filler_screen("Pagination Demo 14"),
            filler_screen("Pagination Demo 15"),
            filler_screen("Pagination Demo 16"),
            filler_screen("Pagination Demo 17"),
            filler_screen("Pagination Demo 18"),
            filler_screen("Pagination Demo 19"),
            filler_screen("Pagination Demo 20"),
        ],
    )
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tui::init_panic_hook();

    if let Err(msg) = tui::check_minimum_size() {
        eprintln!("{}", msg);
        return Ok(());
    }

    let mut tui = tui::Tui::new()?;
    let mut event_handler = EventHandler::new();
    let mut app = build_demo();

    while app.running {
        tui.draw(|f| app.render(f))?;

        if let Some(event) = event_handler.next().await {
            app.handle_event(event);
        }
    }

    tui.exit()?;
    Ok(())
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
                "分页填充",
                vec![
                    ContentBlock::text_input("标签", title, "输入标签"),
                    ContentBlock::select("模式", ["A", "B", "C"], 0),
                ],
            ),
            section(
                "控件测试",
                vec![
                    ContentBlock::toggle("启用测试", true),
                    ContentBlock::text_input("说明", "demo", "输入说明").with_height_units(2),
                ],
            ),
        ],
    )
}
