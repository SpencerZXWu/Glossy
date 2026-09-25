//! Counters for the three things that would leak over a day of use: the mouse
//! hook, the clipboard and the speech voice.
//!
//! A day long run cannot be put in a test, but its shape can: every acquisition
//! is paired with a release here, so a test reads the balance and says which of
//! the three is out. Incrementing an atomic at each site is cheap enough to keep
//! the counters always on rather than behind an environment variable, and a
//! count that only ever grows is what makes a leak visible in the first place.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Installations of the low level mouse hook, and their removals.
static HOOK_INSTALLS: AtomicU64 = AtomicU64::new(0);
static HOOK_UNINSTALLS: AtomicU64 = AtomicU64::new(0);
/// Every left button event the hook saw, so a hook that has gone deaf is
/// visible as a number that stopped moving.
static HOOK_EVENTS: AtomicU64 = AtomicU64::new(0);

/// Opens of the clipboard, and their closes: the clipboard is one global lock,
/// and an open that is never closed keeps every other application out of it.
static CLIPBOARD_OPENS: AtomicU64 = AtomicU64::new(0);
static CLIPBOARD_CLOSES: AtomicU64 = AtomicU64::new(0);

/// The voice is one COM object per worker thread, created once and shut down
/// with that thread.
static VOICES_CREATED: AtomicU64 = AtomicU64::new(0);
static VOICES_RELEASED: AtomicU64 = AtomicU64::new(0);

fn bump(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

/// Records that the hook was installed.
pub fn hook_installed() {
    bump(&HOOK_INSTALLS);
}

/// Records that the hook was taken down again.
pub fn hook_uninstalled() {
    bump(&HOOK_UNINSTALLS);
}

/// Records one left button event seen by the hook.
pub fn hook_event() {
    bump(&HOOK_EVENTS);
}

/// Records a clipboard that was opened.
pub fn clipboard_opened() {
    bump(&CLIPBOARD_OPENS);
}

/// Records a clipboard that was closed.
pub fn clipboard_closed() {
    bump(&CLIPBOARD_CLOSES);
}

/// Records a voice that was created.
pub fn voice_created() {
    bump(&VOICES_CREATED);
}

/// Records a voice that was shut down.
pub fn voice_released() {
    bump(&VOICES_RELEASED);
}

/// What the counters say at one moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    pub hook_installs: u64,
    pub hook_uninstalls: u64,
    pub hook_events: u64,
    pub clipboard_opens: u64,
    pub clipboard_closes: u64,
    pub voices_created: u64,
    pub voices_released: u64,
}

impl Snapshot {
    /// Hooks still installed. One is the running application; a second one is a
    /// hook nobody took down.
    pub fn hooks_live(&self) -> u64 {
        self.hook_installs.saturating_sub(self.hook_uninstalls)
    }

    /// Opens of the clipboard without a close, which is the clipboard still
    /// being held.
    pub fn clipboard_outstanding(&self) -> u64 {
        self.clipboard_opens.saturating_sub(self.clipboard_closes)
    }

    /// Voices created and not shut down.
    pub fn voices_live(&self) -> u64 {
        self.voices_created.saturating_sub(self.voices_released)
    }

    /// The first of the three that is out of balance, named so a message can
    /// say which one it is.
    pub fn leak(&self) -> Option<&'static str> {
        if self.clipboard_outstanding() > 0 {
            Some("the clipboard")
        } else if self.hooks_live() > 1 {
            Some("the mouse hook")
        } else if self.voices_live() > 1 {
            Some("the voice")
        } else {
            None
        }
    }
}

/// Reads the counters.
pub fn snapshot() -> Snapshot {
    Snapshot {
        hook_installs: HOOK_INSTALLS.load(Ordering::Relaxed),
        hook_uninstalls: HOOK_UNINSTALLS.load(Ordering::Relaxed),
        hook_events: HOOK_EVENTS.load(Ordering::Relaxed),
        clipboard_opens: CLIPBOARD_OPENS.load(Ordering::Relaxed),
        clipboard_closes: CLIPBOARD_CLOSES.load(Ordering::Relaxed),
        voices_created: VOICES_CREATED.load(Ordering::Relaxed),
        voices_released: VOICES_RELEASED.load(Ordering::Relaxed),
    }
}

/// Says out loud which of the three was not given back, once.
///
/// Called after a selection has been read, which is where a held clipboard or a
/// second hook would show up; the message is written once because a leak that
/// stays leaks on every selection.
pub fn report_leak() -> Option<&'static str> {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    let named = snapshot().leak()?;
    if !REPORTED.swap(true, Ordering::Relaxed) {
        eprintln!("glossy: {named} was not given back");
    }
    Some(named)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_of(opens: u64, closes: u64, installs: u64, uninstalls: u64) -> Snapshot {
        Snapshot {
            hook_installs: installs,
            hook_uninstalls: uninstalls,
            hook_events: 0,
            clipboard_opens: opens,
            clipboard_closes: closes,
            voices_created: 1,
            voices_released: 1,
        }
    }

    #[test]
    fn a_paired_run_leaves_nothing_behind() {
        let run = snapshot_of(40, 40, 1, 1);
        assert_eq!(run.hooks_live(), 0);
        assert_eq!(run.clipboard_outstanding(), 0);
        assert_eq!(run.voices_live(), 0);
        assert_eq!(run.leak(), None);
    }

    #[test]
    fn a_held_clipboard_is_named_as_the_leak() {
        let run = snapshot_of(40, 39, 1, 1);
        assert_eq!(run.clipboard_outstanding(), 1);
        assert_eq!(run.leak(), Some("the clipboard"));
    }

    #[test]
    fn a_second_hook_is_named_as_the_leak() {
        assert_eq!(snapshot_of(1, 1, 1, 1).leak(), None);
        assert_eq!(snapshot_of(1, 1, 2, 1).hooks_live(), 1);
        assert_eq!(snapshot_of(1, 1, 3, 1).leak(), Some("the mouse hook"));
        assert_eq!(snapshot_of(1, 1, 3, 1).hooks_live(), 2);
    }

    #[test]
    fn a_release_that_never_happened_is_not_a_negative_count() {
        // The counters are read as unsigned differences: a stray close must not
        // wrap around into a huge number that hides a real leak.
        assert_eq!(snapshot_of(1, 2, 1, 2).clipboard_outstanding(), 0);
        assert_eq!(snapshot_of(1, 2, 1, 2).hooks_live(), 0);
    }

    #[test]
    fn the_counters_only_ever_grow() {
        let before = snapshot();
        hook_installed();
        hook_uninstalled();
        hook_event();
        clipboard_opened();
        clipboard_closed();
        voice_created();
        voice_released();
        let after = snapshot();
        assert!(after.hook_installs > before.hook_installs);
        assert!(after.hook_uninstalls > before.hook_uninstalls);
        assert!(after.hook_events > before.hook_events);
        assert!(after.clipboard_opens > before.clipboard_opens);
        assert!(after.clipboard_closes > before.clipboard_closes);
        assert!(after.voices_created > before.voices_created);
        assert!(after.voices_released > before.voices_released);
        assert_eq!(after.clipboard_outstanding(), 0);
    }
}
