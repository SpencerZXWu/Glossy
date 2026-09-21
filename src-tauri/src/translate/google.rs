//! Free Google Translate endpoint.
//!
//! `translate_a/single` returns one ragged JSON array that carries the
//! translation, the detected source language, phonetics, a dictionary and an
//! example sentence in a single request.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::classify::Kind;

use super::{Meaning, TranslationResult, MAX_DEFINITIONS, MAX_MEANINGS, MAX_SYNONYMS};

const BASE: &str = "https://translate.googleapis.com/translate_a/single?client=";

/// `dict-chrome-ex` is the client the Chrome dictionary uses and answers with
/// the same JSON shape. The older `gtx` client replies `429 Too Many Requests`
/// to most non-curl HTTP clients, so it is only kept as a fallback.
const CLIENTS: [&str; 2] = ["dict-chrome-ex", "gtx"];

/// Returned when the endpoint answers with its "we are sorry" HTML page instead
/// of JSON, which is how it refuses automated queries.
const BLOCKED: &str = "Google Translate refused the request. Please try again in a \
                       minute, or pick another provider in the settings.";

fn looks_like_html(body: &str) -> bool {
    let head = body.trim_start();
    head.starts_with('<') && head.chars().take(512).collect::<String>().contains("<html")
}

pub fn endpoint(
    client: &str,
    text: &str,
    source: &str,
    target: &str,
    with_dictionary: bool,
) -> String {
    let mut url = format!(
        "{BASE}{client}&sl={}&dt=t&dt=rm&tl={}",
        urlencoding::encode(source),
        urlencoding::encode(target)
    );
    if with_dictionary {
        // `dt=bd` adds the dictionary block, `dt=ex` an example sentence.
        url.push_str("&dt=bd&dt=ex");
    }
    url.push_str("&q=");
    url.push_str(&urlencoding::encode(text));
    url
}

pub async fn translate(
    client: &reqwest::Client,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    run(client, text, source, target, kind, None).await
}

/// How long the whole of `translate_quickly` may take, every client it tries
/// included.
///
/// The extras of a card fill in while the user is already reading the
/// translation, so the lookup is only worth a moment: a budget per client would
/// double the wait on a network that cannot reach the endpoint at all.
const QUICK_BUDGET: Duration = Duration::from_millis(1500);

/// An attempt is not started with less of the budget left than this.
const MIN_ATTEMPT: Duration = Duration::from_millis(250);

/// How long the quick path is skipped after an attempt came back with nothing.
///
/// Without this every card would wait out `QUICK_BUDGET` again on a network
/// where the endpoint is unreachable or rate limiting, and a card that fills in
/// its extras from the dictionary alone would still feel slow.
const QUICK_COOLDOWN: Duration = Duration::from_secs(120);

/// Milliseconds since the Unix epoch when the quick path may be tried again.
/// `0` while it is healthy.
static QUICK_DOWN_UNTIL: AtomicU64 = AtomicU64::new(0);

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Whether the quick path is still inside a cooldown that ends at `until`.
fn cooling_down(now: u64, until: u64) -> bool {
    now < until
}

/// The budget left for one request, or `None` when the lookup has spent it.
fn attempt_budget(budget: Duration, started: Instant, now: Instant) -> Option<Duration> {
    let left = budget.saturating_sub(now.saturating_duration_since(started));
    (left >= MIN_ATTEMPT).then_some(left)
}

/// The same lookup, giving up after `QUICK_BUDGET`.
///
/// Used for the extras of a card — phonetic symbols, meanings, an example —
/// which fill in while the user is already reading the translation: an endpoint
/// that is slow or unreachable on this network must not keep them waiting, and
/// whatever has not arrived is simply left out. A lookup that failed is
/// remembered for `QUICK_COOLDOWN`, so the cards after it answer at once.
pub async fn translate_quickly(
    client: &reqwest::Client,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    if cooling_down(now_millis(), QUICK_DOWN_UNTIL.load(Ordering::Relaxed)) {
        return Err(
            "Google Translate is unavailable right now; skipping the dictionary lookup."
                .to_string(),
        );
    }

    let result = run(client, text, source, target, kind, Some(QUICK_BUDGET)).await;
    let until = if result.is_ok() {
        0
    } else {
        now_millis().saturating_add(QUICK_COOLDOWN.as_millis() as u64)
    };
    QUICK_DOWN_UNTIL.store(until, Ordering::Relaxed);
    result
}

