//! What a click means, driving the "select text -> translate" flow.
//!
//! [`crate::platform::input_hook`] reports raw left button events and this
//! module decides what they meant: whether the click landed on one of Glossy's
//! own cards, whether it was the second click of a double click, whether it
//! dragged. The work that follows — capture the selection, translate it, show
//! the card — is handed to a worker thread over a channel, because the hook
//! that delivered the click must stay cheap.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::classify;
use crate::context;
use crate::notice;
use crate::platform;
use crate::platform::clipboard::{self, Capture};
use crate::platform::hotkey;
use crate::platform::input_hook::Click;
use crate::popup;
use crate::settings::{self, Settings};
use crate::state::AppState;
use crate::text;

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
/// reading in, so they must never start a translation. The card's own windows
/// count as well, which is what keeps the list of a `<select>` from turning a
/// press on one of its entries into a translation of whatever lies behind it.
fn on_overlay(x: i32, y: i32) -> bool {
    popup::contains(x, y) || popup::owns_point(x, y) || notice::contains(x, y)
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

/// Decides what one left button event means and forwards the outcome.
///
/// Called on the hook thread, so it stays short: the click chain is tracked in
/// a thread local and the decision is sent to the worker thread.
fn handle_click(click: Click) {
    let (x, y) = (click.x, click.y);
    TRACKER.with(|tracker| {
        let mut tracker = match tracker.try_borrow_mut() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if click.pressed {
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
        } else {
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
            // A drag that started on one of our own cards (the popup header,
            // the start hint) or that ends on one must never trigger a
            // translation.
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
    });
}

/// Starts the hook thread and the worker thread that performs the capture.
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

    std::thread::spawn(move || {
        let reload_app = app.clone();
        let reload_state = Arc::clone(&state);
        let on_reload = move || {
            // The first call means the hook is live; it is also what registers
            // the accelerator, and every later call re-registers it after a
            // settings change.
            reload_state.hooked.store(true, Ordering::Relaxed);
            hotkey::reload(&reload_state);
            let _ = reload_app.emit("glossy://status", ());
        };

        let fire_app = app.clone();
        let fire_state = Arc::clone(&state);
        let on_fire = move || {
            // Never block: the hook has to stay responsive.
            let app = fire_app.clone();
            let state = Arc::clone(&fire_state);
            std::thread::spawn(move || on_hotkey(&app, &state));
        };

        if let Err(error) = platform::input_hook::run(handle_click, on_reload, on_fire) {
            state.set_hook_error(error);
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
                // A click on the list of a language dropdown counts as a click
                // on the card: the list reaches past the card's edges, so the
                // position alone would call it a click on the program behind.
                if popup::dismisses_click(x, y) && !popup::owns_point(x, y) {
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
        name: platform::desktop::process_under_point(x, y),
    };
    let _ = app.emit("glossy://picked-app", picked);
}

fn on_trigger(app: &AppHandle, state: &AppState, x: i32, y: i32, trigger: Trigger) {
    let settings = state.settings();
    if !allows(&settings, trigger) {
        return;
    }
    // Selections made inside our own windows are handled by the webview.
    if platform::desktop::foreground_is_self() {
        return;
    }
    if is_ignored(&settings) {
        return;
    }
    // A double click on the desktop, the taskbar or the start menu selects no
    // text at all, and the Ctrl+C sent right after only copies whatever the
    // clipboard happened to hold.
    if platform::desktop::shell_surface_at(x, y) {
        return;
    }
    // The click can also hand the foreground to the desktop while the icon stays
    // selected, so check where the copy would land as well.
    if platform::desktop::copy_target_is_shell() {
        return;
    }

    let Capture::Text(text) = clipboard::capture_selection(settings.restore_clipboard) else {
        return;
    };

    let text = text::normalize(&text);
    if !selection_worth_translating(&settings, &text) {
        return;
    }
    if !settings::allows_source(&settings.source_langs, &text) {
        return;
    }

    // Re-read the settings: the toggle may have been switched while we copied.
    if !state.settings().enabled {
        return;
    }

    let context = enclosing_sentence(&state.settings(), &text);
    popup::reveal(app, state, text, context, (x as f64, y as f64));
}

/// The sentence the selection stands in, when that was asked for.
///
/// Read here rather than when the card asks for the word details, because the
/// program the text came from still has the focus at this point: a moment later
/// the popup is on screen and the focused element may be its own webview.
fn enclosing_sentence(settings: &Settings, text: &str) -> Option<String> {
    if !settings.word_sentence || classify::classify(text) != classify::Kind::Word {
        return None;
    }
    context::sentence(text)
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
    if platform::desktop::foreground_is_self() {
        return;
    }
    if is_ignored(&settings) {
        return;
    }

    let selection = clipboard::capture_selection(settings.restore_clipboard);
    let Some(text) = preferred_text(selection, clipboard::read_text()) else {
        return;
    };

    let text = text::normalize(&text);
    if !long_enough(&settings, &text) {
        return;
    }
    if !settings::allows_source(&settings.source_langs, &text) {
        return;
    }

    let (x, y) = platform::desktop::cursor_pos();
    let context = enclosing_sentence(&settings, &text);
    popup::reveal(app, state, text, context, (x as f64, y as f64));
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
    platform::desktop::foreground_process_name()
        .is_some_and(|process| settings::ignores_process(&settings.ignored_apps, &process))
}

/// True when `text` is long enough to be worth a translation request.
fn long_enough(settings: &Settings, text: &str) -> bool {
    text.trim().chars().count() >= settings.min_selection_len
}

/// True when a selection is worth translating.
///
/// Stricter than `long_enough`, because the drag and double click triggers work
/// without the user asking for a translation: a copy that was sent without a
/// selection often leaves a run of digits, punctuation or the path of a file on
/// the clipboard.
fn selection_worth_translating(settings: &Settings, text: &str) -> bool {
    long_enough(settings, text) && text.chars().any(char::is_alphabetic) && !looks_like_a_path(text)
}

/// True for the path of a file or folder, e.g. `C:\Windows` or `\\nas\share`.
fn looks_like_a_path(text: &str) -> bool {
    if text.contains('\n') {
        return false;
    }
    let mut chars = text.chars();
    let drive = matches!(
        (chars.next(), chars.next()),
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic())
        && matches!(chars.next(), Some('\\') | Some('/'));
    drive || text.starts_with("\\\\") || text.starts_with("//")
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
    fn a_selection_needs_a_letter_to_be_worth_translating() {
        let settings = settings();
        assert!(selection_worth_translating(&settings, "hello"));
        assert!(selection_worth_translating(&settings, "你好"));
        assert!(selection_worth_translating(&settings, "3 apples"));
        // Numbers, punctuation and symbols carry no language.
        assert!(!selection_worth_translating(&settings, "1234"));
        assert!(!selection_worth_translating(&settings, "12.34%"));
        assert!(!selection_worth_translating(&settings, "->"));
    }

    #[test]
    fn a_file_path_is_not_a_selection() {
        let settings = settings();
        assert!(looks_like_a_path("C:\\Windows"));
        assert!(looks_like_a_path("d:/photos/holiday.jpg"));
        assert!(looks_like_a_path("\\\\nas\\share\\notes.txt"));
        assert!(looks_like_a_path("//nas/share"));
        assert!(!looks_like_a_path("The file is at C:\\Windows"));
        assert!(!looks_like_a_path("Ratio 16:9"));
        assert!(!looks_like_a_path("C:"));
        assert!(!looks_like_a_path("hello"));
        assert!(!looks_like_a_path("C:\\a\nD:\\b"));
        assert!(!selection_worth_translating(
            &settings,
            "C:\\Windows\\System32"
        ));
    }

    #[test]
    fn a_source_language_whitelist_lets_other_languages_through_when_empty() {
        let settings = settings();
        assert!(settings.source_langs.is_empty());
        assert!(settings::allows_source(&settings.source_langs, "hello"));
    }

    #[test]
    fn only_the_listed_source_languages_trigger() {
        let wanted = vec!["en".to_string()];
        assert!(settings::allows_source(
            &wanted,
            "The quick brown fox is here"
        ));
        assert!(!settings::allows_source(
            &wanted,
            "Le chat est dans la maison"
        ));
        // A region variant of a listed language still matches.
        assert!(settings::allows_source(
            &["zh-CN".to_string()],
            "翻译选中的文字"
        ));
        // A language that cannot be told apart is translated anyway.
        assert!(settings::allows_source(&wanted, "ok"));
        assert!(settings::allows_source(&wanted, "1234"));
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

    /// The click tracker is a thread local and [`handle_click`] reports through
    /// the channel the app installs at startup, so this one test owns that
    /// channel and walks the whole chain: a click that selects nothing, a drag,
    /// and the second click of a double click.
    #[test]
    fn the_hook_reports_a_click_a_drag_and_a_double_click() {
        let (tx, rx) = channel::<HookEvent>();
        let _ = EVENT_TX.set(tx);

        let click = |pressed: bool, x: i32, y: i32| handle_click(Click { pressed, x, y });
        let next = || rx.try_recv().expect("an event was reported");

        // A click that never moved selects nothing.
        click(true, 10, 10);
        assert!(matches!(next(), HookEvent::ButtonDown { x: 10, y: 10 }));
        click(false, 10, 10);
        assert!(rx.try_recv().is_err());

        // Press, move, release is a drag.
        click(true, 500, 500);
        assert!(matches!(next(), HookEvent::ButtonDown { x: 500, y: 500 }));
        click(false, 540, 500);
        assert!(matches!(
            next(),
            HookEvent::ButtonUp {
                trigger: Trigger::Drag,
                ..
            }
        ));

        // The second click of a double click reports the word under the cursor.
        click(true, 500, 500);
        assert!(matches!(next(), HookEvent::ButtonDown { x: 500, y: 500 }));
        click(false, 500, 500);
        assert!(rx.try_recv().is_err());
        click(true, 500, 500);
        assert!(matches!(next(), HookEvent::ButtonDown { x: 500, y: 500 }));
        click(false, 500, 500);
        assert!(matches!(
            next(),
            HookEvent::ButtonUp {
                trigger: Trigger::DoubleClick,
                ..
            }
        ));
    }
}
