//! Console handling for a GUI executable.
//!
//! The binary is linked as a Windows GUI application so that starting it never
//! opens a console window. Diagnostics written with `eprintln!` would then be
//! lost, so the parent console — the terminal Glossy was started from, if any —
//! is borrowed instead.

use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

/// Reconnects stdout and stderr to the console of the calling terminal.
///
/// Does nothing when Glossy was started from Explorer or the notification area,
/// because there is no console to attach to.
pub fn attach_parent() {
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
    // Rust resolves the standard handles per write, so nothing else is needed
    // here for `println!` and `eprintln!` to reach the borrowed console.
}
