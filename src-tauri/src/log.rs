//! The file the failures Glossy prints end up in.
//!
//! The binary is linked as a Windows GUI application, so there is usually no
//! console to read: everything written with `eprintln!` reached nobody when the
//! app was started from Explorer or from the notification area, which is how it
//! is normally started. [`note!`](crate::log::note) writes the same line to the
//! log and to stderr, so a terminal that is there still sees it and a machine
//! without one keeps a record.
//!
//! The log sits next to `settings.json` and is capped: a full one becomes
//! `glossy.log.1` and a new one is started, so a machine that runs for months
//! leaves two bounded files rather than one that grows without end.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// The name the log is written under, next to the settings file.
pub const FILE_NAME: &str = "glossy.log";

/// The name the previous log is kept under once a new one is started.
const PREVIOUS_NAME: &str = "glossy.log.1";

/// How large the log may grow before it gives way to a new one.
pub const CAP: u64 = 256 * 1024;

/// Where the log goes, once the app has said where that is. Until then — and in
/// a test, which never calls [`keep`] — a line only reaches the console.
static DESTINATION: OnceLock<PathBuf> = OnceLock::new();

/// Held while a line is written: two threads writing at once would interleave
/// their lines.
static WRITER: Mutex<()> = Mutex::new(());

/// Writes one line to the log and to stderr: `note!("glossy: {error}")`.
macro_rules! note {
    ($($arg:tt)*) => {
        $crate::log::record(&format!($($arg)*))
    };
}

pub(crate) use note;

/// Tells the log where its file is, and answers with the path.
///
/// Called once, early, so that as little as possible happens before there is
/// somewhere to write about it. A second call is ignored.
pub fn keep(dir: &Path) -> PathBuf {
    let path = dir.join(FILE_NAME);
    let _ = DESTINATION.set(path.clone());
    path
}

/// The file the log is written to, once [`keep`] has been called.
pub fn path() -> Option<&'static Path> {
    DESTINATION.get().map(PathBuf::as_path)
}

/// What the log takes up right now.
pub fn size() -> u64 {
    path().map(size_of).unwrap_or(0)
}

/// Empties the log, and the file the last one was rotated into.
///
/// Nothing else is touched: the failures already written are gone, which is what
/// a user who is about to hand the file to somebody wants, and the settings are
/// where they were.
pub fn clear() -> Result<(), String> {
    let Some(path) = path() else {
        return Err("The log has nowhere to be written yet.".to_string());
    };
    empty(path)
}

/// Writes one line, stamped, to the log and to stderr.
///
/// Nothing here may fail loudly: the log is worth less than the message it was
/// about, and a panic in here would take down whatever was being reported on.
pub fn record(message: &str) {
    eprintln!("{message}");
    let Some(path) = path() else {
        return;
    };
    let line = format!("{} {message}\n", stamp(now()));
    let _guard = WRITER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    append(path, &line);
}

/// Makes a panic leave a line in the log before the process goes.
///
/// The default hook still runs, so a terminal sees what it has always seen; what
/// this adds is the record that outlives the window closing, which is the only
/// way a crash that happened while nobody was looking can still be read.
pub fn catch_panics() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        note!("glossy: it went down: {info}");
        previous(info);
    }));
}

/// Appends one line, starting a new file first when this one is full.
fn append(path: &Path, line: &str) {
    if rotate_needed(size_of(path)) {
        let previous = path.with_file_name(PREVIOUS_NAME);
        // The previous file is the one before this session's; keeping two is the
        // whole point, keeping three is not.
        let _ = fs::remove_file(&previous);
        if fs::rename(path, &previous).is_err() {
            let _ = fs::remove_file(path);
        }
    }
    let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = file.write_all(line.as_bytes());
}

/// Empties a log file, and the one it was last rotated into.
fn empty(path: &Path) -> Result<(), String> {
    let _ = fs::remove_file(path.with_file_name(PREVIOUS_NAME));
    fs::write(path, b"").map_err(|error| format!("The log could not be emptied: {error}"))
}

/// Whether a log of this size has to give way to a new one.
fn rotate_needed(bytes: u64) -> bool {
    bytes >= CAP
}

