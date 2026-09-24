//! Translation providers and the orchestration that decides what to show.

mod cloud;
mod dictionary;
mod google;
pub mod languages;

use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::classify::{self, Kind};
use crate::settings::{Service, Settings};

pub use self::cloud::{
    new_install_id, quota as cloud_quota, rates as cloud_rates,
    resolved_endpoint as cloud_endpoint, Quota as CloudQuota,
};

/// The free public endpoints reject requests without a browser like agent.
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// Longer selections are rejected before hitting the network.
pub const MAX_TEXT_CHARS: usize = 1800;

/// Parts of speech shown on one card.
const MAX_MEANINGS: usize = 4;

/// Definitions shown under one part of speech.
const MAX_DEFINITIONS: usize = 4;

/// Synonyms shown under a word.
const MAX_SYNONYMS: usize = 6;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meaning {
    pub part_of_speech: String,
    pub definitions: Vec<String>,
}

/// One sentence of the original next to its translation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentencePair {
    pub source: String,
    pub translation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResult {
    /// `word` for dictionary style output, `sentence` for a plain translation.
    pub kind: String,
    pub source_text: String,
    pub translation: String,
    pub source_lang: String,
    pub target_lang: String,
    pub phonetic: Option<String>,
    pub meanings: Vec<Meaning>,
    pub example: Option<String>,
    pub provider: String,
    /// Words that mean roughly the same as the selected one, in the language it
    /// was written in.
    #[serde(default)]
    pub synonyms: Vec<String>,
    /// The forms the word takes, English only.
    #[serde(default)]
    pub forms: Vec<crate::morphology::Form>,
    /// The service the user chose, when another one had to answer because the
    /// chosen one failed. `None` when the chosen service answered, or when it
    /// is the only one that was asked.
    #[serde(default)]
    pub fallback_from: Option<String>,
    /// The translation split sentence by sentence against the original, for the
    /// optional side-by-side view. Empty unless the setting is on, and a single
    /// pair when the two do not divide the same way, which reads as "no
    /// sentence view".
    #[serde(default)]
    pub pairs: Vec<SentencePair>,
    /// Measurements in the translation that a reader of the target language
    /// would not expect, annotated with the switch to the units they do.
    #[serde(default)]
    pub conversions: Vec<crate::units::Conversion>,
}

impl TranslationResult {
    pub fn new(kind: Kind, provider: &str, text: &str, target: &str) -> Self {
        TranslationResult {
            kind: kind.as_str().to_string(),
            source_text: text.to_string(),
            translation: String::new(),
            source_lang: "auto".to_string(),
            target_lang: target.to_string(),
            phonetic: None,
            meanings: Vec::new(),
            example: None,
            provider: provider.to_string(),
            synonyms: Vec::new(),
            forms: Vec::new(),
            fallback_from: None,
            pairs: Vec::new(),
            conversions: Vec::new(),
        }
    }

    /// Copies over anything this result is still missing.
    pub fn fill_gaps_from(&mut self, other: &TranslationResult) {
        if self.phonetic.is_none() {
            self.phonetic = other.phonetic.clone();
        }
        if self.meanings.is_empty() {
            self.meanings = other.meanings.clone();
        } else {
            merge_meanings(&mut self.meanings, &other.meanings);
        }
        if self.example.is_none() {
            self.example = other.example.clone();
        }
        merge_synonyms(&mut self.synonyms, &other.synonyms);
        if self.forms.is_empty() {
            self.forms = other.forms.clone();
        }
        if self.source_lang == "auto" && other.source_lang != "auto" {
            self.source_lang = normalize_lang_code(&other.source_lang);
        }
    }
}

/// Adds the definitions of `extra` to `meanings`, one part of speech at a time.
///
/// Two dictionaries describe the same word in different words, so whichever
/// answered first is kept as it is and the other only adds what is not already
/// there: a card that lists `noun` twice with the same sentence under both is
/// worse than one source alone.
fn merge_meanings(meanings: &mut Vec<Meaning>, extra: &[Meaning]) {
    for meaning in extra {
        let index = meanings.iter().position(|existing| {
            existing
                .part_of_speech
                .trim()
                .eq_ignore_ascii_case(meaning.part_of_speech.trim())
        });
        match index {
            Some(index) => {
                let existing = &mut meanings[index];
                for definition in &meaning.definitions {
                    let known = existing
                        .definitions
                        .iter()
                        .any(|kept| kept.eq_ignore_ascii_case(definition));
                    if !known && existing.definitions.len() < MAX_DEFINITIONS {
                        existing.definitions.push(definition.clone());
                    }
                }
            }
            None if meanings.len() < MAX_MEANINGS => meanings.push(meaning.clone()),
            None => {}
        }
    }
}

