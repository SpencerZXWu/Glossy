//! Subtitle translation: a part of the screen that is read again and again,
//! with its translation drawn where the user put it.
//!
//! It is the screenshot flow made to repeat. The user picks two rectangles with
//! the same overlay a screenshot is picked with — where the subtitles are, and
//! where the translation is to be drawn — and then a thread of its own reads
//! the first one on a timer and asks for the translation whenever what it read
//! is not what it read last time. The picture never leaves the machine: the
//! reading is done by the same local engine a screenshot uses, and only the
//! text that comes out of it goes to the translator.
//!
//! Three things keep it cheap. One: the region is read with the one language
//! picked for it rather than with every language that is checked, which is a
//! fraction of the work. Two: the translation is only asked for when the text
//! changes, so a still subtitle costs nothing at all. Three: a reading that is
//! still running when the next tick comes is simply the next reading, because
//! the loop waits for its own work rather than for the clock.
//!
//! The **Subtitles** page is only on show in developer mode: this is the
//! feature that is still being built.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};

use crate::log::note;
use crate::platform::{self, ScreenRect};
use crate::settings::Settings;
use crate::state::AppState;

/// Label of the window the translation is drawn in.
pub const SUBTITLE_LABEL: &str = "subtitle";

/// Label of the window drawn around the part of the screen that is read.
///
/// It is a box and nothing else: the area the subtitles are in is otherwise
/// invisible, and a reading that suddenly comes back with nothing is worth being
/// able to point at.
pub const AREA_LABEL: &str = "subtitle-area";

/// Label of the window the two boxes are moved in.
pub const EDIT_LABEL: &str = "subtitle-edit";

/// How long between two readings of the region.
///
/// A subtitle changes at the speed of speech, and a reading costs a couple of
/// hundred milliseconds, so a tick a second keeps up with the fastest talking
/// without the processor ever being busy: the loop sleeps first and works
/// after, so a slow reading simply makes the next tick later.
const TICK: Duration = Duration::from_millis(700);

/// Shortest reading that is worth translating. One character that flickers
/// between frames is a stray mark rather than a subtitle.
const MIN_CHARS: usize = 2;

/// The smallest box the translation is drawn in, in CSS pixels.
const MIN_WIDTH: f64 = 160.0;
const MIN_HEIGHT: f64 = 44.0;

/// The area the subtitles are in, between the two picks and for as long as a
/// reading runs.
static AREA: Mutex<Option<ScreenRect>> = Mutex::new(None);

/// Where the translation is drawn, which is also the window holding it.
static PLACE: Mutex<Option<ScreenRect>> = Mutex::new(None);

/// Which run is the current one, and whether there is one at all.
///
/// The same shape the overlay uses: the number goes up whenever a run starts or
/// stops, so the thread of a run that was stopped knows it is over even while it
/// is in the middle of a reading.
static SESSION: AtomicU64 = AtomicU64::new(0);
static RUNNING: AtomicBool = AtomicBool::new(false);

/// Whether subtitles are being read right now.
pub fn running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

/// The area the subtitles are read from.
fn area() -> Option<ScreenRect> {
    AREA.lock().ok().and_then(|guard| *guard)
}

/// Where the translation is drawn, as the settings page asks for it.
pub fn place() -> Option<ScreenRect> {
    PLACE.lock().ok().and_then(|guard| *guard)
}

/// Both boxes, once there are two of them.
pub fn boxes() -> Option<(ScreenRect, ScreenRect)> {
    Some((area()?, place()?))
}

/// Remembers the pair a run is made of.
///
/// Both halves are written every time, and on purpose: a reading asks for the
/// area on every tick and draws in the other one, so a pair that is half kept is
/// a reading that starts, says it started, and then reads nothing at all.
fn keep(area: ScreenRect, place: ScreenRect) {
    if let Ok(mut guard) = AREA.lock() {
        *guard = Some(area);
    }
    if let Ok(mut guard) = PLACE.lock() {
        *guard = Some(place);
    }
}

/// Forgets the pair, which is what makes the next reading pick it again.
fn forget() {
    if let Ok(mut guard) = AREA.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = PLACE.lock() {
        *guard = None;
    }
}

/// Remembers the subtitle area and asks for the place to draw in.
///
/// The second pick is the same overlay with a different hint, which is why this
/// does not show the translation window yet: the user is still choosing.
pub fn area_picked(app: &AppHandle, rect: ScreenRect) {
    if let Ok(mut guard) = AREA.lock() {
        *guard = Some(rect);
    }
    crate::ocr::begin_pick(app, crate::ocr::Pick::SubtitlePlace);
}

