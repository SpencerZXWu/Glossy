//! Glossy: select text anywhere on the desktop and translate it in a floating
//! popup window.

mod classify;
mod clipboard;
pub mod console;
mod hotkey;
mod input;
mod instance;
mod notice;
mod platform;
mod popup;
mod secrets;
mod selection;
mod settings;
mod state;
mod translate;
mod tray;

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

use notice::{notice_close, notice_open, NOTICE_LABEL};
use popup::POPUP_LABEL;
use settings::Settings;
use state::AppState;
use translate::TranslationResult;

/// Label of the settings window, which now doubles as the application window.
pub const MAIN_LABEL: &str = "main";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureStatus {
    /// Whether the global mouse hook is listening.
    hooked: bool,
    error: Option<String>,
    /// Canonical spelling of the registered global hotkey, when it is active.
    hotkey: Option<String>,
    /// Why the configured hotkey could not be registered.
    hotkey_error: Option<String>,
}

#[tauri::command]
fn get_settings(state: State<'_, Arc<AppState>>) -> Settings {
    state.settings()
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: Settings,
) -> Result<Settings, String> {
    let mut settings = settings.sanitized();
    // Only the very first launch opens the settings window, and that launch is
    // over by now, so the interface never gets to ask for it again.
    settings.first_run = false;
    settings.save(&app)?;
    state.set_settings(settings.clone());

    if !settings.enabled {
        popup::hide(&app);
    }
    // The accelerator lives on the hook thread, which re-reads it on demand.
    hotkey::request_reload();
    let _ = app.emit("glossy://settings", settings.clone());
    Ok(settings)
}

/// Translates a selection. Called by the popup window and by the demo pane.
/// `source_lang` / `target_lang` are optional overrides coming from the popup's
/// language bar; both fall back to the settings when they are missing.
#[tauri::command]
async fn translate_text(
    state: State<'_, Arc<AppState>>,
    text: String,
    source_lang: Option<String>,
    target_lang: Option<String>,
) -> Result<TranslationResult, String> {
    let settings = state.settings();
    let languages = translate::Languages {
        source: source_lang,
        target: target_lang,
    };
    translate::translate(&text, &settings, &languages).await
}

/// Shows the floating popup for `text`, anchored to the mouse cursor. The demo
/// pane uses this to exercise the popup without a global text selection.
#[tauri::command]
fn show_popup(app: AppHandle, state: State<'_, Arc<AppState>>, text: String) {
    let (x, y) = platform::cursor_pos();
    popup::reveal(&app, &state, text, (x as f64, y as f64));
}

/// Sizes and shows the popup; returns the usable height of its monitor in CSS
/// pixels so the interface can cap the card to that screen.
#[tauri::command]
fn popup_present(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    width: f64,
    height: f64,
) -> Result<f64, String> {
    popup::place(&app, &state, width, height, true)
}

/// Sizes the popup that is already on screen; returns the usable height of its
/// monitor in CSS pixels.
#[tauri::command]
fn popup_resize(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    width: f64,
    height: f64,
) -> Result<f64, String> {
    popup::place(&app, &state, width, height, false)
}

/// Called after the user dragged the popup so later resizes keep it in place.
#[tauri::command]
fn popup_sync_anchor(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    popup::sync_anchor(&app, &state)
}

#[tauri::command]
fn popup_close(app: AppHandle) {
    popup::hide(&app);
}

#[tauri::command]
fn copy_text(text: String) -> bool {
    clipboard::copy_to_clipboard(&text)
}

#[tauri::command]
fn capture_status(state: State<'_, Arc<AppState>>) -> CaptureStatus {
    let (hotkey, hotkey_error) = hotkey::status();
    CaptureStatus {
        hooked: state.hooked.load(std::sync::atomic::Ordering::Relaxed),
        error: state.hook_error.lock().ok().and_then(|guard| guard.clone()),
        hotkey,
        hotkey_error,
    }
}