async fn run(
    client: &reqwest::Client,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
    budget: Option<Duration>,
) -> Result<TranslationResult, String> {
    let started = Instant::now();
    let mut failure = None;
    for name in CLIENTS {
        let left = match budget {
            // The clients share one budget: a fallback is only tried while
            // something of it is left.
            Some(budget) => match attempt_budget(budget, started, Instant::now()) {
                Some(left) => Some(left),
                None => break,
            },
            None => None,
        };
        match request(client, name, text, source, target, kind, left).await {
            Ok(result) => return Ok(result),
            Err(error) => failure = Some(error),
        }
    }
    Err(failure.unwrap_or_else(|| "Google Translate is unavailable.".to_string()))
}

async fn request(
    client: &reqwest::Client,
    name: &str,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
    budget: Option<Duration>,
) -> Result<TranslationResult, String> {
    let url = endpoint(name, text, source, target, kind == Kind::Word);
    let mut call = client.get(&url);
    if let Some(budget) = budget {
        call = call.timeout(budget);
    }
    let response = call
        .send()
        .await
        .map_err(|error| format!("Could not reach Google Translate: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(match status.as_u16() {
            429 => "Google Translate is rate limiting requests right now. Please try again in a \
                    minute, or pick another provider in the settings."
                .to_string(),
            _ => format!("Google Translate returned HTTP {status}."),
        });
    }

    let body = response
        .text()
        .await
        .map_err(|error| format!("Could not read the Google Translate response: {error}"))?;
    parse(&body, text, target, kind)
}

pub fn parse(
    body: &str,
    text: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    let body = body.trim_start_matches('\u{feff}');
    if looks_like_html(body) {
        return Err(BLOCKED.to_string());
    }

    let data: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "Google Translate sent a response Glossy could not read.".to_string())?;

    let segments = data
        .get(0)
        .and_then(|value| value.as_array())
        .ok_or_else(|| "Google Translate sent a response Glossy could not read.".to_string())?;

    let mut result = TranslationResult::new(kind, "google", text, target);

    // A sentence can come back as several segments that have to be joined.
    let mut translation = String::new();
    for segment in segments {
        if let Some(part) = segment.get(0).and_then(|value| value.as_str()) {
            translation.push_str(part);
        }
    }
    if translation.trim().is_empty() {
        return Err("Google Translate returned an empty translation.".to_string());
    }
    result.translation = translation;

    if let Some(language) = data.get(2).and_then(|value| value.as_str()) {
        result.source_lang = language.to_string();
    }

    // Google appends the phonetics of the selected text as `[null, null,
    // <translation romanization>, <source phonetics>]` to the first segment it
    // belongs to, so the entry has to be searched for.
    result.phonetic = segments
        .iter()
        .filter_map(|segment| segment.get(3))
        .filter_map(|value| value.as_str())
        .map(clean)
        .find(|value| !value.is_empty());

    result.meanings = parse_meanings(&data);

    result.synonyms = parse_synonyms(&data);

    result.example = data
        .get(13)
        .and_then(|value| value.get(0))
        .and_then(|value| value.get(0))
        .and_then(|value| value.get(0))
        .and_then(|value| value.as_str())
        .map(clean)
        .filter(|value| !value.is_empty());

    Ok(result)
}

fn parse_meanings(data: &serde_json::Value) -> Vec<Meaning> {
    let Some(entries) = data.get(1).and_then(|value| value.as_array()) else {
        return Vec::new();
    };

    let mut meanings = Vec::new();
    for entry in entries.iter().take(MAX_MEANINGS) {
        let Some(fields) = entry.as_array() else {
            continue;
        };
        let part_of_speech = fields
            .first()
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        let definitions: Vec<String> = fields
            .get(1)
            .and_then(|value| value.as_array())
            .map(|list| {
                list.iter()
                    .filter_map(|value| value.as_str())
                    .map(clean)
                    .filter(|value| !value.is_empty())
                    .take(MAX_DEFINITIONS)
                    .collect()
            })
            .unwrap_or_default();
        if !definitions.is_empty() {
            meanings.push(Meaning {
                part_of_speech,
                definitions,
            });
        }
    }
    meanings
}

