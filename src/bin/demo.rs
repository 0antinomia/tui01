//! TUI01 框架演示程序

use tui01::components::{ContentBlock, ContentBlueprint, ContentSection};
use tui01::event::EventHandler;
use tui01::showcase::{ShowcaseApp, ShowcaseCopy, ShowcaseScreen};
use tui01::tui;

fn build_demo() -> ShowcaseApp {
    ShowcaseApp::new(
        ShowcaseCopy {
            title_text: "TUI01 Framework Demo\n\n第一个菜单集中展示当前所有控件类型，并为可交互控件提供成功/失败两组示例。",
            status_controls: "Controls:\n↑/↓ 或 j/k  当前焦点内移动\nShift+J/K  当前焦点区域翻页\nEnter / l  从菜单进入右下\nEnter  确认文本框/下拉框\nEsc  取消当前编辑或返回左下\n←/→/j/k/l  调整下拉与开关\nq  退出",
        },
        vec![
            screen(
                "Showcase",
                vec![
                    section(
                        "文本输入",
                        vec![
                            ContentBlock::text_input("服务名称", "alpha", "输入名称")
                                .with_operation_success(900),
                            ContentBlock::text_input("项目名称", "broken-demo", "输入名称")
                                .with_operation_failure(1100),
                        ],
                    ),
                    section(
                        "数值输入",
                        vec![
                            ContentBlock::number_input("HTTP 端口", "8080", "输入端口")
                                .with_operation_success(700),
                            ContentBlock::number_input("重试次数", "3", "输入次数")
                                .with_operation_failure(900),
                        ],
                    ),
                    section(
                        "下拉选择",
                        vec![
                            ContentBlock::select("部署环境", ["dev", "stage", "prod"], 0)
                                .with_operation_success(900),
                            ContentBlock::select("高亮风格", ["bold", "inverse", "plain"], 1)
                                .with_operation_failure(1000),
                        ],
                    ),
                    section(
                        "开关控件",
                        vec![
                            ContentBlock::toggle("启用缓存", true)
                                .with_operation_success(800),
                            ContentBlock::toggle("开启追踪", false)
                                .with_operation_failure(1000),
                        ],
                    ),
                    section(
                        "动作控件",
                        vec![
                            ContentBlock::refresh_button("刷新工作区", "刷新")
                                .with_id("action_refresh_success")
                                .with_result_target("action_log_success")
                                .with_operation_success(1000),
                            ContentBlock::action_button("重建缓存", "执行")
                                .with_id("action_rebuild_failure")
                                .with_result_target("action_log_failure")
                                .with_operation_failure(900),
                        ],
                    ),
                    section(
                        "静态展示",
                        vec![
                            ContentBlock::static_data("当前版本", "v0.1.0-success"),
                            ContentBlock::static_data("目标版本", "v0.1.0-failure"),
                        ],
                    ),
                    section(
                        "动态展示",
                        vec![
                            ContentBlock::dynamic_data("队列状态", "2 running / 0 failed"),
                            ContentBlock::dynamic_data("任务状态", "1 running / 3 failed"),
                        ],
                    ),
                    section(
                        "日志输出",
                        vec![
                            ContentBlock::log_output("成功日志", "等待成功结果")
                                .with_id("action_log_success")
                                .with_height_units(4),
                            ContentBlock::log_output("失败日志", "等待失败结果")
                                .with_id("action_log_failure")
                                .with_height_units(4),
                        ],
                    ),
                ],
            ),
            screen(
                "Menu",
                vec![
                    section(
                        "菜单行为",
                        vec![
                            ContentBlock::toggle("允许分页", true)
                                .with_operation_success(900),
                            ContentBlock::select("高亮样式", ["加粗", "反色", "下划线"], 0)
                                .with_operation_failure(1100),
                        ],
                    ),
                    section(
                        "菜单文案",
                        vec![ContentBlock::text_input("默认标签", "Item 01", "输入标签")
                            .with_operation_success(1000)],
                    ),
                ],
            ),
            screen(
                "Content",
                vec![
                    section(
                        "输入控件",
                        vec![
                            ContentBlock::text_input("用户名", "demo-user", "输入用户名")
                                .with_operation_success(900),
                            ContentBlock::number_input("端口", "8080", "输入端口")
                                .with_operation_success(700),
                            ContentBlock::text_input("项目名", "tui01", "输入项目名")
                                .with_operation_failure(1300),
                        ],
                    ),
                    section(
                        "选择控件",
                        vec![
                            ContentBlock::select("语言", ["Rust", "Go", "Zig"], 0)
                                .with_operation_success(900),
                            ContentBlock::select("主题", ["Classic", "Ocean", "Mono"], 1)
                                .with_operation_failure(1000),
                        ],
                    ),
                    section(
                        "开关控件",
                        vec![
                            ContentBlock::toggle("启用缓存", true)
                                .with_operation_success(700),
                            ContentBlock::toggle("显示日志", false)
                                .with_height_units(2)
                                .with_operation_failure(1000),
                        ],
                    ),
                    section(
                        "数据展示",
                        vec![
                            ContentBlock::static_data("当前版本", "v0.1.0"),
                            ContentBlock::dynamic_data("任务状态", "3 running / 1 queued")
                                .with_height_units(2),
                            ContentBlock::log_output(
                                "最近输出",
                                "build started\nchecking modules\nbuild finished",
                            )
                            .with_height_units(4),
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
                            ContentBlock::toggle("键盘事件", true)
                                .with_operation_success(800),
                            ContentBlock::toggle("尺寸事件", true)
                                .with_operation_success(800),
                        ],
                    ),
                    section(
                        "调试配置",
                        vec![
                            ContentBlock::select("日志级别", ["info", "debug", "trace"], 1)
                                .with_operation_success(1100),
                            ContentBlock::text_input("事件前缀", "evt", "输入前缀")
                                .with_operation_failure(1000),
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
                            ContentBlock::toggle("Raw Mode", true)
                                .with_operation_success(1000),
                            ContentBlock::toggle("Alt Screen", true)
                                .with_operation_success(1000),
                        ],
                    ),
                    section(
                        "清理策略",
                        vec![
                            ContentBlock::select("退出恢复", ["完整", "最小", "跳过"], 0)
                                .with_operation_success(900),
                            ContentBlock::text_input("最小宽度", "80", "输入宽度")
                                .with_operation_failure(1000),
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
                    ContentBlock::text_input("标签", title, "输入标签")
                        .with_operation_success(700),
                    ContentBlock::select("模式", ["A", "B", "C"], 0)
                        .with_operation_success(700),
                ],
            ),
            section(
                "控件测试",
                vec![
                    ContentBlock::toggle("启用测试", true)
                        .with_operation_failure(900),
                    ContentBlock::text_input("说明", "demo", "输入说明")
                        .with_height_units(2)
                        .with_operation_success(900),
                ],
            ),
        ],
    )
}
