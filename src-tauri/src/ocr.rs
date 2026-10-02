//! Reading the text off a part of the screen and translating it.
//!
//! The screenshot translation is the select-to-translate flow with a different
//! source: the shortcut takes a picture of the monitor the cursor is on, the
//! overlay lets the user drag a rectangle over it, and the text read out of that
//! rectangle goes to the popup exactly like a selection does.
//!
//! The picture is taken *before* the overlay goes up, and the rectangle is cut
//! out of that copy. Nothing then depends on how quickly Windows takes the
//! overlay off the screen again, which is what makes this reliable rather than
//! a race against the compositor.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow};

use crate::platform::{self, input, screen::Shot, ScreenRect};
use crate::settings::Settings;
use crate::state::AppState;
use crate::{popup, settings, text};

pub mod models;
mod pipeline;
mod vision;

pub use models::Status;

/// Label of the overlay window that picks the region.
pub const OCR_LABEL: &str = "ocr";

/// Smallest region worth reading, in physical pixels. A stray click would
/// otherwise ask the engine about a couple of pixels of a desktop icon.
pub const MIN_REGION: i64 = 8;

/// Longest a screenshot may stay up, however the page inside it behaves. The
/// overlay covers a monitor and sits above everything; a page that failed to
/// load would otherwise leave the desktop unusable until the process is killed.
const AUTO_CLOSE: Duration = Duration::from_secs(30);

/// How often the way out is looked for while the overlay is up. Short enough to
/// feel immediate, long enough to cost nothing.
const POLL: Duration = Duration::from_millis(50);

/// Which screenshot is the current one. It goes up by one every time an overlay
/// appears or is put away, so a watchdog armed for one screenshot knows it has
/// nothing left to watch once another one is in its place — and the overlay the
/// user is looking at is never closed by the watchdog of a previous one.
static SESSION: AtomicU64 = AtomicU64::new(0);

/// What the rectangle the user is about to drag is wanted for.
///
/// The overlay is the same one in all three cases — a dimmed screen with a band
/// on it — and so is the command it calls back through. What changes is what
/// happens to the rectangle once it is known, and what the page asks the user
/// for while it is being dragged.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    /// Read it once and translate what is in it.
    Once,
    /// The part of the screen subtitles appear in.
    SubtitleArea,
    /// Where the translation of those subtitles is to be drawn.
    SubtitlePlace,
}

impl Pick {
    /// The word the overlay page is told, which is how it knows what to ask for.
    fn name(self) -> &'static str {
        match self {
            Pick::Once => "once",
            Pick::SubtitleArea => "area",
            Pick::SubtitlePlace => "place",
        }
    }
}

/// Held between the shortcut and the rectangle, so the callback knows what it is
/// a rectangle for.
static PICK: Mutex<Pick> = Mutex::new(Pick::Once);

/// The picture the region is cut out of, and how the overlay reports itself.
struct Frozen {
    shot: Shot,
    /// Scale factor of the monitor the picture was taken on, so the rectangle
    /// the page reports in CSS pixels can be found in it.
    scale: f64,
}

/// Held between the shortcut and the rectangle, and dropped as soon as one is
/// known, so a stray second call can never cut the picture twice.
static FROZEN: Mutex<Option<Frozen>> = Mutex::new(None);

/// Where the overlay sits while it is up.
///
/// The picker covers a whole monitor and swallows every press inside it, and
/// the mouse hook has to reach the same answer without taking a lock. Reading
/// it is also what keeps a drag made to mark a region from being read as a
/// text selection in the program behind the overlay, which would spend a
/// translation the user never asked for.
struct OverlayBounds {
    left: AtomicI32,
    top: AtomicI32,
    right: AtomicI32,
    bottom: AtomicI32,
    visible: AtomicBool,
}

static OVERLAY: OverlayBounds = OverlayBounds {
    left: AtomicI32::new(0),
    top: AtomicI32::new(0),
    right: AtomicI32::new(0),
    bottom: AtomicI32::new(0),
    visible: AtomicBool::new(false),
};

