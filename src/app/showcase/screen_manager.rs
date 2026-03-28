//! 屏幕管理：面板同步、内容持久化和焦点切换。

use super::ShowcaseApp;
use crate::components::Component;

pub(super) fn sync_panels(app: &mut ShowcaseApp) {
    app.content_panel.set_theme(app.theme);
    app.title_panel.set_text(app.copy.title_text.clone());

    let selected = app
        .menu
        .selected_item()
        .map(|item| item.label.as_str())
        .unwrap_or("None");

    let active = app
        .screens
        .get(app.active_screen)
        .map(|screen| screen.title.as_str())
        .unwrap_or("None");

    let focus_label = match app.focus {
        super::FocusTarget::Menu => "MenuComponent",
        super::FocusTarget::Content => "ContentPanel",
    };

    app.status_panel.set_text(format!(
        "Focus: {}\nSelected: {}\nActive: {}\n\n{}",
        focus_label, selected, active, app.copy.status_controls
    ));

    load_active_screen_content(app);
}

pub(super) fn persist_active_screen_content(app: &mut ShowcaseApp) {
    if let Some(screen) = app.screens.get_mut(app.active_screen) {
        screen.content = app.content_panel.blueprint();
    }
}

pub(super) fn load_active_screen_content(app: &mut ShowcaseApp) {
    if app.loaded_screen == Some(app.active_screen) {
        return;
    }

    let Some(screen) = app.screens.get(app.active_screen) else {
        return;
    };

    app.content_panel.set_blueprint(screen.content.clone());
    app.loaded_screen = Some(app.active_screen);
}

pub(super) fn sync_active_to_menu_selection(app: &mut ShowcaseApp) {
    let index = app.menu.selected_index();
    if index < app.screens.len() {
        if index != app.active_screen {
            persist_active_screen_content(app);
        }
        app.active_screen = index;
        load_active_screen_content(app);
    }
    sync_panels(app);
}

pub(super) fn focus_menu(app: &mut ShowcaseApp) {
    app.focus = super::FocusTarget::Menu;
    app.content_panel.blur();
    app.menu.focus();
}

pub(super) fn focus_content(app: &mut ShowcaseApp) {
    app.focus = super::FocusTarget::Content;
    app.menu.blur();
    app.content_panel.focus();
    let height = app.current_content_rect().height;
    if app.content_panel.has_selectable_blocks(height) {
        app.content_panel.ensure_visible_selection(height);
    }
}
