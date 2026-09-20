//! Clipboard access plus the "press Ctrl+C and watch the clipboard" trick that
//! reads the current text selection out of the foreground application.

use std::time::{Duration, Instant};

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardSequenceNumber, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};

/// `CF_UNICODETEXT`; declared locally so no extra Win32 feature is required.
const CF_UNICODETEXT: u32 = 13;

/// Formats that publish a GDI handle instead of a block of memory. Their content
/// cannot be read with the global memory functions, and locking one of those
/// handles as if it were memory corrupts the heap. The picture itself is also on
/// the clipboard as `CF_DIB`, which is captured normally.
const CF_BITMAP: u32 = 2;
const CF_PALETTE: u32 = 9;
const CF_ENHMETAFILE: u32 = 14;

fn is_handle_format(format: u32) -> bool {
    matches!(format, CF_BITMAP | CF_PALETTE | CF_ENHMETAFILE)
}

const MAX_TEXT_BYTES: usize = 1 << 21;

/// `CF_HDROP`, the list of files a copy in the file explorer publishes.
const CF_HDROP: u32 = 15;

/// How often the clipboard is looked at again after a restore, and how long to
/// wait in between. Short enough that a copy the user makes right afterwards
/// survives.
const RESTORE_SETTLE_TRIES: usize = 4;
const RESTORE_SETTLE_INTERVAL: Duration = Duration::from_millis(20);

/// Upper bound for a single captured format; larger payloads are skipped rather
/// than copied into memory (a 4K screenshot is well below this).
const MAX_FORMAT_BYTES: usize = 1 << 26;

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
            let _ = GlobalFree(Some(block));
            return false;
        }
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), memory as *mut u16, utf16.len());
        let _ = GlobalUnlock(block);

        if !open_retry() {
            let _ = GlobalFree(Some(block));
            return false;
        }
        let _ = EmptyClipboard();
        // Ownership of the block transfers to the clipboard on success.
        let stored = SetClipboardData(CF_UNICODETEXT, Some(HANDLE(block.0))).is_ok();
        let _ = CloseClipboard();
        if !stored {
            let _ = GlobalFree(Some(block));
        }
        stored
    }
}

fn sequence_number() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

/// Reads the text the copy shortcut just put on the clipboard.
///
/// A copy in the file explorer publishes the selected files as `CF_HDROP`
/// alongside the path it also offers as text; that path is not a selection the
/// user wants translated.
fn read_copied_text() -> Option<String> {
    if !open_retry() {
        return None;
    }
    let text = if unsafe { GetClipboardData(CF_HDROP) }.is_ok() {
        None
    } else {
        read_text_locked()
    };
    unsafe {
        let _ = CloseClipboard();
    }
    text
}

/// A verbatim copy of everything the clipboard holds.
///
/// The Ctrl+C used to read a selection replaces the whole clipboard, so keeping
/// only its text would make images and file lists disappear for good.
struct Snapshot {
    entries: Vec<(u32, Vec<u8>)>,
}

impl Snapshot {
    /// Captures every published format, or `None` when the clipboard is empty.
    fn of_clipboard() -> Option<Snapshot> {
        if !open_retry() {
            return None;
        }
        let entries = unsafe { collect_formats() };
        unsafe {
            let _ = CloseClipboard();
        }
        if entries.is_empty() {
            None
        } else {
            Some(Snapshot { entries })
        }
    }

    /// Writes the captured formats back over the text we copied.
    fn restore(&self) {
        if self.entries.is_empty() || !open_retry() {
            return;
        }
        unsafe {
            let _ = EmptyClipboard();
            for (format, bytes) in &self.entries {
                put_format(*format, bytes);
            }
            let _ = CloseClipboard();
        }
    }

    /// Restores, then repairs the clipboard if it changes again right away.
    ///
    /// Some applications fill the clipboard from a worker thread, so their write
    /// can land just after ours and undo the restore. A copy the user makes
    /// themselves also bumps the sequence number, which is why this only watches
    /// the clipboard for a moment instead of keeping an eye on it.
    fn restore_settled(&self) {
        self.restore();
        let mut ours = sequence_number();
        for _ in 0..RESTORE_SETTLE_TRIES {
            std::thread::sleep(RESTORE_SETTLE_INTERVAL);
            let current = sequence_number();
            if current == ours {
                return;
            }
            self.restore();
            ours = sequence_number();
        }
    }
}

unsafe fn collect_formats() -> Vec<(u32, Vec<u8>)> {
    let mut entries = Vec::new();
    let mut format = EnumClipboardFormats(0);
    while format != 0 {
        // `EnumClipboardFormats` only reports formats that really exist, so no
        // synthesized duplicates (CF_TEXT and friends) end up in the list.
        if !is_handle_format(format) {
            if let Some(bytes) = format_bytes(format) {
                entries.push((format, bytes));
            }
        }
        format = EnumClipboardFormats(format);
    }
    entries
}

/// Copies one format out of the clipboard. Formats that are rendered on demand
/// (no handle yet) or cannot be locked are skipped instead of aborting the run.
unsafe fn format_bytes(format: u32) -> Option<Vec<u8>> {
    let handle = GetClipboardData(format).ok()?;
    if handle.0.is_null() {
        return None;
    }
    let block = HGLOBAL(handle.0);
    let published = GlobalSize(block);
    if published == 0 || published > MAX_FORMAT_BYTES {
        return None;
    }
    let ptr = GlobalLock(block);
    if ptr.is_null() {
        return None;
    }
    let bytes = std::slice::from_raw_parts(ptr as *const u8, published).to_vec();
    let _ = GlobalUnlock(block);
    Some(bytes)
}

/// Puts one captured format back; ownership of the block moves to the clipboard
/// on success and is released again on failure.
unsafe fn put_format(format: u32, bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let Ok(block) = GlobalAlloc(GMEM_MOVEABLE, bytes.len()) else {
        return false;
    };
    if block.0.is_null() {
        return false;
    }
    let memory = GlobalLock(block);
    if memory.is_null() {
        let _ = GlobalFree(Some(block));
        return false;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), memory as *mut u8, bytes.len());
    let _ = GlobalUnlock(block);
    if SetClipboardData(format, Some(HANDLE(block.0))).is_ok() {
        return true;
    }
    let _ = GlobalFree(Some(block));
    false
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
///
/// With `restore` set, the clipboard is captured beforehand and written back once
/// the selection has been read out of it.
pub fn capture_selection(restore: bool) -> Capture {
    let snapshot = if restore {
        Snapshot::of_clipboard()
    } else {
        None
    };
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
    let captured = read_copied_text();

    let result = match captured {
        Some(text) if !text.trim().is_empty() => Capture::Text(text),
        _ => Capture::NoSelection,
    };

    // An empty clipboard is left as it is, so the copied selection stays available.
    if let Some(snapshot) = snapshot.as_ref() {
        snapshot.restore_settled();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_the_formats_that_are_gdi_handles() {
        assert!(is_handle_format(CF_BITMAP));
        assert!(is_handle_format(CF_PALETTE));
        assert!(is_handle_format(CF_ENHMETAFILE));
        // The memory based formats are the ones worth capturing.
        assert!(!is_handle_format(3));
        assert!(!is_handle_format(8));
        assert!(!is_handle_format(CF_UNICODETEXT));
        assert!(!is_handle_format(15));
        assert!(!is_handle_format(17));
    }
}
