//! Placement, sizing and dismissal of the floating translation popup.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewWindow};

use crate::platform::ScreenRect;
use crate::state::AppState;

pub const POPUP_LABEL: &str = "popup";

/// Vertical distance between the mouse cursor and the top edge of the popup.
const CURSOR_GAP: f64 = 18.0;
/// Keep the popup this far away from the edges of the work area.
const EDGE_MARGIN: f64 = 8.0;
/// Fallback work area used when the monitor cannot be queried.
const FALLBACK_AREA: ScreenRect = ScreenRect {
    left: 0,
    top: 0,
    right: 1280,
    bottom: 720,
};

/// Screen geometry of the popup window.
///
/// Kept in process wide atomics so the low level mouse hook can test whether a
/// click landed on the popup without ever taking a lock.
struct PopupBounds {
    left: AtomicI32,
    top: AtomicI32,
    right: AtomicI32,
    bottom: AtomicI32,
    visible: AtomicBool,
}

static BOUNDS: PopupBounds = PopupBounds {
    left: AtomicI32::new(0),
    top: AtomicI32::new(0),
    right: AtomicI32::new(0),
    bottom: AtomicI32::new(0),
    visible: AtomicBool::new(false),
};

fn bounds() -> Option<ScreenRect> {
    if !BOUNDS.visible.load(Ordering::Relaxed) {
        return None;
    }
    Some(ScreenRect {
        left: BOUNDS.left.load(Ordering::Relaxed),
        top: BOUNDS.top.load(Ordering::Relaxed),
        right: BOUNDS.right.load(Ordering::Relaxed),
        bottom: BOUNDS.bottom.load(Ordering::Relaxed),
    })
}

fn store_bounds(rect: ScreenRect) {
    BOUNDS.left.store(rect.left, Ordering::Relaxed);
    BOUNDS.top.store(rect.top, Ordering::Relaxed);
    BOUNDS.right.store(rect.right, Ordering::Relaxed);
    BOUNDS.bottom.store(rect.bottom, Ordering::Relaxed);
    BOUNDS.visible.store(true, Ordering::Relaxed);
}

/// True when the screen point is currently covered by the popup.
pub fn contains(x: i32, y: i32) -> bool {
    bounds().is_some_and(|rect| rect.contains_padded(x, y, 0))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionPayload {
    pub text: String,
}

fn popup_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(POPUP_LABEL)
}

/// Positions the popup below `anchor`, clamped so it never leaves the screen.
///
/// Pure geometry, kept separate so it can be unit tested.
pub fn clamp_in(anchor: (f64, f64), width: f64, height: f64, area: ScreenRect) -> (f64, f64) {
    let (ax, ay) = anchor;
    let mut x = ax - width / 2.0;
    let mut y = ay + CURSOR_GAP;

    let min_x = area.left as f64 + EDGE_MARGIN;
    let max_x = area.right as f64 - EDGE_MARGIN - width;
    if x > max_x {
        x = max_x;
    }
    if x < min_x {
        x = min_x;
    }

    // Not enough room underneath: flip the popup above the cursor instead.
    let min_y = area.top as f64 + EDGE_MARGIN;
    let max_y = area.bottom as f64 - EDGE_MARGIN - height;
    if y > max_y {
        y = ay - CURSOR_GAP - height;
    }
    if y > max_y {
        y = max_y;
    }
    if y < min_y {
        y = min_y;
    }

    (x.round(), y.round())
}

/// Keeps a window of `width` x `height` anchored at `point`, nudged inside
/// `area`. Used when the popup is already placed (and possibly dragged).
fn clamp_point(point: (f64, f64), width: f64, height: f64, area: ScreenRect) -> (f64, f64) {
    let min_x = area.left as f64 + EDGE_MARGIN;
    let max_x = (area.right as f64 - EDGE_MARGIN - width).max(min_x);
    let min_y = area.top as f64 + EDGE_MARGIN;
    let max_y = (area.bottom as f64 - EDGE_MARGIN - height).max(min_y);
    (point.0.clamp(min_x, max_x), point.1.clamp(min_y, max_y))
}

/// Tells the popup window about a new selection and remembers where it belongs.
pub fn reveal(app: &AppHandle, state: &AppState, text: String, anchor: (f64, f64)) {
    let Some(window) = popup_window(app) else {
        return;
    };
    state.set_anchor(anchor);
    let _ = window.emit("glossy://selection", SelectionPayload { text });
}

