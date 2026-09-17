//! Baidu Translate provider. Requires an APP ID together with a secret key;
//! the endpoint only returns a plain translation, so word details are filled in
//! by the orchestrator.

use std::time::{SystemTime, UNIX_EPOCH};

use md5::{Digest, Md5};

use crate::classify::Kind;

use super::TranslationResult;

const ENDPOINT: &str = "https://fanyi-api.baidu.com/api/trans/vip/translate";

/// Baidu spells most languages with three letters and a few with its own
/// abbreviations, so the codes have to be mapped instead of uppercased.
pub fn baidu_target(target: &str) -> String {
    let lower = target.to_ascii_lowercase();
    let base = lower.split(['-', '_']).next().unwrap_or("");
    match (base, lower.as_str()) {
        ("zh", _) if lower.contains("tw") || lower.contains("hk") => "cht".to_string(),
        ("zh", _) => "zh".to_string(),
        ("ja", _) => "jp".to_string(),
        ("ko", _) => "kor".to_string(),
        ("fr", _) => "fra".to_string(),
        ("es", _) => "spa".to_string(),
        ("ar", _) => "ara".to_string(),
        ("uk", _) => "ukr".to_string(),
        ("vi", _) => "vie".to_string(),
        ("ms", _) => "may".to_string(),
        ("da", _) => "dan".to_string(),
        ("fi", _) => "fin".to_string(),
        ("he", _) => "heb".to_string(),
        ("no", _) => "nor".to_string(),
        ("ro", _) => "rom".to_string(),
        ("sv", _) => "swe".to_string(),
        ("", _) => "en".to_string(),
        _ => base.to_string(),
    }
}

pub async fn translate(
    client: &reqwest::Client,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
    app_id: &str,
    api_key: &str,
) -> Result<TranslationResult, String> {
    let app_id = app_id.trim();
    let key = api_key.trim();
    if app_id.is_empty() || key.is_empty() {
        return Err(
            "Baidu needs both an APP ID and a key. Add them in Settings, or switch to another provider."
                .to_string(),
        );
    }

    let source_language = if source == "auto" {
        "auto".to_string()
    } else {
        baidu_target(source)
    };
    let target_language = baidu_target(target);
    let salt = salt();
    let sign = sign(app_id, text, &salt, key);

    let response = client
        .post(ENDPOINT)
        .form(&[
            ("q", text),
            ("from", source_language.as_str()),
            ("to", target_language.as_str()),
            ("appid", app_id),
            ("salt", salt.as_str()),
            ("sign", sign.as_str()),
        ])
        .send()
        .await
        .map_err(|error| format!("Could not reach Baidu: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Could not read the Baidu response: {error}"))?;

    if !status.is_success() {
        return Err(format!("Baidu returned HTTP {status}.{}", detail(&body)));
    }

    parse(&body, "baidu", text, target, kind)
}

/// Reads a `POST /api/trans/vip/translate` body. Baidu reports failures as HTTP
/// 200 with an `error_code`, so the payload decides whether the call worked.
pub fn parse(
    body: &str,
    provider: &str,
    text: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    let data: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "Baidu sent a response Glossy could not read.".to_string())?;

    if let Some(code) = data.get("error_code").and_then(|value| value.as_str()) {
        return Err(error_message(code, body));
    }

    // Long selections come back split into several parts that have to be
    // stitched back together.
    let translation = data
        .get("trans_result")
        .and_then(|value| value.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get("dst").and_then(|value| value.as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    if translation.trim().is_empty() {
        return Err("Baidu returned an empty translation.".to_string());
    }

    let mut result = TranslationResult::new(kind, provider, text, target);
    result.translation = translation;
    if let Some(language) = data.get("from").and_then(|value| value.as_str()) {
        if !language.eq_ignore_ascii_case("auto") {
            result.source_lang = language.to_ascii_lowercase();
        }
    }
    Ok(result)
}

/// Baidu only requires the salt to differ between requests.
fn salt() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    nanos.to_string()
}