/// Starts reading, with the translation drawn in `place`.
///
/// Called with the rectangle the user picked for the translation, which is also
/// the window that holds it: the window is exactly that box, so what is drawn
/// goes where the user pointed and nothing else on screen moves.
pub fn start(app: &AppHandle, state: &AppState, place: ScreenRect) {
    let Some(rect) = area() else {
        return crate::ocr::fail(app, "The subtitle area was not picked.");
    };
    let width = (rect.right - rect.left) as i64;
    let height = (rect.bottom - rect.top) as i64;
    if width < crate::ocr::MIN_REGION || height < crate::ocr::MIN_REGION {
        return crate::ocr::fail(app, "The subtitle area is too small to read.");
    }

    end_run(app);
    let session = SESSION.load(Ordering::SeqCst);
    RUNNING.store(true, Ordering::SeqCst);
    // The pair is stored after the run before it was ended: a start keeps its
    // own boxes, and the stop that ends the old run is only there to take its
    // windows off the screen.
    keep(rect, place);
    frames(app);

    let settings = state.settings();
    note!(
        "glossy: reading subtitles from {}x{} at {},{} into {}",
        width,
        height,
        rect.left,
        rect.top,
        settings.subtitle_target_lang
    );

    let app = app.clone();
    std::thread::spawn(move || {
        if let Err(error) = read_forever(&app, settings, session) {
            note!("glossy: the subtitle reading stopped: {error}");
            if SESSION.load(Ordering::SeqCst) == session {
                let _ = app.emit("glossy://subtitle", Kind::Failed(error).payload());
            }
        }
        // Whatever the reason, a run that has ended is no longer running: the
        // page that started it has to be able to start another one.
        if SESSION.load(Ordering::SeqCst) == session {
            RUNNING.store(false, Ordering::SeqCst);
        }
    });
}

/// Ends whatever run is going on and takes its windows off the screen.
///
/// The boxes are deliberately left where they are: this is the half of a stop
/// that a start needs, and a start is about to be handed a new pair — a stop
/// that cleared them first is what a reading that never started once looked
/// like.
fn end_run(app: &AppHandle) {
    SESSION.fetch_add(1, Ordering::SeqCst);
    RUNNING.store(false, Ordering::SeqCst);
    hide(app, SUBTITLE_LABEL);
    hide(app, AREA_LABEL);
    hide(app, EDIT_LABEL);
}

/// Stops reading, forgets the two boxes and takes the translation off the screen.
pub fn stop(app: &AppHandle) {
    end_run(app);
    forget();
}

fn hide(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.hide();
    }
}

/// What the settings window is told about the reading.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub running: bool,
}

/// The reading itself, one tick at a time, until the run is over.
fn read_forever(app: &AppHandle, settings: Settings, session: u64) -> Result<(), String> {
    // The language the region is read with, and where its two files are. Both
    // are decided once: a run reads with one language from beginning to end.
    let pack = crate::ocr::subtitle_pack(app, &settings);
    let paths = crate::ocr::models::paths(app, &pack)?;
    let from = settings.subtitle_source_lang.clone();
    let to = settings.subtitle_target_lang.clone();
    let mut last = String::new();
    let mut announced = false;

    while SESSION.load(Ordering::SeqCst) == session {
        std::thread::sleep(TICK);
        if SESSION.load(Ordering::SeqCst) != session {
            return Ok(());
        }
        // Where the region is is asked for every tick rather than remembered:
        // the box can be moved while the reading runs, and the next line should
        // come from where the user left it.
        let Some(rect) = area() else {
            return Ok(());
        };

        let text = match read_once(app, rect, &paths) {
            Ok(text) => text,
            Err(error) => {
                // One failed reading is a frame that could not be taken — a
                // screen that changed under the rectangle, a monitor that was
                // unplugged. The next tick tries again; only a run that cannot
                // read at all is worth giving up on.
                note!("glossy: a subtitle reading failed: {error}");
                continue;
            }
        };
        if text.chars().count() < MIN_CHARS || text == last {
            continue;
        }
        last = text.clone();

        let translated = tauri::async_runtime::block_on(crate::translate::translate_pair(
            &text, &from, &to, &settings,
        ));
        match translated {
            Ok(result) => {
                if !announced {
                    announced = true;
                    note!("glossy: the subtitle reading is running");
                }
                let translation = result.translation.trim().to_string();
                if !translation.is_empty() {
                    let _ = app.emit("glossy://subtitle", Kind::Line(translation).payload());
                }
            }
            Err(error) => {
                note!("glossy: a subtitle line could not be translated: {error}");
                let _ = app.emit("glossy://subtitle", Kind::Failed(error).payload());
            }
        }
    }
    Ok(())
}