/// Adds the synonyms that are not already listed, keeping the first spelling of
/// each.
fn merge_synonyms(synonyms: &mut Vec<String>, extra: &[String]) {
    for synonym in extra {
        let synonym = synonym.trim();
        if synonym.is_empty() || synonyms.len() >= MAX_SYNONYMS {
            return;
        }
        let known = synonyms
            .iter()
            .any(|kept| kept.eq_ignore_ascii_case(synonym));
        if !known {
            synonyms.push(synonym.to_string());
        }
    }
}

/// Removes the synonyms that just repeat the headword in another spelling.
fn drop_the_headword(synonyms: &mut Vec<String>, word: &str) {
    let headword = word.trim().to_lowercase();
    synonyms.retain(|synonym| synonym.trim().to_lowercase() != headword);
}

/// The dictionary style extra of a word: phonetic symbols, meanings and one
/// example. Fetched apart from the translation, because both sources for them
/// answer in anything from half a second to twenty, and a popup that waits for
/// them stays empty for that long.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordDetails {
    pub phonetic: Option<String>,
    pub meanings: Vec<Meaning>,
    pub example: Option<String>,
    /// Words that mean roughly the same, in the language the word was written
    /// in.
    #[serde(default)]
    pub synonyms: Vec<String>,
    /// The forms the word takes, English only.
    #[serde(default)]
    pub forms: Vec<crate::morphology::Form>,
    /// The sentence the word was selected from, translated, when the setting for
    /// it is on and the program in front let the sentence be read.
    #[serde(default)]
    pub context: Option<SentenceContext>,
}

/// The sentence a word was selected from, with its own translation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentenceContext {
    pub text: String,
    pub translation: String,
}

impl WordDetails {
    /// Whether the lookup came back with nothing worth sending to the card.
    pub fn is_empty(&self) -> bool {
        self.phonetic.is_none()
            && self.meanings.is_empty()
            && self.example.is_none()
            && self.synonyms.is_empty()
            && self.forms.is_empty()
            && self.context.is_none()
    }
}

/// Language pair chosen for one translation. Both sides are optional: an absent
/// (or `auto`) source is detected by the provider, and an absent target follows
/// the settings.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Languages {
    pub source: Option<String>,
    pub target: Option<String>,
}

impl Languages {
    /// Source code to send to the providers; `auto` when nothing was forced.
    pub fn source_code(&self) -> &str {
        match self.source.as_deref().filter(|code| is_concrete(code)) {
            Some(code) => code,
            None => "auto",
        }
    }

    /// Target code chosen by hand, when there is one.
    pub fn target_code(&self) -> Option<&str> {
        self.target.as_deref().filter(|code| is_concrete(code))
    }
}

/// Whether a language code names a language instead of standing for "detect it".
fn is_concrete(code: &str) -> bool {
    let code = code.trim();
    !code.is_empty() && !code.eq_ignore_ascii_case("auto")
}

/// Shared HTTP client, created on first use.
pub fn client() -> Result<&'static reqwest::Client, String> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(REQUEST_TIMEOUT)
                .build()
                .map_err(|error| format!("Could not create the network client: {error}"))
        })
        .as_ref()
        .map_err(|error| error.clone())
}

