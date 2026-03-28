//! 内容面板渲染：页面绘制、内容块渲染和文本输出。

use super::layout::{ContentPage, current_page_body, effective_height, gutter_width, pagination_rows, truncate_to_width};
use super::ContentPanel;
use crate::controls::{AnyControl, ControlFeedback};
use crate::runtime::{ContentBlock, OperationStatus};
use crate::theme::RenderContext;

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};
use unicode_width::UnicodeWidthChar;

fn render_control(
    control: &AnyControl,
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    ctx: &RenderContext,
) {
    control.render(area, buf, ctx)
}

pub(super) fn render_content_panel(panel: &mut ContentPanel, f: &mut Frame, rect: Rect) {
    let gw = gutter_width(panel, rect.width);
    let rows = pagination_rows(panel, rect.height);
    for (idx, (glyph, color, label)) in rows.into_iter().enumerate() {
        let y = rect.y + idx as u16 + ContentPanel::TOP_PADDING;
        if y >= rect.y + rect.height {
            break;
        }
        f.buffer_mut()[(rect.x, y)].set_char(glyph).set_fg(color);

        if let Some(label) = label {
            let label_x = rect.x.saturating_add(2);
            let available = gw.saturating_sub(2) as usize;
            write_text(panel, f, label_x, y, &label, available, Color::White, None);
        }
    }

    let content_rect = Rect::new(
        rect.x + gw.min(rect.width),
        rect.y.saturating_add(ContentPanel::TOP_PADDING),
        rect.width.saturating_sub(gw),
        effective_height(panel, rect.height),
    );

    if let Some(page) = current_page_body(panel, rect.width, rect.height) {
        render_page(panel, f, content_rect, &page);
    } else {
        let widget = Paragraph::new("No content").style(Style::default().fg(Color::White));
        f.render_widget(widget, content_rect);
    }
}

pub(super) fn render_page(panel: &ContentPanel, f: &mut Frame, rect: Rect, page: &ContentPage) {
    let content_width = rect.width as usize;
    if content_width == 0 || rect.height == 0 {
        return;
    }

    let blocks_rect = rect;
    if blocks_rect.height == 0 {
        return;
    }

    let content_width = blocks_rect.width.saturating_sub(2);
    let distributable = content_width.saturating_sub(ContentPanel::COLUMN_GAP);
    let text_width = (distributable * 3 / 10).max(1);
    let control_width = distributable.saturating_sub(text_width);

    let mut cursor_y = blocks_rect.y;
    for block in &page.blocks {
        if cursor_y >= blocks_rect.y + blocks_rect.height {
            break;
        }

        let intended_height = block.row_height() as u16;
        let block_height = intended_height.min(blocks_rect.y + blocks_rect.height - cursor_y);
        let block_rect = Rect::new(blocks_rect.x, cursor_y, blocks_rect.width, block_height);
        render_content_block(
            panel,
            f,
            block_rect,
            &block.block,
            block.index,
            text_width,
            control_width,
            block.index == panel.runtime.selected_block,
        );
        cursor_y = cursor_y.saturating_add(intended_height);
    }

    while cursor_y < blocks_rect.y + blocks_rect.height {
        write_text(
            panel,
            f,
            blocks_rect.x,
            cursor_y,
            "",
            content_width as usize,
            Color::White,
            None,
        );
        cursor_y += 1;
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_content_block(
    panel: &ContentPanel,
    f: &mut Frame,
    rect: Rect,
    block: &ContentBlock,
    block_index: usize,
    text_width: u16,
    control_width: u16,
    selected: bool,
) {
    if rect.height == 0 || rect.width == 0 {
        return;
    }

    let content_rect = rect;
    let baseline_y = content_rect
        .y
        .saturating_add(1)
        .min(content_rect.y + content_rect.height.saturating_sub(1));
    f.buffer_mut()[(content_rect.x, baseline_y)]
        .set_char(if selected && panel.focused { '>' } else { ' ' })
        .set_fg(Color::Yellow);

    let text_rect = Rect::new(
        content_rect.x + 2,
        content_rect.y,
        text_width,
        content_rect.height,
    );
    let control_x = content_rect
        .x
        .saturating_add(2)
        .saturating_add(text_width)
        .saturating_add(ContentPanel::COLUMN_GAP);
    let label_width = text_rect.width as usize;
    let truncated = truncate_to_width(block.label.as_str(), label_width.max(1));
    write_text(
        panel,
        f,
        text_rect.x,
        baseline_y,
        &truncated,
        label_width.max(1),
        Color::Cyan,
        if selected && panel.focused {
            Some(Style::default().add_modifier(Modifier::BOLD))
        } else {
            None
        },
    );

    let control_rect = Rect::new(
        control_x,
        content_rect.y,
        control_width,
        content_rect.height,
    );
    let feedback = feedback_for(panel, block_index);
    render_control(
        panel.block_control(block_index).unwrap_or(&block.control),
        control_rect,
        f.buffer_mut(),
        &RenderContext {
            theme: panel.theme,
            selected: selected && panel.focused,
            active: selected && panel.focused && panel.control_active,
            feedback,
        },
    );
}

pub(super) fn feedback_for(panel: &ContentPanel, block_index: usize) -> ControlFeedback {
    match panel
        .block_state(block_index)
        .map(|state| &state.status)
        .unwrap_or(&OperationStatus::Idle)
    {
        OperationStatus::Idle => ControlFeedback::Idle,
        OperationStatus::Running { .. } => ControlFeedback::Running(panel.runtime.spinner_tick),
        OperationStatus::Success => ControlFeedback::Success,
        OperationStatus::Failure => ControlFeedback::Failure,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_text(
    _panel: &ContentPanel,
    f: &mut Frame,
    x: u16,
    y: u16,
    text: &str,
    max_width: usize,
    fg: Color,
    style: Option<Style>,
) {
    let style = style.unwrap_or_default().fg(fg);
    let mut cursor_x = x;
    let mut used_width = 0usize;

    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if ch_width == 0 {
            continue;
        }
        if used_width + ch_width > max_width {
            break;
        }

        f.buffer_mut()[(cursor_x, y)].set_char(ch).set_style(style);

        cursor_x = cursor_x.saturating_add(ch_width as u16);
        used_width += ch_width;
    }
}
