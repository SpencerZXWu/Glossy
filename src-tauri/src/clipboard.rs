//! Clipboard access plus the "press Ctrl+C and watch the clipboard" trick that
//! reads the current text selection out of the foreground application.

use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

/// `CF_UNICODETEXT`; declared locally so no extra Win32 feature is required.
const CF_UNICODETEXT: u32 = 13;

const MAX_TEXT_BYTES: usize = 1 << 21;

fn open_retry() -> bool {
    // The clipboard is often briefly locked by the application that just wrote to it.
    for _ in 0..12 {
        if unsafe { OpenClipboard(None) }.is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

/// Reads the current unicode text content of the clipboard.
pub fn read_text() -> Option<String> {
    if !open_retry() {
        return None;
    }
    let text = read_text_locked();
    unsafe {
        let _ = CloseClipboard();
    }
    text
}

fn read_text_locked() -> Option<String> {
    unsafe {
        let handle = GetClipboardData(CF_UNICODETEXT).ok()?;
        if handle.0.is_null() {
            return None;
        }
        let block = HGLOBAL(handle.0);
        let ptr = GlobalLock(block);
        if ptr.is_null() {
            return None;
        }
        let mut length = 0usize;
        let words = ptr as *const u16;
        while length * 2 < MAX_TEXT_BYTES && *words.add(length) != 0 {
            length += 1;
        }
        let slice = std::slice::from_raw_parts(words, length);
        let text = String::from_utf16_lossy(slice);
        let _ = GlobalUnlock(block);
        Some(text)
    }
}

/// Replaces the clipboard content with `text`.
pub fn write_text(text: &str) -> bool {
    let mut utf16: Vec<u16> = text.encode_utf16().collect();
    utf16.push(0);
    let bytes = utf16.len() * 2;

    unsafe {
        let Ok(block) = GlobalAlloc(GMEM_MOVEABLE, bytes) else {
            return false;
        };
        if block.0.is_null() {
            return false;
        }
        let memory = GlobalLock(block);
        if memory.is_null() {
            return false;
        }
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), memory as *mut u16, utf16.len());
        let _ = GlobalUnlock(block);

        if !open_retry() {
            return false;
        }
        let _ = EmptyClipboard();
        // Ownership of the block transfers to the clipboard on success.
        let stored = SetClipboardData(CF_UNICODETEXT, Some(HANDLE(block.0))).is_ok();
        let _ = CloseClipboard();
        stored
    }
}

fn sequence_number() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

/// Result of attempting to lift the current selection from another application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capture {
    Text(String),
    /// Nothing was selected in the foreground application.
    NoSelection,
    /// The clipboard never changed, so the copy shortcut was ignored.
    NoResponse,
}

/// Presses Ctrl+C and waits for the clipboard to change.
pub fn capture_selection(restore: bool) -> Capture {
    let previous = read_text();
    let before = sequence_number();

    crate::input::send_copy();

    let deadline = Instant::now() + Duration::from_millis(700);
    let mut changed = false;
    while Instant::now() < deadline {
        if sequence_number() != before {
            changed = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(8));
    }

    if !changed {
        return Capture::NoResponse;
    }

    // The foreground application may still be filling the clipboard.
    std::thread::sleep(Duration::from_millis(20));
    let captured = read_text();

    let result = match captured {
        Some(text) if !text.trim().is_empty() => Capture::Text(text),
        _ => Capture::NoSelection,
    };

    if restore {
        if let (Some(original), Capture::Text(ref taken)) = (previous.as_ref(), &result) {
            if original != taken {
                write_text(original);
            }
        }
    }

    result
}

/// Puts `text` on the clipboard, used by the popup copy button.
pub fn copy_to_clipboard(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    write_text(text)
}
