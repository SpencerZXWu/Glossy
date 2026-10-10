//! Starting hint, and the one other card that borrows it.
//!
//! The settings window only opens on the very first launch ever, so every later
//! launch is silent: the icon Windows parks in the overflow of the notification
//! area would be the only sign that Glossy is up. This little card appears in the
//! corner that holds that icon for a few seconds, opens the settings window when
//! clicked, and goes away by itself.
//!
//! The same card also carries a line the relay asked every App to pass on. It is
//! the same window with a different sentence on it, and the relay is the only
//! one who can put something there: the App never invents one.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

use crate::platform::{self, ScreenRect};

pub const NOTICE_LABEL: &str = "notice";

/// What the card listens for when the line on it changes.
const EVENT: &str = "glossy://notice";

/// Distance between the hint and the edges of the work area, in the CSS pixels
/// the window is laid out in.
const EDGE_MARGIN: f64 = 16.0;

/// Fallback size in CSS pixels, used until the window reports its own.
const FALLBACK_SIZE: (f64, f64) = (340.0, 104.0);

/// How long the hint stays on screen before it takes itself away.
const LINGER: Duration = Duration::from_millis(6500);

/// Number of the show the hint is meant to be on screen for.
///
/// The window is created once and reused, so the countdown cannot live in the
/// page: its document loads once and a timer armed there would only ever run
/// for the first hint. It is kept here instead, and this counter lets a
/// countdown recognise that a later show has taken over — a start of Glossy
/// while the previous hint is still up must not be cut short by the countdown
/// of the one before it.
static SHOW_EPOCH: AtomicU64 = AtomicU64::new(0);

/// Where the hint sits while it is on screen.
///
/// The mouse hook asks for this on every button release. It only ever holds a
/// copy of a rectangle and is never held while anything else runs, so a short
/// lock is cheaper here than the atomics the popup uses.
static BOUNDS: Mutex<Option<ScreenRect>> = Mutex::new(None);

/// The line an announcement put on the card, or `None` while it is the start
/// hint. Empty is not a line: the card falls back on its own text.
static ANNOUNCEMENT: Mutex<Option<String>> = Mutex::new(None);

/// The last line the relay asked to have shown, for this run.
///
/// The relay sends its announcement with every translation it answers, and this
/// is what keeps a line meant to be read once from being pushed at the user
/// again and again. Only the last one is kept: the same sentence arriving twice
/// in a row is the same news, and anything older has already been read.
#[derive(Default)]
struct Announced {
    last: Option<String>,
}

impl Announced {
    /// The line to put on the card, or `None` when there is nothing new in the
    /// message: empty, or the same sentence the last translation carried.
    fn take(&mut self, message: &str) -> Option<String> {
        let line = message.trim();
        if line.is_empty() || self.last.as_deref() == Some(line) {
            return None;
        }
        self.last = Some(line.to_string());
        Some(line.to_string())
    }
}

static ANNOUNCED: Mutex<Announced> = Mutex::new(Announced { last: None });

/// True when the screen point is covered by the hint.
pub fn contains(x: i32, y: i32) -> bool {
    BOUNDS
        .lock()
        .ok()
        .and_then(|guard| *guard)
        .is_some_and(|rect| rect.contains_padded(x, y, 0))
}

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(NOTICE_LABEL)
}

/// Claims the next show number, retiring the one before it.
fn start_show() -> u64 {
    SHOW_EPOCH.fetch_add(1, Ordering::SeqCst) + 1
}

/// Whether `epoch` is still the show that owns the hint.
fn is_current(epoch: u64) -> bool {
    SHOW_EPOCH.load(Ordering::SeqCst) == epoch
}

/// Ends the current show without hiding the window.
fn retire_show() {
    SHOW_EPOCH.fetch_add(1, Ordering::SeqCst);
}

/// Places the card in the bottom right corner of the screen the cursor is on —
/// the corner that holds the notification area — and shows it, with the start
/// hint on it.
///
/// The countdown that takes it away again is started here, so every show gets a
/// full one however often the window has been on screen before.
pub fn show(app: &AppHandle) {
    set_line(app, None);
    reveal(app);
}

/// Shows a line the relay asked every App to pass on, in place of the hint.
///
/// A line that has already been shown this run is dropped: it comes back with
/// every translation, and repeating it would turn news into a nag.
pub fn announce(app: &AppHandle, message: &str) {
    let Some(line) = ANNOUNCED.lock().ok().and_then(|mut guard| guard.take(message)) else {
        return;
    };
    set_line(app, Some(&line));
    reveal(app);
}

/// Puts a different sentence on the card and tells the window to read it.
///
/// The window asks for the line afterwards instead of being handed it here: it
/// exists from the first launch, and a line announced while its document is
/// still loading would reach nobody, while it is still here to be asked for.
fn set_line(app: &AppHandle, line: Option<&str>) {
    let next = line.map(str::to_string);
    let changed = match ANNOUNCEMENT.lock() {
        Ok(mut guard) => {
            let changed = *guard != next;
            if changed {
                *guard = next;
            }
            changed
        }
        Err(_) => false,
    };
    if !changed {
        return;
    }
    if let Some(window) = window(app) {
        let _ = window.emit(EVENT, ());
    }
}

