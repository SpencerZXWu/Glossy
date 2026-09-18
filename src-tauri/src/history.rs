//! What Glossy has translated, newest first.
//!
//! The list lives in memory and is mirrored to `history.json`, so it survives a
//! restart. A translation is not a secret — it is what the user just read on
//! screen — so the file is written as plain JSON, unlike the settings file with
//! its API keys.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::translate::TranslationResult;

/// Largest history the settings can ask for.
pub const MAX_LIMIT: u32 = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Identifies one entry for reopening, copying or removing it.
    pub id: u64,
    /// Unix seconds, so the list can be grouped by day.
    pub at: u64,
    /// The card as the popup showed it, stored whole: reopening shows the same
    /// phonetic symbols, meanings and example without asking the provider again.
    pub result: TranslationResult,
}

static ENTRIES: Mutex<Vec<Entry>> = Mutex::new(Vec::new());
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("history.json"))
}

/// Reads the stored history back, keeping at most `limit` entries.
pub fn load(app: &AppHandle, limit: u32) {
    let Some(path) = path(app) else {
        return;
    };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return;
    };
    let mut entries: Vec<Entry> = serde_json::from_str(&raw).unwrap_or_default();
    entries.truncate(limit as usize);
    let next = entries
        .iter()
        .map(|entry| entry.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    NEXT_ID.store(next, Ordering::Relaxed);
    if let Ok(mut guard) = ENTRIES.lock() {
        *guard = entries;
    }
}

/// Adds a translation to the history. A `limit` of 0 turns the history off.
pub fn record(app: &AppHandle, result: &TranslationResult, limit: u32) {
    if limit == 0 {
        return;
    }
    let at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let entry = Entry {
        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        at,
        result: result.clone(),
    };

    if let Ok(mut guard) = ENTRIES.lock() {
        remember(&mut guard, entry, limit);
    }
    store(app);
}

/// Puts an entry at the front, dropping a repeat and anything past the cap.
fn remember(entries: &mut Vec<Entry>, entry: Entry, limit: u32) {
    // Translating the same text into the same language twice in a row — a
    // retry, or the hotkey pressed again — is one entry, not two.
    let repeated = entries.first().is_some_and(|first| {
        first.result.source_text == entry.result.source_text
            && first.result.target_lang == entry.result.target_lang
    });
    if repeated {
        entries.remove(0);
    }
    entries.insert(0, entry);
    entries.truncate(limit as usize);
}

/// Entries from newest to oldest.
pub fn list() -> Vec<Entry> {
    ENTRIES
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

pub fn get(id: u64) -> Option<Entry> {
    ENTRIES
        .lock()
        .ok()
        .and_then(|guard| guard.iter().find(|entry| entry.id == id).cloned())
}

pub fn remove(app: &AppHandle, id: u64) {
    if let Ok(mut guard) = ENTRIES.lock() {
        guard.retain(|entry| entry.id != id);
    }
    store(app);
}

pub fn clear(app: &AppHandle) {
    if let Ok(mut guard) = ENTRIES.lock() {
        guard.clear();
    }
    store(app);
}

/// Drops everything past `limit`, for when the user lowers the cap.
pub fn set_limit(app: &AppHandle, limit: u32) {
    let changed = match ENTRIES.lock() {
        Ok(mut guard) => {
            if guard.len() > limit as usize {
                guard.truncate(limit as usize);
                true
            } else {
                // Turning the history off also forgets what is already there.
                limit == 0 && !guard.is_empty()
            }
        }
        Err(_) => false,
    };
    if limit == 0 || changed {
        clear(app);
    } else {
        store(app);
    }
}

fn store(app: &AppHandle) {
    let Some(path) = path(app) else {
        return;
    };
    let snapshot = match ENTRIES.lock() {
        // A poisoned lock still holds data worth keeping, so read it anyway.
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let Ok(raw) = serde_json::to_string(&snapshot) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(error) = std::fs::write(&path, raw) {
        eprintln!("Glossy could not write its translation history: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translate::TranslationResult;

    fn result(text: &str, translation: &str) -> TranslationResult {
        TranslationResult {
            kind: "sentence".to_string(),
            source_text: text.to_string(),
            translation: translation.to_string(),
            source_lang: "en".to_string(),
            target_lang: "zh-CN".to_string(),
            phonetic: None,
            meanings: Vec::new(),
            example: None,
            provider: "google".to_string(),
        }
    }

    /// A history entry built from a translation, the way `record` builds it.
    fn entry(id: u64, result: &TranslationResult) -> Entry {
        Entry {
            id,
            at: 0,
            result: result.clone(),
        }
    }

    #[test]
    fn keeps_the_newest_entry_first_and_caps_the_list() {
        let mut entries = Vec::new();
        for (id, text) in [(1, "one"), (2, "two"), (3, "three")] {
            remember(&mut entries, entry(id, &result(text, "x")), 2);
        }
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].result.source_text, "three");
        assert_eq!(entries[1].result.source_text, "two");
    }

    #[test]
    fn collapses_a_repeated_translation() {
        let mut entries = Vec::new();
        remember(&mut entries, entry(1, &result("same", "x")), 5);
        remember(&mut entries, entry(2, &result("same", "x")), 5);
        assert_eq!(entries.len(), 1);
        // The entry that survives is the newer one.
        assert_eq!(entries[0].id, 2);
    }

    #[test]
    fn keeps_the_same_text_translated_into_another_language() {
        let mut entries = Vec::new();
        remember(&mut entries, entry(1, &result("same", "x")), 5);
        let mut other = result("same", "y");
        other.target_lang = "ja".to_string();
        remember(&mut entries, entry(2, &other), 5);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn an_entry_survives_a_round_trip_through_json() {
        let mut stored = result("running", "跑步");
        stored.kind = "word".to_string();
        stored.phonetic = Some("/ˈrʌnɪŋ/".to_string());
        stored.example = Some("He is running.".to_string());

        let entry = Entry {
            id: 7,
            at: 1_700_000_000,
            result: stored,
        };
        let raw = serde_json::to_string(&entry).expect("entry serializes");
        let parsed: Entry = serde_json::from_str(&raw).expect("entry deserializes");

        assert_eq!(parsed.id, 7);
        assert_eq!(parsed.result.phonetic.as_deref(), Some("/ˈrʌnɪŋ/"));
        assert_eq!(parsed.result.example.as_deref(), Some("He is running."));
    }
}
