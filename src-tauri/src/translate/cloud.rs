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
use crate::log::note;

use super::{Failure, TranslationResult};

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
///
/// It is derived from the machine and the Windows account rather than drawn at
/// random, so that installing the app again lands on the id it had — an
/// allowance counted by a random id starts over for anybody who reinstalls,
/// which is exactly the bug this fixes. Nothing about the machine is sent
/// itself: the server only ever sees the 32 hex characters below.
///
/// A machine whose identifier cannot be read falls back on a random one, which
/// is what every install used to have and is still better than refusing to
/// translate.
pub fn new_install_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let machine = crate::platform::desktop::machine_id();
    let seed = match machine {
        Some(machine) => {
            let account = std::env::var("USERNAME").unwrap_or_default();
            format!("glossy-{machine}-{account}")
        }
        None => {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0);
            format!(
                "glossy-{nanos}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            )
        }
    };
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
///
/// It is part of the build rather than a setting: the window has had no field
/// for it for years, and a wrong address in an old file would break the one
/// service that needs nothing set up. A build made without an address — which
/// is what a fork that has not deployed `server/` gets — says so instead.
fn endpoint_of() -> Result<String, String> {
    endpoint_from(DEFAULT_ENDPOINT)
}

/// `endpoint_of` with the build default passed in, so the branch that has
/// nothing to send to stays reachable for a test even though this build ships
/// with an address.
fn endpoint_from(build_default: &str) -> Result<String, String> {
    let endpoint = build_default.trim().trim_end_matches('/');
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
    install_id: &str,
) -> Result<TranslationResult, Failure> {
    let endpoint = endpoint_of().map_err(Failure::from)?;
    let payload = request.payload(install_id);

    let response = client
        .post(format!("{endpoint}/v1/translate"))
        .json(&payload)
        .send()
        .await
        .map_err(|error| {
            Failure::coded(
                format!("Could not reach the Glossy translation server: {error}"),
                RELAY_UNREACHABLE,
            )
        })?;

    let status = response.status();
    let body = response.text().await.map_err(|error| {
        Failure::coded(
            format!("Could not read the Glossy translation server response: {error}"),
            RELAY_UNREACHABLE,
        )
    })?;

    let data: serde_json::Value = serde_json::from_str(&body).map_err(|_| {
        if status.is_success() {
            Failure::coded(
                "The Glossy translation server sent a response the app could not read.".to_string(),
                RELAY_UNREACHABLE,
            )
        } else {
            Failure::coded(
                format!(
                    "The Glossy translation server answered HTTP {status}. Check that the service \
                     deployed from `server/` is still running."
                ),
                RELAY_UNREACHABLE,
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
        return Err(Failure::plain(
            "The Glossy translation server returned an empty translation.".to_string(),
        ));
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
    // The relay walks its own list of backends when the one it was asked for
    // fails, and answers from the one that did. That is a fallback the user never
    // asked for and the app cannot see coming, so it is marked like one the app
    // made itself: the card then names the engine that answered *and* the one
    // that did not, instead of looking as if the choice had moved on its own.
    let asked = request.vendor.trim();
    if !asked.is_empty() && !provider.eq_ignore_ascii_case(asked) {
        let refused = refused_with(&data, asked);
        note!(
            "glossy: the relay answered with {provider} instead of {asked}{}",
            match refused.as_deref() {
                Some(code) => format!(" ({code})"),
                None => String::new(),
            }
        );
        result.fallback_from = Some(asked.to_string());
        result.fallback_code = refused;
    }
    Ok(result)
}

/// The code a vendor was refused with, when the deployment reports it.
///
/// A deployment that walks past a backend without saying why leaves this empty,
/// which is what an older relay does: the card can still say that the answer came
/// from somewhere else, just not why the one that was asked for stepped aside.
fn refused_with(data: &serde_json::Value, vendor: &str) -> Option<String> {
    data.get("attempts")?
        .as_array()?
        .iter()
        .find(|attempt| {
            attempt
                .get("vendor")
                .and_then(|value| value.as_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(vendor))
        })
        .and_then(|attempt| attempt.get("code")?.as_str().map(str::to_string))
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
pub async fn quota(client: &reqwest::Client, install_id: &str) -> Result<Quota, String> {
    let endpoint = endpoint_of()?;
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
        return Err(problem(&data, status).to_string());
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
pub async fn rates(client: &reqwest::Client, base: &str, install_id: &str) -> Option<RateTable> {
    let endpoint = endpoint_of().ok()?;
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
/// The code that says the relay itself never answered: the one thing the card
/// can name when the failure is not a vendor's own refusal.
const RELAY_UNREACHABLE: &str = "relay_unreachable";

fn problem(data: &serde_json::Value, status: reqwest::StatusCode) -> Failure {
    let code = data
        .get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let message = data
        .get("message")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim();

    let message = match code {
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
    };
    Failure {
        message,
        code: vendor_code(code),
    }
}

/// The refusal codes the card has a sentence for.
///
/// Only the ones about a vendor are passed on. The rest are about the relay
/// itself — its own daily allowance, its rate limit, a request it would not
/// take — and a line saying that the engine named in the footer ran out of
/// quota when it was the relay that refused would be a lie. Those keep the
/// message alone: the log has the detail, and the card says only that the
/// engine did not answer.
fn vendor_code(code: &str) -> Option<String> {
    match code {
        "upstream_limit"
        | "upstream_credentials"
        | "upstream_timeout"
        | "upstream_error"
        | "upstream_unreachable"
        | "not_configured"
        | "unsupported_language" => Some(code.to_string()),
        _ => None,
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
    }

    #[test]
    fn the_install_id_of_one_machine_stays_the_same() {
        // The allowance is counted by this id, so drawing it at random would
        // hand a fresh allowance to anybody who installs the app again. It is
        // read off the machine instead, and this is what says so: two calls in
        // one process — which is all a test can see of a second install — have
        // to agree.
        assert_eq!(new_install_id(), new_install_id());
    }

    #[test]
    fn a_missing_address_is_explained_instead_of_sent_somewhere() {
        // A build made without an address — a fork that has not deployed
        // `server/` — has nowhere to send the text.
        let error = endpoint_from("").unwrap_err();
        assert!(error.contains("no server address"));
        // This build has one, and it is the address every request goes to.
        assert_eq!(endpoint_of().unwrap(), DEFAULT_ENDPOINT);
        assert!(DEFAULT_ENDPOINT.starts_with("https://"));
        // Whatever it is, it is trimmed and has to be a URL.
        assert_eq!(
            endpoint_from(" https://glossy.example.workers.dev/ ").unwrap(),
            "https://glossy.example.workers.dev"
        );
        assert!(endpoint_from("glossy.example.workers.dev").is_err());
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
    fn a_vendor_the_relay_walked_past_gives_up_why() {
        let reported = serde_json::json!({
            "ok": true,
            "vendor": "youdao",
            "attempts": [
                {"vendor": "baidu", "code": "upstream_credentials"},
            ],
        });
        assert_eq!(
            refused_with(&reported, "baidu").as_deref(),
            Some("upstream_credentials")
        );
        // The engine that answered is not the one that was refused.
        assert_eq!(refused_with(&reported, "youdao"), None);
        // A deployment that walks past a backend without saying why, which is
        // what a relay older than this build does.
        assert_eq!(
            refused_with(&serde_json::json!({"vendor": "youdao"}), "baidu"),
            None
        );
        assert_eq!(
            refused_with(&serde_json::json!({"attempts": []}), "baidu"),
            None
        );
        // A code that is not a string is not a reason either.
        assert_eq!(
            refused_with(
                &serde_json::json!({"attempts": [{"vendor": "baidu", "code": 12}]}),
                "baidu"
            ),
            None
        );
    }

    #[test]
    fn refusals_become_one_sentence_per_code() {
        let quota = problem(
            &serde_json::json!({"code": "client_quota_exceeded"}),
            reqwest::StatusCode::TOO_MANY_REQUESTS,
        );
        assert!(quota.message.contains("used up"));
        let config = problem(
            &serde_json::json!({"code": "not_configured"}),
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
        );
        assert!(config.message.contains("BAIDU_APP_ID"));
        // An unknown code keeps whatever the server explained.
        let other = problem(
            &serde_json::json!({"code": "mystery", "message": "服务器说得很清楚"}),
            reqwest::StatusCode::BAD_GATEWAY,
        );
        assert_eq!(other.message, "服务器说得很清楚");
    }

    #[test]
    fn only_a_vendors_own_refusal_says_why_on_the_card() {
        // A vendor that ran out of quota is what the line under the engine is
        // for; the relay's own daily allowance has nothing to do with the engine
        // named in the footer, so that one says nothing at all.
        assert_eq!(
            vendor_code("upstream_limit"),
            Some("upstream_limit".to_string())
        );
        assert_eq!(
            vendor_code("upstream_credentials"),
            Some("upstream_credentials".to_string())
        );
        assert_eq!(vendor_code("client_quota_exceeded"), None);
        assert_eq!(vendor_code("rate_limited"), None);
        assert_eq!(vendor_code("mystery"), None);
    }
}
