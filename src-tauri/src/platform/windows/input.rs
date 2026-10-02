//! Synthetic keyboard input used to copy the current selection.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_C,
    VK_CONTROL, VK_ESCAPE, VK_MENU, VK_RBUTTON, VK_SHIFT, VK_V,
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

/// True while the user is holding Escape down.
///
/// Asked by the screenshot overlay, which covers a whole monitor and takes the
/// clicks with it: if the page inside it never came up, the key presses go
/// nowhere, and something outside the page has to notice the way out.
pub fn escape_is_down() -> bool {
    key_is_down(VK_ESCAPE)
}

/// True while the user is holding the right mouse button down, the other way
/// out of the screenshot overlay.
pub fn right_button_is_down() -> bool {
    key_is_down(VK_RBUTTON)
}

/// Sends Ctrl+C to the foreground window so it copies its current selection.
///
/// Held modifier keys are released first, otherwise the combination would be
/// interpreted as Ctrl+Shift+C style shortcuts by the receiving application.
pub fn send_copy() {
    send_chord(VK_C);
}

/// Sends Ctrl+V to the foreground window so it writes the clipboard over its
/// current selection.
///
/// The other half of [`send_copy`]: the selection was read with a copy, and it
/// is written back with a paste.
pub fn send_paste() {
    send_chord(VK_V);
}

fn send_chord(vk: VIRTUAL_KEY) {
    let mut inputs: Vec<INPUT> = Vec::with_capacity(10);

    for held in [VK_SHIFT, VK_MENU] {
        if key_is_down(held) {
            inputs.push(key(held, true));
        }
    }

    inputs.push(key(VK_CONTROL, false));
    inputs.push(key(vk, false));
    inputs.push(key(vk, true));
    inputs.push(key(VK_CONTROL, true));

    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
