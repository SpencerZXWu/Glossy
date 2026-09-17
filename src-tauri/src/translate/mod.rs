//! Translation providers and the orchestration that decides what to show.

mod baidu;
mod chat;
mod deepl;
mod dictionary;
mod google;

use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::classify::{self, Kind};
use crate::settings::{Provider, Settings};

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

#[derive(Debug, Clone, Serialize)]
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
            self.source_lang = other.source_lang.clone();
        }
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

/// Translates `text` according to `settings`, filling in word details when possible.
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

    // Translating a language into itself just echoes the selection back.
    if !forced && same_language(&result.source_lang, &target) && resembles(&result.translation, text)
    {
        if let Some(alternate) = alternate_target(&target) {
            if let Ok(retry) = call_provider(client, settings, text, source, &alternate, kind).await
            {
                target = alternate;
                result = retry;
            }
        }
    }

    if kind == Kind::Word {
        // The local dictionary is free and fast, so try it before spending a
        // request on the free endpoint.
        dictionary::enrich(client, text, &mut result).await;

        let incomplete = result.phonetic.is_none()
            || result.meanings.is_empty()
            || result.example.is_none();
        if incomplete && settings.provider != Provider::Google {
            if let Ok(details) = google::translate(client, text, source, &target, kind).await {
                result.fill_gaps_from(&details);
            }
        }
    }

    result.kind = kind.as_str().to_string();
    result.target_lang = target;
    Ok(result)
}

async fn call_provider(
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
                &chat::ZHIPU,
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
        Provider::OpenAI => {
            chat::translate(
                client,
                &chat::OPENAI,
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
        assert_eq!(resolve_target("你好世界", "zh-CN", &none), ("en".to_string(), false));

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
        assert_eq!(resolve_target("bonjour", "fr", &forced), ("fr".to_string(), true));
    }
}
