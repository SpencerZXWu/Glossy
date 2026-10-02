//! Glossy: select text anywhere on the desktop and translate it in a floating
//! popup window.

mod autostart;
mod classify;
mod context;
mod document;
mod docx;
mod history;
mod lang;
mod log;
mod morphology;
mod notice;
mod ocr;
pub mod offline;
pub mod platform;
mod popup;
mod selection;
mod settings;
mod state;
mod subtitle;
mod surface;
mod text;
mod timing;
mod translate;
mod tray;
mod units;
mod updater;
mod vitals;
mod vocabulary;

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

use notice::{notice_close, notice_open, NOTICE_LABEL};
use popup::POPUP_LABEL;
use settings::{Service, Settings};
use state::AppState;
use translate::TranslationResult;

/// Label of the settings window, which now doubles as the application window.
pub const MAIN_LABEL: &str = "main";

/// The key that turns the features which are still being built on.
///
/// It is checked in the process rather than in the window, so the window never
/// holds it and a page that was edited cannot turn the switch on by itself. It
/// is not a secret — anyone who reads the binary can have it, and it guards
/// nothing but a page of half-finished work — but it keeps those pages out of
/// the way of a build that is handed to somebody else.
const DEVELOPER_KEY: &str = "sPencer0308f";

/// Turns the pages that are still being built on, for the one who knows the key.
///
/// The settings come back so the window can redraw itself around the switch in
/// one step instead of asking for them again.
#[tauri::command]
fn developer_unlock(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    key: String,
) -> Result<Settings, String> {
    if key.trim() != DEVELOPER_KEY {
        log::note!("glossy: a developer key was tried that does not match");
        return Err("dev.key".to_string());
    }
    let mut settings = state.settings();
    settings.developer_mode = true;
    settings.save(&app)?;
    state.set_settings(settings.clone());
    log::note!("glossy: developer mode is on");
    let _ = app.emit("glossy://settings", settings.clone());
    Ok(settings)
}

/// Puts the pages that are still being built away again.
#[tauri::command]
fn developer_lock(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    subtitle::stop(&app);
    let mut settings = state.settings();
    settings.developer_mode = false;
    settings.save(&app)?;
    state.set_settings(settings.clone());
    let _ = app.emit("glossy://settings", settings.clone());
    Ok(settings)
}

/// Picks the two rectangles the subtitle reading needs and starts it.
///
/// The first rectangle is the area the subtitles are in, the second is where
/// their translation is drawn; both come back through the overlay's own
/// command, which is why this only sets the first one going.
#[tauri::command]
fn subtitle_start(app: AppHandle) {
    if !app.state::<Arc<AppState>>().settings().developer_mode {
        log::note!("glossy: a subtitle reading was asked for with developer mode off");
        return;
    }
    ocr::begin_pick(&app, ocr::Pick::SubtitleArea);
}

/// Stops a subtitle reading and takes the translation off the screen.
#[tauri::command]
fn subtitle_stop(app: AppHandle) {
    subtitle::stop(&app);
}

/// Opens the box editor, or asks for the two rectangles when there are none yet.
///
/// The boxes are what a reading is made of, so this is also the way to make one
/// from scratch: with nothing picked there is nothing to move, and the two
/// rectangles are asked for the way the page asks for them.
#[tauri::command]
fn subtitle_edit_start(app: AppHandle) -> Result<(), String> {
    if !app.state::<Arc<AppState>>().settings().developer_mode {
        log::note!("glossy: the subtitle boxes were asked for with developer mode off");
        return Err("dev.key".to_string());
    }
    if subtitle::boxes().is_none() {
        ocr::begin_pick(&app, ocr::Pick::SubtitleArea);
        return Ok(());
    }
    subtitle::begin_edit(&app)
}

/// Puts the box editor away without moving either box.
#[tauri::command]
fn subtitle_edit_cancel(app: AppHandle) {
    subtitle::edit_cancel(&app);
}