fn store_bounds(rect: ScreenRect) {
    OVERLAY.left.store(rect.left, Ordering::Relaxed);
    OVERLAY.top.store(rect.top, Ordering::Relaxed);
    OVERLAY.right.store(rect.right, Ordering::Relaxed);
    OVERLAY.bottom.store(rect.bottom, Ordering::Relaxed);
    OVERLAY.visible.store(true, Ordering::Relaxed);
}

fn clear_bounds() {
    OVERLAY.visible.store(false, Ordering::Relaxed);
}

fn bounds() -> Option<ScreenRect> {
    if !OVERLAY.visible.load(Ordering::Relaxed) {
        return None;
    }
    Some(ScreenRect {
        left: OVERLAY.left.load(Ordering::Relaxed),
        top: OVERLAY.top.load(Ordering::Relaxed),
        right: OVERLAY.right.load(Ordering::Relaxed),
        bottom: OVERLAY.bottom.load(Ordering::Relaxed),
    })
}

fn covers(rect: Option<ScreenRect>, x: i32, y: i32) -> bool {
    rect.is_some_and(|rect| rect.contains_padded(x, y, 0))
}

/// True when the screen point is currently covered by the screenshot overlay.
pub fn contains(x: i32, y: i32) -> bool {
    covers(bounds(), x, y)
}

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(OCR_LABEL)
}

/// Takes the picture, puts the overlay up and waits for the rectangle.
///
/// Called from the shortcut, on a thread of its own: capturing a screen takes
/// long enough that the mouse hook must not be the one waiting for it.
pub fn begin(app: &AppHandle) {
    begin_pick(app, Pick::Once);
}

/// The same, for a rectangle that is wanted for something else.
///
/// The subtitles ask for two rectangles in a row — the area to read and the
/// place to draw — and both are the same overlay with a different sentence on
/// it, so they come through here rather than through a capture of their own.
pub fn begin_pick(app: &AppHandle, pick: Pick) {
    let Some(window) = window(app) else {
        return fail(app, "The screenshot window is not available.");
    };
    if let Ok(mut guard) = PICK.lock() {
        *guard = pick;
    }
    // A second press of the shortcut, or the button in the settings window
    // while an overlay is somehow still up, puts that one away instead of
    // taking a picture behind it. This is also the escape hatch when the page
    // inside the overlay did not come up: the shortcut is handled out here, in
    // the process, and does not depend on anything the page does.
    if window.is_visible().unwrap_or(false) {
        return hide(app);
    }
    let (cursor_x, cursor_y) = platform::desktop::cursor_pos();
    let monitors: Vec<(ScreenRect, f64)> = window
        .available_monitors()
        .unwrap_or_default()
        .into_iter()
        .map(|monitor| {
            let origin = monitor.position();
            let size = monitor.size();
            (
                ScreenRect {
                    left: origin.x,
                    top: origin.y,
                    right: origin.x + size.width as i32,
                    bottom: origin.y + size.height as i32,
                },
                monitor.scale_factor(),
            )
        })
        .collect();
    // A monitor list that came back empty is also a failure worth reporting:
    // the region the user is about to pick could not be placed anywhere.
    let Some((rect, scale)) = platform::screen::monitor_of(&monitors, cursor_x, cursor_y) else {
        return fail(app, "The screen under the cursor could not be found.");
    };

    let shot = match platform::screen::grab(rect) {
        Ok(shot) => shot,
        Err(error) => return fail(app, &error),
    };
    if let Ok(mut guard) = FROZEN.lock() {
        *guard = Some(Frozen { shot, scale });
    }

    // The overlay covers the screen the cursor is on and nothing else.
    let _ = window.set_position(PhysicalPosition::new(rect.left, rect.top));
    let _ = window.set_size(PhysicalSize::new(
        (rect.right - rect.left) as u32,
        (rect.bottom - rect.top) as u32,
    ));
    let _ = window.show();
    let _ = window.set_focus();
    store_bounds(rect);
    // The page keeps its state between screenshots, so it is told that a fresh
    // one has begun and nothing of the last rectangle is left over — and which
    // of the three things this rectangle is for.
    let _ = window.emit("glossy://ocr", pick.name());
    watch(app, SESSION.fetch_add(1, Ordering::SeqCst) + 1);
}