/// The words that mean roughly the same as the selected one.
///
/// Every sense of the entry carries them as `[<the sense translated>,
/// [<synonym>, ...], ...]`, so the second field of a sense lists its synonyms in
/// the language the word was written in — which is the language the card shows
/// the headword in.
fn parse_synonyms(data: &serde_json::Value) -> Vec<String> {
    let Some(entries) = data.get(1).and_then(|value| value.as_array()) else {
        return Vec::new();
    };

    let mut synonyms: Vec<String> = Vec::new();
    for entry in entries.iter().take(MAX_MEANINGS) {
        let Some(groups) = entry.get(2).and_then(|value| value.as_array()) else {
            continue;
        };
        for group in groups {
            let Some(list) = group.get(1).and_then(|value| value.as_array()) else {
                continue;
            };
            for word in list.iter().filter_map(|value| value.as_str()).map(clean) {
                let known = synonyms.iter().any(|kept| kept.eq_ignore_ascii_case(&word));
                if !word.is_empty() && !known && synonyms.len() < MAX_SYNONYMS {
                    synonyms.push(word);
                }
            }
        }
    }
    synonyms
}

/// Longest run of bytes that is still read as a tag.
const MAX_TAG_LEN: usize = 64;
/// Drops any markup and collapses the whitespace of a value from the response.
///
/// Only something that really looks like a tag is removed: a `<`, an optional
/// `/` and a letter, then the `>`. Anything else — a comparison such as
/// `if x < 5`, an operator such as `<<`, or a lone `<` at the end — is text the
/// provider meant to show and is kept as it is.
fn clean(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut text = String::with_capacity(value.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'<' {
            if let Some(length) = tag_length(&bytes[index..]) {
                index += length;
                continue;
            }
        }
        let character = value[index..]
            .chars()
            .next()
            .expect("index is a character boundary");
        text.push(character);
        index += character.len_utf8();
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Length in bytes of the tag that starts at a `<`, or `None` when the input
/// only looks like the beginning of one.
fn tag_length(bytes: &[u8]) -> Option<usize> {
    if bytes.first() != Some(&b'<') {
        return None;
    }
    let mut index = 1;
    if bytes.get(index) == Some(&b'/') {
        index += 1;
    }
    // A tag name starts with a letter (`<b>`, `</b>`, `<a href="…">`), which also
    // rules out the `<` of a comparison.
    if !bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
        return None;
    }
    while index < bytes.len() && index <= MAX_TAG_LEN {
        match bytes[index] {
            b'>' => return Some(index + 1),
            // A nested `<` means this was never a tag.
            b'<' => return None,
            _ => index += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORD: &str = include_str!("../../tests/fixtures/google_word.json");
    const SENTENCE: &str = include_str!("../../tests/fixtures/google_sentence.json");
    const CJK: &str = include_str!("../../tests/fixtures/google_cjk.json");

    #[test]
    fn parses_a_dictionary_entry() {
        let result = parse(WORD, "running", "zh-CN", Kind::Word).expect("fixture parses");

        assert_eq!(result.translation, "跑步");
        assert_eq!(result.source_lang, "en");
        assert_eq!(result.phonetic.as_deref(), Some("ˈrəniNG"));
        assert_eq!(result.meanings[0].part_of_speech, "noun");
        assert_eq!(result.meanings[0].definitions, vec!["赛跑".to_string()]);
        assert!(result.meanings.len() >= 3);
        let example = result.example.expect("example sentence");
        assert!(example.contains("running"), "{example}");
        assert!(!example.contains('<'));
    }

    #[test]
    fn parses_a_sentence_without_dictionary_data() {
        let result = parse(
            SENTENCE,
            "The quick brown fox jumps over the lazy dog.",
            "zh-CN",
            Kind::Sentence,
        )
        .expect("fixture parses");

        assert!(
            result.translation.starts_with("敏捷的棕色狐狸"),
            "{}",
            result.translation
        );
        assert!(result.meanings.is_empty());
        assert!(result.phonetic.is_none());
        assert!(result.example.is_none());
        assert_eq!(result.source_lang, "en");
    }

    #[test]
    fn parses_a_cjk_word_translated_into_english() {
        let result = parse(CJK, "你好", "en", Kind::Word).expect("fixture parses");

        assert_eq!(result.translation, "Hello");
        assert_eq!(result.source_lang, "zh-CN");
        assert_eq!(result.phonetic.as_deref(), Some("Nǐ hǎo"));
        assert_eq!(result.meanings[0].part_of_speech, "interjection");
    }

    #[test]
    fn rejects_an_empty_translation() {
        let body = r#"[[["","",null,null,5]],null,"en"]"#;
        assert!(parse(body, "running", "zh-CN", Kind::Word).is_err());
    }

    #[test]
    fn rejects_a_malformed_payload() {
        assert!(parse("not json at all", "running", "zh-CN", Kind::Word).is_err());
    }

    #[test]
    fn explains_the_blocked_page() {
        let body = "<!DOCTYPE html><html><head><title>Sorry...</title></head></html>";
        let error = parse(body, "running", "zh-CN", Kind::Word).expect_err("blocked");
        assert_eq!(error, BLOCKED);
    }

    #[test]
    fn builds_an_encoded_url() {
        let url = endpoint("dict-chrome-ex", "hello world", "auto", "zh-CN", true);
        assert!(url.contains("client=dict-chrome-ex"));
        assert!(url.contains("sl=auto"));
        assert!(url.contains("tl=zh-CN"));
        assert!(url.contains("dt=bd"));
        assert!(url.contains("q=hello%20world"));

        let plain = endpoint("gtx", "hello world", "en", "zh-CN", false);
        assert!(plain.contains("client=gtx"));
        assert!(plain.contains("sl=en"));
        assert!(!plain.contains("dt=bd"));
    }

    #[test]
    fn prefers_the_client_that_is_not_throttled() {
        assert_eq!(CLIENTS[0], "dict-chrome-ex");
    }

    #[test]
    fn shares_one_budget_between_the_clients() {
        let started = Instant::now();

        // A fresh lookup hands the whole budget to the first client.
        assert_eq!(
            attempt_budget(QUICK_BUDGET, started, started),
            Some(QUICK_BUDGET)
        );

        // What the first client used is gone from the second one's budget.
        let spent = started + Duration::from_millis(400);
        assert_eq!(
            attempt_budget(QUICK_BUDGET, started, spent),
            Some(QUICK_BUDGET - Duration::from_millis(400))
        );

        // A budget that is spent, or nearly spent, is not worth a request.
        let spent = started + QUICK_BUDGET - Duration::from_millis(10);
        assert_eq!(attempt_budget(QUICK_BUDGET, started, spent), None);
        let spent = started + QUICK_BUDGET;
        assert_eq!(attempt_budget(QUICK_BUDGET, started, spent), None);
    }

    #[test]
    fn a_failed_lookup_stays_in_cooldown_until_its_deadline() {
        assert!(cooling_down(1_000, 1_001));
        assert!(!cooling_down(1_001, 1_001));
        assert!(!cooling_down(1_002, 1_001));
        // Nothing is remembered before the first failure.
        assert!(!cooling_down(1_000, 0));
    }

    #[test]
    fn strips_the_markup_of_a_definition() {
        assert_eq!(clean("<b>running</b>"), "running");
        assert_eq!(clean("run<b>ning</b>"), "running");
        assert_eq!(clean("<a href=\"https://example.com\">link</a>"), "link");
        assert_eq!(clean("<i>  a  </i>\n <em>b</em>"), "a b");
    }

    #[test]
    fn keeps_a_less_than_sign_that_is_not_markup() {
        assert_eq!(clean("if x < 5 then stop"), "if x < 5 then stop");
        assert_eq!(clean("a < b > c"), "a < b > c");
        assert_eq!(clean("value <"), "value <");
        assert_eq!(clean("2 << 3"), "2 << 3");
        assert_eq!(clean("< 5"), "< 5");
        assert_eq!(clean("a <b"), "a <b");
    }

    #[test]
    fn keeps_text_after_an_unfinished_tag() {
        // A `<` that never closes must not swallow the rest of the value.
        assert_eq!(clean("cost < 10 and > 5"), "cost < 10 and > 5");
    }
}
