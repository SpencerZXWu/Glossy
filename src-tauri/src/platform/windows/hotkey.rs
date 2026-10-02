//! System wide accelerators, one per action.
//!
//! Windows delivers `WM_HOTKEY` to the thread that called `RegisterHotKey`, so
//! every accelerator is registered on the mouse hook message loop owned by
//! `selection` instead of on a thread of its own. Each [`Slot`] owns an
//! identifier of its own, which is what tells the loop which action was asked
//! for.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN, VIRTUAL_KEY, VK_BACK, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME,
    VK_INSERT, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_APP, WM_HOTKEY};

use crate::settings::Settings;
use crate::state::AppState;

/// Identifier of the first accelerator Glossy owns; the slots count up from it.
const BASE_ID: i32 = 0x6075;
/// Posted to the message loop thread when the accelerator settings changed.
pub const WM_RELOAD: u32 = WM_APP + 7;

/// How many slots a setting drives, so the tables below can be plain arrays.
/// The card's own accelerator is not one of them; see [`Slot::Replace`].
const SLOTS: usize = 3;

/// The combination the card answers to while it is on screen.
///
/// Not a setting: it is registered and released with the card itself, because
/// Ctrl+Enter is what many programs send a message with.
pub const REPLACE_SPEC: &str = "Ctrl+Enter";

static LOOP_THREAD: AtomicU32 = AtomicU32::new(0);
/// Reload requests handed to the loop thread, and the ones it has carried out.
static RELOAD_SENT: AtomicU64 = AtomicU64::new(0);
static RELOAD_DONE: AtomicU64 = AtomicU64::new(0);
/// Registered spelling per slot, indexed by [`Slot::index`].
static ACTIVE: Mutex<[Option<String>; SLOTS]> = Mutex::new([None, None, None]);
/// Why a slot is not registered, when it is not.
static PROBLEM: Mutex<[Option<String>; SLOTS]> = Mutex::new([None, None, None]);

/// One action that can be reached from anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Translate the selection, or the clipboard, and show the popup.
    Translate,
    /// Bring the settings window up.
    Settings,
    /// Pick a region of the screen, read its text, and translate that.
    Ocr,
    /// Write the translation of the card back over the text it came from.
    ///
    /// Deliberately left out of [`Slot::ALL`]: it has no setting, no row on the
    /// shortcuts page, and it is armed only for as long as a card is on screen.
    Replace,
}

impl Slot {
    /// Every slot a setting drives, in the order the shortcuts page lists them.
    pub const ALL: [Slot; SLOTS] = [Slot::Translate, Slot::Settings, Slot::Ocr];

    /// The identifier Windows knows this accelerator by.
    pub fn id(self) -> i32 {
        BASE_ID + self as i32
    }

    /// The settings key that holds the combination.
    pub fn key(self) -> &'static str {
        match self {
            Slot::Translate => "hotkey",
            Slot::Settings => "hotkeySettings",
            Slot::Ocr => "hotkeyOcr",
            Slot::Replace => "hotkeyReplace",
        }
    }

    /// The name the shortcuts page and the status payload use.
    pub fn name(self) -> &'static str {
        match self {
            Slot::Translate => "translate",
            Slot::Settings => "settings",
            Slot::Ocr => "ocr",
            Slot::Replace => "replace",
        }
    }

    /// The combination configured for this slot.
    pub fn spec(self, settings: &Settings) -> String {
        match self {
            Slot::Translate => settings.hotkey.clone(),
            Slot::Settings => settings.hotkey_settings.clone(),
            Slot::Ocr => settings.hotkey_ocr.clone(),
            // The card's combination is fixed and follows the card, so there is
            // nothing to read out of the settings.
            Slot::Replace => REPLACE_SPEC.to_string(),
        }
    }

    /// Row of the per-slot tables, which only hold the configured slots.
    fn index(self) -> usize {
        debug_assert!(
            Slot::ALL.contains(&self),
            "a slot without a setting has no row here"
        );
        self as usize
    }

    fn from_id(id: i32) -> Option<Slot> {
        std::iter::once(Slot::Replace)
            .chain(Slot::ALL)
            .find(|slot| slot.id() == id)
    }
}

/// How one slot is registered, and why it is not when it is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    /// Which action this is about; see [`Slot::name`].
    pub slot: &'static str,
    /// Canonical spelling of the registered combination, when it is active.
    pub spec: Option<String>,
    /// Why the configured combination could not be registered.
    pub error: Option<String>,
}

/// Longest [`status`] waits for a reload that is still on its way.
const RELOAD_WAIT: Duration = Duration::from_millis(300);

