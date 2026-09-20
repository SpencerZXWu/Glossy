//! Live exchange rates, with two independent sources and a cache on disk so a
//! card can still show a conversion while offline.
//!
//! The whole feature is a bonus next to the translation, so nothing in here is
//! allowed to be slow: every request has its own timeout, a chain of failures
//! stops asking for a while, and an answer that is too old is still used, only
//! marked as stale.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

/// A table younger than this is used without asking again.
const FRESH: Duration = Duration::from_secs(6 * 60 * 60);
/// A table older than this is not worth showing at all.
const USABLE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// After a failed round both sources are left alone for a while.
const COOLDOWN: Duration = Duration::from_secs(120);
/// One request may take this long.
const TIMEOUT: Duration = Duration::from_secs(4);

const PRIMARY: &str = "https://open.er-api.com/v6/latest/";
const FALLBACK: &str = "https://api.frankfurter.app/latest?from=";

const SOURCE_PRIMARY: &str = "exchangerate-api.com";
const SOURCE_FALLBACK: &str = "frankfurter.app";
const CACHE_FILE: &str = "rates.json";

/// One rate, ready to be shown.
#[derive(Debug, Clone)]
pub struct Rate {
    pub value: f64,
    pub source: &'static str,
    pub date: Option<String>,
    /// The only rate available was older than the freshness window.
    pub stale: bool,
}

/// Every rate one source returned for one base currency.
#[derive(Debug, Clone)]
struct Table {
    source: &'static str,
    date: Option<String>,
    /// Unix seconds when the table was received.
    fetched: u64,
    rates: HashMap<String, f64>,
}

