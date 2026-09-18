//! OpenAI compatible chat completion providers (OpenAI, Zhipu GLM), used as the
//! key based alternative to the free endpoint.

use std::time::Duration;

use crate::classify::Kind;

use super::{Meaning, TranslationResult};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(25);

/// One OpenAI compatible chat completion backend.
pub struct Chat {
    /// Identifier reported to the user interface.
    pub id: &'static str,
    /// Service name used in error messages.
    pub label: &'static str,
    /// Full chat completions URL.
    pub endpoint: &'static str,
    pub model: &'static str,
    /// Whether the service accepts `response_format`. Where it does not, the
    /// prompt alone has to keep the answer JSON shaped.
    pub json_mode: bool,
}

pub const OPENAI: Chat = Chat {
    id: "openai",
    label: "OpenAI",
    endpoint: "https://api.openai.com/v1/chat/completions",
    model: "gpt-4o-mini",
    json_mode: true,
};

/// Zhipu's free text tier, which is reachable from mainland China.
pub const ZHIPU: Chat = Chat {
    id: "zhipu",
    label: "Zhipu",
    endpoint: "https://open.bigmodel.cn/api/paas/v4/chat/completions",
    model: "glm-4.7-flash",
    json_mode: false,
};

fn prompt(text: &str, source: &str, target: &str, kind: Kind) -> String {
    let known_source = if source == "auto" {
        String::new()
    } else {
        format!("The text is in {source}. ")
    };
    let mut prompt = format!(
        "Translate the text between the triple quotes into {target}. \
         Detect the language of the text. Answer with a single JSON object and nothing else.\n\
         {known_source}Text: \"\"\"{text}\"\"\"\n\
         JSON keys: \"sourceLang\" (BCP-47 code), \"translation\" (the translation)."
    );
    if kind == Kind::Word {
        prompt.push_str(
            ",\n\"phonetic\" (pronunciation of the original text), \
             \"meanings\" (array of {\"partOfSpeech\", \"definitions\" (array of strings)}), \
             \"example\" (one short sentence in the original language using the text).",
        );
    }
    prompt
}

pub async fn translate(
    client: &reqwest::Client,
    chat: &Chat,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
    api_key: &str,
) -> Result<TranslationResult, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err(format!(
            "{} needs an API key. Add one in Settings, or switch back to the free Google provider.",
            chat.label
        ));
    }

    let mut payload = serde_json::json!({
        "model": chat.model,
        "temperature": 0.2,
        "messages": [
            {
                "role": "system",
                "content": "You are a translation engine that only ever replies with JSON."
            },
            { "role": "user", "content": prompt(text, source, target, kind) }
        ]
    });
    if chat.json_mode {
        payload["response_format"] = serde_json::json!({ "type": "json_object" });
    }

    let response = client
        .post(chat.endpoint)
        .timeout(REQUEST_TIMEOUT)
        .bearer_auth(key)
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Could not reach {}: {error}", chat.label))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Could not read the {} response: {error}", chat.label))?;

    if !status.is_success() {
        let detail = error_detail(&body);
        return Err(match status.as_u16() {
            401 => format!(
                "{} rejected the API key. Check it in Settings.{detail}",
                chat.label
            ),
            429 => format!(
                "{} is rate limiting requests or the quota is used up.{detail}",
                chat.label
            ),
            _ => format!("{} returned HTTP {status}.{detail}", chat.label),
        });
    }

    let data: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| format!("{} sent a response Glossy could not read.", chat.label))?;
    let content = data
        .get("choices")
        .and_then(|value| value.get(0))
        .and_then(|value| value.get("message"))
        .and_then(|value| value.get("content"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("{} returned no translation.", chat.label))?;

    parse(content, chat.id, text, target, kind)
}

