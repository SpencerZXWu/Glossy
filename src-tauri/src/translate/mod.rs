//! Translation providers and the orchestration that decides what to show.

mod baidu;
mod chat;
mod cloud;
mod deepl;
mod dictionary;
mod google;
mod local;

use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::classify::{self, Kind};
use crate::settings::{Channel, CloudProvider, Provider, Settings};

pub use self::cloud::{new_install_id, quota as cloud_quota, Quota as CloudQuota};

/// The free public endpoints reject requests without a browser like agent.
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// Longer selections are rejected before hitting the network.
pub const MAX_TEXT_CHARS: usize = 1800;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meaning {
    pub part_of_speech: String,
    pub definitions: Vec<String>,
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
        }
        if self.example.is_none() {
            self.example = other.example.clone();
        }
        if self.source_lang == "auto" && other.source_lang != "auto" {
            self.source_lang = normalize_lang_code(&other.source_lang);
        }
    }
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
}

impl WordDetails {
    /// Whether the lookup came back with nothing worth sending to the card.
    pub fn is_empty(&self) -> bool {
        self.phonetic.is_none() && self.meanings.is_empty() && self.example.is_none()
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
    let (mut target, forced) = resolve_target(text, &settings.target_lang, languages);

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
    let target = match languages.target_code() {
        Some(code) => code.to_string(),
        None => effective_target(text, &settings.target_lang),
    };

    let mut result = TranslationResult::new(Kind::Word, "", text, &target);

    // The dictionary and the free endpoint answer independently, and the free
    // endpoint alone carries an example, so both are asked at once: together
    // they must not take longer than the slower of the two. The free endpoint
    // gets a short budget because on some networks it is simply unreachable,
    // and whatever has not arrived by then is left out.
    let lookup = if settings.uses_google_api() {
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

    Ok(WordDetails {
        phonetic: result.phonetic,
        meanings: result.meanings,
        example: result.example,
    })
}

/// Sends the text to whichever backend the settings name.
///
/// The cloud channel is Glossy's own: either the built in service the app
/// ships with, or a model on this machine - neither asks the user for a key.
/// The api channel is the user's own account with a provider.
async fn call_provider(
    client: &reqwest::Client,
    settings: &Settings,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    if settings.channel == Channel::Cloud {
        return match settings.cloud_provider {
            CloudProvider::Builtin => {
                cloud::translate(
                    client,
                    cloud::Request {
                        text,
                        source,
                        target,
                        kind,
                        vendor: &settings.cloud_vendor,
                    },
                    &settings.cloud_endpoint,
                    &settings.cloud_id,
                )
                .await
            }
            CloudProvider::Local => {
                local::translate(
                    client,
                    &settings.local_endpoint,
                    &settings.local_model,
                    text,
                    source,
                    target,
                    kind,
                )
                .await
            }
        };
    }

    call_api_provider(client, settings, text, source, target, kind).await
}

async fn call_api_provider(
    client: &reqwest::Client,
    settings: &Settings,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    let credentials = settings.active_credentials();
    match settings.provider {
        Provider::Google => google::translate(client, text, source, target, kind).await,
        Provider::Zhipu => {
            chat::translate(
                client,
                &chat::zhipu(),
                text,
                source,
                target,
                kind,
                &credentials.api_key,
            )
            .await
        }
        Provider::Baidu => {
            baidu::translate(
                client,
                text,
                source,
                target,
                kind,
                &credentials.app_id,
                &credentials.api_key,
            )
            .await
        }
        Provider::DeepL => {
            deepl::translate(client, text, source, target, kind, &credentials.api_key).await
        }
        // Kept so a settings file written before the channels existed, one
        // naming `cloud` as its provider, still translates. The cloud channel
        // handles that case long before here.
        Provider::Cloud => {
            cloud::translate(
                client,
                cloud::Request {
                    text,
                    source,
                    target,
                    kind,
                    vendor: &settings.cloud_vendor,
                },
                &settings.cloud_endpoint,
                &settings.cloud_id,
            )
            .await
        }
        Provider::OpenAI => {
            chat::translate(
                client,
                &chat::openai(),
                text,
                source,
                target,
                kind,
                &credentials.api_key,
            )
            .await
        }
    }
}

/// The target for one translation, paired with a flag telling whether the user
/// picked it by hand. An explicit choice is used as it is; otherwise the
/// settings decide and a selection already in the target language is translated
/// away from it.
fn resolve_target(text: &str, configured: &str, languages: &Languages) -> (String, bool) {
    match languages.target_code() {
        Some(code) => (code.to_string(), true),
        None => (effective_target(text, configured), false),
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
    fn a_normalized_code_is_understood_by_every_provider() {
        // The popup feeds the detected language back in when the user swaps the
        // languages, so a private code of one provider must never reach another
        // one: it goes through the shared spelling first.
        let detected = normalize_lang_code("cht");
        assert_eq!(detected, "zh-TW");
        assert_eq!(baidu::baidu_target(&detected), "cht");
        assert_eq!(deepl::deepl_target(&detected), "ZH-HANT");

        let detected = normalize_lang_code("kor");
        assert_eq!(detected, "ko");
        assert_eq!(baidu::baidu_target(&detected), "kor");
        assert_eq!(deepl::deepl_target(&detected), "KO");
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
        assert_eq!(
            resolve_target("你好世界", "zh-CN", &none),
            ("en".to_string(), false)
        );

        let auto = Languages {
            source: Some("Auto".to_string()),
            target: Some("".to_string()),
        };
        assert_eq!(auto.source_code(), "auto");
        assert_eq!(auto.target_code(), None);
        assert_eq!(resolve_target("hello", "zh-CN", &auto).0, "zh-CN");
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
            resolve_target("bonjour", "fr", &forced),
            ("fr".to_string(), true)
        );
    }
}
