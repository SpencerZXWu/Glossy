//! Global low level mouse hook driving the "select text -> translate" flow.
//!
//! The hook callback itself must stay extremely cheap: Windows silently
//! removes hooks that block for too long. It therefore only records click
//! coordinates and forwards a decision to a worker thread over a channel.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, MSG, MSLLHOOKSTRUCT, WH_MOUSE_LL,
    WM_LBUTTONDOWN, WM_LBUTTONUP,
};

use crate::clipboard::{self, Capture};
use crate::hotkey;
use crate::notice;
use crate::platform;
use crate::popup;
use crate::settings::{self, Settings};
use crate::state::AppState;

/// Minimum pointer travel (physical px) for a press/drag/release to count.
const DRAG_MIN_PX: i32 = 5;
/// Maximum delay between the two clicks of a double click.
const DOUBLE_CLICK_MS: u128 = 450;
/// Maximum pointer travel between the two clicks of a double click.
const DOUBLE_CLICK_SLOP: i32 = 6;
/// How long a double click waits for the rest of a click chain. Triple and
/// quadruple clicks select a sentence or a paragraph, and every one of those
/// clicks would otherwise translate on its own.
const CHAIN_SETTLE_MS: u64 = 250;
/// How long "pick the program under the cursor" waits for a click.
const PICK_TIMEOUT_MS: u64 = 30_000;

static EVENT_TX: OnceLock<Sender<HookEvent>> = OnceLock::new();
/// When the pick mode was armed, in milliseconds since the app started.
static PICK_DEADLINE: AtomicU64 = AtomicU64::new(0);
static STARTED: OnceLock<Instant> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// Press, move, release in another place.
    Drag,
    /// Second click of a double click: the word under the cursor.
    DoubleClick,
}

#[derive(Debug, Clone, Copy)]
enum HookEvent {
    ButtonDown {
        x: i32,
        y: i32,
    },
    ButtonUp {
        x: i32,
        y: i32,
        trigger: Trigger,
    },
    /// The next click after a pick request: report the program under it.
    Pick {
        x: i32,
        y: i32,
    },
}

/// Program reported by the "click a window" picker.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PickedApp {
    /// `None` when no program owns the window under the cursor.
    name: Option<String>,
}

/// Arms the pick mode: the next mouse release reports the program under the
/// cursor instead of translating the selection.
pub fn arm_pick() {
    PICK_DEADLINE.store(uptime_ms() + PICK_TIMEOUT_MS, Ordering::Relaxed);
}

/// Consumes an armed pick request. Kept cheap: it runs inside the mouse hook.
fn take_pick() -> bool {
    let deadline = PICK_DEADLINE.load(Ordering::Relaxed);
    if deadline == 0 || uptime_ms() >= deadline {
        PICK_DEADLINE.store(0, Ordering::Relaxed);
        return false;
    }
    PICK_DEADLINE.store(0, Ordering::Relaxed);
    true
}

fn uptime_ms() -> u64 {
    STARTED.get_or_init(Instant::now).elapsed().as_millis() as u64
}

thread_local! {
    static TRACKER: RefCell<Tracker> = RefCell::new(Tracker::default());
}

#[derive(Default)]
struct Tracker {
    down: Option<Down>,
    last_click: Option<(i32, i32, u128)>,
}

struct Down {
    x: i32,
    y: i32,
    inside_overlay: bool,
    repeated: bool,
}

