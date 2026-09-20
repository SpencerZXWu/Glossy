//! Glossy's own translation proxy.
//!
//! The Worker in `server/` holds the provider credentials, so this build ships
//! without any: the app only sends the text plus an install id, and the server
//! decides what is left of the day's quota.

use std::time::{SystemTime, UNIX_EPOCH};

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use crate::classify::Kind;

use super::TranslationResult;

/// Address the build points at when the user has not set one.
///
/// It is the deployment made from `server/`, the same one for every copy of the
/// app, which is what lets the cloud provider work without anybody filling in a
/// key. It is public and harmless to read: the credentials live on the server,
/// and the server answers a device that asks for more than its daily allowance
/// with an error rather than with translations. Emptying this constant makes
/// every build ask for a deployment of its own in the settings window.
pub const DEFAULT_ENDPOINT: &str = "https://1492303375-cwrw0pdztr.ap-guangzhou.tencentscf.com";

/// Characters the server wants an install id to have.
pub fn new_install_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let seed = format!(
        "{nanos}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    // MD5 is already along for the Baidu signature, and its 32 hex characters
    // are exactly the kind of id the server accepts.
    format!("{:x}", Md5::digest(seed.as_bytes()))
}

/// What the server still allows today, shown under the provider dropdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quota {
    pub day: String,
    /// Characters one device may translate per day.
    pub limit: u64,
    pub used: u64,
    pub remaining: u64,
}

/// The address requests go to, or an explanation of why there is none.
fn endpoint_of(configured: &str) -> Result<String, String> {
    endpoint_from(configured, DEFAULT_ENDPOINT)
}

/// `endpoint_of` with the build default passed in, so the branch that has
/// nothing to send to stays reachable for a test even though this build ships
/// with an address.
fn endpoint_from(configured: &str, build_default: &str) -> Result<String, String> {
    let configured = configured.trim().trim_end_matches('/');
    let endpoint = if configured.is_empty() {
        build_default
    } else {
        configured
    };
    if endpoint.is_empty() {
        return Err(
            "The cloud translator has no server address yet. Deploy the Worker from `server/` \
             and paste its address in Settings, or pick another provider."
                .to_string(),
        );
    }
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err("The cloud translator address has to start with `https://`.".to_string());
    }
    Ok(endpoint.to_string())
}

pub async fn translate(
    client: &reqwest::Client,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
    configured_endpoint: &str,
    install_id: &str,
) -> Result<TranslationResult, String> {
    let endpoint = endpoint_of(configured_endpoint)?;
    let payload = serde_json::json!({
        "clientId": install_id,
        "text": text,
        "from": source,
        "to": target,
    });

    let response = client
        .post(format!("{endpoint}/v1/translate"))
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("Could not reach the Glossy translation server: {error}"))?;

    let status = response.status();
    let body = response.text().await.map_err(|error| {
        format!("Could not read the Glossy translation server response: {error}")
    })?;

    let data: serde_json::Value = serde_json::from_str(&body).map_err(|_| {
        if status.is_success() {
            "The Glossy translation server sent a response the app could not read.".to_string()
        } else {
            format!(
                "The Glossy translation server answered HTTP {status}. Is the address in \
                 Settings the one `wrangler deploy` printed?"
            )
        }
    })?;

    if data.get("ok").and_then(|value| value.as_bool()) != Some(true) {
        return Err(problem(&data, status));
    }

    let translation = data
        .get("translation")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    if translation.trim().is_empty() {
        return Err("The Glossy translation server returned an empty translation.".to_string());
    }

    let mut result = TranslationResult::new(kind, "cloud", text, target);
    result.translation = translation;
    result.source_lang = data
        .get("from")
        .and_then(|value| value.as_str())
        .unwrap_or("auto")
        .to_string();
    Ok(result)
}

