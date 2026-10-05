//! One translation channel for every service that speaks the OpenAI chat
//! completions protocol.
//!
//! OpenAI, DeepSeek, Zhipu, Moonshot, a model server on this machine: they all
//! take the same request, so they are one entry in the dropdown whose address,
//! key and model the reader filled in. The model is asked for a JSON object
//! rather than for prose, because a card is not a paragraph: a word gets its
//! phonetics and meanings out of the same answer that carries the translation.
//!
//! `response_format` is deliberately not sent: it is an OpenAI extension, not
//! part of the protocol every compatible server implements, and a prompt that
//! asks for JSON plus a parser that digs the object out of a Markdown fence is
//! enough without it.

use std::time::Duration;

use crate::classify::Kind;
use crate::settings::ChatApi;

use super::{Failure, Meaning, TranslationResult};

/// A model writing a paragraph takes longer than a dictionary lookup, and an
/// entry the reader pointed at a server of their own may be slow to answer.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(25);

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
    api: &ChatApi,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, Failure> {
    let key = api.api_key.trim();
    let model = api.model.trim();
    if key.is_empty() || model.is_empty() {
        return Err(Failure::plain(
            "This channel needs an API key and a model name. Add them on the General page, or \
             choose another channel."
                .to_string(),
        ));
    }
    let endpoint = api.endpoint.trim();
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err(Failure::plain(
            "The API address has to start with `https://` (or `http://` for a server on this \
             machine). Set it on the General page."
                .to_string(),
        ));
    }

    let payload = serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "messages": [
            {
                "role": "system",
                "content": "You are a translation engine that only ever replies with JSON."
            },
            { "role": "user", "content": prompt(text, source, target, kind) }
        ]
    });

    let response = client
        .post(api.completion_url())
        .timeout(REQUEST_TIMEOUT)
        .bearer_auth(key)
        .json(&payload)
        .send()
        .await
        .map_err(|error| {
            Failure::plain(format!(
                "Could not reach the API at {}: {error}",
                api.completion_url()
            ))
        })?;

    let status = response.status();
    let body = response.text().await.map_err(|error| {
        Failure::plain(format!("Could not read the answer from the API: {error}"))
    })?;

    if !status.is_success() {
        let detail = error_detail(&body);
        return Err(Failure::plain(match status.as_u16() {
            401 | 403 => format!("The API rejected the key. Check it on the General page.{detail}"),
            404 => format!(
                "The API has no completion endpoint at {}. Check the address on the General \
                 page.{detail}",
                api.completion_url()
            ),
            429 => format!("The API is rate limiting requests or the quota is used up.{detail}"),
            _ => format!("The API returned HTTP {status}.{detail}"),
        }));
    }

    let data: serde_json::Value = serde_json::from_str(&body).map_err(|_| {
        Failure::plain("The API sent a response Glossy could not read.".to_string())
    })?;
    let content = data
        .get("choices")
        .and_then(|value| value.get(0))
        .and_then(|value| value.get("message"))
        .and_then(|value| value.get("content"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| Failure::plain("The API returned no translation.".to_string()))?;

    parse(content, "api-openai", text, target, kind)
}

pub fn parse(
    content: &str,
    provider: &str,
    text: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, Failure> {
    let json = extract_json(content);

    let data: serde_json::Value = serde_json::from_str(json)
        .map_err(|_| Failure::plain("The model did not answer with JSON.".to_string()))?;

    let translation = data
        .get("translation")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    if translation.is_empty() {
        return Err(Failure::plain(
            "The model returned an empty translation.".to_string(),
        ));
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
            parse(content, "api-openai", "running", "zh-CN", Kind::Word).expect("answer parses");
        assert_eq!(result.translation, "跑步");
        assert_eq!(result.source_lang, "en");
        assert_eq!(result.provider, "api-openai");
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
        let result = parse(
            content,
            "api-openai",
            "Hello world",
            "zh-CN",
            Kind::Sentence,
        )
        .expect("answer parses");
        assert_eq!(result.translation, "你好世界");
        assert_eq!(result.provider, "api-openai");
        assert!(result.meanings.is_empty());
    }

    #[test]
    fn finds_the_object_inside_a_sentence() {
        let content = "Sure! Here it is: {\"translation\":\"你好\"} — anything else?";
        let result =
            parse(content, "api-openai", "hi", "zh-CN", Kind::Sentence).expect("answer parses");

        assert_eq!(result.translation, "你好");
    }

    #[test]
    fn refuses_a_model_that_answered_with_prose() {
        let error = parse(
            "I cannot help with that.",
            "api-openai",
            "hi",
            "zh-CN",
            Kind::Sentence,
        )
        .expect_err("rejected");

        assert!(error.to_string().contains("JSON"), "{error}");
    }

    #[test]
    fn refuses_an_empty_translation() {
        let error = parse(
            r#"{"translation":"  "}"#,
            "api-openai",
            "hi",
            "zh-CN",
            Kind::Sentence,
        )
        .expect_err("rejected");

        assert!(error.to_string().contains("empty"), "{error}");
    }

    #[test]
    fn the_completion_path_is_added_to_the_address_the_reader_named() {
        let api = ChatApi {
            endpoint: "https://api.deepseek.com/v1/".to_string(),
            api_key: "sk-x".to_string(),
            model: "deepseek-chat".to_string(),
        };

        assert_eq!(
            api.completion_url(),
            "https://api.deepseek.com/v1/chat/completions"
        );
    }
}
