//! Glossy: select text anywhere on the desktop and translate it in a floating
//! popup window.

mod classify;
mod clipboard;
mod hotkey;
mod input;
mod platform;
mod popup;
mod selection;
mod settings;
mod state;
mod translate;

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

use popup::POPUP_LABEL;
use settings::Settings;
use state::AppState;
use translate::TranslationResult;

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
    let settings = settings.sanitized();
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

#[tauri::command]
fn popup_present(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    width: f64,
    height: f64,
) -> Result<(), String> {
    popup::place(&app, &state, width, height, true)
}

#[tauri::command]
fn popup_resize(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    width: f64,
    height: f64,
) -> Result<(), String> {
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
        error: state
            .hook_error
            .lock()
            .ok()
            .and_then(|guard| guard.clone()),
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
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let state = Arc::new(AppState::new(Settings::load(&handle)));
            app.manage(Arc::clone(&state));

            if let Some(popup) = app.get_webview_window(POPUP_LABEL) {
                let _ = popup.hide();
                let _ = popup.set_always_on_top(true);
                // Without WS_EX_NOACTIVATE the popup would steal the focus of
                // the application the user is reading in.
                if let Ok(hwnd) = popup.hwnd() {
                    platform::make_non_activating(platform::Handle(hwnd.0 as isize));
                }
            }

            selection::install(handle, Arc::clone(&state));
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } if window.label() == POPUP_LABEL => {
                api.prevent_close();
                let _ = window.hide();
            }
            // The settings window is the application: closing it quits Glossy.
            WindowEvent::CloseRequested { .. } if window.label() == "main" => {
                window.app_handle().exit(0);
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
        ])
        .run(tauri::generate_context!())
        .expect("Glossy could not start");
}