/// One reading: the region as it is now, and what the engine makes of it.
fn read_once(
    app: &AppHandle,
    rect: ScreenRect,
    paths: &crate::ocr::models::Paths,
) -> Result<String, String> {
    let _ = app;
    let shot = platform::screen::grab(rect)?;
    let image = shot.crop(rect)?;
    let (pixels, width, height) = (image.pixels, image.width, image.height);
    let lines = crate::ocr::recognize_one(paths, &pixels, width, height)?;
    Ok(crate::text::normalize(&lines.join(" ")))
}

/// What the overlay window is told.
enum Kind {
    /// The line that is on screen now, translated.
    Line(String),
    /// Why there is nothing to show.
    Failed(String),
}

impl Kind {
    fn payload(self) -> serde_json::Value {
        match self {
            Kind::Line(text) => serde_json::json!({ "text": text, "error": null }),
            Kind::Failed(message) => serde_json::json!({ "text": null, "error": message }),
        }
    }
}

/// Shows the two boxes the reading is made of.
///
/// One is drawn around the area the subtitles are read from, the other is the
/// window the translation goes in, which carries a frame of its own: a faint
/// dashed line is what makes a region that is invisible by nature something the
/// user can see, aim and move.
pub fn frames(app: &AppHandle) {
    let Some((area, place)) = boxes() else {
        return;
    };
    if let Some(window) = app.get_webview_window(AREA_LABEL) {
        let _ = window.set_size(PhysicalSize::new(
            (area.right - area.left) as u32,
            (area.bottom - area.top) as u32,
        ));
        let _ = window.set_position(PhysicalPosition::new(area.left, area.top));
        let _ = window.set_always_on_top(true);
        // It is a guide and never a target: everything under it stays clickable.
        let _ = window.set_ignore_cursor_events(true);
        let _ = window.show();
    }
    place_window(app, place);
}

/// Puts the translation window where the user pointed.
///
/// The rectangle is in the physical pixels of the desktop, the way every
/// rectangle the overlay reports is, and that is what both windows are moved and
/// measured in.
fn place_window(app: &AppHandle, place: ScreenRect) {
    let Some(window) = app.get_webview_window(SUBTITLE_LABEL) else {
        return;
    };
    let scale = monitor_of(app, place)
        .map(|(_, scale)| scale)
        .unwrap_or(1.0);
    let width = ((place.right - place.left) as f64).max(MIN_WIDTH * scale) as u32;
    let height = ((place.bottom - place.top) as f64).max(MIN_HEIGHT * scale) as u32;
    let _ = window.set_size(PhysicalSize::new(width, height));
    let _ = window.set_position(PhysicalPosition::new(place.left, place.top));
    let _ = window.set_always_on_top(true);
    // The window is over a video that is being watched: a click that lands on
    // the translation has to reach whatever is under it, and the reading is
    // stopped from the settings window rather than from here.
    let _ = window.set_ignore_cursor_events(true);
    let _ = window.show();
}

/// The monitor a rectangle sits on, in physical pixels, and its scale factor.
///
/// The scale is what tells the editor's CSS pixels from the physical pixels the
/// boxes are kept in, and the smallest size a box may have is measured in each
/// of them, so both are asked for together.
fn monitor_of(app: &AppHandle, rect: ScreenRect) -> Option<(ScreenRect, f64)> {
    let window = app.get_webview_window(crate::MAIN_LABEL)?;
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
    platform::screen::monitor_of(
        &monitors,
        rect.left + (rect.right - rect.left) / 2,
        rect.top + (rect.bottom - rect.top) / 2,
    )
}

