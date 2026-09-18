//! Single instance guard. Two Glossy processes would install two mouse hooks
//! and both would try to drive the same popup window, so a second launch only
//! reports that Glossy is already in the notification area and exits.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MB_TOPMOST,
};

use crate::settings::{self, UiLanguage};

/// `Local\` scopes the name to the logon session, which is the same thing the
/// notification area icon is scoped to.
const MUTEX_NAME: &str = r"Local\Glossy.SingleInstance";

/// Keeps the name owned for as long as this process runs. The handle is
/// deliberately never closed: releasing the last handle would free the name
/// while Glossy is still alive.
pub struct Instance {
    _handle: isize,
}

/// What the guard found out about other Glossy processes.
pub enum Claim {
    /// No other Glossy is running.
    First(Instance),
    /// Another Glossy already owns the name.
    Taken,
    /// The guard could not be created at all.
    Unavailable,
}

pub fn claim() -> Claim {
    claim_named(MUTEX_NAME)
}

fn claim_named(name: &str) -> Claim {
    let name = wide(name);
    match unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) } {
        // Windows reports "that name already exists" through the last error
        // rather than through the returned handle, which stays valid either way.
        Ok(_) if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS => Claim::Taken,
        Ok(handle) => Claim::First(Instance {
            _handle: handle.0 as isize,
        }),
        Err(_) => Claim::Unavailable,
    }
}

/// Tells the user of a second launch where the running Glossy went.
pub fn report_already_running() {
    let (title, body) = match settings::resolve_ui_language(settings::stored_ui_language()) {
        UiLanguage::Chinese => (
            "Glossy",
            "Glossy 已在后台运行。\n\n点击任务栏通知区域中的 Glossy 图标即可打开主窗口。",
        ),
        _ => (
            "Glossy",
            "Glossy is already running in the background.\n\nClick the Glossy icon in the \
             notification area to open the main window.",
        ),
    };
    let title = wide(title);
    let body = wide(body);
    let style = MB_OK | MB_ICONINFORMATION | MB_SETFOREGROUND | MB_TOPMOST;
    unsafe {
        let _ = MessageBoxW(None, PCWSTR(body.as_ptr()), PCWSTR(title.as_ptr()), style);
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name of its own per test, so the tests never fight over the real one
    /// (which a running Glossy may already own).
    fn test_name(tag: &str) -> String {
        format!(r"Local\Glossy.Test.{tag}")
    }

    #[test]
    fn the_name_is_nul_terminated_utf16() {
        assert_eq!(wide("ab"), vec![0x61, 0x62, 0x00]);
        assert_eq!(wide(""), vec![0x00]);
    }

    #[test]
    fn a_second_claim_of_the_same_name_is_refused() {
        let name = test_name("second-claim");
        let first = claim_named(&name);
        assert!(
            matches!(first, Claim::First(_)),
            "the name should have been free in this process"
        );
        // The guard is still alive, so the name is taken from here on.
        assert!(matches!(claim_named(&name), Claim::Taken));
    }

    #[test]
    fn a_different_name_is_still_free() {
        assert!(matches!(claim_named(&test_name("one")), Claim::First(_)));
        assert!(matches!(claim_named(&test_name("two")), Claim::First(_)));
    }
}
