//! Reading text that is not UTF-8, using the code page Windows was installed
//! with.

use windows::Win32::Globalization::{
    MultiByteToWideChar, CP_ACP, MB_PRECOMPOSED, MULTI_BYTE_TO_WIDE_CHAR_FLAGS,
};

/// Decodes bytes in the code page this copy of Windows uses for programs that
/// are not Unicode aware — 936 on a Chinese install, 1252 on a Western one.
///
/// A `.txt` or an `.srt` downloaded before UTF-8 became the default is usually
/// in that code page, and reading it as UTF-8 would show replacement marks
/// instead of the text. A sequence the code page cannot spell comes back with
/// the replacement character rather than an error: a file with a few broken
/// bytes is still worth translating.
pub fn from_system_code_page(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return Some(String::new());
    }
    let flags = MULTI_BYTE_TO_WIDE_CHAR_FLAGS(MB_PRECOMPOSED.0);
    let needed = unsafe { MultiByteToWideChar(CP_ACP, flags, bytes, None) };
    if needed <= 0 {
        return None;
    }
    let mut wide = vec![0u16; needed as usize];
    let written = unsafe { MultiByteToWideChar(CP_ACP, flags, bytes, Some(&mut wide)) };
    if written <= 0 {
        return None;
    }
    wide.truncate(written as usize);
    Some(String::from_utf16_lossy(&wide))
}
