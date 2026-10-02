//! Notification area icon. It is the only way back to the settings window once
//! that window has been closed, because closing it now hides it instead of
//! quitting the application — and, while that window is closed, it is the only
//! place the shortcuts are written down, so the two actions that need no window
//! are on its menu as well.

use std::sync::Arc;

use tauri::menu::{IsMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

use crate::platform::hotkey::Slot;
use crate::settings::{self, Settings, UiLanguage};
use crate::state::AppState;
use crate::MAIN_LABEL;

const TRAY_ID: &str = "glossy-tray";
const MENU_OPEN: &str = "tray-open";
const MENU_OCR: &str = "tray-ocr";
const MENU_CLIPBOARD: &str = "tray-clipboard";
const MENU_BOXES: &str = "tray-subtitle-boxes";
const MENU_QUIT: &str = "tray-quit";

/// Brings the settings window to the front, restoring it if it was minimised.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// The words of the menu, in one language.
struct Words {
    tooltip: &'static str,
    open: &'static str,
    ocr: &'static str,
    clipboard: &'static str,
    boxes: &'static str,
    quit: &'static str,
}

impl Words {
    fn of(language: UiLanguage) -> Words {
        match language {
            UiLanguage::Chinese => Words {
                tooltip: "Glossy — 选中文本即可翻译",
                open: "打开 Glossy",
                ocr: "截图翻译",
                clipboard: "翻译剪贴板",
                boxes: "调整字幕框",
                quit: "退出",
            },
            _ => Words {
                tooltip: "Glossy — select text to translate",
                open: "Open Glossy",
                ocr: "Translate a screenshot",
                clipboard: "Translate the clipboard",
                boxes: "Adjust the subtitle boxes",
                quit: "Quit",
            },
        }
    }
}

/// An entry that names the shortcut doing the same thing.
///
/// The window that shows the shortcuts may be closed, and the key is the one
/// the settings hold now: the menu is built from them rather than written once,
/// so a combination the user replaced never lingers here. A field that was
/// cleared leaves the entry without one.
fn with_shortcut(text: &str, spec: &str) -> String {
    if spec.trim().is_empty() {
        text.to_string()
    } else {
        format!("{text} ({spec})")
    }
}

/// The menu itself, built from the settings the app is running with.
fn menu(app: &AppHandle, settings: &Settings) -> tauri::Result<Menu<tauri::Wry>> {
    let words = Words::of(settings::resolve_ui_language(settings.ui_lang));
    let mut items: Vec<Box<dyn IsMenuItem<tauri::Wry>>> = vec![
        Box::new(MenuItem::with_id(
            app,
            MENU_OPEN,
            words.open,
            true,
            None::<&str>,
        )?),
        Box::new(MenuItem::with_id(
            app,
            MENU_OCR,
            with_shortcut(words.ocr, &Slot::Ocr.spec(settings)),
            true,
            None::<&str>,
        )?),
        Box::new(MenuItem::with_id(
            app,
            MENU_CLIPBOARD,
            with_shortcut(words.clipboard, &Slot::Translate.spec(settings)),
            true,
            None::<&str>,
        )?),
    ];
    // The subtitle boxes belong to developer mode, and nothing about them is
    // shown while that is off: an entry that could only say no is worse than no
    // entry at all.
    if settings.developer_mode {
        items.push(Box::new(MenuItem::with_id(
            app,
            MENU_BOXES,
            words.boxes,
            true,
            None::<&str>,
        )?));
    }
    items.push(Box::new(MenuItem::with_id(
        app,
        MENU_QUIT,
        words.quit,
        true,
        None::<&str>,
    )?));
    let items: Vec<&dyn IsMenuItem<tauri::Wry>> = items
        .iter()
        .map(|item| item.as_ref() as &dyn IsMenuItem<tauri::Wry>)
        .collect();
    Menu::with_items(app, &items)
}

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let settings = Settings::load(app);
    let words = Words::of(settings::resolve_ui_language(settings.ui_lang));
    let menu = menu(app, &settings)?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip(words.tooltip)
        // The menu belongs to the right button, the left button opens Glossy.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_OPEN => show_main(app),
            // Both of these block for a moment — a screenshot freezes the
            // screen, a translation reads the clipboard — and the thread this
            // menu is answered on has to stay responsive.
            MENU_OCR => {
                let app = app.clone();
                std::thread::spawn(move || crate::ocr::begin(&app));
            }
            MENU_CLIPBOARD => {
                let app = app.clone();
                std::thread::spawn(move || translate_clipboard(&app));
            }
            // The boxes belong to a reading that is already running, and that
            // reading is over a video with the settings window out of the way:
            // the tray is the one place left to reach them from.
            MENU_BOXES => {
                let app = app.clone();
                std::thread::spawn(move || {
                    if crate::subtitle::boxes().is_none() {
                        crate::ocr::begin_pick(&app, crate::ocr::Pick::SubtitleArea);
                    } else if let Err(error) = crate::subtitle::begin_edit(&app) {
                        crate::log::note!(
                            "glossy: the subtitle boxes could not be opened: {error}"
                        );
                    }
                });
            }
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

/// Writes the shortcuts the settings hold into the menu again.
///
/// The menu is built when the app starts and holds the combinations of that
/// moment; a shortcut recorded afterwards would otherwise leave the menu naming
/// the key it replaced until the next launch. Called when the settings are
/// saved, which is where a combination and the two languages can change.
pub fn refresh(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let settings = Settings::load(app);
    match menu(app, &settings) {
        Ok(menu) => {
            if let Err(error) = tray.set_menu(Some(menu)) {
                crate::log::note!("Glossy could not put its shortcuts into the tray menu: {error}");
            }
        }
        Err(error) => {
            crate::log::note!("Glossy could not build its tray menu: {error}");
        }
    }
}

/// Translates what is on the clipboard.
///
/// The shortcut does the same thing from inside another program, where it also
/// presses Ctrl+C to pick up whatever is selected. A menu is clicked after the
/// user has copied something themselves, so what is on the clipboard is what
/// they mean by it — and it needs no window to be in front, which a menu click
/// cannot promise.
fn translate_clipboard(app: &AppHandle) {
    let Some(state) = app.try_state::<Arc<AppState>>() else {
        return;
    };
    let state = state.inner().clone();
    let settings = state.settings();
    let language = settings::resolve_ui_language(settings.ui_lang);
    // A click that does nothing at all reads as a broken menu, so either reason
    // it could not go ahead is said in the card the user was expecting.
    if !settings.enabled {
        crate::popup::fail(app, translations_off(language));
        return;
    }
    let text = crate::platform::clipboard::read_text().unwrap_or_default();
    if !crate::selection::translate_text(app, &state, &text) {
        crate::popup::fail(app, nothing_to_translate(language));
    }
}

fn translations_off(language: UiLanguage) -> &'static str {
    match language {
        UiLanguage::Chinese => "翻译已经关掉了，先在设置里打开 Glossy 再试。",
        _ => "Translation is switched off — turn Glossy on in the settings first.",
    }
}

fn nothing_to_translate(language: UiLanguage) -> &'static str {
    match language {
        UiLanguage::Chinese => "剪贴板里没有可以翻译的文字。",
        _ => "There is no text on the clipboard to translate.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_names_the_shortcut_the_settings_hold() {
        assert_eq!(
            with_shortcut("Translate a screenshot", "Ctrl+Alt+Q"),
            "Translate a screenshot (Ctrl+Alt+Q)"
        );
        // A field the user cleared leaves the entry with the action alone.
        assert_eq!(
            with_shortcut("Translate a screenshot", ""),
            "Translate a screenshot"
        );
        assert_eq!(
            with_shortcut("Translate a screenshot", "  "),
            "Translate a screenshot"
        );
    }

    #[test]
    fn the_menu_is_written_in_the_language_the_interface_is_in() {
        assert_eq!(Words::of(UiLanguage::Chinese).ocr, "截图翻译");
        assert_eq!(Words::of(UiLanguage::English).ocr, "Translate a screenshot");
    }
}