/// Parsed accelerator, together with its canonical spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accelerator {
    modifiers: HOT_KEY_MODIFIERS,
    key: VIRTUAL_KEY,
    label: String,
}

/// Parses a specification such as `ctrl + alt + C` or `Win+Shift+F5`.
pub fn parse(spec: &str) -> Result<Accelerator, String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err("Type an accelerator such as Ctrl+Alt+C.".to_string());
    }

    let mut modifiers = MOD_NOREPEAT;
    let mut modifiers_used = false;
    let mut names: Vec<String> = Vec::new();
    let mut key: Option<(VIRTUAL_KEY, String)> = None;

    for part in spec.split('+') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let modifier = match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Some((MOD_CONTROL, "Ctrl")),
            "alt" => Some((MOD_ALT, "Alt")),
            "shift" => Some((MOD_SHIFT, "Shift")),
            "win" | "meta" | "super" | "cmd" => Some((MOD_WIN, "Win")),
            _ => None,
        };
        if let Some((flag, name)) = modifier {
            modifiers = HOT_KEY_MODIFIERS(modifiers.0 | flag.0);
            modifiers_used = true;
            if !names.iter().any(|existing| existing == name) {
                names.push(name.to_string());
            }
            continue;
        }

        let Some(parsed) = key_code(part) else {
            return Err(format!("\"{part}\" is not a key Glossy knows."));
        };
        if key.is_some() {
            return Err(format!("\"{spec}\" has more than one key."));
        }
        key = Some(parsed);
    }

    let Some((key, label)) = key else {
        return Err(format!("\"{spec}\" needs a key, for example Ctrl+Alt+C."));
    };
    if !modifiers_used {
        return Err("Hold at least one of Ctrl, Alt, Shift or Win.".to_string());
    }
    names.push(label);

    Ok(Accelerator {
        modifiers,
        key,
        label: names.join("+"),
    })
}

fn key_code(part: &str) -> Option<(VIRTUAL_KEY, String)> {
    let lower = part.to_ascii_lowercase();
    let named: Option<(VIRTUAL_KEY, &str)> = match lower.as_str() {
        "space" => Some((VK_SPACE, "Space")),
        "enter" | "return" => Some((VK_RETURN, "Enter")),
        "tab" => Some((VK_TAB, "Tab")),
        "esc" | "escape" => Some((VK_ESCAPE, "Esc")),
        "backspace" => Some((VK_BACK, "Backspace")),
        "delete" | "del" => Some((VK_DELETE, "Delete")),
        "insert" | "ins" => Some((VK_INSERT, "Insert")),
        "home" => Some((VK_HOME, "Home")),
        "end" => Some((VK_END, "End")),
        "pageup" | "pgup" => Some((VK_PRIOR, "PageUp")),
        "pagedown" | "pgdn" => Some((VK_NEXT, "PageDown")),
        "up" => Some((VK_UP, "Up")),
        "down" => Some((VK_DOWN, "Down")),
        "left" => Some((VK_LEFT, "Left")),
        "right" => Some((VK_RIGHT, "Right")),
        _ => None,
    };
    if let Some((key, label)) = named {
        return Some((key, label.to_string()));
    }

    let bytes = lower.as_bytes();
    if bytes.len() == 1 {
        let letter = bytes[0];
        if letter.is_ascii_lowercase() {
            let upper = letter.to_ascii_uppercase();
            // Virtual key codes of the letters and digits follow their ASCII order.
            return Some((VIRTUAL_KEY(upper as u16), (upper as char).to_string()));
        }
        if letter.is_ascii_digit() {
            return Some((VIRTUAL_KEY(0x30 + (letter - b'0') as u16), part.to_string()));
        }
    }

    if lower.len() <= 3 && lower.starts_with('f') {
        if let Ok(number) = lower[1..].parse::<u16>() {
            if (1..=24).contains(&number) {
                return Some((VIRTUAL_KEY(0x70 + number - 1), format!("F{number}")));
            }
        }
    }

    None
}

/// Registers one slot on the calling thread and returns its canonical spelling.
///
/// An empty specification simply switches that slot off.
fn install_on_this_thread(slot: Slot, spec: &str) -> Result<String, String> {
    unsafe {
        let _ = UnregisterHotKey(None, slot.id());
    }

    let spec = spec.trim();
    if spec.is_empty() {
        return Ok(String::new());
    }

    let accelerator = parse(spec)?;
    unsafe {
        RegisterHotKey(
            None,
            slot.id(),
            accelerator.modifiers,
            u32::from(accelerator.key.0),
        )
    }
    .map_err(|error| {
        format!(
            "Windows refused {}. Another program probably owns it ({error}).",
            accelerator.label
        )
    })?;
    Ok(accelerator.label)
}

