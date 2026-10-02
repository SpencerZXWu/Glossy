//! Placement, sizing and dismissal of the floating translation popup.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, AtomicU32, Ordering};
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewWindow};

use crate::platform::ScreenRect;
use crate::platform::{self, speech};
use crate::state::AppState;
use crate::translate::TranslationResult;

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
    /// Set while the user has pinned the card, which is what keeps a click
    /// elsewhere from dismissing it.
    pinned: AtomicBool,
    /// Set while the card's own accelerator (Ctrl+Enter) should be registered.
    /// It follows the card itself, so it is armed only while one is on screen.
    replace: AtomicBool,
    /// Native window the card is drawn in, zero until one has been seen.
    handle: AtomicIsize,
    /// Browser process drawing the card, zero until one has been seen.
    webview: AtomicU32,
}

static BOUNDS: PopupBounds = PopupBounds {
    left: AtomicI32::new(0),
    top: AtomicI32::new(0),
    right: AtomicI32::new(0),
    bottom: AtomicI32::new(0),
    visible: AtomicBool::new(false),
    pinned: AtomicBool::new(false),
    replace: AtomicBool::new(false),
    handle: AtomicIsize::new(0),
    webview: AtomicU32::new(0),
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

/// True while the card is pinned, so a click elsewhere has to leave it alone.
pub fn pinned() -> bool {
    BOUNDS.pinned.load(Ordering::Relaxed)
}

/// Pins or unpins the card, as chosen by the button in its header.
pub fn set_pinned(pinned: bool) {
    BOUNDS.pinned.store(pinned, Ordering::Relaxed);
}

/// True while the card's own accelerator should be registered.
pub fn replace_armed() -> bool {
    BOUNDS.replace.load(Ordering::Relaxed)
}

/// Arms or releases the card's own accelerator.
///
/// The registration happens on the thread that owns the accelerators, so the
/// change is only noted here and the reload is asked for; a state that is
/// already in force asks for nothing. Only a card that holds a translation of a
/// selection still in place can be written back over, so the key is held by the
/// interface while such a card is up, and by nothing else: a badge waiting for
/// its click, an old result from the history and a card that has gone away all
/// leave Ctrl+Enter to the program in front.
pub fn set_replace(armed: bool) {
    if BOUNDS.replace.swap(armed, Ordering::Relaxed) != armed {
        platform::hotkey::request_reload();
    }
}

/// Whether a mouse click at this screen point dismisses the card.
///
/// Only a click outside an unpinned card does: a pinned one was asked to stay,
/// and a click inside it belongs to the card itself. A click on a window the
/// card owns is a click on the card too, even when it falls outside its edges;
/// see [`owns_point`].
pub fn dismisses_click(x: i32, y: i32) -> bool {
    !contains(x, y) && !pinned()
}

/// True when the screen point lands on a window the popup owns.
///
/// The list of a `<select>` is a window of its own, drawn by the browser
/// process of the card, and it is taller than the card: its entries reach well
/// below the card's bottom edge, where a click would otherwise count as a click
/// on the program behind us and take the card away with it.
///
/// Two questions are asked, because the two ways the list can answer for itself
/// are independent: which window owns the one under the point, and which
/// process draws it.
///
/// Both answers are cached while the card is placed, because the mouse hook
/// cannot look them up and it and the worker have to reach the same answer.
pub fn owns_point(x: i32, y: i32) -> bool {
    let handle = BOUNDS.handle.load(Ordering::Relaxed);
    if handle == 0 {
        return false;
    }
    if platform::desktop::owns_point(platform::desktop::Handle(handle), x, y) {
        return true;
    }
    webview_process_id()
        .is_some_and(|pid| platform::desktop::process_id_under_point(x, y) == Some(pid))
}

fn webview_process_id() -> Option<u32> {
    let pid = BOUNDS.webview.load(Ordering::Relaxed);
    if pid == 0 {
        None
    } else {
        Some(pid)
    }
}

/// Remembers the native window the card lives in and the browser process
/// drawing it, so [`owns_point`] can be asked from outside the main thread.
fn store_handle(window: &WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        BOUNDS.handle.store(hwnd.0 as isize, Ordering::Relaxed);
        if let Some(pid) =
            platform::desktop::child_process_id(platform::desktop::Handle(hwnd.0 as isize))
        {
            BOUNDS.webview.store(pid, Ordering::Relaxed);
        }
    }
}