/// Moves both boxes to where the editor left them.
#[tauri::command]
fn subtitle_edit_apply(app: AppHandle, area: subtitle::CssBox, place: subtitle::CssBox) {
    subtitle::edit_apply(&app, area, place);
}

/// Whether subtitles are being read right now.
#[tauri::command]
fn subtitle_status() -> subtitle::Status {
    subtitle::Status {
        running: subtitle::running(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureStatus {
    /// Whether the global mouse hook is listening.
    hooked: bool,
    error: Option<String>,
    /// Registration of every global accelerator.
    hotkeys: Vec<platform::hotkey::HotkeyStatus>,
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
                log::note!("Glossy could not change its login item: {error}");
                before.autostart
            }
        };
    }
    if settings.history_limit != before.history_limit {
        history::set_limit(&app, settings.history_limit);
    }
    settings.save(&app)?;
    state.set_settings(settings.clone());

    // The tray menu names the shortcuts the settings hold, and one of them can
    // have just been recorded or cleared.
    tray::refresh(&app);

    if !settings.enabled {
        popup::hide(&app);
    }
    // The accelerator lives on the hook thread, which re-reads it on demand.
    platform::hotkey::request_reload();
    let _ = app.emit("glossy://settings", settings.clone());
    let _ = app.emit("glossy://history", ());
    Ok(settings)
}

/// Writes the settings to `Documents\glossy-settings.json` and answers with the
/// path it used. The file is plain JSON, so nothing secret ever goes into it.
#[tauri::command]
fn export_settings(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let settings = state.settings();
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

/// Where the log is, how much it holds and how much it may hold.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LogStatus {
    /// Empty when the log has not been given a file yet, which a window can only
    /// see in the moment between the interface starting and the app's setup.
    path: String,
    size: u64,
    cap: u64,
}

#[tauri::command]
fn log_status() -> LogStatus {
    LogStatus {
        path: log::path()
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        size: log::size(),
        cap: log::CAP,
    }
}

/// Shows the log in File Explorer, so it can be read or handed over.
#[tauri::command]
fn log_open_folder() -> Result<(), String> {
    let path = log::path().ok_or("The log has nowhere to be written yet.")?;
    platform::desktop::open_folder(path)
}

/// Puts the log on the clipboard, for pasting into a message.
#[tauri::command]
fn log_copy() -> Result<(), String> {
    let path = log::path().ok_or("The log has nowhere to be written yet.")?;
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("the log could not be read: {error}"))?;
    if platform::clipboard::copy_to_clipboard(&text) {
        Ok(())
    } else {
        Err("the clipboard did not take the log".to_string())
    }
}

/// Writes the log to `Documents\glossy-log-<unix seconds>.txt` and answers with
/// the path it used, the way an exported settings file does.
///
/// The file is the log alone: it starts with the line the app writes when it
/// starts, which names the version, so whoever reads it knows what it came from.
#[tauri::command]
fn log_export(app: AppHandle) -> Result<String, String> {
    let path = log::path().ok_or("The log has nowhere to be written yet.")?;
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let target = app
        .path()
        .document_dir()
        .map_err(|error| format!("the Documents folder is not available: {error}"))?
        .join(format!("glossy-log-{}.txt", log::now()));
    std::fs::write(&target, text)
        .map_err(|error| format!("could not write {}: {error}", target.display()))?;
    Ok(target.display().to_string())
}

/// Empties the log. What went wrong before this is gone afterwards.
#[tauri::command]
fn log_clear() -> Result<(), String> {
    log::clear()
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
        Some(units::Relay {
            install_id: &settings.cloud_id,
        }),
    )
    .await;
}

