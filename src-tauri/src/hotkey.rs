//! System wide accelerator that translates the clipboard.
//!
//! Windows delivers `WM_HOTKEY` to the thread that called `RegisterHotKey`, so
//! the accelerator is registered on the mouse hook message loop owned by
//! `selection` instead of on a thread of its own.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN, VIRTUAL_KEY, VK_BACK, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME,
    VK_INSERT, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_APP, WM_HOTKEY};

use crate::state::AppState;

/// Identifier of the single accelerator Glossy owns.
pub const ID: i32 = 0x6075;
/// Posted to the message loop thread when the accelerator setting changed.
pub const WM_RELOAD: u32 = WM_APP + 7;

static LOOP_THREAD: AtomicU32 = AtomicU32::new(0);
static ACTIVE: Mutex<Option<String>> = Mutex::new(None);
static PROBLEM: Mutex<Option<String>> = Mutex::new(None);

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

/// Registers `spec` on the calling thread and returns its canonical spelling.
///
/// An empty specification simply switches the accelerator off.
pub fn install_on_this_thread(spec: &str) -> Result<String, String> {
    unsafe {
        let _ = UnregisterHotKey(None, ID);
    }

    let spec = spec.trim();
    if spec.is_empty() {
        return Ok(String::new());
    }

    let accelerator = parse(spec)?;
    unsafe { RegisterHotKey(None, ID, accelerator.modifiers, u32::from(accelerator.key.0)) }
        .map_err(|error| {
            format!(
                "Windows refused {}. Another program probably owns it ({error}).",
                accelerator.label
            )
        })?;
    Ok(accelerator.label)
}

/// Re-reads the configured accelerator; the outcome is reported by [`status`].
pub fn reload(state: &AppState) {
    let outcome = install_on_this_thread(&state.settings().hotkey);
    let (active, problem) = match outcome {
        Ok(label) if label.is_empty() => (None, None),
        Ok(label) => (Some(label), None),
        Err(message) => (None, Some(message)),
    };
    *ACTIVE.lock().unwrap() = active;
    *PROBLEM.lock().unwrap() = problem;
}

/// Registered accelerator and, when it is not active, the reason why.
pub fn status() -> (Option<String>, Option<String>) {
    (
        ACTIVE.lock().unwrap().clone(),
        PROBLEM.lock().unwrap().clone(),
    )
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
    unsafe {
        let _ = PostThreadMessageW(thread, WM_RELOAD, WPARAM(0), LPARAM(0));
    }
}

/// True when the message belongs to the accelerator registered by Glossy.
pub fn is_hotkey_message(message: u32, id: usize) -> bool {
    message == WM_HOTKEY && id == ID as usize
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
        assert_eq!(install_on_this_thread("").unwrap(), "");
        assert_eq!(install_on_this_thread("   ").unwrap(), "");
    }

    #[test]
    fn recognises_its_own_hotkey_message() {
        assert!(is_hotkey_message(WM_HOTKEY, ID as usize));
        assert!(!is_hotkey_message(WM_HOTKEY, 0));
        assert!(!is_hotkey_message(WM_RELOAD, ID as usize));
    }
}
