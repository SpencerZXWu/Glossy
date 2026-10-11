//! What the deployment says about its own counters, for the one running it.
//!
//! The server keeps a day's counters to hold the free allowance honest;
//! `/v1/stats` is those same numbers read the other way round — how much of the
//! day is gone, whose it went to, and what the month has spent on readings. It
//! answers only to the token the deployment named, so the token is typed into
//! the window by hand and kept there for as long as the window is open: it is
//! never written to the settings file, and a window that is closed forgets it.
//!
//! Nothing about the report touches the counters: it books no characters and
//! moves no allowance, which is what makes it safe to ask for as often as one
//! likes.

use crate::translate::cloud_endpoint;

/// Answers the window knows how to say.
///
/// A key rather than a sentence: the window is the only thing that knows which
/// language it is drawn in, and a report that could not be read has to read as
/// plainly as anything else on the page.
pub const NO_TOKEN: &str = "dev.noToken";
pub const REFUSED: &str = "dev.refused";
pub const CLOSED: &str = "dev.closed";
pub const UNREADABLE: &str = "dev.unreadable";

/// Asks the deployment this build translates through for its report.
///
/// The address is the one the app already talks to rather than a second field
/// to fill in: a page that could be pointed anywhere would be a page that reads
/// somebody else's counters.
pub async fn report(token: &str) -> Result<serde_json::Value, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err(NO_TOKEN.to_string());
    }
    let endpoint = cloud_endpoint().map_err(|_| UNREADABLE.to_string())?;
    let client = crate::translate::client().map_err(|_| UNREADABLE.to_string())?;
    let response = client
        .get(format!("{endpoint}/v1/stats"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| {
            crate::log::note!("glossy: the server report could not be read: {error}");
            UNREADABLE.to_string()
        })?;

    let status = response.status();
    let body = response.text().await.map_err(|_| UNREADABLE.to_string())?;

    // A refusal is either a token that does not match or a deployment that has
    // not set one at all, and the two are told apart by the code the server
    // sends. What the server said also goes to the log, because it is the only
    // place the difference is visible after the fact.
    if status == reqwest::StatusCode::FORBIDDEN {
        let code = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|data| {
                data.get("code")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_default();
        crate::log::note!("glossy: the server report was refused ({code}): {}", body.trim());
        return Err(if code == "not_configured" { CLOSED } else { REFUSED }.to_string());
    }
    if !status.is_success() {
        crate::log::note!("glossy: the server report answered HTTP {status}: {}", body.trim());
        return Err(UNREADABLE.to_string());
    }

    serde_json::from_str(&body).map_err(|_| UNREADABLE.to_string())
}
