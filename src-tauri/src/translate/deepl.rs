//! DeepL provider. Requires an API key; dictionary details are filled in by the
//! orchestrator because DeepL only returns plain translations.

use crate::classify::Kind;

use super::TranslationResult;

const FREE_ENDPOINT: &str = "https://api-free.deepl.com/v2/translate";
const PRO_ENDPOINT: &str = "https://api.deepl.com/v2/translate";

/// DeepL uses its own uppercase language codes.
pub fn deepl_target(target: &str) -> String {
    let lower = target.to_ascii_lowercase();
    let base = lower.split(['-', '_']).next().unwrap_or("");
    match (base, lower.as_str()) {
        ("zh", _) if lower.contains("tw") || lower.contains("hk") => "ZH-HANT".to_string(),
        ("zh", _) => "ZH".to_string(),
        ("en", _) => "EN-US".to_string(),
        ("pt", _) => "PT-BR".to_string(),
        ("", _) => "EN-US".to_string(),
        _ => base.to_ascii_uppercase(),
    }
}

/// DeepL rejects the region on `source_lang` (`EN-US` is only valid as a
/// target), so the source code is reduced to its base language.
pub fn deepl_source(source: &str) -> String {
    let lower = source.to_ascii_lowercase();
    let base = lower.split(['-', '_']).next().unwrap_or("");
    if base.is_empty() {
        "EN".to_string()
    } else {
        base.to_ascii_uppercase()
    }
}

pub async fn translate(
    client: &reqwest::Client,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
    api_key: &str,
) -> Result<TranslationResult, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err(
            "DeepL needs an API key. Add one in Settings, or switch back to the free Google provider."
                .to_string(),
        );
    }

    let endpoint = if key.ends_with(":fx") {
        FREE_ENDPOINT
    } else {
        PRO_ENDPOINT
    };
    let target_language = deepl_target(target);
    let source_language = deepl_source(source);

    let mut fields = vec![("text", text), ("target_lang", target_language.as_str())];
    // DeepL errors out when the two sides name the same language.
    if source != "auto" && source_language != deepl_source(&target_language) {
        fields.push(("source_lang", source_language.as_str()));
    }

    let response = client
        .post(endpoint)
        .header("Authorization", format!("DeepL-Auth-Key {key}"))
        .form(&fields)
        .send()
        .await
        .map_err(|error| format!("Could not reach DeepL: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Could not read the DeepL response: {error}"))?;

    if !status.is_success() {
        return Err(match status.as_u16() {
            403 => "DeepL rejected the API key. Check it in Settings.".to_string(),
            456 => "The DeepL quota for this key has been used up.".to_string(),
            _ => format!("DeepL returned HTTP {status}."),
        });
    }

    let data: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| "DeepL sent a response Glossy could not read.".to_string())?;
    let entry = data
        .get("translations")
        .and_then(|value| value.get(0))
        .ok_or_else(|| "DeepL returned no translation.".to_string())?;

    let translation = entry
        .get("text")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    if translation.is_empty() {
        return Err("DeepL returned an empty translation.".to_string());
    }

    let mut result = TranslationResult::new(kind, "deepl", text, target);
    result.translation = translation;
    if let Some(language) = entry
        .get("detected_source_language")
        .and_then(|value| value.as_str())
    {
        result.source_lang = language.to_ascii_lowercase();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_language_codes_for_deepl() {
        assert_eq!(deepl_target("zh-CN"), "ZH");
        assert_eq!(deepl_target("zh-TW"), "ZH-HANT");
        assert_eq!(deepl_target("en"), "EN-US");
        assert_eq!(deepl_target("ja"), "JA");
        assert_eq!(deepl_target("pt-BR"), "PT-BR");
        assert_eq!(deepl_target(""), "EN-US");
    }

    #[test]
    fn drops_the_region_from_a_source_language() {
        assert_eq!(deepl_source("en-US"), "EN");
        assert_eq!(deepl_source("zh-TW"), "ZH");
        assert_eq!(deepl_source("pt-BR"), "PT");
        assert_eq!(deepl_source("ja"), "JA");
    }
}