/// Follows the window while the user drags the popup by its header.
///
/// The bounds are normally written when the popup is positioned, so a native
/// drag would leave the click test pointing at the old spot: clicks on the moved
/// popup would then close it and the release would trigger another translation.
/// A drag never changes the size, so the stored extent is kept as is.
pub fn track_move(x: i32, y: i32) {
    let Some(rect) = bounds() else {
        return;
    };
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return;
    }
    store_bounds(ScreenRect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    });
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionPayload {
    pub text: String,
    /// The sentence the selection stands in, when the setting for it is on and
    /// the program in front let it be read. It is read at the moment of the
    /// selection, while that program still has the focus, and travels with the
    /// text so the card can translate it without asking the screen again.
    pub context: Option<String>,
    /// True when the user did something that asks for a translation by itself,
    /// which the shortcut does. A selection made with the mouse only asks for
    /// the badge, and the click on it asks for the translation.
    pub immediate: bool,
}

fn popup_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(POPUP_LABEL)
}

/// Positions the popup below `anchor`, clamped so it never leaves the screen.
///
/// `width`, `height`, `anchor` and the returned point are all physical pixels,
/// while `CURSOR_GAP` and `EDGE_MARGIN` are the CSS pixels the interface
/// promises: `scale` converts them, otherwise the gap and the margin shrink on
/// a display that is not at 100%.
///
/// Pure geometry, kept separate so it can be unit tested.
pub fn clamp_in(
    anchor: (f64, f64),
    width: f64,
    height: f64,
    area: ScreenRect,
    scale: f64,
) -> (f64, f64) {
    let (ax, ay) = anchor;
    let gap = CURSOR_GAP * scale;
    let margin = EDGE_MARGIN * scale;
    let mut x = ax - width / 2.0;
    let mut y = ay + gap;

    let min_x = area.left as f64 + margin;
    let max_x = area.right as f64 - margin - width;
    if x > max_x {
        x = max_x;
    }
    if x < min_x {
        x = min_x;
    }

    // Not enough room underneath: flip the popup above the cursor instead.
    let min_y = area.top as f64 + margin;
    let max_y = area.bottom as f64 - margin - height;
    if y > max_y {
        y = ay - gap - height;
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
fn clamp_point(
    point: (f64, f64),
    width: f64,
    height: f64,
    area: ScreenRect,
    scale: f64,
) -> (f64, f64) {
    let margin = EDGE_MARGIN * scale;
    let min_x = area.left as f64 + margin;
    let max_x = (area.right as f64 - margin - width).max(min_x);
    let min_y = area.top as f64 + margin;
    let max_y = (area.bottom as f64 - margin - height).max(min_y);
    (point.0.clamp(min_x, max_x), point.1.clamp(min_y, max_y))
}

/// Tells the popup window about a new selection and remembers where it belongs.
pub fn reveal(
    app: &AppHandle,
    state: &AppState,
    text: String,
    context: Option<String>,
    anchor: (f64, f64),
    immediate: bool,
) {
    let Some(window) = popup_window(app) else {
        return;
    };
    state.set_anchor(anchor);
    let _ = window.emit(
        "glossy://selection",
        SelectionPayload {
            text,
            context,
            immediate,
        },
    );
}

/// Shows a card that says why there is no translation, where the pointer is.
///
/// The card the user was expecting is not coming, so this one takes its place:
/// the same message the screenshot flow uses when a reading cannot be made.
pub fn fail(app: &AppHandle, message: &str) {
    if let Some(state) = app.try_state::<Arc<AppState>>() {
        let (x, y) = crate::platform::desktop::cursor_pos();
        state.set_anchor((x as f64, y as f64));
    }
    let _ = app.emit("glossy://popup-error", message.to_string());
}

/// Shows the card for a translation that was already made, which is how the
/// history puts an old result back on screen without asking the provider again.
pub fn reveal_result(
    app: &AppHandle,
    state: &AppState,
    result: TranslationResult,
    anchor: (f64, f64),
) {
    let Some(window) = popup_window(app) else {
        return;
    };
    state.set_anchor(anchor);
    let _ = window.emit("glossy://result", result);
}

/// Sizes, places and optionally shows the popup window.
///
/// `width` and `height` are CSS pixels reported by the popup itself. The
/// returned value is the height the monitor under the anchor offers, in CSS
/// pixels, so the interface can cap the card to the screen it actually ends up
/// on instead of the one it is leaving.
pub fn place(
    app: &AppHandle,
    state: &AppState,
    width: f64,
    height: f64,
    show: bool,
) -> Result<f64, String> {
    let window = popup_window(app).ok_or("popup window is not available")?;
    store_handle(&window);

    // The floor is low because the popup is not always the card: the badge that
    // waits for a click is a single icon in a window of its own size, and any
    // slack around it would be transparent window that still swallows clicks.
    let width = width.clamp(40.0, 1200.0);
    // The popup itself keeps the card inside the screen; this is only a guard
    // against nonsense values.
    let height = height.clamp(40.0, 4096.0);

    let anchor = state.anchor().unwrap_or_else(|| {
        let (x, y) = crate::platform::desktop::cursor_pos();
        (x as f64, y as f64)
    });

    let scale = window.scale_factor().unwrap_or(1.0);
    let physical_width = width * scale;
    let physical_height = height * scale;

    let area = crate::platform::desktop::work_area_for_point(anchor.0 as i32, anchor.1 as i32)
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
        Some(point) => clamp_point(point, physical_width, physical_height, area, scale),
        None => clamp_in(anchor, physical_width, physical_height, area, scale),
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

    keep_non_activating(&window);

    store_bounds(ScreenRect {
        left: x as i32,
        top: y as i32,
        right: (x + physical_width).round() as i32,
        bottom: (y + physical_height).round() as i32,
    });

    let anchor_scale = scale_at(&window, anchor, scale);
    Ok((area.bottom - area.top) as f64 / anchor_scale)
}

/// Puts the style that keeps the popup out of the foreground back on.
///
/// The style is set once when the window is created, but showing it takes the
/// style away, and a window without it takes the keyboard when the user clicks
/// the badge or the card. That click is the whole point of the badge, so the
/// program the selection came from is told it is no longer in front — WeChat
/// drops the selection at that moment and the translation would land next to
/// the text instead of over it. The style is therefore restored after every
/// placement, before the click that follows one can arrive.
fn keep_non_activating(window: &WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        crate::platform::desktop::make_non_activating(crate::platform::desktop::Handle(
            hwnd.0 as isize,
        ));
    }
}

/// Hands the keyboard to the popup, so a field in the card can be typed into.
///
/// The card's original is editable, and typing needs the keyboard the popup
/// otherwise keeps away from - see [`keep_non_activating`]. Only the user
/// opening that field asks for this, and the next placement takes the keyboard
/// back off the popup.
pub fn take_focus(app: &AppHandle) {
    let Some(window) = app.get_webview_window(POPUP_LABEL) else {
        return;
    };
    if let Ok(hwnd) = window.hwnd() {
        crate::platform::desktop::make_activating(crate::platform::desktop::Handle(
            hwnd.0 as isize,
        ));
    }
}

/// Scale factor of the monitor that contains `point`.
/// The popup is about to move to that monitor, while the window still reports
/// the scale factor of the one it is leaving, so the interface has to be sized
/// for the monitor the anchor belongs to.
fn scale_at(window: &WebviewWindow, point: (f64, f64), fallback: f64) -> f64 {
    let (x, y) = (point.0 as i32, point.1 as i32);
    window
        .available_monitors()
        .ok()
        .and_then(|monitors| {
            monitors.into_iter().find(|monitor| {
                let origin = monitor.position();
                let size = monitor.size();
                x >= origin.x
                    && x < origin.x + size.width as i32
                    && y >= origin.y
                    && y < origin.y + size.height as i32
            })
        })
        .map_or(fallback, |monitor| monitor.scale_factor())
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = popup_window(app) {
        // The card can have been given the keyboard - a field in it is typed
        // into, and its buttons answer Tab and the arrows - so it may be the
        // window in front when it is closed. That window is going away, and the
        // keyboard left there would go nowhere: it goes back to the program the
        // selection came from, which is the one the user was reading in.
        //
        // Only when the card itself holds it: a card that sent the user to the
        // settings window must not take the keyboard away from it.
        let mine = window
            .hwnd()
            .map(|hwnd| {
                crate::platform::desktop::is_foreground(crate::platform::desktop::Handle(
                    hwnd.0 as isize,
                ))
            })
            .unwrap_or(false);
        let _ = window.hide();
        if mine {
            crate::platform::desktop::restore_focus_owner();
        }
    }
    // A card that is out of sight must not keep talking.
    speech::stop();
    BOUNDS.visible.store(false, Ordering::Relaxed);
    // Nor may it keep the key that writes a translation back: the card it would
    // write into is gone.
    set_replace(false);
    // The pin belongs to the card that is on screen, not to the popup window,
    // which lives on between translations.
    BOUNDS.pinned.store(false, Ordering::Relaxed);
}

/// Tells the card that the user asked to write the translation back over the
/// text it came from.
///
/// The card holds the translation, so the request carries nothing but the
/// question: the button in the card asks for exactly the same thing.
pub fn request_replace(app: &AppHandle) {
    if let Some(window) = popup_window(app) {
        let _ = window.emit("glossy://replace", ());
    }
}

/// Re-anchors the popup after the user dragged it to a new position.
pub fn sync_anchor(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let window = popup_window(app).ok_or("popup window is not available")?;
    store_handle(&window);
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    // The anchor is compared against physical cursor coordinates, so the CSS
    // pixel gap has to be scaled like it is everywhere else.
    let scale = window.scale_factor().unwrap_or(1.0);

    state.set_anchor((
        position.x as f64 + size.width as f64 / 2.0,
        position.y as f64 - CURSOR_GAP * scale,
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

    /// Scale factor of every test that was written for a 100% display.
    const FULL_SCALE: f64 = 1.0;

    #[test]
    fn places_below_and_centered_on_the_cursor() {
        let (x, y) = clamp_in((900.0, 400.0), 384.0, 200.0, AREA, FULL_SCALE);
        assert_eq!(x, 708.0);
        assert_eq!(y, 418.0);
    }

    #[test]
    fn pulls_back_from_the_right_edge() {
        let (x, _) = clamp_in((1900.0, 400.0), 384.0, 200.0, AREA, FULL_SCALE);
        assert_eq!(x, 1920.0 - 8.0 - 384.0);
    }

    #[test]
    fn pulls_back_from_the_left_edge() {
        let (x, _) = clamp_in((10.0, 400.0), 384.0, 200.0, AREA, FULL_SCALE);
        assert_eq!(x, 8.0);
    }

    #[test]
    fn flips_above_the_cursor_near_the_bottom() {
        let (_, y) = clamp_in((900.0, 1000.0), 384.0, 200.0, AREA, FULL_SCALE);
        assert_eq!(y, 1000.0 - CURSOR_GAP - 200.0);
    }

    #[test]
    fn stays_inside_when_taller_than_the_screen() {
        let (_, y) = clamp_in((900.0, 500.0), 384.0, 2000.0, AREA, FULL_SCALE);
        assert_eq!(y, 8.0);
    }

    #[test]
    fn keeps_the_inset_on_a_scaled_display() {
        // A 150% display: the 8px inset and the 18px gap the interface promises
        // have to grow with it, otherwise the popup lands too close to the edge.
        let scale = 1.5;
        let (x, _) = clamp_in((10.0, 400.0), 576.0, 300.0, AREA, scale);
        assert_eq!(x, EDGE_MARGIN * scale);

        let (_, y) = clamp_in((900.0, 400.0), 576.0, 300.0, AREA, scale);
        assert_eq!(y, 400.0 + CURSOR_GAP * scale);

        // ... and the drag clamp uses the same inset.
        let (x, _) = clamp_point((5000.0, 400.0), 576.0, 300.0, AREA, scale);
        assert_eq!(x, 1920.0 - EDGE_MARGIN * scale - 576.0);
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
        let (x, y) = clamp_in((3800.0, 100.0), 384.0, 120.0, secondary, FULL_SCALE);
        assert_eq!(x, 3840.0 - 8.0 - 384.0);
        assert_eq!(y, 118.0);
    }

    #[test]
    fn keeps_an_already_placed_popup_where_it_is() {
        let (x, y) = clamp_point((700.0, 300.0), 384.0, 200.0, AREA, FULL_SCALE);
        assert_eq!((x, y), (700.0, 300.0));
    }

    #[test]
    fn drags_an_already_placed_popup_back_on_screen() {
        let (x, y) = clamp_point((1900.0, 1030.0), 384.0, 200.0, AREA, FULL_SCALE);
        assert_eq!(x, 1920.0 - 8.0 - 384.0);
        assert_eq!(y, 1040.0 - 8.0 - 200.0);
    }

    #[test]
    fn follows_the_window_while_it_is_dragged() {
        store_bounds(ScreenRect {
            left: 100,
            top: 100,
            right: 400,
            bottom: 300,
        });
        assert!(contains(350, 250));

        track_move(700, 500);
        assert!(contains(950, 690));
        assert!(contains(700, 500));
        // The old spot is free again, so a click there starts a new selection.
        assert!(!contains(350, 250));

        // A hidden popup keeps the remembered rectangle, it just stops matching.
        BOUNDS.visible.store(false, Ordering::Relaxed);
        track_move(0, 0);
        assert!(!contains(0, 0));
    }

    #[test]
    fn a_pinned_card_survives_a_click_elsewhere() {
        store_bounds(ScreenRect {
            left: 100,
            top: 100,
            right: 400,
            bottom: 300,
        });

        // Unpinned, the click that starts the next selection dismisses the card.
        assert!(!dismisses_click(350, 250));
        assert!(dismisses_click(500, 500));

        set_pinned(true);
        assert!(pinned());
        assert!(!dismisses_click(500, 500));

        set_pinned(false);
        assert!(!pinned());
        assert!(dismisses_click(500, 500));
    }

    #[test]
    fn the_write_back_key_is_only_held_while_the_card_holds_a_translation() {
        // Nothing on screen owns Ctrl+Enter until a card says it does.
        set_replace(false);
        assert!(!replace_armed());

        set_replace(true);
        assert!(replace_armed());

        set_replace(false);
        assert!(!replace_armed());
    }
}