/// What the cloud translator still allows today.
///
/// The settings window shows this next to the cloud provider, because the
/// allowance belongs to the server and the app cannot know it otherwise. A
/// failure is reported as an error string the same way a translation failure
/// is, and the window shows it in place of the numbers.
#[tauri::command]
async fn cloud_status(state: State<'_, Arc<AppState>>) -> Result<translate::CloudQuota, String> {
    let settings = state.settings();
    let client = translate::client()?;
    translate::cloud_quota(client, &settings.cloud_id).await
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
    context: Option<String>,
) -> Result<translate::WordDetails, String> {
    let settings = state.settings();
    let languages = translate::Languages {
        source: source_lang,
        target: target_lang,
    };
    let mut details = translate::word_details(&text, &settings, &languages).await?;
    // The sentence was read out of the program in front by the selection hook,
    // while that program still had the focus; translating it belongs here, with
    // the rest of the word lookup.
    details.context = translate::sentence_context(&settings, &languages, context).await;
    history::patch_details(&app, text.trim(), &details);
    vocabulary::patch_details(&app, text.trim(), languages.target.as_deref(), &details);
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
    let (x, y) = platform::desktop::cursor_pos();
    popup::reveal_result(&app, &state, result, (x as f64, y as f64));
    Ok(())
}

/// The wordbook as the settings window lists it, newest first.
#[tauri::command]
fn vocabulary_list() -> Vec<vocabulary::Entry> {
    vocabulary::list()
}

/// Keeps a card in the wordbook, or takes it out again when it is already
/// there. Answers whether it is in the book now, which is the state of the star
/// that asked.
#[tauri::command]
fn vocabulary_toggle(app: AppHandle, result: translate::TranslationResult) -> bool {
    let kept = vocabulary::toggle(&app, &result);
    // The settings window keeps its own copy of the book; it reloads on this so
    // a word kept from the card shows up without a restart.
    let _ = app.emit("glossy://vocabulary", ());
    kept
}

/// Whether a text is kept already, in the language it was translated into.
#[tauri::command]
fn vocabulary_keeps(source_text: String, target_lang: String) -> bool {
    vocabulary::keeps(&source_text, &target_lang)
}

#[tauri::command]
fn vocabulary_remove(app: AppHandle, id: u64) {
    vocabulary::remove(&app, id);
    let _ = app.emit("glossy://vocabulary", ());
}

#[tauri::command]
fn vocabulary_clear(app: AppHandle) {
    vocabulary::clear(&app);
    let _ = app.emit("glossy://vocabulary", ());
}

/// Puts a kept card back into the floating window, anchored to the cursor.
#[tauri::command]
async fn vocabulary_reopen(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    let entry = vocabulary::get(id).ok_or("that card is no longer in the wordbook")?;
    let mut result = entry.result;
    // Cards kept before the unit feature existed have nothing to show, so they
    // are filled in on the way out, the same way the history does it.
    if result.conversions.is_empty() {
        let settings = state.settings();
        attach_conversions(&app, &settings, &mut result).await;
    }
    let (x, y) = platform::desktop::cursor_pos();
    popup::reveal_result(&app, &state, result, (x as f64, y as f64));
    Ok(())
}

