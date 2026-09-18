//! Optional fallback that fills in missing phonetics or examples for English
//! words. Strictly best effort: the API is public and rate limited, so any
//! failure is ignored.

use std::time::Duration;

use serde_json::Value;

use super::TranslationResult;

const ENDPOINT: &str = "https://api.dictionaryapi.dev/api/v2/entries/en/";

/// This endpoint is a best effort extra, so it must never hold back a popup for
/// long: the shared client would otherwise wait its full timeout.
const TIMEOUT: Duration = Duration::from_secs(4);

const MAX_MEANINGS: usize = 4;
const MAX_DEFINITIONS: usize = 4;

pub async fn enrich(client: &reqwest::Client, text: &str, result: &mut TranslationResult) {
    if result.phonetic.is_some() && result.example.is_some() {
        return;
    }
    let Some(word) = english_word(text) else {
        return;
    };

    let url = format!("{ENDPOINT}{}", urlencoding::encode(word));
    let Ok(response) = client.get(&url).timeout(TIMEOUT).send().await else {
        return;
    };
    if !response.status().is_success() {
        return;
    }
    let Ok(body) = response.text().await else {
        return;
    };
    let Ok(data) = serde_json::from_str::<Value>(&body) else {
        return;
    };
    let Some(entries) = data.as_array() else {
        return;
    };

    if result.phonetic.is_none() {
        result.phonetic = entries.iter().find_map(phonetic_of);
    }

    if result.meanings.is_empty() {
        for entry in entries.iter().take(1) {
            let Some(meanings) = entry.get("meanings").and_then(|value| value.as_array()) else {
                continue;
            };
            for meaning in meanings.iter().take(MAX_MEANINGS) {
                let definitions: Vec<String> = meaning
                    .get("definitions")
                    .and_then(|value| value.as_array())
                    .map(|list| {
                        list.iter()
                            .filter_map(|definition| {
                                definition
                                    .get("definition")
                                    .and_then(|value| value.as_str())
                            })
                            .map(|value| value.trim().to_string())
                            .filter(|value| !value.is_empty())
                            .take(MAX_DEFINITIONS)
                            .collect()
                    })
                    .unwrap_or_default();
                if definitions.is_empty() {
                    continue;
                }
                result.meanings.push(super::Meaning {
                    part_of_speech: meaning
                        .get("partOfSpeech")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    definitions,
                });
            }
        }
    }

    if result.example.is_none() {
        result.example = entries.iter().find_map(example_of);
    }
}

fn phonetic_of(entry: &Value) -> Option<String> {
    if let Some(text) = entry
        .get("phonetic")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(text.to_string());
    }
    entry
        .get("phonetics")
        .and_then(|value| value.as_array())?
        .iter()
        .filter_map(|value| value.get("text").and_then(|value| value.as_str()))
        .map(str::trim)
        .find(|value| !value.is_empty())
        .map(str::to_string)
}

fn example_of(entry: &Value) -> Option<String> {
    entry
        .get("meanings")?
        .as_array()?
        .iter()
        .flat_map(|meaning| meaning.get("definitions"))
        .filter_map(|definitions| definitions.as_array())
        .flatten()
        .filter_map(|definition| definition.get("example").and_then(|value| value.as_str()))
        .map(str::trim)
        .find(|value| !value.is_empty())
        .map(str::to_string)
}

/// Only plain English words are worth looking up in an English dictionary.
fn english_word(text: &str) -> Option<&str> {
    let word = text.trim().trim_end_matches(['.', ',', '!', '?']);
    if word.is_empty() || word.chars().count() > 32 {
        return None;
    }
    let plain = word
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c == '-' || c == '\'');
    plain.then_some(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_english_words_are_looked_up() {
        assert_eq!(english_word("running"), Some("running"));
        assert_eq!(english_word(" mother-in-law "), Some("mother-in-law"));
        assert_eq!(english_word("你好"), None);
        assert_eq!(english_word("two words"), None);
        assert_eq!(english_word(""), None);
    }

    #[test]
    fn reads_phonetics_meanings_and_examples() {
        let entry: Value = serde_json::from_str(
            r#"[{
                "word": "running",
                "phonetics": [{ "text": "/ˈrʌnɪŋ/" }],
                "meanings": [{
                    "partOfSpeech": "noun",
                    "definitions": [
                        { "definition": "The action of running.", "example": "He likes running." }
                    ]
                }]
            }]"#,
        )
        .expect("fixture parses");
        let entries = entry.as_array().expect("array");

        assert_eq!(
            entries.iter().find_map(phonetic_of),
            Some("/ˈrʌnɪŋ/".to_string())
        );
        assert_eq!(
            entries.iter().find_map(example_of),
            Some("He likes running.".to_string())
        );
    }
}