pub fn parse(
    content: &str,
    provider: &str,
    text: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    let json = extract_json(content);

    let data: serde_json::Value = serde_json::from_str(json)
        .map_err(|_| "The model did not answer with JSON.".to_string())?;

    let translation = data
        .get("translation")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    if translation.is_empty() {
        return Err("The model returned an empty translation.".to_string());
    }

    let mut result = TranslationResult::new(kind, provider, text, target);
    result.translation = translation;
    if let Some(language) = data.get("sourceLang").and_then(|value| value.as_str()) {
        if !language.is_empty() {
            result.source_lang = language.to_string();
        }
    }
    result.phonetic = data
        .get("phonetic")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    result.example = data
        .get("example")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if let Some(meanings) = data.get("meanings").and_then(|value| value.as_array()) {
        for entry in meanings.iter().take(4) {
            let definitions: Vec<String> = entry
                .get("definitions")
                .and_then(|value| value.as_array())
                .map(|list| {
                    list.iter()
                        .filter_map(|value| value.as_str())
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .take(4)
                        .collect()
                })
                .unwrap_or_default();
            if definitions.is_empty() {
                continue;
            }
            result.meanings.push(Meaning {
                part_of_speech: entry
                    .get("partOfSpeech")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
                definitions,
            });
        }
    }

    Ok(result)
}

/// Services report the real reason (unknown model, bad key format) in the body,
/// so the first line of it is worth showing next to the status code.
fn error_detail(body: &str) -> String {
    let Ok(data) = serde_json::from_str::<serde_json::Value>(body) else {
        return String::new();
    };
    let message = data
        .get("error")
        .map(|error| {
            error
                .get("message")
                .and_then(|value| value.as_str())
                .or_else(|| error.as_str())
                .unwrap_or_default()
        })
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if message.is_empty() {
        return String::new();
    }
    format!(" {}", message.chars().take(200).collect::<String>())
}

/// Models occasionally wrap the object in a Markdown fence or in a sentence.
fn extract_json(content: &str) -> &str {
    let trimmed = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    match (trimmed.find('{'), trimmed.rfind('}')) {
        (Some(start), Some(end)) if end > start => &trimmed[start..=end],
        _ => trimmed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_word_answer() {
        let content = r#"```json
        {
          "sourceLang": "en",
          "translation": "跑步",
          "phonetic": "ˈrəniNG",
          "meanings": [{ "partOfSpeech": "noun", "definitions": ["赛跑", "奔跑"] }],
          "example": "I went running this morning."
        }
        ```"#;

        let result =
            parse(content, "openai", "running", "zh-CN", Kind::Word).expect("answer parses");
        assert_eq!(result.translation, "跑步");
        assert_eq!(result.source_lang, "en");
        assert_eq!(result.provider, "openai");
        assert_eq!(result.phonetic.as_deref(), Some("ˈrəniNG"));
        assert_eq!(result.meanings[0].definitions.len(), 2);
        assert_eq!(
            result.example.as_deref(),
            Some("I went running this morning.")
        );
    }

    #[test]
    fn parses_a_sentence_answer() {
        let content = r#"{"sourceLang":"en","translation":"你好世界"}"#;
        let result =
            parse(content, "zhipu", "Hello world", "zh-CN", Kind::Sentence).expect("answer parses");
        assert_eq!(result.translation, "你好世界");
        assert_eq!(result.provider, "zhipu");
        assert!(result.meanings.is_empty());
    }

    #[test]
    fn parses_an_answer_wrapped_in_prose() {
        let content = "Sure, here it is: {\"translation\":\"你好\"} — hope that helps.";
        let result =
            parse(content, "zhipu", "Hello", "zh-CN", Kind::Sentence).expect("answer parses");
        assert_eq!(result.translation, "你好");
    }

    #[test]
    fn shows_the_reason_a_request_was_rejected() {
        let body = r#"{"error":{"code":"1002","message":"Authorization Token 非法"}}"#;
        assert_eq!(error_detail(body), " Authorization Token 非法");
        assert_eq!(error_detail("<html>nope</html>"), "");
    }

    #[test]
    fn rejects_an_answer_without_a_translation() {
        assert!(parse(
            r#"{"sourceLang":"en"}"#,
            "openai",
            "hello",
            "zh-CN",
            Kind::Word
        )
        .is_err());
        assert!(parse("not json at all", "openai", "hello", "zh-CN", Kind::Word).is_err());
    }
}
