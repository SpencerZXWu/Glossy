//! Starting hint.
//!
//! The settings window only opens on the very first launch ever, so every later
//! launch is silent: the icon Windows parks in the overflow of the notification
//! area would be the only sign that Glossy is up. This little card appears in the
//! corner that holds that icon for a few seconds, opens the settings window when
//! clicked, and goes away by itself.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

use crate::platform::{self, ScreenRect};

pub const NOTICE_LABEL: &str = "notice";

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

/// Places the hint in the bottom right corner of the screen the cursor is on —
/// the corner that holds the notification area — and shows it.
///
/// The countdown that takes it away again is started here, so every show gets a
/// full one however often the window has been on screen before.
pub fn show(app: &AppHandle) {
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

/// Hides the hint without opening anything.
#[tauri::command]
pub fn notice_close(app: AppHandle) {
    hide(&app);
}

/// Clicking the hint is a request for the settings window.
#[tauri::command]
pub fn notice_open(app: AppHandle) {
    hide(&app);
    // The runtime is spelled out: leaving it to inference makes the macro expand
    // to a never type fallback.
    crate::tray::show_main::<tauri::Wry>(&app);
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
}