/// Arms or releases the card's own accelerator, on the loop thread.
///
/// Called from [`reload`] with the state of the card, so it rides the same
/// reloads as the configured accelerators and nothing has to remember which
/// thread registered what. A state that is already in force does nothing.
pub fn install_replace(armed: bool) {
    static ARMED: AtomicBool = AtomicBool::new(false);
    if ARMED.load(Ordering::SeqCst) == armed {
        return;
    }
    let outcome = install_on_this_thread(Slot::Replace, if armed { REPLACE_SPEC } else { "" });
    // A refusal (another program owning the key) leaves it off; the card still
    // works through its own button, and the next reload tries again.
    ARMED.store(armed && outcome.is_ok(), Ordering::SeqCst);
}

/// Re-reads every configured accelerator; the outcome is reported by [`status`].
pub fn reload(state: &AppState) {
    let settings = state.settings();
    {
        let mut active = ACTIVE.lock().unwrap();
        let mut problem = PROBLEM.lock().unwrap();
        for slot in Slot::ALL {
            let (found, error) = match install_on_this_thread(slot, &slot.spec(&settings)) {
                Ok(label) if label.is_empty() => (None, None),
                Ok(label) => (Some(label), None),
                Err(message) => (None, Some(message)),
            };
            active[slot.index()] = found;
            problem[slot.index()] = error;
        }
    }
    install_replace(crate::popup::replace_armed());
    RELOAD_DONE.fetch_add(1, Ordering::SeqCst);
}

