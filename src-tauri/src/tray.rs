//! Notification area icon. It is the only way back to the settings window once
//! that window has been closed, because closing it now hides it instead of
//! quitting the application.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

use crate::settings::UiLanguage;
use crate::MAIN_LABEL;

const TRAY_ID: &str = "glossy-tray";
const MENU_OPEN: &str = "tray-open";
const MENU_QUIT: &str = "tray-quit";

/// Brings the settings window to the front, restoring it if it was minimised.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn install<R: Runtime>(app: &AppHandle<R>, language: UiLanguage) -> tauri::Result<()> {
    let (open, quit, tooltip) = match language {
        UiLanguage::Chinese => ("打开 Glossy", "退出", "Glossy — 选中文本即可翻译"),
        _ => ("Open Glossy", "Quit", "Glossy — select text to translate"),
    };
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, MENU_OPEN, open, true, None::<&str>)?,
            &MenuItem::with_id(app, MENU_QUIT, quit, true, None::<&str>)?,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip(tooltip)
        // The menu belongs to the right button, the left button opens Glossy.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_OPEN => show_main(app),
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder.build(app)?;
    Ok(())
}
