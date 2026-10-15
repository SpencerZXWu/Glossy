//! The relay's dictionary lookup: one word's entry, asked for from the
//! deployment's own network.
//!
//! The card's meanings used to come from a public endpoint in the reader's
//! network alone, and that endpoint answers in anything from half a second to
//! twenty and is sometimes unreachable from where the reader sits. The relay is
//! the same data from a network that reaches it: the deployment asks, the
//! reader waits for one round trip instead of a timeout.
//!
//! Nothing here is required for a card: the word card is complete without
//! meanings, so every failure is silent and leaves the lookup to the sources
//! that were there before.

use serde_json::Value;

use super::cloud::dictionary;
use super::{Meaning, TranslationResult};

/// Whether the relay's dictionary can say anything about this lookup.
///
/// It carries English entries explained in Chinese, so a card reading into
/// another language is not asked for: an answer in the wrong language is worse
/// than no answer, because it looks like one.
pub fn wanted(source: &str, target: &str) -> bool {
    let target = target.trim().to_ascii_lowercase();
    if !target.starts_with("zh") {
        return false;
    }
    let source = source.trim().to_ascii_lowercase();
    let source = source.split(['-', '_']).next().unwrap_or("");
    // `auto` is what a word selected anywhere comes back as: English is the
    // only thing the endpoint has entries for, so it is assumed and a word it
    // does not know simply comes back empty.
    matches!(source, "" | "auto" | "en")
}

/// One entry, as the card holds it.
///
/// The same three things the public endpoint fills in, so both sources are
/// merged the same way and the first one to answer wins.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entry {
    pub phonetic: Option<String>,
    pub meanings: Vec<Meaning>,
    pub example: Option<String>,
    pub synonyms: Vec<String>,
}

/// The text of one field, when it is there and not empty.
fn text_of(value: Option<&Value>) -> Option<String> {
    let text = value?.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// The parts of speech and their definitions, skipping anything malformed.
fn meanings_of(value: Option<&Value>) -> Vec<Meaning> {
    let Some(rows) = value.and_then(Value::as_array) else {
        return Vec::new();
    };
    rows.iter()
        .filter_map(|row| {
            let definitions: Vec<String> = row
                .get("definitions")?
                .as_array()?
                .iter()
                .filter_map(|definition| text_of(Some(definition)))
                .collect();
            if definitions.is_empty() {
                return None;
            }
            Some(Meaning {
                part_of_speech: row
                    .get("partOfSpeech")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
                definitions,
            })
        })
        .collect()
}

/// The words themselves, ignoring anything that is not a string.
fn strings_of(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(|row| text_of(Some(row))).collect())
        .unwrap_or_default()
}

/// Reads one answer from the relay into an entry.
///
/// Split out from the request so it can be tested against a stored answer: what
/// a deployment sends is a shape, and a shape is worth checking without a
/// network call.
pub fn entry_from(data: &Value) -> Option<Entry> {
    if data.get("ok").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    if data.get("found").and_then(Value::as_bool) != Some(true) {
        return None;
    }

    let entry = Entry {
        phonetic: text_of(data.get("phonetic")),
        meanings: meanings_of(data.get("meanings")),
        example: text_of(data.get("example")),
        synonyms: strings_of(data.get("synonyms")),
    };
    (!entry.meanings.is_empty() || entry.phonetic.is_some()).then_some(entry)
}

/// Asks the deployment for one word, or nothing when it cannot answer.
pub async fn lookup(client: &reqwest::Client, word: &str, install_id: &str) -> Option<Entry> {
    entry_from(&dictionary(client, word, install_id).await?)
}

