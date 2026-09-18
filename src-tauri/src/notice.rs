//! Starting hint.
//!
//! The settings window only opens on the very first launch ever, so every later
//! launch is silent: the icon Windows parks in the overflow of the notification
//! area would be the only sign that Glossy is up. This little card appears in the
//! corner that holds that icon for a few seconds, opens the settings window when
//! clicked, and goes away by itself.

use std::sync::Mutex;

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

use crate::platform::{self, ScreenRect};

pub const NOTICE_LABEL: &str = "notice";

/// Distance between the hint and the edges of the work area, in the CSS pixels
/// the window is laid out in.
const EDGE_MARGIN: f64 = 16.0;

/// Fallback size in CSS pixels, used until the window reports its own.
const FALLBACK_SIZE: (f64, f64) = (340.0, 104.0);

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

/// Places the hint in the bottom right corner of the screen the cursor is on —
/// the corner that holds the notification area — and shows it.
pub fn show(app: &AppHandle) {
    let Some(window) = window(app) else {
        return;
    };
    let (cursor_x, cursor_y) = platform::cursor_pos();
    let area = platform::work_area_for_point(cursor_x, cursor_y)
        .or_else(|| platform::work_area_for_point(0, 0));
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
}

pub fn hide(app: &AppHandle) {
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
}