/// The way out of the overlay that does not go through the page inside it.
///
/// The overlay is transparent, covers a whole monitor and is always on top, so
/// it is also what receives the clicks and the keys while it is up. That is fine
/// while the page is running — it cancels on Escape, on a right click and on a
/// click that drags nothing — but a page that never loaded cancels nothing at
/// all, and the desktop is then covered by something that cannot be dismissed.
///
/// So Escape and the right button are also watched from here, where the process
/// is, and the overlay is put away on its own after `AUTO_CLOSE` whatever the
/// page does. The watchdog stops as soon as the next screenshot takes over.
fn watch(app: &AppHandle, session: u64) {
    let app = app.clone();
    std::thread::spawn(move || {
        let deadline = Instant::now() + AUTO_CLOSE;
        loop {
            let asked_to_close = input::escape_is_down() || input::right_button_is_down();
            let decision = step(
                session,
                SESSION.load(Ordering::SeqCst),
                asked_to_close,
                Instant::now() >= deadline,
            );
            match decision {
                Step::Wait => std::thread::sleep(POLL),
                Step::Close => {
                    hide(&app);
                    return;
                }
                Step::Retire => return,
            }
        }
    });
}

/// What one look at the overlay's state means for its watchdog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// Still up, and nothing asked it to go away.
    Wait,
    /// The user asked for it to go, or it has been up for long enough.
    Close,
    /// This watchdog is watching a screenshot that is no longer the current
    /// one: the page closed it, or another screenshot replaced it.
    Retire,
}

/// `session` is the screenshot this watchdog was armed for and `current` the one
/// that is up now.
fn step(session: u64, current: u64, asked_to_close: bool, timed_out: bool) -> Step {
    if session != current {
        Step::Retire
    } else if asked_to_close || timed_out {
        Step::Close
    } else {
        Step::Wait
    }
}

/// Puts the overlay away and forgets the picture it was showing.
pub fn hide(app: &AppHandle) {
    // Whatever watchdog was armed for the screenshot that is ending has nothing
    // left to do, and the one the next screenshot arms starts from here.
    SESSION.fetch_add(1, Ordering::SeqCst);
    clear_bounds();
    if let Ok(mut guard) = FROZEN.lock() {
        *guard = None;
    }
    if let Some(window) = window(app) {
        let _ = window.hide();
    }
}

/// Tells the user what went wrong, in the card they were expecting an answer in,
/// and puts the overlay away.
/// Reports a reading that could not be made, in the card the user was
/// expecting. `pub` because the subtitle reading fails into the same card.
pub fn fail(app: &AppHandle, message: &str) {
    hide(app);
    // The card that reports the failure takes the place of the one the user was
    // expecting, so it is shown where the pointer is.
    if let Some(state) = app.try_state::<Arc<AppState>>() {
        let (x, y) = platform::desktop::cursor_pos();
        state.set_anchor((x as f64, y as f64));
    }
    let _ = app.emit("glossy://popup-error", message.to_string());
}

/// Starts a screenshot from inside the settings window, where the same flow is
/// offered as a button next to the shortcut.
#[tauri::command]
pub fn ocr_start(app: AppHandle) {
    begin(&app);
}

/// How long the card is given to leave the screen before the picture is taken.
///
/// Hiding a window takes effect in the frame the desktop is composited from, and
/// the picture is taken from a screen that has already been drawn: without a
/// moment in between, the card the user just clicked would be in the picture they
/// are about to draw a rectangle on.
const CARD_SETTLE: Duration = Duration::from_millis(120);

/// Starts a screenshot from the button in the card itself.
///
/// The card is always on top and normally sits right next to the text the user
/// wants to read from the screen, so it is taken off the screen first — and only
/// for as long as the reading takes: the translation, or the reason there is none,
/// comes back to the same card.
#[tauri::command]
pub fn ocr_start_from_card(app: AppHandle) {
    crate::popup::hide(&app);
    // Off the main thread, like the shortcut: the wait and the picture are not
    // worth stalling the interface for.
    std::thread::spawn(move || {
        std::thread::sleep(CARD_SETTLE);
        begin(&app);
    });
}

