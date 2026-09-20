//! Glossy: select text anywhere on the desktop and translate it in a floating
//! popup window.

mod autostart;
mod classify;
mod clipboard;
pub mod console;
mod history;
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
mod surface;
mod translate;
mod tray;
mod units;
mod updater;

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
    // The login item lives outside the settings file, so changing it is part of
    // saving. What the system reports back is what gets stored, which keeps the
    // checkbox from claiming a state Windows does not have.
    let before = state.settings();
    if settings.autostart != before.autostart {
        settings.autostart = match autostart::apply(&app, settings.autostart) {
            Ok(actual) => actual,
            Err(error) => {
                eprintln!("Glossy could not change its login item: {error}");
                before.autostart
            }
        };
    }
    if settings.history_limit != before.history_limit {
        history::set_limit(&app, settings.history_limit);
    }
    settings.save(&app)?;
    state.set_settings(settings.clone());

    if !settings.enabled {
        popup::hide(&app);
    }
    // The accelerator lives on the hook thread, which re-reads it on demand.
    hotkey::request_reload();
    let _ = app.emit("glossy://settings", settings.clone());
    let _ = app.emit("glossy://history", ());
    Ok(settings)
}

/// Writes the settings to `Documents\glossy-settings.json` and answers with the
/// path it used. `include_credentials` is asked for separately because the file
/// is plain JSON: without it the export holds everything but the API keys.
#[tauri::command]
fn export_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    include_credentials: bool,
) -> Result<String, String> {
    let mut settings = state.settings();
    if !include_credentials {
        settings.credentials.clear();
    }
    let path = app
        .path()
        .document_dir()
        .map_err(|error| format!("the Documents folder is not available: {error}"))?
        .join("glossy-settings.json");
    let json = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    std::fs::write(&path, json)
        .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    Ok(path.display().to_string())
}

/// Replaces the current settings with those of an exported file. The file holds
/// unprotected keys, so saving it again is what gets them encrypted.
#[tauri::command]
fn import_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    json: String,
) -> Result<Settings, String> {
    save_settings(app, state, Settings::import(&json)?)
}

/// Translates a selection. Called by the popup window and by the demo pane./// `source_lang` / `target_lang` are optional overrides coming from the popup's
/// language bar; both fall back to the settings when they are missing.
#[tauri::command]
async fn translate_text(
    app: AppHandle,
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
    let mut result = translate::translate(&text, &settings, &languages).await?;
    attach_conversions(&app, &settings, &mut result).await;
    history::record(&app, &result, settings.history_limit);
    // The application window keeps its own copy of the history; it reloads on
    // this so a translation made while it was open shows up without a restart.
    let _ = app.emit("glossy://history", ());
    Ok(result)
}

/// Adds the unit conversions of a translation to its result.
///
/// Best effort: the card is complete without them, so a failure (or a machine
/// that is offline) simply leaves the list empty.
async fn attach_conversions(app: &AppHandle, settings: &Settings, result: &mut TranslationResult) {
    if !settings.units_enabled {
        return;
    }
    let Ok(client) = translate::client() else {
        return;
    };
    let cache = app.path().app_config_dir().ok();
    result.conversions = units::conversions_for(
        client,
        &result.source_text,
        &result.translation,
        &result.target_lang,
        cache.as_deref(),
    )
    .await;
}

/// The dictionary style extra of a word: phonetic symbols, meanings and one
/// example.
///
/// Called by the card after it already shows the translation, because both
/// sources answer slowly and the card must not wait for them. The details are
/// also written into the stored history entry, so reopening it shows the same
/// card.
#[tauri::command]
async fn word_details(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    text: String,
    source_lang: Option<String>,
    target_lang: Option<String>,
) -> Result<translate::WordDetails, String> {
    let settings = state.settings();
    let languages = translate::Languages {
        source: source_lang,
        target: target_lang,
    };
    let details = translate::word_details(&text, &settings, &languages).await?;
    history::patch_details(&app, text.trim(), &details);
    Ok(details)
}

/// Every translation this session (and the previous ones) remembers, newest
/// first.
#[tauri::command]
fn history_list() -> Vec<history::Entry> {
    history::list()
}

#[tauri::command]
fn history_clear(app: AppHandle) {
    history::clear(&app);
}

#[tauri::command]
fn history_remove(app: AppHandle, id: u64) {
    history::remove(&app, id);
}

/// Puts an old translation back into the floating card, anchored to the cursor.
#[tauri::command]
async fn history_reopen(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    let entry = history::get(id).ok_or("that translation is no longer in the history")?;
    let mut result = entry.result;
    // Entries recorded before the unit feature existed (or while it was
    // switched off) have nothing to show, so they are filled in on the way out.
    if result.conversions.is_empty() {
        let settings = state.settings();
        attach_conversions(&app, &settings, &mut result).await;
    }
    let (x, y) = platform::cursor_pos();
    popup::reveal_result(&app, &state, result, (x as f64, y as f64));
    Ok(())
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

/// Called when the pin button of the card is used: a pinned card survives the
/// clicks that would otherwise dismiss it.
#[tauri::command]
fn popup_set_pinned(pinned: bool) {
    popup::set_pinned(pinned);
}

#[tauri::command]
fn copy_text(text: String) -> bool {
    clipboard::copy_to_clipboard(&text)
}

/// What the clipboard holds, for the paste button of the settings window.
#[tauri::command]
fn read_clipboard() -> String {
    clipboard::read_text().unwrap_or_default()
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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![autostart::FLAG]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
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

            // Only the settings window has a frame and a desktop behind it. Both
            // calls are best effort: they change how it looks, nothing else.
            surface::prepare(&handle, MAIN_LABEL, state.settings().theme);

            history::load(&handle, state.settings().history_limit);

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
            } else if autostart::started_by_system() {
                // Nobody asked for this start, so it must not put anything on
                // screen; the icon in the notification area is enough.
            } else {
                // Nothing else would tell the user that Glossy came up: its icon
                // usually sits in the overflow of the notification area.
                notice::show(&handle);
            }

            // A login item that points at a build the user since moved or
            // renamed would silently stop working; rewrite it while it is on.
            if state.settings().autostart {
                if let Err(error) = autostart::apply(&handle, true) {
                    eprintln!("Glossy could not repair its login item: {error}");
                }
            }

            // The check runs off the main thread and says nothing unless a newer
            // release really exists.
            if state.settings().check_updates {
                updater::check_in_background(&handle);
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
            surface::surface_info,
            surface::set_window_theme,
            export_settings,
            import_settings,
            translate_text,
            word_details,
            show_popup,
            popup_present,
            popup_resize,
            popup_sync_anchor,
            popup_close,
            popup_set_pinned,
            copy_text,
            read_clipboard,
            history_list,
            history_clear,
            history_remove,
            history_reopen,
            capture_status,
            running_apps,
            pick_app,
            notice_open,
            notice_close,
            updater::update_capability,
            updater::check_for_update,
            updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("Glossy could not start");
}