/// A rectangle as the box editor measures it: the CSS pixels of its own window.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct CssBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl CssBox {
    /// The same rectangle in physical pixels, grown to the smallest size it may
    /// have and kept inside `screen`.
    fn to_screen(
        self,
        origin: (i32, i32),
        scale: f64,
        min: (i32, i32),
        screen: ScreenRect,
    ) -> ScreenRect {
        let width =
            ((self.width * scale).round() as i32).max(min.0.min(screen.right - screen.left));
        let height =
            ((self.height * scale).round() as i32).max(min.1.min(screen.bottom - screen.top));
        let left = origin.0 + (self.x * scale).round() as i32;
        let top = origin.1 + (self.y * scale).round() as i32;
        let left = left.clamp(screen.left, (screen.right - width).max(screen.left));
        let top = top.clamp(screen.top, (screen.bottom - height).max(screen.top));
        ScreenRect {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    /// The same rectangle as the editor draws it, given the window it is drawn in.
    fn of(rect: ScreenRect, origin: (i32, i32), scale: f64) -> serde_json::Value {
        serde_json::json!({
            "x": (rect.left - origin.0) as f64 / scale,
            "y": (rect.top - origin.1) as f64 / scale,
            "width": (rect.right - rect.left) as f64 / scale,
            "height": (rect.bottom - rect.top) as f64 / scale,
        })
    }
}

/// Opens the box editor over the monitor the boxes are on.
///
/// The two rectangles are drawn where they are now and can be dragged and
/// resized in place; the window covers that one monitor, so the mouse is the
/// editor's for as long as it is up and the reading is out of the way behind it.
pub fn begin_edit(app: &AppHandle) -> Result<(), String> {
    let Some((area, place)) = boxes() else {
        return Err("The subtitle boxes have not been picked yet.".to_string());
    };
    let Some(window) = app.get_webview_window(EDIT_LABEL) else {
        return Err("The box editor window is not available.".to_string());
    };
    let Some((screen, scale)) = monitor_of(app, area) else {
        return Err("The screen under the boxes could not be found.".to_string());
    };

    hide(app, SUBTITLE_LABEL);
    hide(app, AREA_LABEL);
    let _ = window.set_position(PhysicalPosition::new(screen.left, screen.top));
    let _ = window.set_size(PhysicalSize::new(
        (screen.right - screen.left) as u32,
        (screen.bottom - screen.top) as u32,
    ));
    let _ = window.set_always_on_top(true);
    let _ = window.show();
    let _ = window.set_focus();
    // The page measures itself in CSS pixels, so the boxes are handed over the
    // same way and come back the same way.
    let _ = window.emit(
        "glossy://subtitle-edit",
        serde_json::json!({
            "area": CssBox::of(area, (screen.left, screen.top), scale),
            "place": CssBox::of(place, (screen.left, screen.top), scale),
        }),
    );
    Ok(())
}

/// Puts the editor away without moving either box.
pub fn edit_cancel(app: &AppHandle) {
    hide(app, EDIT_LABEL);
    frames(app);
}

/// Moves both boxes to where the editor left them.
///
/// The two are remembered rather than re-picked, and a reading that is running
/// keeps running: it asks for the area every tick, so the next line is read from
/// the box the user just drew, and the translation is drawn in the other one.
pub fn edit_apply(app: &AppHandle, area: CssBox, place: CssBox) {
    let Some((screen, scale)) = (boxes()
        .map(|(area, _)| area)
        .and_then(|area| monitor_of(app, area)))
    .or_else(|| {
        let (x, y) = platform::desktop::cursor_pos();
        monitor_of(
            app,
            ScreenRect {
                left: x,
                top: y,
                right: x + 1,
                bottom: y + 1,
            },
        )
    }) else {
        hide(app, EDIT_LABEL);
        return;
    };
    let origin = (screen.left, screen.top);
    let smallest = crate::ocr::MIN_REGION as i32;
    let area = area.to_screen(origin, scale, (smallest, smallest), screen);
    let place = place.to_screen(
        origin,
        scale,
        (
            (MIN_WIDTH * scale).round() as i32,
            (MIN_HEIGHT * scale).round() as i32,
        ),
        screen,
    );
    if let Ok(mut guard) = AREA.lock() {
        *guard = Some(area);
    }
    if let Ok(mut guard) = PLACE.lock() {
        *guard = Some(place);
    }
    hide(app, EDIT_LABEL);
    if RUNNING.load(Ordering::SeqCst) {
        frames(app);
        // The window the editor left is the one the line is drawn in, so the
        // page inside it is told to measure itself again.
        if let Some(window) = app.get_webview_window(SUBTITLE_LABEL) {
            let _ = window.emit("glossy://subtitle-resize", ());
        }
    } else {
        hide(app, SUBTITLE_LABEL);
        hide(app, AREA_LABEL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: ScreenRect = ScreenRect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };

    fn boxed(x: f64, y: f64, width: f64, height: f64) -> CssBox {
        CssBox {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn a_box_dragged_off_the_screen_comes_back_to_it() {
        // Dragging a box past the edge of the monitor is how a user aims at the
        // very edge of the picture: the box is kept whole instead of being
        // taken off the screen, where nothing could be read or drawn.
        let moved = boxed(1900.0, 1060.0, 400.0, 200.0).to_screen((0, 0), 1.0, (8, 8), SCREEN);
        assert_eq!(
            moved,
            ScreenRect {
                left: 1520,
                top: 880,
                right: 1920,
                bottom: 1080
            }
        );
        let above = boxed(-300.0, -40.0, 400.0, 200.0).to_screen((0, 0), 1.0, (8, 8), SCREEN);
        assert_eq!(
            above,
            ScreenRect {
                left: 0,
                top: 0,
                right: 400,
                bottom: 200
            }
        );
    }

    #[test]
    fn a_box_is_never_smaller_than_what_it_has_to_hold() {
        // Shrinking a box to nothing would leave a reading that can never come
        // back with a line: the smallest sizes are the ones the feature needs,
        // not the ones the mouse happened to stop at.
        let tiny = boxed(100.0, 100.0, 2.0, 2.0).to_screen((0, 0), 1.0, (8, 8), SCREEN);
        assert_eq!(tiny.right - tiny.left, 8);
        assert_eq!(tiny.bottom - tiny.top, 8);
        let line = boxed(100.0, 100.0, 2.0, 2.0).to_screen((0, 0), 1.0, (160, 44), SCREEN);
        assert_eq!(line.right - line.left, 160);
        assert_eq!(line.bottom - line.top, 44);
    }

    #[test]
    fn a_box_on_a_scaled_or_offset_monitor_is_measured_where_it_was_drawn() {
        // The editor measures in the CSS pixels of its own window, which sits at
        // the origin of the monitor it covers: a 150% monitor with the origin at
        // 1920 comes back as the same box in physical pixels, and goes out as
        // the same numbers it was drawn with.
        let screen = ScreenRect {
            left: 1920,
            top: 0,
            right: 3840,
            bottom: 1440,
        };
        let drawn = boxed(100.0, 50.0, 400.0, 80.0);
        let physical = drawn.to_screen((1920, 0), 1.5, (8, 8), screen);
        assert_eq!(
            physical,
            ScreenRect {
                left: 2070,
                top: 75,
                right: 2670,
                bottom: 195
            }
        );
        assert_eq!(
            CssBox::of(physical, (1920, 0), 1.5),
            serde_json::json!({ "x": 100.0, "y": 50.0, "width": 400.0, "height": 80.0 })
        );
    }

    #[test]
    fn a_reading_that_starts_right_after_a_stop_still_has_its_two_boxes() {
        // The order a reading starts in, without a screen: whatever was going on
        // is ended, and then the pair this run is made of is kept. A start that
        // kept only half of it read nothing at all while reporting that it was
        // running, and drew neither of its two boxes.
        let area = ScreenRect {
            left: 10,
            top: 20,
            right: 1010,
            bottom: 220,
        };
        let place = ScreenRect {
            left: 100,
            top: 400,
            right: 900,
            bottom: 480,
        };
        forget();
        keep(area, place);
        assert_eq!(boxes(), Some((area, place)));
    }

    #[test]
    fn a_stop_forgets_the_pair_so_the_next_reading_picks_it_again() {
        keep(
            ScreenRect {
                left: 0,
                top: 0,
                right: 100,
                bottom: 100,
            },
            ScreenRect {
                left: 0,
                top: 200,
                right: 400,
                bottom: 260,
            },
        );
        forget();
        assert_eq!(boxes(), None);
    }

    #[test]
    fn a_line_that_has_not_changed_is_not_translated_again() {
        // The check the loop makes, without a screen to read: the same reading
        // twice is one translation, and a reading too short to be a subtitle is
        // none at all.
        let mut last = String::new();
        let mut translated = 0;
        for reading in ["Hello there", "Hello there", "Hi", "", "Hello there!"] {
            let normalised = crate::text::normalize(reading);
            if normalised.chars().count() < MIN_CHARS || normalised == last {
                continue;
            }
            last = normalised;
            translated += 1;
        }
        assert_eq!(translated, 3);
    }

    #[test]
    fn the_window_is_never_narrower_than_a_subtitle() {
        assert_eq!(
            (
                (100.0f64 / 1.0).max(MIN_WIDTH),
                (10.0f64 / 1.0).max(MIN_HEIGHT)
            ),
            (160.0, 44.0)
        );
    }
}
