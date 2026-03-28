//! TEA 核心：事件处理、键盘分发和动作应用。

use super::ShowcaseApp;
use super::super::action::Action;
use crate::components::Component;
use crate::event::{Event, Key};
use super::{operation_poll, screen_manager};

pub(super) fn handle_event(app: &mut ShowcaseApp, event: Event) {
    match event {
        Event::Tick => {
            app.content_panel.tick();
            operation_poll::poll_operation_results(app);
        }
        Event::Resize(w, h) => apply_action(app, Action::Resize(w, h)),
        Event::Quit => apply_action(app, Action::Quit),
        Event::Key(key) => handle_key(app, key),
    }
}

fn handle_key(app: &mut ShowcaseApp, key: Key) {
    if key == Key::Char('q') {
        apply_action(app, Action::Quit);
        return;
    }

    match app.focus {
        super::FocusTarget::Menu => handle_menu_key(app, key),
        super::FocusTarget::Content => handle_content_key(app, key),
    }
}

fn handle_menu_key(app: &mut ShowcaseApp, key: Key) {
    match key {
        Key::Enter | Key::Char('l') => {
            screen_manager::sync_active_to_menu_selection(app);
            screen_manager::focus_content(app);
        }
        Key::Char('K') => {
            app.content_panel
                .previous_page_with_height(app.current_content_rect().height);
            let action = app.menu.handle_events(Some(Event::Key(key)));
            apply_action(app, action);
            screen_manager::sync_active_to_menu_selection(app);
        }
        Key::Char('J') => {
            let content_rect = app.current_content_rect();
            app.content_panel
                .next_page(content_rect.width, content_rect.height);
            let action = app.menu.handle_events(Some(Event::Key(key)));
            apply_action(app, action);
            screen_manager::sync_active_to_menu_selection(app);
        }
        _ => {
            let action = app.menu.handle_events(Some(Event::Key(key)));
            apply_action(app, action);
            screen_manager::sync_active_to_menu_selection(app);
        }
    }
}

fn handle_content_key(app: &mut ShowcaseApp, key: Key) {
    let content_rect = app.current_content_rect();
    if app.content_panel.is_control_active() {
        match key {
            Key::Char('h') if app.content_panel.active_control_uses_h_as_cancel() => {
                app.content_panel.cancel_control();
                screen_manager::persist_active_screen_content(app);
            }
            Key::Esc => {
                app.content_panel.cancel_control();
                screen_manager::persist_active_screen_content(app);
            }
            Key::Enter => {
                let operation_id = operation_poll::next_operation_id(app);
                if let Some(request) = app
                    .content_panel
                    .confirm_control(operation_id, app.active_screen)
                {
                    operation_poll::submit_operation(app, request);
                }
                screen_manager::persist_active_screen_content(app);
            }
            Key::Char('l') if app.content_panel.active_control_uses_l_as_confirm() => {
                let operation_id = operation_poll::next_operation_id(app);
                if let Some(request) = app
                    .content_panel
                    .confirm_control(operation_id, app.active_screen)
                {
                    operation_poll::submit_operation(app, request);
                }
                screen_manager::persist_active_screen_content(app);
            }
            Key::Left => {
                if app.content_panel.handle_control_key(key) {
                    screen_manager::persist_active_screen_content(app);
                }
            }
            Key::Right | Key::Char('l') => {
                if app.content_panel.handle_control_key(key) {
                    screen_manager::persist_active_screen_content(app);
                }
            }
            _ => {
                if app.content_panel.handle_control_key(key) {
                    screen_manager::persist_active_screen_content(app);
                }
            }
        }
        return;
    }

    match key {
        Key::Up | Key::Char('k') => {
            app.content_panel
                .select_previous_block(content_rect.height);
        }
        Key::Down | Key::Char('j') => {
            app.content_panel.select_next_block(content_rect.height);
        }
        Key::Char('K') => {
            app.content_panel
                .previous_page_with_height(content_rect.height);
        }
        Key::Char('J') => {
            app.content_panel
                .next_page(content_rect.width, content_rect.height);
        }
        Key::Char('l') | Key::Enter => {
            let operation_id = operation_poll::next_operation_id(app);
            if let Some(request) = app
                .content_panel
                .activate_selected_control(operation_id, app.active_screen)
            {
                operation_poll::submit_operation(app, request);
            }
            screen_manager::persist_active_screen_content(app);
        }
        Key::Char('h') | Key::Esc => {
            screen_manager::focus_menu(app);
        }
        _ => {}
    }
}

pub(super) fn apply_action(app: &mut ShowcaseApp, action: Action) {
    match action {
        Action::Quit => app.running = false,
        Action::Resize(w, h) => {
            app.size_error = super::ShowcaseApp::check_size(w, h);
        }
        Action::MenuSelect(index) => {
            if index < app.screens.len() {
                if index != app.active_screen {
                    screen_manager::persist_active_screen_content(app);
                }
                app.active_screen = index;
                screen_manager::load_active_screen_content(app);
            }
        }
        Action::Noop => {}
    }

    screen_manager::sync_panels(app);
}