/// True when the point lands on one of Glossy's own floating cards.
///
/// Clicks and drags there belong to the card, not to the program the user is
/// reading in, so they must never start a translation.
fn on_overlay(x: i32, y: i32) -> bool {
    popup::contains(x, y) || notice::contains(x, y)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn send(event: HookEvent) {
    if let Some(tx) = EVENT_TX.get() {
        let _ = tx.send(event);
    }
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let message = wparam.0 as u32;
        if message == WM_LBUTTONDOWN || message == WM_LBUTTONUP {
            let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
            let (x, y) = (info.pt.x, info.pt.y);
            TRACKER.with(|tracker| {
                let mut tracker = match tracker.try_borrow_mut() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };
                match message {
                    WM_LBUTTONDOWN => {
                        let now = now_ms();
                        let repeated = tracker.last_click.is_some_and(|(lx, ly, lt)| {
                            now.saturating_sub(lt) <= DOUBLE_CLICK_MS
                                && (x - lx).abs() <= DOUBLE_CLICK_SLOP
                                && (y - ly).abs() <= DOUBLE_CLICK_SLOP
                        });
                        tracker.last_click = Some((x, y, now));
                        tracker.down = Some(Down {
                            x,
                            y,
                            inside_overlay: on_overlay(x, y),
                            repeated,
                        });
                        send(HookEvent::ButtonDown { x, y });
                    }
                    _ => {
                        // A pick request swallows the click it was armed for.
                        if take_pick() {
                            tracker.down = None;
                            tracker.last_click = None;
                            send(HookEvent::Pick { x, y });
                            return;
                        }
                        let Some(down) = tracker.down.take() else {
                            return;
                        };
                        // A drag that started on one of our own cards (the popup
                        // header, the start hint) or that ends on one must never
                        // trigger a translation.
                        if down.inside_overlay || on_overlay(x, y) {
                            return;
                        }
                        let moved = (x - down.x).abs().max((y - down.y).abs());
                        let trigger = if down.repeated {
                            Some(Trigger::DoubleClick)
                        } else if moved >= DRAG_MIN_PX {
                            Some(Trigger::Drag)
                        } else {
                            None
                        };
                        if moved >= DRAG_MIN_PX {
                            tracker.last_click = None;
                        } else {
                            tracker.last_click = Some((x, y, now_ms()));
                        }
                        if let Some(trigger) = trigger {
                            send(HookEvent::ButtonUp { x, y, trigger });
                        }
                    }
                }
            });
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Installs the hook and starts the worker thread that performs the capture.
///
/// Returns immediately; hook failures are recorded in `AppState::hook_error`.
pub fn install(app: AppHandle, state: Arc<AppState>) {
    let (tx, rx) = channel::<HookEvent>();
    if EVENT_TX.set(tx).is_err() {
        return;
    }

    let worker_app = app.clone();
    let worker_state = Arc::clone(&state);
    std::thread::spawn(move || worker(worker_app, worker_state, rx));

    std::thread::spawn(move || unsafe {
        // The handle is kept alive for the lifetime of the message loop.
        let _hook = match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), None, 0) {
            Ok(hook) => hook,
            Err(error) => {
                state.set_hook_error(format!("could not install the mouse hook: {error}"));
                return;
            }
        };

        state.hooked.store(true, Ordering::Relaxed);

        // Hotkeys are delivered to the thread that registered them, so the
        // accelerator shares this message loop with the hook.
        hotkey::register_loop_thread();
        hotkey::reload(&state);
        let _ = app.emit("glossy://status", ());

        let mut message = MSG::default();
        // A message loop is required for low level hooks to be delivered.
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            if message.message == hotkey::WM_RELOAD {
                hotkey::reload(&state);
                let _ = app.emit("glossy://status", ());
            } else if hotkey::is_hotkey_message(message.message, message.wParam.0) {
                // Never block: the hook has to stay responsive.
                let hotkey_app = app.clone();
                let hotkey_state = Arc::clone(&state);
                std::thread::spawn(move || on_hotkey(&hotkey_app, &hotkey_state));
            }
        }
    });
}

fn worker(app: AppHandle, state: Arc<AppState>, rx: Receiver<HookEvent>) {
    let mut pending: Option<HookEvent> = None;
    loop {
        let event = match pending.take() {
            Some(event) => event,
            None => match rx.recv() {
                Ok(event) => event,
                Err(_) => return,
            },
        };
        match event {
            HookEvent::ButtonDown { x, y } => {
                if popup::dismisses_click(x, y) {
                    popup::hide(&app);
                }
            }
            HookEvent::ButtonUp { x, y, trigger } => {
                let (x, y) = settle_click_chain(&rx, &mut pending, x, y, trigger);
                on_trigger(&app, &state, x, y, trigger);
            }
            HookEvent::Pick { x, y } => on_pick(&app, x, y),
        }
    }
}