fn size_of(path: &Path) -> u64 {
    fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

/// Seconds since the epoch.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

/// `YYYY-MM-DDTHH:MM:SSZ`, in UTC.
///
/// UTC rather than the machine's own zone: a local stamp would need a timezone
/// table the app has no other use for, and a log read to find out what went
/// wrong is easier to place when the offset never moves.
fn stamp(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let rest = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

/// The calendar date `days` after 1970-01-01, as `(year, month, day)`.
///
/// Howard Hinnant's civil calendar algorithm: exact for every day the app could
/// ever see, and no table to get wrong.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    // Shift the epoch to 0000-03-01, so leap days fall at the end of the cycle.
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// What a caught panic says, for a line in the log.
///
/// A payload carries either the message that was formatted into it or a value
/// somebody chose to panic with; anything else is named for what it was.
pub fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    "a value that is not a message".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder of this test's own, emptied first.
    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("glossy-log-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("a scratch folder");
        dir
    }

    #[test]
    fn a_stamp_is_utc_to_the_second() {
        assert_eq!(stamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(stamp(1), "1970-01-01T00:00:01Z");
        assert_eq!(stamp(1_700_000_000), "2023-11-14T22:13:20Z");
        // A leap day, which is where a date built from arithmetic usually slips.
        assert_eq!(stamp(1_709_164_800), "2024-02-29T00:00:00Z");
        assert_eq!(stamp(1_780_000_000), "2026-05-28T20:26:40Z");
    }

    #[test]
    fn only_a_full_log_gives_way() {
        assert!(!rotate_needed(0));
        assert!(!rotate_needed(CAP - 1));
        assert!(rotate_needed(CAP));
        assert!(rotate_needed(CAP + 10_000));
    }

    #[test]
    fn a_line_reaches_the_file_with_its_time() {
        let dir = scratch("line");
        let path = dir.join(FILE_NAME);

        append(
            &path,
            &format!("{} glossy: something went wrong\n", stamp(0)),
        );

        let written = fs::read_to_string(&path).expect("the log");
        assert_eq!(
            written,
            "1970-01-01T00:00:00Z glossy: something went wrong\n"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_full_log_becomes_the_previous_one() {
        let dir = scratch("rotate");
        let path = dir.join(FILE_NAME);
        let filler = "x".repeat(CAP as usize);
        fs::write(&path, &filler).expect("a full log");

        append(&path, "1970-01-01T00:00:00Z glossy: after the rotation\n");

        assert_eq!(
            fs::read_to_string(dir.join(PREVIOUS_NAME)).expect("the previous log"),
            filler
        );
        assert_eq!(
            fs::read_to_string(&path).expect("the new log"),
            "1970-01-01T00:00:00Z glossy: after the rotation\n"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn emptying_the_log_takes_the_previous_one_with_it() {
        let dir = scratch("clear");
        let path = dir.join(FILE_NAME);
        fs::write(&path, "an old failure\n").expect("a log");
        fs::write(dir.join(PREVIOUS_NAME), "an older failure\n").expect("a previous log");

        empty(&path).expect("the log to be emptied");

        assert_eq!(fs::read_to_string(&path).expect("the log"), "");
        assert!(!dir.join(PREVIOUS_NAME).exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_line_that_has_nowhere_to_go_is_simply_said_out_loud() {
        // `keep` is what gives the log its file, and only the app calls it: a
        // line written before that must not fail, and asking to empty a file
        // that does not exist yet must not invent a path to write to.
        note!("glossy: a line with no file behind it");
        assert!(path().is_none());
    }

    #[test]
    fn a_caught_panic_gives_up_what_it_said() {
        let text = std::panic::catch_unwind(|| panic!("the popup is gone"))
            .expect_err("the panic to be caught");
        assert_eq!(panic_message(&*text), "the popup is gone");

        // `panic_any` carries whatever value it was given, which is not always
        // something that can be read as a message.
        let value = std::panic::catch_unwind(|| std::panic::panic_any(7u8))
            .expect_err("the panic to be caught");
        assert_eq!(panic_message(&*value), "a value that is not a message");
    }
}