/// Abandons the screenshot; the Escape key and a click that selects nothing go
/// through here.
#[tauri::command]
pub fn ocr_cancel(app: AppHandle) {
    hide(&app);
}

/// Translates the text inside the rectangle the user dragged.
///
/// `x`, `y`, `width` and `height` are the rectangle in the CSS pixels of the
/// overlay, which is how the page measures a drag.
#[tauri::command]
pub async fn ocr_region(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let Some(frozen) = FROZEN.lock().ok().and_then(|mut guard| guard.take()) else {
        hide(&app);
        return Err("The screenshot is no longer available.".to_string());
    };
    // The overlay has to be gone before anything is shown in its place.
    hide(&app);

    let origin = frozen.shot.origin;
    let rect = ScreenRect {
        left: origin.0 + (x * frozen.scale).round() as i32,
        top: origin.1 + (y * frozen.scale).round() as i32,
        right: origin.0 + ((x + width) * frozen.scale).round() as i32,
        bottom: origin.1 + ((y + height) * frozen.scale).round() as i32,
    };
    let width = (rect.right - rect.left) as i64;
    let height = (rect.bottom - rect.top) as i64;
    if width < MIN_REGION || height < MIN_REGION {
        return Ok(());
    }

    let pick = PICK.lock().map(|guard| *guard).unwrap_or(Pick::Once);
    match pick {
        // The subtitles ask for two rectangles and translate neither of them
        // here: the reading starts once the second one is known.
        Pick::SubtitleArea => {
            crate::subtitle::area_picked(&app, rect);
            return Ok(());
        }
        Pick::SubtitlePlace => {
            crate::subtitle::start(&app, &state, rect);
            return Ok(());
        }
        Pick::Once => {}
    }

    // Cropping a screenshot is a few tens of milliseconds of plain arithmetic,
    // and the answers the app is waiting for must not be held up by it.
    let image = tauri::async_runtime::spawn_blocking(move || {
        frozen
            .shot
            .crop(rect)
            .map(|shot| (shot.pixels, shot.width, shot.height))
    })
    .await
    .map_err(|error| {
        let message = format!("The screenshot could not be taken: {error}");
        fail(&app, &message);
        message
    })?;
    let (pixels, width, height) = match image {
        Ok(image) => image,
        Err(error) => {
            fail(&app, &error);
            return Err(error);
        }
    };

    // The card that will hold the translation is put up as soon as the
    // rectangle is accepted, because reading it takes long enough for an empty
    // screen to look like nothing happened. What it says depends on what the
    // wait is for: the first screenshot of a language has to fetch the models
    // first, which is the longer of the two.
    let wanted = settings::Settings::load(&app).ocr_packs;
    let installed = languages_ready(&app, &wanted);
    let (cursor_x, cursor_y) = platform::desktop::cursor_pos();
    state.set_anchor((cursor_x as f64, cursor_y as f64));
    let _ = app.emit(
        "glossy://popup-wait",
        if installed.is_empty() {
            "engine"
        } else {
            "reading"
        },
    );
    // None of the checked languages is on the disk: the first screenshot asks
    // about the download, as it always has, and the rest of the checked ones
    // wait until the page has fetched them.
    if installed.is_empty() {
        let first = wanted
            .first()
            .cloned()
            .unwrap_or_else(|| models::DEFAULT_PACK.to_string());
        if let Err(error) = ensure_reader(&app, &first).await {
            fail(&app, &error);
            return Err(error);
        }
    }
    let installed = languages_ready(&app, &wanted);
    let paths: Vec<models::Paths> = installed
        .iter()
        .filter_map(|pack| models::paths(&app, pack).ok())
        .collect();
    if paths.is_empty() {
        let message = "The text reader is not installed.".to_string();
        fail(&app, &message);
        return Err(message);
    }
    let recognized = tauri::async_runtime::spawn_blocking(move || {
        pipeline::recognize(&paths, &pixels, width, height)
    })
    .await
    .map_err(|error| {
        let message = format!("The text could not be read: {error}");
        fail(&app, &message);
        message
    })?;
    let recognized = match recognized {
        Ok(recognized) => recognized,
        Err(error) => {
            fail(&app, &error);
            return Err(error);
        }
    };

    let recognized = recognized
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let recognized = text::normalize(&recognized);
    if recognized.trim().is_empty() {
        let message = "No text was found in that part of the screen.".to_string();
        fail(&app, &message);
        return Err(message);
    }

    // The same entry point the selection uses: the card appears under the
    // cursor and translates straight away, because the user asked for it when
    // they dragged the rectangle.
    let (cursor_x, cursor_y) = platform::desktop::cursor_pos();
    popup::reveal(
        &app,
        &state,
        recognized,
        None,
        (cursor_x as f64, cursor_y as f64),
        true,
    );
    Ok(())
}