/// True when `b` looks like another click of the chain started at `a`.
fn same_click_chain(a: (i32, i32), b: (i32, i32)) -> bool {
    (a.0 - b.0).abs() <= DOUBLE_CLICK_SLOP && (a.1 - b.1).abs() <= DOUBLE_CLICK_SLOP
}

/// Waits out the rest of a click chain and returns the coordinates to translate.
///
/// Triple and quadruple clicks select a sentence or a paragraph, and each of
/// those clicks reports the same double click trigger near the same spot. Waiting
/// briefly collapses the chain into a single translation of whatever the last
/// click of the chain selected. Dragging is never delayed because it cannot be
/// followed by another click of the same chain.
fn settle_click_chain(
    rx: &Receiver<HookEvent>,
    pending: &mut Option<HookEvent>,
    x: i32,
    y: i32,
    trigger: Trigger,
) -> (i32, i32) {
    if trigger != Trigger::DoubleClick {
        return (x, y);
    }

    let (mut x, mut y) = (x, y);
    let mut deadline = Instant::now() + Duration::from_millis(CHAIN_SETTLE_MS);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        let Ok(event) = rx.recv_timeout(remaining) else {
            break;
        };
        match event {
            HookEvent::ButtonUp {
                x: next_x,
                y: next_y,
                trigger: Trigger::DoubleClick,
            } if same_click_chain((x, y), (next_x, next_y)) => {
                x = next_x;
                y = next_y;
                // A longer chain keeps the wait open for the next click.
                deadline = Instant::now() + Duration::from_millis(CHAIN_SETTLE_MS);
            }
            other => {
                // Not part of the chain: keep it for the main loop.
                *pending = Some(other);
                break;
            }
        }
    }
    (x, y)
}

/// Reports the program the user clicked on while the pick mode was armed.
fn on_pick(app: &AppHandle, x: i32, y: i32) {
    let picked = PickedApp {
        name: platform::process_under_point(x, y),
    };
    let _ = app.emit("glossy://picked-app", picked);
}

fn on_trigger(app: &AppHandle, state: &AppState, x: i32, y: i32, trigger: Trigger) {
    let settings = state.settings();
    if !allows(&settings, trigger) {
        return;
    }
    // Selections made inside our own windows are handled by the webview.
    if platform::foreground_is_self() {
        return;
    }
    if is_ignored(&settings) {
        return;
    }

    let Capture::Text(text) = clipboard::capture_selection(settings.restore_clipboard) else {
        return;
    };

    let text = text.trim().to_string();
    if !long_enough(&settings, &text) {
        return;
    }

    // Re-read the settings: the toggle may have been switched while we copied.
    if !state.settings().enabled {
        return;
    }

    popup::reveal(app, state, text, (x as f64, y as f64));
}

/// Translates the text the user has selected when the hotkey is pressed.
///
/// The live selection is the primary source, so a stale clipboard from an
/// earlier copy can never be translated by mistake; the clipboard only serves
/// as the fallback for "copy first, then press the hotkey".
fn on_hotkey(app: &AppHandle, state: &AppState) {
    let settings = state.settings();
    if !settings.enabled {
        return;
    }
    // The hotkey is pressed inside another program, so there is nothing to do
    // when one of our own windows is in front or the program is ignored.
    if platform::foreground_is_self() {
        return;
    }
    if is_ignored(&settings) {
        return;
    }

    let selection = clipboard::capture_selection(settings.restore_clipboard);
    let Some(text) = preferred_text(selection, clipboard::read_text()) else {
        return;
    };

    let text = text.trim().to_string();
    if !long_enough(&settings, &text) {
        return;
    }

    let (x, y) = platform::cursor_pos();
    popup::reveal(app, state, text, (x as f64, y as f64));
}

/// The text to translate: the selection that was just copied, or the clipboard
/// content when the foreground program had nothing selected.
fn preferred_text(selection: Capture, clipboard_text: Option<String>) -> Option<String> {
    if let Capture::Text(text) = selection {
        if !text.trim().is_empty() {
            return Some(text);
        }
    }
    clipboard_text.filter(|text| !text.trim().is_empty())
}

