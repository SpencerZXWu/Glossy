//! The words and sentences the user kept, in their own book.
//!
//! A wordbook entry is a card the popup showed, stored whole: the same text
//! translated into the same language is kept once, so starring it again is the
//! way to take it back out. The list is written to `vocabulary.json` beside the
//! history, as plain JSON, so it survives a restart and outlives the history it
//! was collected from.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::log::note;
use crate::translate::{TranslationResult, WordDetails};

/// Largest wordbook the file keeps. Kept well above the history cap, because a
/// book the user filled on purpose is not a log that overwrites itself.
pub const MAX_ENTRIES: usize = 2000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Identifies one entry for reopening, revealing or removing it.
    pub id: u64,
    /// Unix seconds of the moment the entry was kept.
    pub at: u64,
    /// The card as the popup showed it, stored whole: the wordbook can redraw a
    /// kept entry — phonetic symbols, meanings and example included — without
    /// asking the provider again.
    pub result: TranslationResult,
}

static ENTRIES: Mutex<Vec<Entry>> = Mutex::new(Vec::new());
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("vocabulary.json"))
}

/// Reads the stored wordbook back into memory.
pub fn load(app: &AppHandle) {
    let Some(path) = path(app) else {
        return;
    };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return;
    };
    let mut entries: Vec<Entry> = serde_json::from_str(&raw).unwrap_or_default();
    entries.truncate(MAX_ENTRIES);
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

/// Whether a text is in the wordbook already, in the language it was translated
/// into. The star of a card asks this when that card appears.
pub fn keeps(source_text: &str, target_lang: &str) -> bool {
    ENTRIES
        .lock()
        .map(|guard| {
            guard
                .iter()
                .any(|entry| same_word(entry, source_text, target_lang))
        })
        .unwrap_or(false)
}

/// Keeps `result`, or drops it again when it is already there. Answers whether
/// it is in the wordbook after the call, which is what the star button shows.
pub fn toggle(app: &AppHandle, result: &TranslationResult) -> bool {
    let at = now();
    let kept = match ENTRIES.lock() {
        Ok(mut guard) => flip(&mut guard, result, at),
        Err(poisoned) => flip(&mut poisoned.into_inner(), result, at),
    };
    store(app);
    kept
}

/// The newest entry first: the word just kept is the one the reader wants to
/// see at the top of the book.
fn flip(entries: &mut Vec<Entry>, result: &TranslationResult, at: u64) -> bool {
    if let Some(index) = entries
        .iter()
        .position(|entry| same_word(entry, &result.source_text, &result.target_lang))
    {
        entries.remove(index);
        return false;
    }
    entries.insert(
        0,
        Entry {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            at,
            result: result.clone(),
        },
    );
    entries.truncate(MAX_ENTRIES);
    true
}

/// One kept word is one text in one target language; translating the same text
/// into another language is a second entry, because it is a second card.
fn same_word(entry: &Entry, source_text: &str, target_lang: &str) -> bool {
    entry.result.source_text == source_text && entry.result.target_lang == target_lang
}

/// Fills the details of a kept word that was starred before they were looked up.
///
/// The card is kept the moment the star is clicked, and the lookups that add
/// phonetic symbols, meanings and an example can still be on their way. Only
/// missing fields are written, so the entry the card is showing keeps what it
/// already has.
pub fn patch_details(
    app: &AppHandle,
    source_text: &str,
    target_lang: Option<&str>,
    details: &WordDetails,
) {
    if details.is_empty() {
        return;
    }
    let filled = match ENTRIES.lock() {
        Ok(mut guard) => fill_details(&mut guard, source_text, target_lang, details),
        Err(_) => false,
    };
    if filled {
        store(app);
    }
}

