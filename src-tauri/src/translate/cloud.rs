//! Glossy's own translation proxy.
//!
//! The Worker in `server/` holds the provider credentials, so this build ships
//! without any: the app only sends the text plus an install id, and the server
//! decides what is left of the day's quota.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
///
/// The address this build was made with wins over the one in the settings file:
/// the window has no field for it any more, so a value found there is a leftover
/// from an older version, and a wrong one breaks the only service that needs
/// nothing set up. A build made without an address still honors the setting.
fn endpoint_from(configured: &str, build_default: &str) -> Result<String, String> {
    let build_default = build_default.trim().trim_end_matches('/');
    let configured = configured.trim().trim_end_matches('/');
    let endpoint = if build_default.is_empty() {
        configured
    } else {
        build_default
    };
    if endpoint.is_empty() {
        return Err(
            "The cloud translator has no server address yet. Deploy the service from `server/` \
             and build Glossy with its address, or pick another provider."
                .to_string(),
        );
    }
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err("The cloud translator address has to start with `https://`.".to_string());
    }
    Ok(endpoint.to_string())
}

/// The address this build sends requests to, for the callers that need the same
/// choice `endpoint_of` makes without wanting its error path: what they do with
/// a missing address is skip the server entirely, so an empty string is the
/// answer for a build that carries no address of its own.
pub fn resolved_endpoint(configured: &str) -> String {
    endpoint_from(configured, DEFAULT_ENDPOINT).unwrap_or_default()
}

/// What one translation asks the server for.
pub struct Request<'a> {
    pub text: &'a str,
    pub source: &'a str,
    pub target: &'a str,
    pub kind: Kind,
    /// Which vendor the server should translate with — `baidu`, `youdao`, or
    /// empty to let it walk its own list of backends.
    pub vendor: &'a str,
}

impl Request<'_> {
    /// The body a translation is asked with.
    fn payload(&self, install_id: &str) -> serde_json::Value {
        serde_json::json!({
            "clientId": install_id,
            "text": self.text,
            "from": self.source,
            "to": self.target,
            "vendor": self.vendor,
        })
    }
}

pub async fn translate(
    client: &reqwest::Client,
    request: Request<'_>,
    configured_endpoint: &str,
    install_id: &str,
) -> Result<TranslationResult, String> {
    let endpoint = endpoint_of(configured_endpoint)?;
    let payload = request.payload(install_id);

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
                "The Glossy translation server answered HTTP {status}. Check that the service \
                 deployed from `server/` is still running."
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

    // The server names the engine that answered, so the footer can say which
    // one it was; a server too old to name it leaves the vendor the request
    // asked for, because the card showing the engine that ran is worth more
    // than it showing that something in the cloud ran.
    let provider = answered_by(&data, request.vendor);
    let mut result = TranslationResult::new(request.kind, provider, request.text, request.target);
    result.translation = translation;
    result.source_lang = data
        .get("from")
        .and_then(|value| value.as_str())
        .unwrap_or("auto")
        .to_string();
    Ok(result)
}

