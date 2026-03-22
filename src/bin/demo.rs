//! TUI01 框架演示程序

use tui01::builder::{page, screen, section, AppSpec};
use tui01::event::EventHandler;
use tui01::schema::FieldSpec;
use tui01::showcase::{ShowcaseApp, ShowcaseScreen};
use tui01::tui;

fn build_demo() -> ShowcaseApp {
    AppSpec::new()
        .title_text("TUI01 Framework Demo\n\n第一个菜单集中展示当前所有控件类型，并为可交互控件提供成功/失败两组示例。")
        .status_controls("Controls:\n↑/↓ 或 j/k  当前焦点内移动\nShift+J/K  当前焦点区域翻页\nEnter / l  从菜单进入右下\nEnter  确认文本框/下拉框\nEsc  取消当前编辑或返回左下\n←/→/j/k/l  调整下拉与开关\nq  退出")
        .screen(screen("Showcase", page("Showcase")
            .section(section("文本输入")
                .field(FieldSpec::text_input("服务名称", "alpha", "输入名称").with_operation_success(900))
                .field(FieldSpec::text_input("项目名称", "broken-demo", "输入名称").with_operation_failure(1100)))
            .section(section("数值输入")
                .field(FieldSpec::number_input("HTTP 端口", "8080", "输入端口").with_operation_success(700))
                .field(FieldSpec::number_input("重试次数", "3", "输入次数").with_operation_failure(900)))
            .section(section("下拉选择")
                .field(FieldSpec::select("部署环境", ["dev", "stage", "prod"], 0).with_operation_success(900))
                .field(FieldSpec::select("高亮风格", ["bold", "inverse", "plain"], 1).with_operation_failure(1000)))
            .section(section("开关控件")
                .field(FieldSpec::toggle("启用缓存", true).with_operation_success(800))
                .field(FieldSpec::toggle("开启追踪", false).with_operation_failure(1000)))
            .section(section("动作控件")
                .field(FieldSpec::refresh_button("刷新工作区", "刷新").with_id("action_refresh_success").with_result_target("action_log_success").with_operation_success(1000))
                .field(FieldSpec::action_button("重建缓存", "执行").with_id("action_rebuild_failure").with_result_target("action_log_failure").with_operation_failure(900)))
            .section(section("静态展示")
                .field(FieldSpec::static_data("当前版本", "v0.1.0-success"))
                .field(FieldSpec::static_data("目标版本", "v0.1.0-failure")))
            .section(section("动态展示")
                .field(FieldSpec::dynamic_data("队列状态", "2 running / 0 failed"))
                .field(FieldSpec::dynamic_data("任务状态", "1 running / 3 failed")))
            .section(section("日志输出")
                .field(FieldSpec::log_output("成功日志", "等待成功结果").with_id("action_log_success").with_height_units(4))
                .field(FieldSpec::log_output("失败日志", "等待失败结果").with_id("action_log_failure").with_height_units(4)))))
        .screen(screen("Menu", page("Menu")
            .section(section("菜单行为")
                .field(FieldSpec::toggle("允许分页", true).with_operation_success(900))
                .field(FieldSpec::select("高亮样式", ["加粗", "反色", "下划线"], 0).with_operation_failure(1100)))
            .section(section("菜单文案")
                .field(FieldSpec::text_input("默认标签", "Item 01", "输入标签").with_operation_success(1000)))))
        .screen(screen("Content", page("Content")
            .section(section("输入控件")
                .field(FieldSpec::text_input("用户名", "demo-user", "输入用户名").with_operation_success(900))
                .field(FieldSpec::number_input("端口", "8080", "输入端口").with_operation_success(700))
                .field(FieldSpec::text_input("项目名", "tui01", "输入项目名").with_operation_failure(1300)))
            .section(section("选择控件")
                .field(FieldSpec::select("语言", ["Rust", "Go", "Zig"], 0).with_operation_success(900))
                .field(FieldSpec::select("主题", ["Classic", "Ocean", "Mono"], 1).with_operation_failure(1000)))
            .section(section("开关控件")
                .field(FieldSpec::toggle("启用缓存", true).with_operation_success(700))
                .field(FieldSpec::toggle("显示日志", false).with_height_units(2).with_operation_failure(1000)))
            .section(section("数据展示")
                .field(FieldSpec::static_data("当前版本", "v0.1.0"))
                .field(FieldSpec::dynamic_data("任务状态", "3 running / 1 queued").with_height_units(2))
                .field(FieldSpec::log_output("最近输出", "build started\nchecking modules\nbuild finished").with_height_units(4)))))
        .screen(screen("Events", page("Events")
            .section(section("输入流")
                .field(FieldSpec::toggle("键盘事件", true).with_operation_success(800))
                .field(FieldSpec::toggle("尺寸事件", true).with_operation_success(800)))
            .section(section("调试配置")
                .field(FieldSpec::select("日志级别", ["info", "debug", "trace"], 1).with_operation_success(1100))
                .field(FieldSpec::text_input("事件前缀", "evt", "输入前缀").with_operation_failure(1000)))))
        .screen(screen("Lifecycle", page("Lifecycle")
            .section(section("终端行为")
                .field(FieldSpec::toggle("Raw Mode", true).with_operation_success(1000))
                .field(FieldSpec::toggle("Alt Screen", true).with_operation_success(1000)))
            .section(section("清理策略")
                .field(FieldSpec::select("退出恢复", ["完整", "最小", "跳过"], 0).with_operation_success(900))
                .field(FieldSpec::text_input("最小宽度", "80", "输入宽度").with_operation_failure(1000)))))
        .screen(filler_screen("Pagination Demo 01"))
        .screen(filler_screen("Pagination Demo 02"))
        .screen(filler_screen("Pagination Demo 03"))
        .screen(filler_screen("Pagination Demo 04"))
        .screen(filler_screen("Pagination Demo 05"))
        .screen(filler_screen("Pagination Demo 06"))
        .screen(filler_screen("Pagination Demo 07"))
        .screen(filler_screen("Pagination Demo 08"))
        .screen(filler_screen("Pagination Demo 09"))
        .screen(filler_screen("Pagination Demo 10"))
        .screen(filler_screen("Pagination Demo 11"))
        .screen(filler_screen("Pagination Demo 12"))
        .screen(filler_screen("Pagination Demo 13"))
        .screen(filler_screen("Pagination Demo 14"))
        .screen(filler_screen("Pagination Demo 15"))
        .screen(filler_screen("Pagination Demo 16"))
        .screen(filler_screen("Pagination Demo 17"))
        .screen(filler_screen("Pagination Demo 18"))
        .screen(filler_screen("Pagination Demo 19"))
        .screen(filler_screen("Pagination Demo 20"))
        .into_showcase_app()
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

fn filler_screen(title: &'static str) -> ShowcaseScreen {
    screen(
        title,
        page(title)
            .section(
                section("分页填充")
                    .field(
                        FieldSpec::text_input("标签", title, "输入标签")
                            .with_operation_success(700),
                    )
                    .field(
                        FieldSpec::select("模式", ["A", "B", "C"], 0).with_operation_success(700),
                    ),
            )
            .section(
                section("控件测试")
                    .field(FieldSpec::toggle("启用测试", true).with_operation_failure(900))
                    .field(
                        FieldSpec::text_input("说明", "demo", "输入说明")
                            .with_height_units(2)
                            .with_operation_success(900),
                    ),
            ),
    )
}
