//! How long the road from a selection to the painted popup is.
//!
//! The performance budget promises a number, so the number is measured rather
//! than guessed: the mouse up that ends a selection stamps the time, and the
//! popup reports back once it has painted. Collecting is off unless
//! `GLOSSY_TIMING` is set, because the instrument is only wanted while someone
//! is measuring and a background tool should not carry a counter it never reads.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// A mark older than this belongs to a selection that is long over; a popup
/// painted much later says nothing about the click that asked for it.
const STALE_MS: u64 = 5_000;
/// How many samples the summary keeps. A measuring run is a few dozen drags.
const CAPACITY: usize = 512;
/// Where the summary is written as well, when `GLOSSY_TIMING_LOG` names a file.
const LOG_VAR: &str = "GLOSSY_TIMING_LOG";

/// Set when the command line or the environment asked for measurements.
static ON: OnceLock<bool> = OnceLock::new();
static STARTED: OnceLock<Instant> = OnceLock::new();
/// When the selection gesture ended, in microseconds since the app started.
/// Zero means nothing is waiting to be measured.
static MARK: AtomicU64 = AtomicU64::new(0);
static SAMPLES: Mutex<Vec<f64>> = Mutex::new(Vec::new());

fn enabled() -> bool {
    *ON.get_or_init(|| match std::env::var("GLOSSY_TIMING") {
        Ok(value) => value != "0",
        Err(_) => false,
    })
}

fn uptime_us() -> u64 {
    STARTED.get_or_init(Instant::now).elapsed().as_micros() as u64
}

/// Stamps the moment a selection gesture ended.
///
/// Called from the mouse hook, so it does nothing at all unless measuring was
/// asked for.
pub fn mark() {
    if !enabled() {
        return;
    }
    MARK.store(uptime_us().max(1), Ordering::Relaxed);
}

/// Records the time the popup took to paint after the last mark, if any.
pub fn painted() {
    if !enabled() {
        return;
    }
    // Taken rather than read: one mark belongs to one popup, and a second
    // report must not measure the same selection twice.
    let marked = MARK.swap(0, Ordering::Relaxed);
    if marked == 0 {
        return;
    }
    let spent_ms = uptime_us().saturating_sub(marked) as f64 / 1_000.0;
    if spent_ms > STALE_MS as f64 {
        return;
    }

    let mut samples = SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
    samples.push(spent_ms);
    if samples.len() > CAPACITY {
        samples.remove(0);
    }
    let summary = Summary::of(&samples);
    report(&format!(
        "glossy: latency {spent_ms:.1} ms — p50 {:.1}, p95 {:.1}, max {:.1}, n = {}",
        summary.p50, summary.p95, summary.max, summary.count
    ));
}

/// Writes one line to the console and, when a log file was named, to it as well.
///
/// A GUI build started from Explorer has no console to attach to, which is the
/// reason the file exists: `GLOSSY_TIMING_LOG` is how a measuring run started
/// from a shortcut is read afterwards.
fn report(line: &str) {
    eprintln!("{line}");
    let Ok(path) = std::env::var(LOG_VAR) else {
        return;
    };
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(file, "{line}");
    }
}

/// The numbers a run is read by.
#[derive(Debug, PartialEq)]
pub struct Summary {
    pub count: usize,
    pub p50: f64,
    pub p95: f64,
    pub max: f64,
}

impl Summary {
    /// Percentiles of `samples` in milliseconds, nearest rank.
    pub fn of(samples: &[f64]) -> Summary {
        if samples.is_empty() {
            return Summary {
                count: 0,
                p50: 0.0,
                p95: 0.0,
                max: 0.0,
            };
        }
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Summary {
            count: sorted.len(),
            p50: percentile(&sorted, 0.50),
            p95: percentile(&sorted, 0.95),
            max: *sorted.last().unwrap_or(&0.0),
        }
    }
}

/// The value below which `share` of the samples fall, nearest rank.
fn percentile(sorted: &[f64], share: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (share * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_middle_and_the_tail() {
        let samples: Vec<f64> = (1..=100).map(|n| n as f64).collect();
        let summary = Summary::of(&samples);
        assert_eq!(summary.count, 100);
        assert_eq!(summary.p50, 50.0);
        assert_eq!(summary.p95, 95.0);
        assert_eq!(summary.max, 100.0);
    }

    #[test]
    fn a_short_run_still_has_a_tail() {
        let summary = Summary::of(&[120.0, 90.0, 200.0]);
        assert_eq!(summary.count, 3);
        assert_eq!(summary.p50, 120.0);
        assert_eq!(summary.p95, 200.0);
    }

    #[test]
    fn nothing_measured_is_not_a_division_by_zero() {
        assert_eq!(Summary::of(&[]).count, 0);
        assert_eq!(Summary::of(&[7.0]).max, 7.0);
    }
}