/// Translates `text` according to `settings`.
///
/// The answer arrives as soon as the provider has it; the word details go
/// through `word_details`, so nothing here waits for the slow public lookups.
pub async fn translate(
    text: &str,
    settings: &Settings,
    languages: &Languages,
) -> Result<TranslationResult, String> {
    let text = text.trim();
    let characters = text.chars().count();
    if characters == 0 {
        return Err("Nothing was selected.".to_string());
    }
    if characters > MAX_TEXT_CHARS {
        return Err(format!(
            "That selection is too long ({characters} characters, limit {MAX_TEXT_CHARS})."
        ));
    }

    let kind = classify::classify(text);
    let client = client()?;
    let source = languages.source_code();
    let (mut target, forced) = resolve_target(text, &settings.target_lang, languages, settings);

    let mut result = call_provider(client, settings, text, source, &target, kind).await?;
    result.source_lang = normalize_lang_code(&result.source_lang);

    // Translating a language into itself just echoes the selection back.
    if !forced
        && same_language(&result.source_lang, &target)
        && resembles(&result.translation, text)
    {
        if let Some(alternate) = alternate_target(&target) {
            if let Ok(retry) = call_provider(client, settings, text, source, &alternate, kind).await
            {
                target = alternate;
                result = retry;
                result.source_lang = normalize_lang_code(&result.source_lang);
            }
        }
    }

    // Phonetic symbols, meanings and an example are looked up separately, by
    // `word_details`, so that the card can show the translation straight away.

    result.kind = kind.as_str().to_string();
    result.target_lang = target;

    // The sentence view only makes sense once the translation is there, and only
    // for text that is long enough to have more than one sentence in it.
    if settings.sentence_pairs && kind == Kind::Sentence && !result.translation.trim().is_empty() {
        result.pairs = crate::context::pairs(text, &result.translation)
            .into_iter()
            .map(|(source, translation)| SentencePair {
                source,
                translation,
            })
            .collect();
    }

    Ok(result)
}

/// The dictionary style extra of a word, looked up on demand after the card
/// already shows the translation.
///
/// Best effort throughout: a selection that is not a word, or a lookup where
/// both sources come back with nothing, answers with an empty result instead of
/// an error, because the card is complete without it.
pub async fn word_details(
    text: &str,
    settings: &Settings,
    languages: &Languages,
) -> Result<WordDetails, String> {
    let text = text.trim();
    if text.is_empty() || classify::classify(text) != Kind::Word {
        return Ok(WordDetails::default());
    }

    let client = client()?;
    let source = languages.source_code();
    // The same target the translation went into, so the dictionary is looked up
    // in the language the card is showing.
    let target = resolve_target(text, &settings.target_lang, languages, settings).0;

    let mut result = TranslationResult::new(Kind::Word, "", text, &target);
    // The dictionary and the free endpoint answer independently, and the free
    // endpoint alone carries an example, so both are asked at once: together
    // they must not take longer than the slower of the two. The free endpoint
    // gets a short budget because on some networks it is simply unreachable,
    // and whatever has not arrived by then is left out.
    // The free endpoint is not asked when it is the service the card is
    // already using: it answered the translation itself, so the extra lookup
    // would only pay for the same result twice.
    let lookup = if settings.service() == Service::Google {
        None
    } else {
        let client = client.clone();
        let word = text.to_string();
        let source = source.to_string();
        Some(tauri::async_runtime::spawn(async move {
            google::translate_quickly(&client, &word, &source, &target, Kind::Word)
                .await
                .ok()
        }))
    };

    dictionary::enrich(client, text, &mut result).await;
    // The dictionary may already carry everything the free endpoint could add;
    // waiting for it then only holds the card back. When something is still
    // missing the card gives the free endpoint a moment, not its whole budget:
    // a card that renders its translation, phonetics and meanings at once is
    // worth more than an example that lands a second later. A lookup that runs
    // on in the background still records that the endpoint is unreachable, so
    // the next card does not pay for it at all.
    let complete =
        result.phonetic.is_some() && !result.meanings.is_empty() && result.example.is_some();
    if let (Some(lookup), false) = (lookup, complete) {
        const GRACE: Duration = Duration::from_millis(600);
        if let Ok(Ok(Some(details))) = tokio::time::timeout(GRACE, lookup).await {
            result.fill_gaps_from(&details);
        }
    }

    drop_the_headword(&mut result.synonyms, text);
    // The forms follow the parts of speech the dictionary gave the word: a noun
    // has a plural, a verb a past tense, and a word the dictionary does not
    // know gets nothing guessed for it.
    let parts: Vec<String> = result
        .meanings
        .iter()
        .map(|meaning| meaning.part_of_speech.clone())
        .collect();
    let forms = crate::morphology::forms(text, &parts);

    Ok(WordDetails {
        phonetic: result.phonetic,
        meanings: result.meanings,
        example: result.example,
        synonyms: result.synonyms,
        forms,
        // Filled in by the command layer, which has the sentence the selection
        // came from.
        context: None,
    })
}