impl Table {
    fn age(&self) -> Duration {
        Duration::from_secs(now().saturating_sub(self.fetched))
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

fn memory() -> &'static Mutex<HashMap<String, Table>> {
    static MEMORY: OnceLock<Mutex<HashMap<String, Table>>> = OnceLock::new();
    MEMORY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn quiet_until() -> &'static Mutex<u64> {
    static UNTIL: OnceLock<Mutex<u64>> = OnceLock::new();
    UNTIL.get_or_init(|| Mutex::new(0))
}

/// The rate from one currency to another, or `None` when no source could say.
pub async fn rate(
    client: &reqwest::Client,
    from: &str,
    to: &str,
    cache: Option<&Path>,
) -> Option<Rate> {
    let from = from.to_ascii_uppercase();
    let to = to.to_ascii_uppercase();
    if from == to {
        return None;
    }

    if let Some(table) = remembered(&from, cache) {
        if table.age() < FRESH {
            return lookup(&table, &to);
        }
    }

    let cooling_down = *quiet_until().lock().expect("poisoned") > now();
    if !cooling_down {
        match fetch(client, &from).await {
            Some(table) => {
                remember(&from, &table, cache);
                return lookup(&table, &to);
            }
            None => {
                *quiet_until().lock().expect("poisoned") = now() + COOLDOWN.as_secs();
            }
        }
    }

    // Nothing fresh: an old table is better than an empty annotation.
    remembered(&from, cache).and_then(|table| {
        lookup(&table, &to).map(|rate| Rate {
            stale: true,
            ..rate
        })
    })
}

fn lookup(table: &Table, to: &str) -> Option<Rate> {
    let value = *table.rates.get(to)?;
    Some(Rate {
        value,
        source: table.source,
        date: table.date.clone(),
        stale: false,
    })
}

/// The table held in memory, falling back to the one left on disk.
fn remembered(from: &str, cache: Option<&Path>) -> Option<Table> {
    if let Some(table) = memory().lock().expect("poisoned").get(from) {
        if table.age() < USABLE {
            return Some(table.clone());
        }
    }
    let table = read_cache(cache)?.remove(from)?;
    if table.age() >= USABLE {
        return None;
    }
    memory()
        .lock()
        .expect("poisoned")
        .insert(from.to_string(), table.clone());
    Some(table)
}

fn remember(from: &str, table: &Table, cache: Option<&Path>) {
    memory()
        .lock()
        .expect("poisoned")
        .insert(from.to_string(), table.clone());
    let mut tables = read_cache(cache).unwrap_or_default();
    tables.insert(from.to_string(), table.clone());
    write_cache(cache, &tables);
}

fn cache_path(cache: Option<&Path>) -> Option<PathBuf> {
    Some(cache?.join(CACHE_FILE))
}

fn read_cache(cache: Option<&Path>) -> Option<HashMap<String, Table>> {
    let text = std::fs::read_to_string(cache_path(cache)?).ok()?;
    let data: Value = serde_json::from_str(&text).ok()?;
    let mut tables = HashMap::new();
    for (base, entry) in data.as_object()? {
        let source = match entry.get("source").and_then(Value::as_str) {
            Some(SOURCE_PRIMARY) => SOURCE_PRIMARY,
            Some(SOURCE_FALLBACK) => SOURCE_FALLBACK,
            _ => SOURCE_FALLBACK,
        };
        let Some(fetched) = entry.get("fetched").and_then(Value::as_u64) else {
            continue;
        };
        let mut rates = HashMap::new();
        for (code, value) in entry.get("rates").and_then(Value::as_object)?.iter() {
            if let Some(value) = value.as_f64() {
                rates.insert(code.to_ascii_uppercase(), value);
            }
        }
        if rates.is_empty() {
            continue;
        }
        tables.insert(
            base.to_ascii_uppercase(),
            Table {
                source,
                date: entry
                    .get("date")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                fetched,
                rates,
            },
        );
    }
    Some(tables)
}

fn write_cache(cache: Option<&Path>, tables: &HashMap<String, Table>) {
    let Some(path) = cache_path(cache) else {
        return;
    };
    let data: serde_json::Map<String, Value> = tables
        .iter()
        .map(|(base, table)| {
            let rates: serde_json::Map<String, Value> = table
                .rates
                .iter()
                .map(|(code, value)| (code.clone(), Value::from(*value)))
                .collect();
            let entry = serde_json::json!({
                "source": table.source,
                "date": table.date,
                "fetched": table.fetched,
                "rates": Value::Object(rates),
            });
            (base.clone(), entry)
        })
        .collect();
    let text = Value::Object(data).to_string();
    if let Some(directory) = path.parent() {
        let _ = std::fs::create_dir_all(directory);
    }
    let _ = std::fs::write(path, text);
}

/// Asks the two sources in turn and returns the first usable table.
async fn fetch(client: &reqwest::Client, base: &str) -> Option<Table> {
    if let Some(table) = fetch_primary(client, base).await {
        return Some(table);
    }
    fetch_fallback(client, base).await
}

async fn fetch_primary(client: &reqwest::Client, base: &str) -> Option<Table> {
    let url = format!("{PRIMARY}{base}");
    let body = client
        .get(&url)
        .timeout(TIMEOUT)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let data: Value = serde_json::from_str(&body).ok()?;
    if data.get("result").and_then(Value::as_str) != Some("success") {
        return None;
    }
    Some(Table {
        source: SOURCE_PRIMARY,
        date: data
            .get("time_last_update_utc")
            .and_then(Value::as_str)
            .and_then(iso_date),
        fetched: now(),
        rates: rates_of(&data)?,
    })
}

async fn fetch_fallback(client: &reqwest::Client, base: &str) -> Option<Table> {
    let url = format!("{FALLBACK}{base}");
    let body = client
        .get(&url)
        .timeout(TIMEOUT)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let data: Value = serde_json::from_str(&body).ok()?;
    let mut rates = rates_of(&data)?;
    // The fallback answers relative to the requested base, which it leaves out
    // of the map.
    rates.insert(base.to_string(), 1.0);
    Some(Table {
        source: SOURCE_FALLBACK,
        date: data.get("date").and_then(Value::as_str).map(str::to_string),
        fetched: now(),
        rates,
    })
}

/// The `rates` object of either source, upper cased.
fn rates_of(data: &Value) -> Option<HashMap<String, f64>> {
    let mut rates = HashMap::new();
    for (code, value) in data.get("rates")?.as_object()?.iter() {
        if let Some(value) = value.as_f64() {
            if value.is_finite() && value > 0.0 {
                rates.insert(code.to_ascii_uppercase(), value);
            }
        }
    }
    if rates.is_empty() {
        None
    } else {
        Some(rates)
    }
}

/// `Thu, 05 Feb 2026 00:02:31 +0000` -> `2026-02-05`.
fn iso_date(text: &str) -> Option<String> {
    let mut parts = text.split_whitespace();
    let (_, day, month, year) = (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .position(|name| *name == month)?
        + 1;
    let day: u32 = day.parse().ok()?;
    let year: u32 = year.parse().ok()?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(source: &'static str, fetched: u64, rates: &[(&str, f64)]) -> Table {
        Table {
            source,
            date: Some("2026-02-05".to_string()),
            fetched,
            rates: rates
                .iter()
                .map(|(code, value)| (code.to_string(), *value))
                .collect(),
        }
    }

    #[test]
    fn a_rate_is_read_out_of_a_table() {
        let table = table(SOURCE_PRIMARY, now(), &[("USD", 7.12), ("EUR", 7.8)]);
        let rate = lookup(&table, "USD").expect("a rate");
        assert_eq!(rate.value, 7.12);
        assert_eq!(rate.source, SOURCE_PRIMARY);
        assert!(!rate.stale);
        assert!(lookup(&table, "XYZ").is_none());
    }

    #[test]
    fn the_primary_answer_carries_a_date_and_its_rates() {
        let body = r#"{"result":"success","time_last_update_utc":"Thu, 05 Feb 2026 00:02:31 +0000",
            "base_code":"USD","rates":{"USD":1,"CNY":7.1234,"JPY":150.2}}"#;
        let data: Value = serde_json::from_str(body).expect("json");
        assert_eq!(
            data.get("time_last_update_utc")
                .and_then(Value::as_str)
                .and_then(iso_date),
            Some("2026-02-05".to_string())
        );
        let rates = rates_of(&data).expect("rates");
        assert_eq!(rates.get("CNY"), Some(&7.1234));
        assert_eq!(rates.get("jpy"), None);
        assert_eq!(rates.get("JPY"), Some(&150.2));
    }

    #[test]
    fn the_fallback_answers_are_read_too() {
        let body =
            r#"{"amount":1.0,"base":"USD","date":"2026-02-05","rates":{"CNY":7.11,"JPY":149.5}}"#;
        let data: Value = serde_json::from_str(body).expect("json");
        let rates = rates_of(&data).expect("rates");
        assert_eq!(rates.get("CNY"), Some(&7.11));
        assert_eq!(data.get("date").and_then(Value::as_str), Some("2026-02-05"));
    }

    #[test]
    fn a_table_that_carries_nothing_usable_is_rejected() {
        let empty: Value = serde_json::from_str(r#"{"rates":{}}"#).expect("json");
        assert!(rates_of(&empty).is_none());
        let negative: Value = serde_json::from_str(r#"{"rates":{"CNY":0}}"#).expect("json");
        assert!(rates_of(&negative).is_none());
    }

    #[test]
    fn a_date_is_formatted_the_way_a_card_shows_it() {
        assert_eq!(
            iso_date("Thu, 05 Feb 2026 00:02:31 +0000").as_deref(),
            Some("2026-02-05")
        );
        assert_eq!(iso_date("2026-02-05"), None);
        assert_eq!(iso_date(""), None);
    }

    #[test]
    fn a_stored_table_survives_a_round_trip_and_expires() {
        let directory = std::env::temp_dir().join(format!("glossy-rates-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        let tables = HashMap::from([
            (
                "USD".to_string(),
                table(SOURCE_PRIMARY, now(), &[("CNY", 7.12)]),
            ),
            (
                "EUR".to_string(),
                table(SOURCE_FALLBACK, now(), &[("CNY", 7.8)]),
            ),
        ]);
        write_cache(Some(&directory), &tables);

        let read = read_cache(Some(&directory)).expect("a cache");
        assert_eq!(read.len(), 2);
        let usd = read.get("USD").expect("USD");
        assert_eq!(usd.source, SOURCE_PRIMARY);
        assert_eq!(usd.rates.get("CNY"), Some(&7.12));
        assert_eq!(read.get("EUR").expect("EUR").source, SOURCE_FALLBACK);

        // A table that was fetched too long ago is not read again.
        let old = directory.join("old");
        write_cache(
            Some(&old),
            &HashMap::from([(
                "USD".to_string(),
                table(
                    SOURCE_PRIMARY,
                    now() - USABLE.as_secs() - 1,
                    &[("CNY", 7.12)],
                ),
            )]),
        );
        memory().lock().expect("poisoned").remove("USD");
        assert!(remembered("USD", Some(&old)).is_none());

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_fresh_table_in_memory_is_used_without_a_source() {
        let fetched = now();
        memory().lock().expect("poisoned").insert(
            "GBP".to_string(),
            table(SOURCE_PRIMARY, fetched, &[("CNY", 8.9)]),
        );
        let remembered = remembered("GBP", None).expect("a table");
        assert!(remembered.age() < FRESH);
        assert_eq!(remembered.rates.get("CNY"), Some(&8.9));
    }
}