/// Sizes, places and optionally shows the popup window.
///
/// `width` and `height` are CSS pixels reported by the popup itself.
pub fn place(
    app: &AppHandle,
    state: &AppState,
    width: f64,
    height: f64,
    show: bool,
) -> Result<(), String> {
    let window = popup_window(app).ok_or("popup window is not available")?;

    let width = width.clamp(160.0, 1200.0);
    // The popup itself keeps the card inside the screen; this is only a guard
    // against nonsense values.
    let height = height.clamp(40.0, 4096.0);

    let anchor = state.anchor().unwrap_or_else(|| {
        let (x, y) = crate::platform::cursor_pos();
        (x as f64, y as f64)
    });

    let scale = window.scale_factor().unwrap_or(1.0);
    let physical_width = width * scale;
    let physical_height = height * scale;

    let area = crate::platform::work_area_for_point(anchor.0 as i32, anchor.1 as i32)
        .unwrap_or(FALLBACK_AREA);

    // While the popup is already on screen the user may have dragged it, so it
    // keeps its current corner. A fresh selection is anchored to the cursor.
    let placed = if show {
        None
    } else {
        window
            .outer_position()
            .ok()
            .filter(|_| window.is_visible().unwrap_or(false))
            .map(|position| (position.x as f64, position.y as f64))
    };

    let (x, y) = match placed {
        Some(point) => clamp_point(point, physical_width, physical_height, area),
        None => clamp_in(anchor, physical_width, physical_height, area),
    };

    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    if show {
        window.set_always_on_top(true).map_err(|e| e.to_string())?;
        window.show().map_err(|e| e.to_string())?;
    }

    store_bounds(ScreenRect {
        left: x as i32,
        top: y as i32,
        right: (x + physical_width).round() as i32,
        bottom: (y + physical_height).round() as i32,
    });

    Ok(())
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = popup_window(app) {
        let _ = window.hide();
    }
    BOUNDS.visible.store(false, Ordering::Relaxed);
}

/// Re-anchors the popup after the user dragged it to a new position.
pub fn sync_anchor(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let window = popup_window(app).ok_or("popup window is not available")?;
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;

    state.set_anchor((
        position.x as f64 + size.width as f64 / 2.0,
        position.y as f64 - CURSOR_GAP,
    ));
    store_bounds(ScreenRect {
        left: position.x,
        top: position.y,
        right: position.x + size.width as i32,
        bottom: position.y + size.height as i32,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: ScreenRect = ScreenRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1040,
    };

    #[test]
    fn places_below_and_centered_on_the_cursor() {
        let (x, y) = clamp_in((900.0, 400.0), 384.0, 200.0, AREA);
        assert_eq!(x, 708.0);
        assert_eq!(y, 418.0);
    }

    #[test]
    fn pulls_back_from_the_right_edge() {
        let (x, _) = clamp_in((1900.0, 400.0), 384.0, 200.0, AREA);
        assert_eq!(x, 1920.0 - 8.0 - 384.0);
    }

    #[test]
    fn pulls_back_from_the_left_edge() {
        let (x, _) = clamp_in((10.0, 400.0), 384.0, 200.0, AREA);
        assert_eq!(x, 8.0);
    }

    #[test]
    fn flips_above_the_cursor_near_the_bottom() {
        let (_, y) = clamp_in((900.0, 1000.0), 384.0, 200.0, AREA);
        assert_eq!(y, 1000.0 - CURSOR_GAP - 200.0);
    }

    #[test]
    fn stays_inside_when_taller_than_the_screen() {
        let (_, y) = clamp_in((900.0, 500.0), 384.0, 2000.0, AREA);
        assert_eq!(y, 8.0);
    }

    #[test]
    fn honours_a_secondary_monitor_offset() {
        let secondary = ScreenRect {
            left: 1920,
            top: -200,
            right: 3840,
            bottom: 880,
        };
        // The cursor sits close to the right edge of the secondary monitor, so
        // the popup has to be pulled back against *its* edge.
        let (x, y) = clamp_in((3800.0, 100.0), 384.0, 120.0, secondary);
        assert_eq!(x, 3840.0 - 8.0 - 384.0);
        assert_eq!(y, 118.0);
    }

    #[test]
    fn keeps_an_already_placed_popup_where_it_is() {
        let (x, y) = clamp_point((700.0, 300.0), 384.0, 200.0, AREA);
        assert_eq!((x, y), (700.0, 300.0));
    }

    #[test]
    fn drags_an_already_placed_popup_back_on_screen() {
        let (x, y) = clamp_point((1900.0, 1030.0), 384.0, 200.0, AREA);
        assert_eq!(x, 1920.0 - 8.0 - 384.0);
        assert_eq!(y, 1040.0 - 8.0 - 200.0);
    }
}