/// Translates the sentence a word was selected from.
///
/// The sentence itself is read out of the program in front by the selection
/// hook, which is the only place that still sees it; this only translates what
/// it brought back. A failure, or a card that was opened from the history
/// rather than from a selection, simply leaves the card without the line.
pub async fn sentence_context(
    settings: &Settings,
    languages: &Languages,
    sentence: Option<String>,
) -> Option<SentenceContext> {
    if !settings.word_sentence {
        return None;
    }
    let text = sentence.filter(|text| !text.trim().is_empty())?;
    let client = client().ok()?;
    let source = languages.source_code();
    let (target, _) = resolve_target(&text, &settings.target_lang, languages, settings);
    let result = call_provider(client, settings, &text, source, &target, Kind::Sentence)
        .await
        .ok()?;
    let translation = result.translation.trim().to_string();
    (!translation.is_empty()).then_some(SentenceContext { text, translation })
}

/// Sends the text to whichever backend the settings name, and to the next one
/// in the fallback order when it fails.
///
/// A public endpoint that rate-limits or drops a request is the normal case
/// rather than an error, so the chosen service is tried first and, unless the
/// fallback is switched off, the others after it. The card names the service
/// that answered; `fallback_from` tells it whether that was the chosen one.
async fn call_provider(
    client: &reqwest::Client,
    settings: &Settings,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    let chosen = settings.service();
    let order = settings.service_order();
    let mut last: Option<String> = None;

    for (index, service) in order.iter().enumerate() {
        match call_service(client, settings, *service, text, source, target, kind).await {
            Ok(mut result) => {
                if *service != chosen {
                    result.fallback_from = Some(chosen.id().to_string());
                }
                return Ok(result);
            }
            Err(error) => {
                if index + 1 < order.len() {
                    eprintln!(
                        "glossy: {} failed ({error}), trying the next service",
                        service.id()
                    );
                }
                last = Some(error);
            }
        }
    }

    // Only the first failure is worth showing: the ones after it are the same
    // problem met again by another service.
    Err(last.unwrap_or_else(|| "No translation service is available.".to_string()))
}

/// Sends the text to one service, whoever it is.
async fn call_service(
    client: &reqwest::Client,
    settings: &Settings,
    service: Service,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    match service {
        Service::Google => google::translate(client, text, source, target, kind).await,
        Service::CloudBaidu | Service::CloudYoudao => {
            let vendor = match service {
                Service::CloudYoudao => "youdao",
                _ => "baidu",
            };
            cloud::translate(
                client,
                cloud::Request {
                    text,
                    source,
                    target,
                    kind,
                    vendor,
                },
                &settings.cloud_endpoint,
                &settings.cloud_id,
            )
            .await
        }
    }
}

/// The target for one translation, paired with a flag telling whether the user
/// picked it by hand. An explicit choice is used as it is; otherwise the
/// settings decide and a selection already in the target language is translated
/// away from it.
///
/// A language the channel does not translate never reaches it: a choice the
/// card kept from another channel falls back on the configured target, and a
/// configured target the channel does not take falls back on the first language
/// it does.
fn resolve_target(
    text: &str,
    configured: &str,
    languages: &Languages,
    settings: &Settings,
) -> (String, bool) {
    let service = settings.service();
    let picked = languages
        .target_code()
        .filter(|code| languages::serves(service, code));

    match picked {
        Some(code) => (code.to_string(), true),
        None => {
            if languages::serves(service, configured) {
                (effective_target(text, configured), false)
            } else {
                (languages::fallback(service).to_string(), false)
            }
        }
    }
}

/// Swaps the target language when the selection is already written in it.
pub fn effective_target(text: &str, target: &str) -> String {
    let lower = target.to_ascii_lowercase();
    // Kana means the text is Japanese, so a Chinese target is still useful.
    let swap = if lower.starts_with("zh") {
        classify::has_cjk(text) && !classify::contains_kana(text) && mostly_han(text)
    } else if lower.starts_with("ja") {
        classify::contains_kana(text)
    } else if lower.starts_with("ko") {
        classify::contains_hangul(text)
    } else {
        false
    };

    if swap {
        "en".to_string()
    } else {
        target.to_string()
    }
}