/// Reads today's allowance for this installation.
pub async fn quota(
    client: &reqwest::Client,
    configured_endpoint: &str,
    install_id: &str,
) -> Result<Quota, String> {
    let endpoint = endpoint_of(configured_endpoint)?;
    let response = client
        .get(format!("{endpoint}/v1/quota"))
        .query(&[("client", install_id)])
        .send()
        .await
        .map_err(|_| {
            format!(
                "Could not reach the Glossy translation server at {endpoint}. Check that it is \
                 deployed and that the address above is the one `wrangler deploy` printed."
            )
        })?;

    let status = response.status();
    let body = response.text().await.map_err(|error| {
        format!("Could not read the Glossy translation server response: {error}")
    })?;
    let data: serde_json::Value = serde_json::from_str(&body).map_err(|_| {
        format!("The Glossy translation server answered HTTP {status} instead of a quota.")
    })?;

    if data.get("ok").and_then(|value| value.as_bool()) != Some(true) {
        return Err(problem(&data, status));
    }

    let number = |path: &[&str]| -> u64 {
        let mut cursor = &data;
        for key in path {
            cursor = &cursor[key];
        }
        cursor.as_u64().unwrap_or(0)
    };

    Ok(Quota {
        day: data
            .get("day")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        limit: number(&["limits", "charsPerClient"]),
        used: number(&["usage", "client"]),
        remaining: number(&["remaining"]),
    })
}

/// Turns a refusal from the server into a sentence the popup can show.
///
/// The server answers in Chinese, and a desktop app that may be running in
/// either language should not show one language's text in the other, so the
/// codes carry the wording and the server text is only the last resort.
fn problem(data: &serde_json::Value, status: reqwest::StatusCode) -> String {
    let code = data
        .get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let message = data
        .get("message")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim();

    match code {
        "client_quota_exceeded" | "ip_quota_exceeded" | "global_quota_exceeded" => {
            "The free cloud translation quota for today is used up. It starts over at 00:00 UTC."
                .to_string()
        }
        "rate_limited" => {
            "The cloud translator is being hit too often right now. Try again in a moment."
                .to_string()
        }
        "too_long" => {
            "That selection is longer than the cloud translator accepts in one request.".to_string()
        }
        "unsupported_language" => {
            "The cloud translator does not support that language pair.".to_string()
        }
        "not_configured" => "The cloud translator has no provider credentials yet: set \
             BAIDU_APP_ID and BAIDU_KEY on the server with `wrangler secret put`."
            .to_string(),
        "upstream_credentials" => "The cloud translator could not authenticate with the provider. \
             Check the APP ID and key set on the server."
            .to_string(),
        "upstream_limit" => {
            "The account behind the cloud translator has run out of quota.".to_string()
        }
        "upstream_timeout" | "upstream_error" | "upstream_unreachable" => {
            "The cloud translator could not reach the translation service. Try again in a moment."
                .to_string()
        }
        "invalid_request" if !message.is_empty() => {
            format!("The cloud translator refused the request: {message}")
        }
        _ if !message.is_empty() => message.to_string(),
        _ => format!("The cloud translator answered HTTP {status}."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_install_id_looks_the_way_the_server_asks_for_it() {
        let id = new_install_id();
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(id, new_install_id());
    }

    #[test]
    fn a_missing_address_is_explained_instead_of_sent_somewhere() {
        // A build without an address of its own, and nothing set here, has
        // nowhere to send the text.
        let error = endpoint_from("", "").unwrap_err();
        assert!(error.contains("no server address"));
        // Otherwise the build's own address is what an empty setting means.
        assert_eq!(endpoint_of("").unwrap(), DEFAULT_ENDPOINT);
        assert!(DEFAULT_ENDPOINT.starts_with("https://"));
        // And a setting overrides it. The trailing slash and the path a user
        // copies from the dashboard are not part of the address requests are
        // built from.
        assert_eq!(
            endpoint_of(" https://glossy.example.workers.dev/ ").unwrap(),
            "https://glossy.example.workers.dev"
        );
        assert!(endpoint_of("glossy.example.workers.dev").is_err());
    }

    #[test]
    fn refusals_become_one_sentence_per_code() {
        let quota = problem(
            &serde_json::json!({"code": "client_quota_exceeded"}),
            reqwest::StatusCode::TOO_MANY_REQUESTS,
        );
        assert!(quota.contains("used up"));
        let config = problem(
            &serde_json::json!({"code": "not_configured"}),
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
        );
        assert!(config.contains("BAIDU_APP_ID"));
        // An unknown code keeps whatever the server explained.
        let other = problem(
            &serde_json::json!({"code": "mystery", "message": "服务器说得很清楚"}),
            reqwest::StatusCode::BAD_GATEWAY,
        );
        assert_eq!(other, "服务器说得很清楚");
    }
}
