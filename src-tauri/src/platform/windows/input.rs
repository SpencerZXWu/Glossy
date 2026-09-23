//! Synthetic keyboard input used to copy the current selection.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_C,
    VK_CONTROL, VK_MENU, VK_SHIFT,
};

fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_KEYBOARD,
        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn key_is_down(vk: VIRTUAL_KEY) -> bool {
    unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000
            != 0
    }
}

/// Sends Ctrl+C to the foreground window so it copies its current selection.
///
/// Held modifier keys are released first, otherwise the combination would be
/// interpreted as Ctrl+Shift+C style shortcuts by the receiving application.
pub fn send_copy() {
    let mut inputs: Vec<INPUT> = Vec::with_capacity(10);

    for vk in [VK_SHIFT, VK_MENU] {
        if key_is_down(vk) {
            inputs.push(key(vk, true));
        }
    }

    inputs.push(key(VK_CONTROL, false));
    inputs.push(key(VK_C, false));
    inputs.push(key(VK_C, true));
    inputs.push(key(VK_CONTROL, true));

    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
