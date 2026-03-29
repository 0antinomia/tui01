//! 内容面板布局：分页计算、页码标签和尺寸辅助函数。

use super::ContentPanel;
use crate::runtime::ContentBlock;
use ratatui::style::Color;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContentPage {
    pub(super) subtitle: String,
    pub(super) blocks: Vec<VisibleBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VisibleBlock {
    pub(super) index: usize,
    pub(super) block: ContentBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectedControlKind {
    TextInput,
    NumberInput,
    Select,
    Toggle,
    ActionButton,
    StaticData,
    DynamicData,
    LogOutput,
    Custom,
}

impl VisibleBlock {
    pub(super) fn row_height(&self) -> usize {
        self.block.row_height()
    }
}

pub(super) fn layout_pages(panel: &ContentPanel, height: u16) -> Vec<ContentPage> {
    let page_height = height.max(1) as usize;
    let mut pages = Vec::new();

    if panel.blueprint.sections.is_empty() {
        return vec![ContentPage { subtitle: panel.blueprint.title.clone(), blocks: Vec::new() }];
    }
    let mut global_index = 0usize;
    let available_height = page_height.max(1);

    for section in &panel.blueprint.sections {
        let subtitle = if section.subtitle.trim().is_empty() {
            panel.blueprint.title.clone()
        } else {
            section.subtitle.clone()
        };

        if section.blocks.is_empty() {
            pages.push(ContentPage { subtitle, blocks: Vec::new() });
            continue;
        }

        let mut current_page = ContentPage { subtitle: subtitle.clone(), blocks: Vec::new() };
        let mut remaining = available_height.max(1);

        for block in &section.blocks {
            let block_height = block.row_height();
            let fits_current = block_height <= remaining;
            let has_blocks = !current_page.blocks.is_empty();

            if !fits_current && has_blocks {
                pages.push(current_page);
                current_page = ContentPage { subtitle: subtitle.clone(), blocks: Vec::new() };
                remaining = available_height.max(1);
            }

            current_page.blocks.push(VisibleBlock { index: global_index, block: block.clone() });
            global_index += 1;

            let consumed = block_height.min(remaining.max(1));
            remaining = remaining.saturating_sub(consumed);

            if remaining == 0 {
                pages.push(current_page);
                current_page = ContentPage { subtitle: subtitle.clone(), blocks: Vec::new() };
                remaining = available_height.max(1);
            }
        }

        if !current_page.blocks.is_empty() {
            pages.push(current_page);
        }
    }

    pages
}

pub(super) fn current_page_body(
    panel: &ContentPanel,
    _width: u16,
    height: u16,
) -> Option<ContentPage> {
    let pages = layout_pages(panel, effective_height(panel, height));
    let page_index = panel.runtime.current_page.min(pages.len().saturating_sub(1));
    pages.get(page_index).cloned()
}

pub(super) fn current_page_block_indices(panel: &ContentPanel, height: u16) -> Vec<usize> {
    current_page_body(panel, 0, height)
        .map(|page| page.blocks.into_iter().map(|block| block.index).collect())
        .unwrap_or_default()
}

pub(super) fn clamp_selection_to_page(panel: &mut ContentPanel, height: u16) {
    let visible = current_page_block_indices(panel, height);
    if visible.is_empty() {
        panel.runtime.selected_block = 0;
        return;
    }

    if !visible.contains(&panel.runtime.selected_block) {
        panel.runtime.selected_block = visible[0];
        panel.control_active = false;
        panel.clear_selected_snapshot();
    }
}

pub(super) fn gutter_width(_panel: &ContentPanel, width: u16) -> u16 {
    ContentPanel::GUTTER_WIDTH.min(width.saturating_sub(1).max(1))
}

pub(super) fn page_label(panel: &ContentPanel, height: u16, page: usize) -> String {
    let pages = layout_pages(panel, effective_height(panel, height));
    pages
        .get(page)
        .map(|page| page.subtitle.clone())
        .unwrap_or_else(|| panel.blueprint.title.clone())
}

pub(super) fn truncated_page_label(
    panel: &ContentPanel,
    height: u16,
    page: usize,
    max_width: usize,
) -> String {
    let label = page_label(panel, height, page);
    let max_width = max_width.min(ContentPanel::PAGE_LABEL_MAX_CHARS);
    truncate_chars(&label, max_width)
}

pub(super) fn pagination_rows(
    panel: &ContentPanel,
    height: u16,
) -> Vec<(char, Color, Option<String>)> {
    let height = effective_height(panel, height);
    if height == 0 {
        return Vec::new();
    }

    let total_pages = panel.total_pages(0, height);
    let visible_pages = (height / ContentPanel::PAGE_SLOT_HEIGHT).max(1) as usize;
    let total_visible = total_pages.min(visible_pages);
    let current_page = panel.runtime.current_page.min(total_pages.saturating_sub(1));
    let start_page = current_page.saturating_add(1).saturating_sub(total_visible);
    let end_page = (start_page + total_visible).min(total_pages);
    let slots = (total_visible as u16 * ContentPanel::PAGE_SLOT_HEIGHT) as usize;

    let inactive = ('\u{2502}', Color::Rgb(170, 170, 170), None);
    let active = ('\u{2503}', Color::White, None);

    if total_pages <= 1 {
        let mut rows = vec![active.clone(); slots];
        if rows.len() > 1 {
            rows[1] = (
                '\u{2503}',
                Color::White,
                Some(truncated_page_label(
                    panel,
                    height,
                    0,
                    ContentPanel::GUTTER_WIDTH.saturating_sub(3) as usize,
                )),
            );
        }
        return rows;
    }

    let mut rows = vec![inactive.clone(); slots];
    for page in start_page..end_page {
        let local_page = page - start_page;
        let row_start = local_page * ContentPanel::PAGE_SLOT_HEIGHT as usize;
        let row_mid = row_start + 1;
        let row_end = row_start + ContentPanel::PAGE_SLOT_HEIGHT as usize;
        let glyph = if page == current_page { active.clone() } else { inactive.clone() };

        for row in rows.iter_mut().take(row_end.min(slots)).skip(row_start) {
            *row = glyph.clone();
        }

        if page == current_page && row_mid < rows.len() {
            rows[row_mid] = (
                '\u{2503}',
                Color::White,
                Some(truncated_page_label(
                    panel,
                    height,
                    page,
                    ContentPanel::GUTTER_WIDTH.saturating_sub(3) as usize,
                )),
            );
        }
    }

    rows
}

pub(super) fn effective_height(_panel: &ContentPanel, height: u16) -> u16 {
    height.saturating_sub(ContentPanel::TOP_PADDING)
}

pub(super) fn truncate_chars(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let char_count = text.chars().count();
    if char_count <= max_width {
        return text.to_string();
    }

    if max_width == 1 {
        return "\u{2026}".to_string();
    }

    text.chars().take(max_width - 1).collect::<String>() + "\u{2026}"
}

pub(super) fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }

    if max_width == 1 {
        return "\u{2026}".to_string();
    }

    let target = max_width.saturating_sub(1);
    let mut out = String::new();
    let mut width = 0usize;

    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > target {
            break;
        }
        out.push(ch);
        width += ch_width;
    }

    out.push('\u{2026}');
    out
}
