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

use crate::translate::{TranslationResult, WordDetails};

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

/// Fills the details of the newest word entry that matches `source_text`.
///
/// The card is recorded the moment the translation lands, before the phonetic
/// symbols, meanings and example have been looked up. Reopening that entry is
/// meant to show the same card the popup ended up with, so the details are
/// written into the stored entry as well.
pub fn patch_details(app: &AppHandle, source_text: &str, details: &WordDetails) {
    if details.is_empty() {
        return;
    }
    let filled = match ENTRIES.lock() {
        Ok(mut guard) => fill_details(&mut guard, source_text, details),
        Err(_) => false,
    };
    if filled {
        store(app);
    }
}

/// Copies `details` into the newest word entry for `source_text`, leaving
/// everything the entry already holds alone. Answers whether anything changed.
fn fill_details(entries: &mut [Entry], source_text: &str, details: &WordDetails) -> bool {
    let Some(entry) = entries
        .iter_mut()
        .find(|entry| entry.result.kind == "word" && entry.result.source_text == source_text)
    else {
        return false;
    };
    let mut filled = false;
    if entry.result.phonetic.is_none() && details.phonetic.is_some() {
        entry.result.phonetic = details.phonetic.clone();
        filled = true;
    }
    if entry.result.meanings.is_empty() && !details.meanings.is_empty() {
        entry.result.meanings = details.meanings.clone();
        filled = true;
    }
    if entry.result.example.is_none() && details.example.is_some() {
        entry.result.example = details.example.clone();
        filled = true;
    }
    filled
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
            conversions: Vec::new(),
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

    #[test]
    fn late_details_reach_the_newest_card_of_that_word() {
        let mut word = result("ephemeral", "短暂的");
        word.kind = "word".to_string();
        let mut entries = vec![entry(2, &word), entry(1, &word)];
        let details = details();

        assert!(fill_details(&mut entries, "ephemeral", &details));
        assert_eq!(entries[0].result.phonetic.as_deref(), Some("/ɪˈfem.ər.əl/"));
        assert_eq!(
            entries[0].result.example.as_deref(),
            Some("an ephemeral joy")
        );
        assert_eq!(entries[0].result.meanings.len(), 1);
        // Only the newest card of that word is touched.
        assert!(entries[1].result.phonetic.is_none());
        // Nothing is left to add the second time around.
        assert!(!fill_details(&mut entries, "ephemeral", &details));
    }

    #[test]
    fn late_details_never_touch_another_card() {
        let mut sentence = result("ephemeral", "短暂的");
        sentence.kind = "sentence".to_string();
        let mut entries = vec![entry(1, &sentence)];

        assert!(!fill_details(&mut entries, "ephemeral", &details()));
        assert!(!fill_details(&mut entries, "resilient", &details()));
        assert!(entries[0].result.phonetic.is_none());
    }

    #[test]
    fn late_details_keep_what_the_card_already_showed() {
        let mut word = result("ephemeral", "短暂的");
        word.kind = "word".to_string();
        word.phonetic = Some("/from the dictionary/".to_string());
        word.example = Some("kept".to_string());
        let mut entries = vec![entry(1, &word)];

        assert!(fill_details(&mut entries, "ephemeral", &details()));
        assert_eq!(
            entries[0].result.phonetic.as_deref(),
            Some("/from the dictionary/")
        );
        assert_eq!(entries[0].result.example.as_deref(), Some("kept"));
        assert_eq!(entries[0].result.meanings.len(), 1);
    }

    fn details() -> WordDetails {
        WordDetails {
            phonetic: Some("/ɪˈfem.ər.əl/".to_string()),
            meanings: vec![crate::translate::Meaning {
                part_of_speech: "adjective".to_string(),
                definitions: vec!["lasting a very short time".to_string()],
            }],
            example: Some("an ephemeral joy".to_string()),
        }
    }
}