/// The line the card should be showing, or an empty string for the start hint.
#[tauri::command]
pub fn notice_text() -> String {
    ANNOUNCEMENT
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_default()
}

/// Shows a line the relay asked every App to pass on.
#[tauri::command]
pub fn notice_announce(app: AppHandle, message: String) {
    announce(&app, &message);
}

/// The placement and the countdown, with whatever sentence is on the card.
fn reveal(app: &AppHandle) {
    let Some(window) = window(app) else {
        return;
    };
    let (cursor_x, cursor_y) = platform::desktop::cursor_pos();
    let area = platform::desktop::work_area_for_point(cursor_x, cursor_y)
        .or_else(|| platform::desktop::work_area_for_point(0, 0));
    // The work area and the window position are physical pixels while the
    // margin and the fallback size are CSS pixels: on a display that is not at
    // 100% they only agree after this conversion.
    let scale = window.scale_factor().unwrap_or(1.0);
    let mut bounds = None;
    if let Some(area) = area {
        let size = window
            .outer_size()
            .map(|size| (size.width as f64, size.height as f64))
            .unwrap_or((FALLBACK_SIZE.0 * scale, FALLBACK_SIZE.1 * scale));
        let margin = EDGE_MARGIN * scale;
        let x = area.right as f64 - margin - size.0;
        let y = area.bottom as f64 - margin - size.1;
        let x = x.max(area.left as f64);
        let y = y.max(area.top as f64);
        let _ = window.set_position(PhysicalPosition::new(x, y));
        bounds = Some(ScreenRect {
            left: x as i32,
            top: y as i32,
            right: (x + size.0) as i32,
            bottom: (y + size.1) as i32,
        });
    }
    if let Ok(mut guard) = BOUNDS.lock() {
        *guard = bounds;
    }
    let _ = window.show();
    let epoch = start_show();
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(LINGER).await;
        if is_current(epoch) {
            hide(&handle);
        }
    });
}

pub fn hide(app: &AppHandle) {
    retire_show();
    if let Ok(mut guard) = BOUNDS.lock() {
        *guard = None;
    }
    if let Some(window) = window(app) {
        let _ = window.hide();
    }
}

/// Hides the card without opening anything.
#[tauri::command]
pub fn notice_close(app: AppHandle) {
    hide(&app);
}

/// Clicking the start hint is a request for the settings window; clicking an
/// announcement is only ever an acknowledgement.
#[tauri::command]
pub fn notice_open(app: AppHandle) {
    hide(&app);
    if !from_announcement() {
        // The runtime is spelled out: leaving it to inference makes the macro
        // expand to a never type fallback.
        crate::tray::show_main::<tauri::Wry>(&app);
    }
}

/// Whether the card is showing a line from the relay rather than the hint.
fn from_announcement() -> bool {
    ANNOUNCEMENT
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hit box only exists while the hint is on screen; a stale one would
    /// swallow clicks on whatever the user is actually working in.
    #[test]
    fn a_hint_that_is_not_shown_covers_nothing() {
        assert!(!contains(0, 0));
        assert!(!contains(1356, 904));
    }

    /// A hint that comes back a second time gets its own countdown, and the one
    /// left over from the show before it must not cut that short.
    #[test]
    fn a_new_show_retires_the_countdown_of_the_previous_one() {
        let first = start_show();
        assert!(is_current(first));

        let second = start_show();
        assert!(is_current(second));
        assert!(!is_current(first));

        // Closing the hint by hand ends the show it belonged to as well.
        retire_show();
        assert!(!is_current(second));
    }

    /// The line rides on every translation the relay answers, so the same one has
    /// to be shown once rather than once per translation.
    #[test]
    fn a_line_that_came_with_the_last_translation_is_not_shown_again() {
        let mut announced = Announced::default();
        assert_eq!(
            announced.take("  明天上午维护  ").as_deref(),
            Some("明天上午维护")
        );
        assert_eq!(announced.take("明天上午维护"), None);
        // News that is not the last thing said is news again: the relay has no
        // way to know what this App has already shown.
        assert_eq!(announced.take("维护已结束").as_deref(), Some("维护已结束"));
        assert_eq!(announced.take("明天上午维护").as_deref(), Some("明天上午维护"));
    }

    /// An empty line is not a line: a relay with nothing to say leaves the card
    /// on the start hint rather than blanking it.
    #[test]
    fn an_empty_line_is_nothing_to_show() {
        let mut announced = Announced::default();
        assert_eq!(announced.take(""), None);
        assert_eq!(announced.take("   "), None);
        // …and it does not count as the last thing said.
        assert_eq!(announced.take("有话说").as_deref(), Some("有话说"));
    }
}
