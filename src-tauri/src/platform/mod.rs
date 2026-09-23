//! Everything Glossy asks the operating system for, behind one door.
//!
//! The application code never names a platform API: it calls
//! `crate::platform::…`, and this module routes the call to the implementation
//! for the OS the build targets. Windows is the only implementation today, so
//! the value of the split is not portability yet — it is that the surface which
//! has to be ported is written down in one place, grouped by what it does
//! rather than by the order the files happened to appear in:
//!
//! | Module | What it answers |
//! | --- | --- |
//! | `clipboard` | read and write the system clipboard, capture a selection |
//! | `console` | attach to the terminal the process was started from |
//! | `desktop` | cursor, monitors, window handles and what they own, the foreground program, locale |
//! | `hotkey` | register the global accelerator and route its messages |
//! | `input` | type the keystrokes that copy a selection |
//! | `input_hook` | the low level mouse hook and the message loop it runs on |
//! | `instance` | one Glossy per login, and how a second start announces itself |
//! | `secrets` | protect a stored credential with the OS key store |
//! | `speech` | read text aloud |
//! | `uia` | the text around the current selection, from the accessibility API |
//!
//! Portable work that used to share a file with a platform call stays with the
//! behaviour that owns it (`context::sentence_in` narrows a paragraph to one
//! sentence, `selection` decides what a click meant), and the geometry below
//! moved here because it holds no platform type at all. Some helpers written
//! beside a Win32 call are still spelled in Win32 terms — `hotkey`'s
//! accelerator grammar, `speech`'s queue, `context`'s UIA reader — and lifting
//! those is the first thing a second implementation has to do, not the last.
//! ROADMAP.md records that debt instead of pretending the layer is finished.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::{
    clipboard, console, desktop, hotkey, input, input_hook, instance, secrets, speech, uia,
};

#[cfg(not(windows))]
compile_error!(
    "Glossy has no implementation for this platform yet. Add `src/platform/<os>/` holding the \
     modules documented in `src/platform/mod.rs` and route to it here; only Windows builds today."
);

/// Region of the desktop: `(left, top, right, bottom)` in physical pixels.
///
/// The popup and the start hint both measure themselves against this, so it
/// lives next to the dispatch rather than inside an implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ScreenRect {
    /// True when the point falls inside the rectangle, grown by `pad` pixels.
    pub fn contains_padded(self, x: i32, y: i32, pad: i32) -> bool {
        x >= self.left - pad && x < self.right + pad && y >= self.top - pad && y < self.bottom + pad
    }
}