/// Every slot's registration, with the reason when it is not active.
///
/// The registration happens on the hook thread, so a reload that was just asked
/// for is waited for here: a caller that saves a setting and reads the status
/// right afterwards would otherwise be told about the previous accelerators.
pub fn status() -> Vec<HotkeyStatus> {
    let deadline = Instant::now() + RELOAD_WAIT;
    while RELOAD_DONE.load(Ordering::SeqCst) < RELOAD_SENT.load(Ordering::SeqCst) {
        if Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let active = ACTIVE.lock().unwrap();
    let problem = PROBLEM.lock().unwrap();
    Slot::ALL
        .into_iter()
        .map(|slot| HotkeyStatus {
            slot: slot.name(),
            spec: active[slot.index()].clone(),
            error: problem[slot.index()].clone(),
        })
        .collect()
}

/// Remembers the message loop thread so settings changes can reach it.
pub fn register_loop_thread() {
    LOOP_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
}

/// Asks the message loop thread to pick up a changed accelerator.
pub fn request_reload() {
    let thread = LOOP_THREAD.load(Ordering::Relaxed);
    if thread == 0 {
        return;
    }
    // Counted before the message is posted, so a status check that follows the
    // save waits for this reload instead of reading the previous accelerator.
    RELOAD_SENT.fetch_add(1, Ordering::SeqCst);
    let posted = unsafe { PostThreadMessageW(thread, WM_RELOAD, WPARAM(0), LPARAM(0)) };
    if posted.is_err() {
        // Nothing will be reloaded, so do not let `status` wait for it.
        RELOAD_DONE.fetch_add(1, Ordering::SeqCst);
    }
}

/// The slot whose accelerator the message carries, when it is one of ours.
pub fn slot_of(message: u32, id: usize) -> Option<Slot> {
    if message != WM_HOTKEY {
        return None;
    }
    Slot::from_id(i32::try_from(id).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modifier_bits(accelerator: &Accelerator) -> u32 {
        accelerator.modifiers.0
    }

    #[test]
    fn parses_a_lower_case_specification() {
        let accelerator = parse(" ctrl + alt + c ").expect("should parse");

        assert_eq!(accelerator.label, "Ctrl+Alt+C");
        assert_eq!(accelerator.key, VIRTUAL_KEY(0x43));
        assert_ne!(modifier_bits(&accelerator) & MOD_CONTROL.0, 0);
        assert_ne!(modifier_bits(&accelerator) & MOD_ALT.0, 0);
        assert_eq!(modifier_bits(&accelerator) & MOD_SHIFT.0, 0);
        assert_eq!(modifier_bits(&accelerator) & MOD_NOREPEAT.0, MOD_NOREPEAT.0);
    }

    #[test]
    fn parses_named_keys_and_function_keys() {
        assert_eq!(parse("Win+Space").unwrap().key, VK_SPACE);
        assert_eq!(parse("Ctrl+Shift+F5").unwrap().key, VIRTUAL_KEY(0x74));
        assert_eq!(parse("Ctrl+Alt+7").unwrap().label, "Ctrl+Alt+7");
        assert_eq!(parse("Alt+PageDown").unwrap().label, "Alt+PageDown");
    }

    #[test]
    fn rejects_specifications_that_would_not_work() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
        assert!(parse("C").is_err());
        assert!(parse("Ctrl").is_err());
        assert!(parse("Ctrl+Alt+Q?").is_err());
        assert!(parse("Ctrl+A+B").is_err());
        assert!(parse("Ctrl+F25").is_err());
    }

    #[test]
    fn installs_nothing_for_an_empty_specification() {
        assert_eq!(install_on_this_thread(Slot::Translate, "").unwrap(), "");
        assert_eq!(install_on_this_thread(Slot::Settings, "   ").unwrap(), "");
        assert_eq!(install_on_this_thread(Slot::Ocr, "").unwrap(), "");
    }

    #[test]
    fn every_slot_has_its_own_identifier_key_and_name() {
        let mut ids: Vec<i32> = Slot::ALL.iter().map(|slot| slot.id()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), Slot::ALL.len(), "two slots share an identifier");

        let mut names: Vec<&str> = Slot::ALL.iter().map(|slot| slot.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), Slot::ALL.len(), "two slots share a name");

        assert_eq!(Slot::Translate.id(), BASE_ID);
        assert_eq!(Slot::Translate.key(), "hotkey");
        assert_eq!(Slot::Settings.key(), "hotkeySettings");
        assert_eq!(Slot::Ocr.key(), "hotkeyOcr");
    }

    #[test]
    fn a_message_names_the_slot_that_was_pressed() {
        assert_eq!(
            slot_of(WM_HOTKEY, Slot::Translate.id() as usize),
            Some(Slot::Translate)
        );
        assert_eq!(
            slot_of(WM_HOTKEY, Slot::Settings.id() as usize),
            Some(Slot::Settings)
        );
        assert_eq!(slot_of(WM_HOTKEY, Slot::Ocr.id() as usize), Some(Slot::Ocr));
        assert_eq!(
            slot_of(WM_HOTKEY, Slot::Replace.id() as usize),
            Some(Slot::Replace)
        );
        // An identifier nobody registered, and a message that is not ours.
        assert_eq!(slot_of(WM_HOTKEY, 0), None);
        assert_eq!(slot_of(WM_RELOAD, Slot::Translate.id() as usize), None);
    }

    #[test]
    fn the_card_has_an_identifier_of_its_own_but_no_row_of_its_own() {
        // A row of its own would put Ctrl+Enter on the shortcuts page, where it
        // would look like something the user can change.
        assert!(!Slot::ALL.contains(&Slot::Replace));
        let mut ids: Vec<i32> = Slot::ALL.iter().map(|slot| slot.id()).collect();
        ids.push(Slot::Replace.id());
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), SLOTS + 1, "two slots share an identifier");

        // The spelling is the one the card promises, so a change to it cannot
        // quietly leave the card unreachable.
        assert_eq!(
            parse(REPLACE_SPEC)
                .expect("the card's combination parses")
                .label,
            "Ctrl+Enter"
        );
    }

    #[test]
    fn a_slot_reads_its_own_combination() {
        let settings = Settings {
            hotkey: "Ctrl+Alt+C".to_string(),
            hotkey_settings: "Ctrl+Alt+G".to_string(),
            hotkey_ocr: "Ctrl+Alt+Q".to_string(),
            ..Settings::default()
        };

        assert_eq!(Slot::Translate.spec(&settings), "Ctrl+Alt+C");
        assert_eq!(Slot::Settings.spec(&settings), "Ctrl+Alt+G");
        assert_eq!(Slot::Ocr.spec(&settings), "Ctrl+Alt+Q");
    }

    #[test]
    fn reading_the_status_waits_for_a_pending_reload_only() {
        let started = Instant::now();
        let _ = status();
        assert!(
            started.elapsed() < RELOAD_WAIT,
            "waited without a reload pending"
        );

        RELOAD_SENT.fetch_add(1, Ordering::SeqCst);
        let started = Instant::now();
        let _ = status();
        let waited = started.elapsed();
        // Nothing is going to complete this reload, so the wait has to end on
        // its own instead of blocking forever.
        RELOAD_DONE.store(RELOAD_SENT.load(Ordering::SeqCst), Ordering::SeqCst);

        assert!(waited >= RELOAD_WAIT, "gave up after {waited:?}");
        assert!(waited < RELOAD_WAIT * 8, "waited far too long: {waited:?}");
    }
}
