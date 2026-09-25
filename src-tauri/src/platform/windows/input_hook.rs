//! The low level mouse hook and the message loop that delivers it.
//!
//! Windows calls a `WH_MOUSE_LL` hook on the thread that installed it and
//! silently drops hooks whose callback blocks, so the callback here does
//! nothing but decode the click and hand it to the handler the application
//! installed. The message loop belongs to this module because the hook cannot
//! fire without one, and the global accelerator rides the same loop — hotkeys
//! are posted to the thread that registered them.

use std::cell::RefCell;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, MSG, MSLLHOOKSTRUCT,
    WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP,
};

use super::hotkey;
use crate::vitals;

/// One left button event, in physical screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Click {
    /// True for the press, false for the release.
    pub pressed: bool,
    pub x: i32,
    pub y: i32,
}

struct Handlers {
    click: Box<dyn FnMut(Click)>,
    reload: Box<dyn FnMut()>,
    fire: Box<dyn FnMut()>,
}

thread_local! {
    static HANDLERS: RefCell<Option<Handlers>> = const { RefCell::new(None) };
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let message = wparam.0 as u32;
        if message == WM_LBUTTONDOWN || message == WM_LBUTTONUP {
            vitals::hook_event();
            let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
            let click = Click {
                pressed: message == WM_LBUTTONDOWN,
                x: info.pt.x,
                y: info.pt.y,
            };
            with_handlers(|handlers| (handlers.click)(click));
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Installs the hook and runs the message loop under it until the loop ends.
///
/// `on_click` sees every left button event, `on_reload` is called when the
/// settings that pick the accelerator have changed, and `on_fire` when the
/// accelerator was pressed. Blocks the calling thread, so the caller owns a
/// thread for it, and returns `Err` when the hook could not be installed — in
/// which case the accelerator is not registered either, since it is delivered
/// through this same loop.
pub fn run(
    on_click: impl FnMut(Click) + 'static,
    on_reload: impl FnMut() + 'static,
    on_fire: impl FnMut() + 'static,
) -> Result<(), String> {
    HANDLERS.with(|handlers| {
        *handlers.borrow_mut() = Some(Handlers {
            click: Box::new(on_click),
            reload: Box::new(on_reload),
            fire: Box::new(on_fire),
        });
    });

    unsafe {
        // The handle is kept alive for the lifetime of the message loop, and
        // given back when the loop ends: a hook left installed would keep this
        // module on the stack of every click of the desktop.
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), None, 0)
            .map_err(|error| format!("could not install the mouse hook: {error}"))?;
        vitals::hook_installed();

        hotkey::register_loop_thread();
        with_handlers(|handlers| (handlers.reload)());

        let mut message = MSG::default();
        // A message loop is required for low level hooks to be delivered.
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            if message.message == hotkey::WM_RELOAD {
                with_handlers(|handlers| (handlers.reload)());
            } else if hotkey::is_hotkey_message(message.message, message.wParam.0) {
                // Never block: the hook has to stay responsive.
                with_handlers(|handlers| (handlers.fire)());
            }
        }

        let _ = UnhookWindowsHookEx(hook);
        vitals::hook_uninstalled();
    }

    Ok(())
}

fn with_handlers(action: impl FnOnce(&mut Handlers)) {
    HANDLERS.with(|handlers| {
        // The hook can be re-entered while a handler runs; the event is dropped
        // rather than waited for.
        if let Ok(mut handlers) = handlers.try_borrow_mut() {
            if let Some(handlers) = handlers.as_mut() {
                action(handlers);
            }
        }
    });
}