/// `sign = md5(appid + q + salt + key)`, with no separators.
fn sign(app_id: &str, text: &str, salt: &str, key: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(app_id.as_bytes());
    hasher.update(text.as_bytes());
    hasher.update(salt.as_bytes());
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn error_message(code: &str, body: &str) -> String {
    let message = match code {
        "52001" => "Baidu timed out on that request. Try again in a moment.",
        "52002" => "Baidu had an internal error. Try again in a moment.",
        "52003" => "Baidu rejected the APP ID. Check it in Settings.",
        "54000" => "Baidu rejected the request because a field was missing.",
        "54001" => "Baidu rejected the signature. Check the key in Settings.",
        "54003" => "Baidu is limiting how often this account may translate. Wait a moment.",
        "54004" => "The Baidu account has no quota left. Check the free monthly quota in the Baidu console.",
        "54005" => "Baidu is limiting long requests from this account. Wait a few seconds.",
        "58000" => {
            "Baidu rejected this computer's IP address. Check the IP whitelist in the Baidu console."
        }
        "58001" => "Baidu does not support that language pair.",
        "58002" => "Baidu has text translation switched off for this account. Enable 通用文本翻译 in the Baidu console.",
        "90107" => "Baidu says this account is not verified yet. Finish the 认证 step in the Baidu console.",
        _ => "Baidu could not translate that selection.",
    };
    format!("{message} (error {code}){}", detail(body))
}

/// Surfaces Baidu's own explanation, which is usually more specific than the code.
fn detail(body: &str) -> String {
    let message = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|data| {
            data.get("error_msg")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if message.is_empty() {
        String::new()
    } else {
        format!(" {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_language_codes_to_the_baidu_spelling() {
        assert_eq!(baidu_target("zh-CN"), "zh");
        assert_eq!(baidu_target("zh-TW"), "cht");
        assert_eq!(baidu_target("ja"), "jp");
        assert_eq!(baidu_target("ko"), "kor");
        assert_eq!(baidu_target("fr"), "fra");
        assert_eq!(baidu_target("es"), "spa");
        assert_eq!(baidu_target("vi"), "vie");
        assert_eq!(baidu_target("ms"), "may");
        assert_eq!(baidu_target("en"), "en");
        assert_eq!(baidu_target("de"), "de");
        assert_eq!(baidu_target(""), "en");
    }

    #[test]
    fn signs_the_request_the_way_the_documentation_example_does() {
        // appid 2015063000000001, q apple, salt 1435660288, key 12345678.
        assert_eq!(
            sign("2015063000000001", "apple", "1435660288", "12345678"),
            "f89f9594663708c1605f3d736d01d2d4"
        );
    }

    #[test]
    fn reads_the_translation_and_the_detected_language() {
        let body = r#"{"from":"en","to":"zh","trans_result":[{"src":"apple","dst":"苹果"}]}"#;
        let result = parse(body, "baidu", "apple", "zh-CN", Kind::Word).expect("parsed");

        assert_eq!(result.translation, "苹果");
        assert_eq!(result.source_lang, "en");
        assert_eq!(result.provider, "baidu");
        assert_eq!(result.kind, "word");
    }

    #[test]
    fn joins_a_long_selection_that_baidu_split_into_parts() {
        let body = r#"{"trans_result":[{"dst":"第一段"},{"dst":"第二段"}]}"#;
        let result = parse(body, "baidu", "one two", "zh-CN", Kind::Sentence).expect("parsed");

        assert_eq!(result.translation, "第一段\n第二段");
    }

    #[test]
    fn explains_the_error_code_instead_of_showing_the_raw_body() {
        let body = r#"{"error_code":"52003","error_msg":"UNAUTHORIZED USER"}"#;
        let error = parse(body, "baidu", "apple", "zh-CN", Kind::Word).expect_err("rejected");

        assert!(error.contains("rejected the APP ID"), "{error}");
        assert!(error.contains("52003"), "{error}");
    }
}