/// Copies an entry onto a result that has only its headword so far, so the rest
/// of the card is filled from one dictionary and the next only adds to it.
pub fn apply(result: &mut TranslationResult, entry: Entry) {
    let mut relayed = TranslationResult::new(
        super::Kind::Word,
        "",
        &result.source_text,
        &result.target_lang,
    );
    relayed.phonetic = entry.phonetic;
    relayed.meanings = entry.meanings;
    relayed.example = entry.example;
    relayed.synonyms = entry.synonyms;
    result.fill_gaps_from(&relayed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// One answer from a deployment, shaped the way `/v1/dict` sends it.
    fn answer() -> Value {
        json!({
            "ok": true,
            "source": "youdao",
            "word": "hello",
            "found": true,
            "phonetic": "həˈloʊ",
            "phonetics": { "uk": "həˈləʊ", "us": "həˈloʊ" },
            "meanings": [
                { "partOfSpeech": "int.", "definitions": ["喂，你好"] },
                { "partOfSpeech": "n.", "definitions": ["招呼，问候"] },
            ],
            "example": "'Hello, Paul,' they chorused.",
            "synonyms": ["hallo", "holloo"],
        })
    }

    #[test]
    fn reads_an_answer_the_way_the_card_holds_it() {
        let entry = entry_from(&answer()).expect("an entry");
        assert_eq!(entry.phonetic.as_deref(), Some("həˈloʊ"));
        assert_eq!(entry.meanings.len(), 2);
        assert_eq!(entry.meanings[0].part_of_speech, "int.");
        assert_eq!(entry.meanings[0].definitions, vec!["喂，你好".to_string()]);
        assert_eq!(
            entry.example.as_deref(),
            Some("'Hello, Paul,' they chorused.")
        );
        assert_eq!(
            entry.synonyms,
            vec!["hallo".to_string(), "holloo".to_string()]
        );
    }

    #[test]
    fn a_word_the_deployment_does_not_have_is_not_an_entry() {
        let unknown = json!({ "ok": true, "word": "qwertyzx", "found": false, "meanings": [] });
        assert!(entry_from(&unknown).is_none());
    }

    #[test]
    fn a_refusal_from_the_relay_is_not_an_entry() {
        let refused = json!({ "ok": false, "code": "upstream_unreachable", "message": "连不上" });
        assert!(entry_from(&refused).is_none());
        assert!(entry_from(&json!({})).is_none());
    }

    #[test]
    fn keeps_the_meaning_it_can_read_and_drops_the_rest() {
        let partly = json!({
            "ok": true,
            "found": true,
            "phonetic": "",
            "meanings": [
                { "partOfSpeech": "n.", "definitions": [] },
                { "partOfSpeech": "v.", "definitions": ["打招呼", "  "] },
                { "partOfSpeech": "x." },
            ],
            "synonyms": ["ok", 7, ""],
        });
        let entry = entry_from(&partly).expect("an entry");
        assert_eq!(entry.phonetic, None);
        assert_eq!(entry.meanings.len(), 1);
        assert_eq!(entry.meanings[0].definitions, vec!["打招呼".to_string()]);
        assert_eq!(entry.synonyms, vec!["ok".to_string()]);
    }

    #[test]
    fn only_asks_about_a_word_read_into_chinese() {
        for (source, target) in [
            ("en", "zh-CN"),
            ("auto", "zh"),
            ("EN", "zh-TW"),
            ("", "zh-Hans"),
        ] {
            assert!(wanted(source, target), "{source} -> {target}");
        }
        for (source, target) in [("en", "ja"), ("en", "fr"), ("fr", "zh-CN"), ("ja", "zh-CN")] {
            assert!(!wanted(source, target), "{source} -> {target}");
        }
    }

    #[test]
    fn fills_a_card_that_has_only_its_headword() {
        let mut result = TranslationResult::new(super::super::Kind::Word, "", "hello", "zh-CN");
        result.translation = "你好".to_string();
        apply(&mut result, entry_from(&answer()).expect("an entry"));
        assert_eq!(result.phonetic.as_deref(), Some("həˈloʊ"));
        assert_eq!(result.meanings.len(), 2);
        assert_eq!(
            result.translation, "你好",
            "the translation itself is untouched"
        );
    }
}