/// Where to send a selection that is already in the requested language.
fn alternate_target(target: &str) -> Option<String> {
    let lower = target.to_ascii_lowercase();
    if lower.starts_with("en") {
        None
    } else {
        Some("en".to_string())
    }
}

fn mostly_han(text: &str) -> bool {
    let mut han = 0usize;
    let mut total = 0usize;
    for c in text.chars() {
        if c.is_whitespace() || c.is_ascii_punctuation() {
            continue;
        }
        total += 1;
        if matches!(c as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF) {
            han += 1;
        }
    }
    total > 0 && han * 2 >= total
}

fn same_language(a: &str, b: &str) -> bool {
    let primary = |value: &str| {
        value
            .split(['-', '_'])
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase()
    };
    let (a, b) = (primary(a), primary(b));
    !a.is_empty() && a == b
}

/// Maps a language code a provider reported onto the codes the rest of Glossy
/// speaks.
///
/// Baidu answers with `jp`, `fra` or `cht` and DeepL with `EN` or `ZH-HANS`.
/// Left alone those codes end up in the language menus of the popup, where no
/// entry matches them, and - worse - in the next request as a source or a
/// target, which the other providers reject or silently ignore.
pub fn normalize_lang_code(code: &str) -> String {
    let lower = code.trim().to_ascii_lowercase().replace('_', "-");
    if lower.is_empty() {
        return String::new();
    }
    let base = lower.split('-').next().unwrap_or("");

    if base == "zh" || base == "cht" {
        return if base == "cht"
            || lower.contains("tw")
            || lower.contains("hk")
            || lower.contains("hant")
        {
            "zh-TW".to_string()
        } else {
            "zh-CN".to_string()
        };
    }

    match base {
        "ja" | "jp" | "jpn" => "ja",
        "ko" | "kor" => "ko",
        "fr" | "fra" | "fre" => "fr",
        "de" | "deu" | "ger" => "de",
        "es" | "spa" => "es",
        "pt" | "por" => "pt",
        "it" | "ita" => "it",
        "ru" | "rus" => "ru",
        "uk" | "ukr" => "uk",
        "vi" | "vie" => "vi",
        "ms" | "may" | "msa" => "ms",
        "da" | "dan" => "da",
        "fi" | "fin" => "fi",
        "he" | "heb" | "iw" => "he",
        "no" | "nor" | "nb" | "nn" => "no",
        "ro" | "ron" | "rom" => "ro",
        "sv" | "swe" => "sv",
        "ar" | "ara" => "ar",
        "cs" | "ces" | "cze" | "cz" => "cs",
        "el" | "ell" | "gre" => "el",
        "hu" | "hun" => "hu",
        "sk" | "slk" | "slo" => "sk",
        "pl" | "pol" => "pl",
        "nl" | "nld" | "dut" => "nl",
        "tr" | "tur" => "tr",
        "th" | "tha" => "th",
        "hi" | "hin" => "hi",
        "id" | "ind" => "id",
        other => return other.to_string(),
    }
    .to_string()
}

