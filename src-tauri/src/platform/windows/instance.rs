//! Single instance guard. Two Glossy processes would install two mouse hooks
//! and both would try to drive the same popup window, so a second launch exits
//! instead.
//!
//! Exiting silently would look like nothing happened, so a second launch asks
//! the running Glossy to show what a start of its own would have shown, through
//! a named event the running Glossy waits on.

use std::ffi::c_void;
use std::thread;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject, EVENT_MODIFY_STATE,
    INFINITE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONINFORMATION, MB_OK, MB_SETFOREGROUND, MB_TOPMOST,
};

use crate::settings::{self, UiLanguage};

/// `Local\` scopes the name to the logon session, which is the same thing the
/// notification area icon is scoped to.
const MUTEX_NAME: &str = r"Local\Glossy.SingleInstance";

/// Owned by the running Glossy, who waits on it; every later launch sets it.
const EVENT_NAME: &str = r"Local\Glossy.Launch";

/// Keeps the names owned for as long as this process runs. The handles are
/// deliberately never closed: releasing the last handle would free the names
/// while Glossy is still alive.
pub struct Instance {
    _handle: isize,
    /// `None` when the event could not be created, which only costs the launch
    /// announcement.
    launch: Option<isize>,
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
    claim_named(MUTEX_NAME, EVENT_NAME)
}

fn claim_named(name: &str, event_name: &str) -> Claim {
    let name = wide(name);
    match unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) } {
        // Windows reports "that name already exists" through the last error
        // rather than through the returned handle, which stays valid either way.
        Ok(_) if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS => Claim::Taken,
        Ok(handle) => Claim::First(Instance {
            _handle: handle.0 as isize,
            launch: create_event(event_name),
        }),
        Err(_) => Claim::Unavailable,
    }
}

impl Instance {
    /// Calls `show` once for every later launch that asks for it.
    ///
    /// The event is auto-resetting, so each signal wakes exactly one wait — and
    /// a signal that arrives while nothing waits is kept until the next wait,
    /// which is what makes a launch arriving during startup still show up.
    pub fn watch(&self, show: impl Fn() + Send + 'static) {
        let Some(handle) = self.launch else {
            return;
        };
        thread::spawn(move || {
            let handle = HANDLE(handle as *mut c_void);
            loop {
                unsafe { WaitForSingleObject(handle, INFINITE) };
                show();
            }
        });
    }
}

/// Asks the Glossy that owns the name to show what a start of its own shows.
///
/// `false` means no one answered, which is the one case where this launch has to
/// tell the user something itself.
pub fn announce_launch() -> bool {
    announce_launch_named(EVENT_NAME)
}

fn announce_launch_named(event_name: &str) -> bool {
    let event_name = wide(event_name);
    let handle = match unsafe { OpenEventW(EVENT_MODIFY_STATE, false, PCWSTR(event_name.as_ptr())) }
    {
        Ok(handle) => handle,
        Err(_) => return false,
    };
    let announced = unsafe { SetEvent(handle) }.is_ok();
    let _ = unsafe { CloseHandle(handle) };
    announced
}

/// Creates the event a later launch signals, and the running Glossy waits on.
fn create_event(event_name: &str) -> Option<isize> {
    let event_name = wide(event_name);
    match unsafe { CreateEventW(None, false, false, PCWSTR(event_name.as_ptr())) } {
        Ok(handle) => Some(handle.0 as isize),
        Err(_) => None,
    }
}

/// Tells the user of a second launch where the running Glossy went.
///
/// Only used when the announcement went unanswered, so the running Glossy is not
/// there to show anything.
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

    /// The event counterpart of [`test_name`].
    fn test_event(tag: &str) -> String {
        format!(r"Local\Glossy.Test.Event.{tag}")
    }

    #[test]
    fn the_name_is_nul_terminated_utf16() {
        assert_eq!(wide("ab"), vec![0x61, 0x62, 0x00]);
        assert_eq!(wide(""), vec![0x00]);
    }

    #[test]
    fn a_second_claim_of_the_same_name_is_refused() {
        let name = test_name("second-claim");
        let first = claim_named(&name, &test_event("second-claim"));
        assert!(
            matches!(first, Claim::First(_)),
            "the name should have been free in this process"
        );
        // The guard is still alive, so the name is taken from here on.
        assert!(matches!(
            claim_named(&name, &test_event("second-claim")),
            Claim::Taken
        ));
    }

    #[test]
    fn a_different_name_is_still_free() {
        let name = test_name("one");
        let event = test_event("one");
        assert!(matches!(claim_named(&name, &event), Claim::First(_)));
        let name = test_name("two");
        let event = test_event("two");
        assert!(matches!(claim_named(&name, &event), Claim::First(_)));
    }

    #[test]
    fn a_later_launch_reaches_the_running_one() {
        let event = test_event("wake");
        let guard = match claim_named(&test_name("wake"), &event) {
            Claim::First(guard) => guard,
            _ => panic!("the name should have been free in this process"),
        };
        let (sender, receiver) = std::sync::mpsc::channel();
        guard.watch(move || {
            let _ = sender.send(());
        });

        assert!(announce_launch_named(&event), "the event should exist");

        assert!(
            receiver
                .recv_timeout(std::time::Duration::from_secs(5))
                .is_ok(),
            "the running instance should have been asked to show itself"
        );
    }

    #[test]
    fn a_launch_nobody_listens_for_goes_unanswered() {
        assert!(
            !announce_launch_named(&test_event("nobody")),
            "no instance owns this event"
        );
    }

    #[test]
    fn a_signal_waiting_for_a_listener_is_not_lost() {
        let event = test_event("early");
        let guard = match claim_named(&test_name("early"), &event) {
            Claim::First(guard) => guard,
            _ => panic!("the name should have been free in this process"),
        };

        // Signalled before anything waits, as a second launch during startup
        // would be.
        assert!(announce_launch_named(&event));

        let (sender, receiver) = std::sync::mpsc::channel();
        guard.watch(move || {
            let _ = sender.send(());
        });
        assert!(
            receiver
                .recv_timeout(std::time::Duration::from_secs(5))
                .is_ok(),
            "the pending signal should have woken the listener"
        );
    }
}