/// The languages whose files are on the disk, in the order the page shows them,
/// so a screenshot is read with every one of them that is there.
fn languages_ready(app: &AppHandle, wanted: &[String]) -> Vec<String> {
    wanted
        .iter()
        .filter(|pack| models::ready(app, pack))
        .cloned()
        .collect()
}

/// The one language subtitle reading uses.
///
/// The user picks it, and a name whose files are not on the disk falls back to
/// the first checked language that is: a run that cannot read at all is worse
/// than one read in the script it was not asked for.
pub fn subtitle_pack(app: &AppHandle, settings: &Settings) -> String {
    if models::ready(app, &settings.subtitle_pack) {
        return settings.subtitle_pack.clone();
    }
    settings
        .ocr_packs
        .iter()
        .find(|pack| models::ready(app, pack))
        .cloned()
        .unwrap_or_else(|| settings.subtitle_pack.clone())
}

/// Reads one picture with one language, which is what the subtitle loop does
/// every tick.
///
/// One language rather than every checked one: the region is read again and
/// again, and a subtitle is only ever written in one of them.
pub fn recognize_one(
    paths: &models::Paths,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<String>, String> {
    Ok(
        pipeline::recognize(std::slice::from_ref(paths), pixels, width, height)?
            .into_iter()
            .map(|line| line.text)
            .collect(),
    )
}

/// The model files are a download of their own, so the first screenshot asks
/// about them instead of failing. `pack` is the language the settings asked for;
/// its two files are fetched along with whatever of the shared ones is missing.
/// `Ok` means the engine is ready to be used; anything else is a message for the
/// user, and the screenshot is abandoned.
async fn ensure_reader(app: &AppHandle, pack: &str) -> Result<(), String> {
    if models::ready(app, pack) {
        return Ok(());
    }
    // A download that is already under way is waited for instead of being asked
    // about a second time.
    if models::status(app).downloading {
        return wait_for_reader(app, pack).await;
    }

    let language = settings::resolve_ui_language(settings::stored_ui_language());
    let agreed = {
        use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
        // The size the prompt names is what this machine is missing, so a second
        // language is not presented as a second full engine.
        let megabytes = models::missing_bytes(app, pack).div_ceil(1024 * 1024);
        let wording = prompt_wording(language, megabytes);
        app.dialog()
            .message(&wording.body)
            .title(wording.title)
            .kind(MessageDialogKind::Info)
            .buttons(MessageDialogButtons::OkCancelCustom(
                wording.accept.to_string(),
                wording.cancel.to_string(),
            ))
            .blocking_show()
    };
    if !agreed {
        return Err(declined_wording(language).to_string());
    }
    models::download(app, pack).await?;
    wait_for_reader(app, pack).await
}

/// Waits out a download that is already running.
async fn wait_for_reader(app: &AppHandle, pack: &str) -> Result<(), String> {
    loop {
        let status = models::status(app);
        if models::ready(app, pack) {
            return Ok(());
        }
        if !status.downloading {
            return Err(status
                .error
                .unwrap_or_else(|| "The text reader is not installed.".to_string()));
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// What the first-use prompt says, in the language the app is set to.
struct Wording {
    title: &'static str,
    body: String,
    accept: &'static str,
    cancel: &'static str,
}

fn prompt_wording(language: settings::UiLanguage, megabytes: u64) -> Wording {
    match language {
        settings::UiLanguage::Chinese => Wording {
            title: "本地文字识别",
            body: format!(
                "截屏翻译需要在本机读取文字。第一次使用要下载识别引擎和使用的那一种文字的模型，共约 {megabytes} MB（下载后一直保存在本机，不再联网）。现在下载吗？"
            ),
            accept: "下载",
            cancel: "取消",
        },
        _ => Wording {
            title: "Local text recognition",
            body: format!(
                "Screenshot translation reads the text on this machine. The first use downloads the recognition engine and the model of the language it reads with — about {megabytes} MB, kept on this machine afterwards and never uploaded. Download it now?"
            ),
            accept: "Download",
            cancel: "Cancel",
        },
    }
}

/// Told when the user turns the download down.
fn declined_wording(language: settings::UiLanguage) -> &'static str {
    match language {
        settings::UiLanguage::Chinese => {
            "截屏翻译需要先下载本机文字识别引擎。可以在设置窗口的“资源”页面里下载。"
        }
        _ => {
            "Screenshot translation needs the local text recognition engine. You can download it on the Resources page of the settings window."
        }
    }
}

/// What the settings page is told about the engine.
#[tauri::command]
pub fn ocr_model_status(app: AppHandle) -> Status {
    models::status(&app)
}

/// Downloads one language pack, or joins a download that is already running.
///
/// `pack` names the language to end up ready to read with; without it the first
/// of the languages the settings name is fetched, which is what the first-use
/// prompt asks for.
#[tauri::command]
pub async fn ocr_model_download(app: AppHandle, pack: Option<String>) -> Result<Status, String> {
    let packs = settings::Settings::load(&app).ocr_packs;
    let wanted = pack
        .filter(|id| models::offered(id))
        .or_else(|| packs.first().cloned())
        .unwrap_or_else(|| models::DEFAULT_PACK.to_string());
    // The button and the first-use prompt can be racing, so a download that is
    // already going is waited for rather than answered with a refusal.
    if models::status(&app).downloading {
        wait_for_reader(&app, &wanted).await?;
    } else {
        models::download(&app, &wanted).await?;
    }
    Ok(models::status(&app))
}

/// Throws one language pack away, or every file with `pack` left out, so the
/// user can get the space back.
#[tauri::command]
pub fn ocr_model_remove(app: AppHandle, pack: Option<String>) -> Status {
    let wanted = pack.filter(|id| models::offered(id));
    // A file the engine still holds open is not a reason to report the removal
    // as failed: the state that is answered below is what the page draws.
    let _ = models::remove(&app, wanted.as_deref());
    models::status(&app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_watchdog_waits_while_its_own_screenshot_is_up() {
        assert_eq!(step(7, 7, false, false), Step::Wait);
    }

    #[test]
    fn the_watchdog_closes_on_escape_a_right_click_or_the_time_limit() {
        assert_eq!(step(7, 7, true, false), Step::Close);
        assert_eq!(step(7, 7, false, true), Step::Close);
    }

    #[test]
    fn the_watchdog_of_a_put_away_screenshot_never_closes_another_one() {
        // The page, the shortcut or the user closed the overlay it was armed
        // for: `hide` moved the session on, so the next one is none of its
        // business even if the way out is still being held down.
        assert_eq!(step(7, 8, true, false), Step::Retire);
        assert_eq!(step(7, 8, false, true), Step::Retire);
    }

    #[test]
    fn only_the_screen_the_overlay_covers_counts_as_inside_it() {
        let screen = ScreenRect {
            left: 0,
            top: 0,
            right: 2560,
            bottom: 1600,
        };
        assert!(covers(Some(screen), 1200, 800));
        assert!(covers(Some(screen), 0, 1599));
        assert!(!covers(Some(screen), 2560, 800));
        assert!(!covers(Some(screen), -1, 800));
    }

    #[test]
    fn a_put_away_overlay_covers_nothing() {
        // The window is kept around and only hidden between screenshots, so
        // `contains` has to answer for the flag and not for the window.
        assert!(!covers(None, 1200, 800));
    }
}