/// Copies `details` into the newest word entry for `source_text`, in the target
/// language the lookups ran for when one was given. Answers whether anything
/// changed.
fn fill_details(
    entries: &mut [Entry],
    source_text: &str,
    target_lang: Option<&str>,
    details: &WordDetails,
) -> bool {
    let Some(entry) = entries.iter_mut().find(|entry| {
        entry.result.kind == "word"
            && entry.result.source_text == source_text
            && match target_lang {
                Some(wanted) => entry.result.target_lang == wanted,
                None => true,
            }
    }) else {
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

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
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
        note!("Glossy could not write its wordbook: {error}");
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
            synonyms: Vec::new(),
            forms: Vec::new(),
            fallback_from: None,
            fallback_code: None,
            pairs: Vec::new(),
            conversions: Vec::new(),
            notice: None,
        }
    }

    fn details() -> WordDetails {
        WordDetails {
            phonetic: Some("/ɪˈfem.ər.əl/".to_string()),
            meanings: Vec::new(),
            example: Some("an ephemeral joy".to_string()),
            ..WordDetails::default()
        }
    }

    #[test]
    fn keeping_the_same_word_again_takes_it_out() {
        let mut entries = Vec::new();
        assert!(flip(&mut entries, &result("same", "x"), 1));
        assert_eq!(entries.len(), 1);
        assert!(!flip(&mut entries, &result("same", "x"), 2));
        assert!(entries.is_empty());
    }

    #[test]
    fn keeps_the_newest_word_first() {
        let mut entries = Vec::new();
        flip(&mut entries, &result("one", "x"), 1);
        flip(&mut entries, &result("two", "x"), 2);
        assert_eq!(entries[0].result.source_text, "two");
        assert_eq!(entries[1].result.source_text, "one");
    }

    #[test]
    fn one_text_in_two_languages_is_two_entries() {
        let mut entries = Vec::new();
        flip(&mut entries, &result("same", "x"), 1);
        let mut other = result("same", "y");
        other.target_lang = "ja".to_string();
        flip(&mut entries, &other, 2);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn counts_a_word_as_kept_in_the_language_it_was_translated_into() {
        let entries = [Entry {
            id: 1,
            at: 0,
            result: result("same", "x"),
        }];
        assert!(entries
            .iter()
            .any(|entry| same_word(entry, "same", "zh-CN")));
        assert!(!entries.iter().any(|entry| same_word(entry, "same", "ja")));
        assert!(!entries
            .iter()
            .any(|entry| same_word(entry, "other", "zh-CN")));
    }

    #[test]
    fn caps_the_book_at_the_largest_size_it_keeps() {
        let mut entries = Vec::new();
        for index in 0..(MAX_ENTRIES + 5) {
            flip(&mut entries, &result(&format!("word {index}"), "x"), 0);
        }
        assert_eq!(entries.len(), MAX_ENTRIES);
        // The oldest words are the ones that fell out.
        assert_eq!(
            entries[0].result.source_text,
            format!("word {}", MAX_ENTRIES + 4)
        );
        assert!(entries
            .iter()
            .all(|entry| entry.result.source_text != "word 0"));
    }

    #[test]
    fn late_details_reach_the_newest_card_of_that_word_only() {
        let mut word = result("ephemeral", "短暂的");
        word.kind = "word".to_string();
        let mut entries = vec![
            Entry {
                id: 2,
                at: 0,
                result: word.clone(),
            },
            Entry {
                id: 1,
                at: 0,
                result: word,
            },
        ];

        assert!(fill_details(&mut entries, "ephemeral", None, &details()));
        assert_eq!(entries[0].result.phonetic.as_deref(), Some("/ɪˈfem.ər.əl/"));
        assert_eq!(
            entries[0].result.example.as_deref(),
            Some("an ephemeral joy")
        );
        // Only the newest card of that word is touched, and nothing is left to
        // add the second time around.
        assert!(entries[1].result.phonetic.is_none());
        assert!(!fill_details(&mut entries, "ephemeral", None, &details()));
    }

    #[test]
    fn late_details_stay_with_the_target_language_they_were_looked_up_for() {
        let mut english = result("ephemeral", "短暂的");
        english.kind = "word".to_string();
        let mut japanese = english.clone();
        japanese.target_lang = "ja".to_string();
        let mut entries = vec![
            Entry {
                id: 1,
                at: 0,
                result: japanese,
            },
            Entry {
                id: 2,
                at: 0,
                result: english,
            },
        ];

        assert!(fill_details(
            &mut entries,
            "ephemeral",
            Some("ja"),
            &details()
        ));
        assert!(entries[0].result.phonetic.is_some());
        assert!(entries[1].result.phonetic.is_none());
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