/// One program that currently owns a visible window.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunningApp {
    /// Executable file name, e.g. `chrome.exe`.
    name: String,
    /// Title of the window that represents the program.
    title: String,
}

/// Programs the user can pick from for the "never translate here" list.
#[tauri::command]
fn running_apps() -> Vec<RunningApp> {
    platform::visible_apps()
        .into_iter()
        .map(|(name, title)| RunningApp { name, title })
        .collect()
}

/// Arms the picker: the next click anywhere reports the program under it
/// through the `glossy://picked-app` event.
#[tauri::command]
fn pick_app() {
    selection::arm_pick();
}

pub fn run() {
    // Two instances would install two mouse hooks and race over one popup.
    let _guard = match instance::claim() {
        instance::Claim::First(guard) => Some(guard),
        instance::Claim::Taken => {
            instance::report_already_running();
            return;
        }
        instance::Claim::Unavailable => None,
    };

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let mut settings = Settings::load(&handle);
            let language = settings::resolve_ui_language(settings.ui_lang);
            // Only the very first launch ever opens the settings window; every
            // later one starts quietly in the notification area.
            let first_run = settings.first_run;
            if first_run {
                settings.first_run = false;
                // Recorded before the window is shown, so a crash later on still
                // leaves Glossy starting quietly next time. A failed write only
                // means the window opens again on the next launch.
                let _ = settings.save(&handle);
            }
            let state = Arc::new(AppState::new(settings));
            app.manage(Arc::clone(&state));

            if let Err(error) = tray::install(&handle, language) {
                eprintln!("Glossy could not add its notification area icon: {error}");
            }

            if let Some(popup) = app.get_webview_window(POPUP_LABEL) {
                let _ = popup.hide();
                let _ = popup.set_always_on_top(true);
                // Without WS_EX_NOACTIVATE the popup would steal the focus of
                // the application the user is reading in.
                if let Ok(hwnd) = popup.hwnd() {
                    platform::make_non_activating(platform::Handle(hwnd.0 as isize));
                }
            }

            if let Some(hint) = app.get_webview_window(NOTICE_LABEL) {
                let _ = hint.hide();
                let _ = hint.set_always_on_top(true);
                // Clicking the hint must not pull the focus out of whatever the
                // user is doing while it is on screen.
                if let Ok(hwnd) = hint.hwnd() {
                    platform::make_non_activating(platform::Handle(hwnd.0 as isize));
                }
            }

            if first_run {
                tray::show_main(&handle);
            } else {
                // Nothing else would tell the user that Glossy came up: its icon
                // usually sits in the overflow of the notification area.
                notice::show(&handle);
            }

            selection::install(handle, Arc::clone(&state));
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } if window.label() == POPUP_LABEL => {
                api.prevent_close();
                let _ = window.hide();
            }
            // Dragging the popup moves its hit box with it.
            WindowEvent::Moved(position) if window.label() == POPUP_LABEL => {
                popup::track_move(position.x, position.y);
            }
            // Closing the settings window only hides it: Glossy keeps watching
            // for selections and stays reachable through the notification area.
            WindowEvent::CloseRequested { api, .. } if window.label() == MAIN_LABEL => {
                api.prevent_close();
                // The page writes a change 180ms after the last edit, which can
                // be later than this click: ask it to write right away.
                let _ = window.emit("glossy://flush-settings", ());
                let _ = window.hide();
            }
            // The hint is a status message, not a window the user manages.
            WindowEvent::CloseRequested { api, .. } if window.label() == NOTICE_LABEL => {
                api.prevent_close();
                notice::hide(window.app_handle());
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            translate_text,
            show_popup,
            popup_present,
            popup_resize,
            popup_sync_anchor,
            popup_close,
            copy_text,
            capture_status,
            running_apps,
            pick_app,
            notice_open,
            notice_close,
        ])
        .run(tauri::generate_context!())
        .expect("Glossy could not start");
}