/// Shows the floating popup for `text`, anchored to the mouse cursor. The demo
/// pane uses this to exercise the popup without a global text selection.
#[tauri::command]
fn show_popup(app: AppHandle, state: State<'_, Arc<AppState>>, text: String) {
    let (x, y) = platform::desktop::cursor_pos();
    // The demo pane stands in for a selection, so there is no program in front
    // to read a sentence out of. It stands in for the mouse, too, and so leaves
    // the click that asks for the translation to be made on the badge.
    popup::reveal(&app, &state, text, None, (x as f64, y as f64), false);
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

/// Hands the keyboard to the popup, so the editable original in the card can be
/// typed into; the next placement takes it back off.
#[tauri::command]
fn popup_take_focus(app: AppHandle) {
    popup::take_focus(&app);
}

/// Reports that a freshly revealed popup has painted.
///
/// The other end of the measurement the mouse hook starts; it only writes
/// anything down when the run was asked for with `GLOSSY_TIMING`.
#[tauri::command]
fn popup_painted() {
    timing::painted();
}

/// Called when the pin button of the card is used: a pinned card survives the
/// clicks that would otherwise dismiss it.
#[tauri::command]
fn popup_set_pinned(pinned: bool) {
    popup::set_pinned(pinned);
}

/// Called by the card when it starts or stops holding a translation that can be
/// written back over the selection behind it, which is what Ctrl+Enter belongs
/// to: the key is registered for as long as that is true and no longer.
#[tauri::command]
fn popup_set_replace(armed: bool) {
    popup::set_replace(armed);
}

/// The gear button of the card is a request for the settings window. A card
/// that was pinned keeps its place on screen; an unpinned one has already been
/// dismissed by the interface.
#[tauri::command]
fn open_settings(app: AppHandle) {
    // The runtime is spelled out: leaving it to inference makes the macro expand
    // to a never type fallback.
    tray::show_main::<tauri::Wry>(&app);
}

#[tauri::command]
fn copy_text(text: String) -> bool {
    platform::clipboard::copy_to_clipboard(&text)
}

/// Where the newest build of Glossy can be downloaded by hand.
///
/// The address is part of the build rather than a value the window sends: a
/// release that cannot update itself yet is the only thing that ever needs a
/// page opened, and nothing else may be asked for this way.
const RELEASES_PAGE: &str = "https://github.com/SpencerZXWu/Glossy/releases/latest";

/// Opens the releases page in the browser, for a build that cannot update
/// itself and for anyone who would rather read the notes before installing.
#[tauri::command]
fn open_releases_page() -> Result<(), String> {
    platform::desktop::open_url(RELEASES_PAGE)
}

/// Writes `text` over the selection it was translated from.
///
/// Called by the card when the user asks for the translation to take the place
/// of the original — with the button under the translation or with Ctrl+Enter,
/// which the card registers while it is on screen.
///
/// The write is a paste, so it blocks for a moment while the receiving
/// application reads the clipboard; the caller has to be free to keep drawing
/// the card in the meantime, hence the blocking thread.
#[tauri::command]
async fn replace_selection(state: State<'_, Arc<AppState>>, text: String) -> Result<bool, String> {
    let restore = state.settings().restore_clipboard;
    tauri::async_runtime::spawn_blocking(move || {
        platform::clipboard::replace_selection(&text, restore)
    })
    .await
    .map_err(|error| error.to_string())
}

/// Reads text out loud, using the voices Windows already has.
///
/// `language` is the language of the text, so a voice that pronounces it can be
/// chosen; the default voice answers when the machine has none for it.
#[tauri::command]
fn say(
    state: State<'_, Arc<AppState>>,
    text: String,
    language: Option<String>,
) -> Result<(), String> {
    let rate = state.settings().speech_rate;
    platform::speech::speak(&text, rate, language.as_deref())
}

/// Stops the reading that is in progress, if any.
#[tauri::command]
fn stop_speaking() {
    platform::speech::stop();
}

/// Whether the voice is reading something out loud right now, which the card
/// polls to know when its pronunciation buttons should go back to rest.
#[tauri::command]
fn speaking() -> bool {
    platform::speech::is_speaking()
}

/// The translation service the settings currently name.
///
/// The window asks rather than reading the settings file itself, so a `1.x`
/// file is folded into one service in a single place.
#[tauri::command]
fn current_service(state: State<'_, Arc<AppState>>) -> String {
    state.settings().service().id().to_string()
}

/// Which languages each service translates, for both language bars.
///
/// A service that cannot take a language would refuse the request, so the bars
/// leave it out; the tables live in one place rather than in a copy per window.
#[tauri::command]
fn service_languages() -> Vec<translate::languages::ServiceLanguages> {
    translate::languages::table()
}

/// Switches the translation service, exactly as picking another entry in the
/// settings window would.
///
/// Everything else about the settings is left alone, which is what lets the
/// card offer the choice without sending the reader to that window.
#[tauri::command]
fn set_service(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Settings, String> {
    let service = Service::from_id(&id)
        .ok_or_else(|| format!("`{id}` is not a translation service this build offers."))?;
    let mut settings = state.settings();
    if settings.service() == service {
        return Ok(settings);
    }
    settings.set_service(service);
    settings.save(&app)?;
    state.set_settings(settings.clone());
    let _ = app.emit("glossy://settings", settings.clone());
    Ok(settings)
}

/// Remembers the language the reader translated into, so the next selection
/// starts from it instead of from the target the settings used to hold.
///
/// What arrives is stored as it is, whichever service is chosen at the time:
/// the two language bars only ever offer the languages of the service they are
/// showing, so all this has to do is spell the code the way the menus do.
#[tauri::command]
fn set_target_lang(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    code: String,
) -> Result<Settings, String> {
    let code = translate::normalize_lang_code(&code);
    if code.is_empty() {
        return Err("A target language is needed.".to_string());
    }
    let mut settings = state.settings();
    if settings.target_lang == code {
        return Ok(settings);
    }
    settings.target_lang = code;
    settings.save(&app)?;
    state.set_settings(settings.clone());
    let _ = app.emit("glossy://settings", settings.clone());
    Ok(settings)
}

/// What the clipboard holds, for the paste button of the settings window.
#[tauri::command]
fn read_clipboard() -> String {
    platform::clipboard::read_text().unwrap_or_default()
}

#[tauri::command]
fn capture_status(state: State<'_, Arc<AppState>>) -> CaptureStatus {
    CaptureStatus {
        hooked: state.hooked.load(std::sync::atomic::Ordering::Relaxed),
        error: state.hook_error.lock().ok().and_then(|guard| guard.clone()),
        hotkeys: platform::hotkey::status(),
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
    platform::desktop::visible_apps()
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

/// Shows what a start of Glossy shows: the settings window when it is already
/// open, and otherwise the hint in the corner of the screen that says Glossy is
/// up.
fn show_launch_surface(handle: &AppHandle) {
    let open = handle
        .get_webview_window(MAIN_LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    if open {
        tray::show_main(handle);
    } else {
        notice::show(handle);
    }
}

pub fn run() {
    // A crash has to be readable after the window it belonged to is gone; the
    // hook is installed before anything else so that as little as possible can
    // fail unrecorded.
    log::catch_panics();
    // Two instances would install two mouse hooks and race over one popup.
    let _guard = match platform::instance::claim() {
        platform::instance::Claim::First(guard) => Some(Arc::new(guard)),
        platform::instance::Claim::Taken => {
            // The running instance answers by showing what a start of its own
            // shows; only a launch nobody answers has to speak for itself.
            if !platform::instance::announce_launch() {
                platform::instance::report_already_running();
            }
            return;
        }
        platform::instance::Claim::Unavailable => None,
    };
    // A later launch reaches this process through the guard, so the watcher
    // needs a handle on it that outlives `run`.
    let watcher = _guard.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![autostart::FLAG]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            // From here on everything the app reports also lands in a file beside
            // its settings: a build with no console has nowhere else to put it,
            // and the failure that only happened once is the one worth keeping.
            if let Ok(dir) = handle.path().app_config_dir() {
                log::keep(&dir);
            }
            log::note!("Glossy {} started", env!("CARGO_PKG_VERSION"));
            let mut settings = Settings::load(&handle);
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
            vocabulary::load(&handle);

            if let Err(error) = tray::install(&handle) {
                log::note!("Glossy could not add its notification area icon: {error}");
            }

            // The window the subtitle translation is drawn in: it is shown by
            // the reading itself, and it never takes the keyboard or the mouse,
            // because what it lies over is the video being watched.
            if let Some(subtitles) = app.get_webview_window(subtitle::SUBTITLE_LABEL) {
                let _ = subtitles.hide();
                let _ = subtitles.set_always_on_top(true);
                let _ = subtitles.set_ignore_cursor_events(true);
                if let Ok(hwnd) = subtitles.hwnd() {
                    platform::desktop::make_non_activating(platform::desktop::Handle(
                        hwnd.0 as isize,
                    ));
                }
            }

            // The two boxes the reading is made of are drawn in windows of their
            // own: the area is marked out by one that lets every click through,
            // and both are moved in a third that covers the monitor while it is
            // up. Only that third one takes the mouse.
            if let Some(frame) = app.get_webview_window(subtitle::AREA_LABEL) {
                let _ = frame.hide();
                let _ = frame.set_always_on_top(true);
                let _ = frame.set_ignore_cursor_events(true);
                if let Ok(hwnd) = frame.hwnd() {
                    platform::desktop::make_non_activating(platform::desktop::Handle(
                        hwnd.0 as isize,
                    ));
                }
            }
            if let Some(editor) = app.get_webview_window(subtitle::EDIT_LABEL) {
                let _ = editor.hide();
                let _ = editor.set_always_on_top(true);
            }

            // A second launch asks this instance to show itself, which it does
            // the same way a start of its own would. Registered once the icon
            // exists, so there is something to focus either way.
            if let Some(guard) = watcher {
                let handle = handle.clone();
                guard.watch(move || {
                    let handle = handle.clone();
                    let shown = handle.clone();
                    let _ = handle.run_on_main_thread(move || show_launch_surface(&shown));
                });
            }

            if let Some(popup) = app.get_webview_window(POPUP_LABEL) {
                let _ = popup.hide();
                let _ = popup.set_always_on_top(true);
                // Without WS_EX_NOACTIVATE the popup would steal the focus of
                // the application the user is reading in.
                if let Ok(hwnd) = popup.hwnd() {
                    platform::desktop::make_non_activating(platform::desktop::Handle(
                        hwnd.0 as isize,
                    ));
                }
            }

            if let Some(hint) = app.get_webview_window(NOTICE_LABEL) {
                let _ = hint.hide();
                let _ = hint.set_always_on_top(true);
                // Clicking the hint must not pull the focus out of whatever the
                // user is doing while it is on screen.
                if let Ok(hwnd) = hint.hwnd() {
                    platform::desktop::make_non_activating(platform::desktop::Handle(
                        hwnd.0 as isize,
                    ));
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
                    log::note!("Glossy could not repair its login item: {error}");
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
            log_status,
            log_open_folder,
            log_copy,
            log_export,
            log_clear,
            open_releases_page,
            translate_text,
            word_details,
            cloud_status,
            show_popup,
            popup_present,
            popup_painted,
            popup_resize,
            popup_sync_anchor,
            popup_close,
            popup_take_focus,
            popup_set_pinned,
            popup_set_replace,
            open_settings,
            copy_text,
            replace_selection,
            say,
            stop_speaking,
            speaking,
            current_service,
            set_service,
            document::document_cancel,
            document::document_open,
            document::document_pick_directory,
            document::document_save,
            document::document_start,
            service_languages,
            set_target_lang,
            read_clipboard,
            history_list,
            history_clear,
            history_remove,
            history_reopen,
            vocabulary_list,
            vocabulary_toggle,
            vocabulary_keeps,
            vocabulary_remove,
            vocabulary_clear,
            vocabulary_reopen,
            capture_status,
            running_apps,
            pick_app,
            notice_open,
            notice_close,
            ocr::ocr_cancel,
            ocr::ocr_model_download,
            ocr::ocr_model_remove,
            ocr::ocr_model_status,
            offline::offline_model_download,
            offline::offline_model_remove,
            offline::offline_model_status,
            ocr::ocr_region,
            ocr::ocr_start,
            ocr::ocr_start_from_card,
            developer_unlock,
            developer_lock,
            subtitle_edit_apply,
            subtitle_edit_cancel,
            subtitle_edit_start,
            subtitle_start,
            subtitle_stop,
            subtitle_status,
            updater::app_version,
            updater::update_capability,
            updater::check_for_update,
            updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("Glossy could not start");
}