fn allows(settings: &Settings, trigger: Trigger) -> bool {
    if !settings.enabled {
        return false;
    }
    match trigger {
        Trigger::Drag => settings.trigger_on_drag,
        Trigger::DoubleClick => settings.trigger_on_double_click,
    }
}

/// True when the foreground application is on the ignored list.
fn is_ignored(settings: &Settings) -> bool {
    if settings.ignored_apps.is_empty() {
        return false;
    }
    platform::foreground_process_name()
        .is_some_and(|process| settings::ignores_process(&settings.ignored_apps, &process))
}

/// True when `text` is long enough to be worth a translation request.
fn long_enough(settings: &Settings, text: &str) -> bool {
    text.trim().chars().count() >= settings.min_selection_len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Provider;

    fn settings() -> Settings {
        Settings {
            provider: Provider::Google,
            ..Settings::default()
        }
    }

    #[test]
    fn respects_the_master_toggle() {
        let mut settings = settings();
        settings.enabled = false;
        assert!(!allows(&settings, Trigger::Drag));
        assert!(!allows(&settings, Trigger::DoubleClick));
    }

    #[test]
    fn respects_individual_triggers() {
        let mut settings = settings();
        settings.trigger_on_drag = false;
        settings.trigger_on_double_click = true;
        assert!(!allows(&settings, Trigger::Drag));
        assert!(allows(&settings, Trigger::DoubleClick));
    }

    #[test]
    fn honours_the_configured_minimum_length() {
        let mut settings = settings();
        assert!(long_enough(&settings, "hi"));
        assert!(!long_enough(&settings, "h"));

        settings.min_selection_len = 4;
        assert!(!long_enough(&settings, "abc"));
        assert!(long_enough(&settings, "  abcd  "));
    }

    #[test]
    fn groups_the_clicks_of_one_chain() {
        // A triple click stays within the slop of the previous click.
        assert!(same_click_chain((100, 200), (103, 197)));
        assert!(same_click_chain((100, 200), (100, 200)));
        // A click somewhere else is a new selection.
        assert!(!same_click_chain((100, 200), (140, 200)));
        assert!(!same_click_chain((100, 200), (100, 260)));
    }

    #[test]
    fn keeps_a_queued_event_that_is_not_part_of_the_chain() {
        let (tx, rx) = channel();
        let mut pending = None;
        tx.send(HookEvent::ButtonDown { x: 900, y: 900 }).unwrap();

        let (x, y) = settle_click_chain(&rx, &mut pending, 100, 200, Trigger::DoubleClick);
        assert_eq!((x, y), (100, 200));
        assert!(matches!(pending, Some(HookEvent::ButtonDown { .. })));
    }

    #[test]
    fn dragging_is_never_delayed() {
        let (_tx, rx) = channel();
        let mut pending = None;
        let start = Instant::now();
        let (x, y) = settle_click_chain(&rx, &mut pending, 100, 200, Trigger::Drag);
        assert_eq!((x, y), (100, 200));
        assert!(start.elapsed() < Duration::from_millis(CHAIN_SETTLE_MS));
        assert!(pending.is_none());
    }

    #[test]
    fn the_hotkey_prefers_the_live_selection() {
        let text = preferred_text(
            Capture::Text("selected".to_string()),
            Some("copied earlier".to_string()),
        );
        assert_eq!(text.as_deref(), Some("selected"));
    }

    #[test]
    fn the_hotkey_falls_back_to_the_clipboard() {
        let clipboard = Some("copied earlier".to_string());
        for empty in [Capture::NoSelection, Capture::NoResponse] {
            let text = preferred_text(empty, clipboard.clone());
            assert_eq!(text.as_deref(), Some("copied earlier"));
        }
        assert_eq!(
            preferred_text(Capture::Text("   ".to_string()), clipboard).as_deref(),
            Some("copied earlier")
        );
    }

    #[test]
    fn the_hotkey_needs_text_from_somewhere() {
        assert_eq!(preferred_text(Capture::NoSelection, None), None);
        assert_eq!(
            preferred_text(Capture::NoResponse, Some("  \n ".to_string())),
            None
        );
    }
}