fn resembles(a: &str, b: &str) -> bool {
    let normalize = |value: &str| {
        value
            .chars()
            .filter(|c| !c.is_whitespace())
            .flat_map(|c| c.to_lowercase())
            .collect::<String>()
    };
    let (a, b) = (normalize(a), normalize(b));
    !a.is_empty() && (a == b || a.contains(&b) || b.contains(&a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_target_for_foreign_selections() {
        assert_eq!(effective_target("hello", "zh-CN"), "zh-CN");
        assert_eq!(effective_target("你好", "en"), "en");
        assert_eq!(effective_target("ありがとう", "zh-CN"), "zh-CN");
    }

    #[test]
    fn swaps_the_target_for_selections_already_in_it() {
        assert_eq!(effective_target("你好世界", "zh-CN"), "en");
        assert_eq!(effective_target("ありがとう", "ja"), "en");
        assert_eq!(effective_target("감사합니다", "ko"), "en");
    }

    #[test]
    fn language_comparison_ignores_the_region() {
        assert!(same_language("zh-CN", "zh"));
        assert!(same_language("EN", "en-US"));
        assert!(same_language("zh-CN", "zh-TW"));
        assert!(!same_language("en", "ja"));
    }

    #[test]
    fn provider_private_codes_are_mapped_onto_the_shared_ones() {
        // Baidu
        assert_eq!(normalize_lang_code("jp"), "ja");
        assert_eq!(normalize_lang_code("kor"), "ko");
        assert_eq!(normalize_lang_code("fra"), "fr");
        assert_eq!(normalize_lang_code("spa"), "es");
        assert_eq!(normalize_lang_code("zh"), "zh-CN");
        assert_eq!(normalize_lang_code("cht"), "zh-TW");
        // DeepL
        assert_eq!(normalize_lang_code("EN"), "en");
        assert_eq!(normalize_lang_code("ZH-HANS"), "zh-CN");
        assert_eq!(normalize_lang_code("ZH-HANT"), "zh-TW");
        assert_eq!(normalize_lang_code("PT-BR"), "pt");
        // Anything else keeps the spelling the provider chose.
        assert_eq!(normalize_lang_code("  EN "), "en");
        assert_eq!(normalize_lang_code("zh_TW"), "zh-TW");
        assert_eq!(normalize_lang_code("fil"), "fil");
        assert_eq!(normalize_lang_code(""), "");
    }

    #[test]
    fn detects_echoed_translations() {
        assert!(resembles("Hello world", "hello  world"));
        assert!(!resembles("你好", "hello"));
    }

    #[test]
    fn no_alternate_target_for_english() {
        assert_eq!(alternate_target("en"), None);
        assert_eq!(alternate_target("zh-TW"), Some("en".to_string()));
    }

    #[test]
    fn an_absent_language_falls_back_to_auto_and_the_settings() {
        let none = Languages::default();
        assert_eq!(none.source_code(), "auto");
        assert_eq!(none.target_code(), None);
        let settings = Settings::default();
        assert_eq!(
            resolve_target("你好世界", "zh-CN", &none, &settings),
            ("en".to_string(), false)
        );

        let auto = Languages {
            source: Some("Auto".to_string()),
            target: Some("".to_string()),
        };
        assert_eq!(auto.source_code(), "auto");
        assert_eq!(auto.target_code(), None);
        assert_eq!(
            resolve_target("hello", "zh-CN", &auto, &settings).0,
            "zh-CN"
        );
    }

    /// The two languages a channel does not take, kept from a channel that did.
    #[test]
    fn an_unserved_language_never_reaches_the_provider() {
        let baidu = Settings::default();
        assert_eq!(baidu.service(), Service::CloudBaidu);

        // A language the card kept from another channel falls back on the
        // configured target, and is still unforced, so the away-from-it
        // handling is left to run.
        let kept = Languages {
            source: Some("auto".to_string()),
            target: Some("tr".to_string()),
        };
        assert_eq!(
            resolve_target("hello", "fr", &kept, &baidu),
            ("fr".to_string(), false)
        );
        assert_eq!(
            resolve_target("你好", "zh-CN", &kept, &baidu),
            ("en".to_string(), false)
        );
        // A configured target the channel does not take falls back on the first
        // language it does, rather than being sent and refused.
        assert_eq!(
            resolve_target("hello", "tr", &Languages::default(), &baidu),
            ("en".to_string(), false)
        );
        // A channel that does take it is left exactly as it was.
        let mut google = Settings::default();
        google.set_service(Service::Google);
        assert_eq!(google.service(), Service::Google);
        assert_eq!(
            resolve_target("hello", "tr", &kept, &google),
            ("tr".to_string(), true)
        );
    }

    #[test]
    fn a_chosen_language_overrides_detection_and_the_settings() {
        let forced = Languages {
            source: Some("ja".to_string()),
            target: Some("fr".to_string()),
        };
        assert_eq!(forced.source_code(), "ja");
        assert_eq!(forced.target_code(), Some("fr"));
        // Even a selection that is already French keeps the chosen target.
        assert_eq!(
            resolve_target("bonjour", "fr", &forced, &Settings::default()),
            ("fr".to_string(), true)
        );
    }
}