/// The name the card shows for the engine that answered.
///
/// The server's own word for it is trusted when it is one of the two vendors
/// this build can ask for; anything else falls back to the vendor the request
/// named, because a card is more useful saying "baidu" than saying nothing.
fn answered_by<'a>(data: &'a serde_json::Value, vendor: &'a str) -> &'a str {
    data.get("vendor")
        .and_then(|value| value.as_str())
        .filter(|name| matches!(*name, "baidu" | "youdao"))
        .unwrap_or(vendor)
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
                "Could not reach the Glossy translation server at {endpoint}. Check that the \
                 service deployed from `server/` is still running."
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

/// One exchange-rate table the server fetched for this device.
#[derive(Debug, Clone)]
pub struct RateTable {
    /// Which vendor the numbers came from, shown under the conversion.
    pub source: String,
    /// The day the table was published, `YYYY-MM-DD`, when the vendor said.
    pub date: Option<String>,
    /// Rates per currency code, all relative to the base that was asked for.
    pub rates: HashMap<String, f64>,
}

/// How long the rate lookup may take. The card asks for this on the way to the
/// first paint, so it is bounded well below the translation's own budget.
const RATE_TIMEOUT: Duration = Duration::from_secs(4);

/// The exchange-rate table for one base currency, or `None` when the server has
/// nothing to say — a deployment from before this route existed, or no network.
///
/// A conversion is an annotation next to the translation rather than the thing
/// the user asked for, so every failure here is silent: `currency` keeps asking
/// the vendors directly and the card simply carries no rate when none answers.
///
/// The server books these calls against its per-minute limit and nothing
/// against the daily characters, so a card that only holds a currency
/// conversion still costs no quota.
pub async fn rates(
    client: &reqwest::Client,
    base: &str,
    configured_endpoint: &str,
    install_id: &str,
) -> Option<RateTable> {
    let endpoint = endpoint_of(configured_endpoint).ok()?;
    let response = client
        .get(format!("{endpoint}/v1/rates"))
        .query(&[("base", base), ("client", install_id)])
        .timeout(RATE_TIMEOUT)
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }

    let data: serde_json::Value = response.json().await.ok()?;
    if data.get("ok").and_then(|value| value.as_bool()) != Some(true) {
        return None;
    }

    let mut rates = HashMap::new();
    for (code, value) in data.get("rates")?.as_object()?.iter() {
        if let Some(value) = value.as_f64() {
            if value.is_finite() && value > 0.0 {
                rates.insert(code.to_ascii_uppercase(), value);
            }
        }
    }
    if rates.is_empty() {
        return None;
    }

    Some(RateTable {
        source: data
            .get("source")
            .and_then(|value| value.as_str())
            .unwrap_or("glossy-cloud")
            .to_string(),
        date: data
            .get("date")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        rates,
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
        // Otherwise the build's own address is what every setting means.
        assert_eq!(endpoint_of("").unwrap(), DEFAULT_ENDPOINT);
        assert!(DEFAULT_ENDPOINT.starts_with("https://"));
        // A build made without an address of its own is the one case where the
        // settings file is still read, because there is nothing else to use.
        assert_eq!(
            endpoint_from(" https://glossy.example.workers.dev/ ", "").unwrap(),
            "https://glossy.example.workers.dev"
        );
        // The address in the settings file is a leftover the window cannot show,
        // so it never replaces the one the build was made with.
        assert_eq!(
            endpoint_of("https://glossy.example.workers.dev").unwrap(),
            DEFAULT_ENDPOINT
        );
        assert!(endpoint_from("glossy.example.workers.dev", "").is_err());
        // Callers that answer with "no server" instead of an error get the same
        // address: the one the build was made with, whatever the file says.
        assert_eq!(
            resolved_endpoint("https://glossy.example.workers.dev"),
            DEFAULT_ENDPOINT
        );
        assert_eq!(resolved_endpoint("  "), DEFAULT_ENDPOINT);
    }

    #[test]
    fn the_request_names_the_vendor_and_keeps_the_rest() {
        let request = |vendor: &'static str| Request {
            text: "hello",
            source: "en",
            target: "zh",
            kind: Kind::Sentence,
            vendor,
        };
        let body = request("youdao").payload("abc");
        assert_eq!(body["clientId"], "abc");
        assert_eq!(body["text"], "hello");
        assert_eq!(body["from"], "en");
        assert_eq!(body["to"], "zh");
        assert_eq!(body["vendor"], "youdao");
        // The empty string is how the app asks the server to choose.
        assert_eq!(request("").payload("abc")["vendor"], "");
    }

    #[test]
    fn the_card_names_the_engine_that_answered() {
        let answered =
            |body: serde_json::Value, vendor: &'static str| answered_by(&body, vendor).to_string();
        // The server's word for it wins, which is the case that matters when
        // the server walked its own list of backends and picked another one.
        assert_eq!(
            answered(serde_json::json!({"vendor": "youdao"}), "baidu"),
            "youdao"
        );
        assert_eq!(
            answered(serde_json::json!({"vendor": "baidu"}), "youdao"),
            "baidu"
        );
        // A server too old to name it leaves the vendor that was asked for,
        // because the card never has to fall back to saying "cloud".
        assert_eq!(answered(serde_json::json!({}), "youdao"), "youdao");
        assert_eq!(
            answered(serde_json::json!({"vendor": ""}), "baidu"),
            "baidu"
        );
        // A name this build cannot place is not shown either.
        assert_eq!(
            answered(serde_json::json!({"vendor": "deepl"}), "baidu"),
            "baidu"
        );
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
