//! A translation model running on this machine.
//!
//! The cloud channel's second backend: instead of asking a server, the app
//! sends the text to an OpenAI compatible service on localhost - Ollama, LM
//! Studio, llama.cpp, vLLM, anything that speaks `/v1/chat/completions`. It is
//! the only backend with no account, no key, no quota and no terms of service
//! to weigh, because the model runs on the user's own hardware.

use crate::classify::Kind;

use super::{chat, TranslationResult};

/// Builds the request URL from whatever the user typed: a base URL such as
/// `http://127.0.0.1:11434/v1`, the full endpoint, or either with a trailing
/// slash.
fn endpoint_of(configured: &str) -> Result<String, String> {
    let base = configured.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("The local model has no address yet. Fill one in under Settings.".to_string());
    }
    if !base.starts_with("http://") && !base.starts_with("https://") {
        return Err(
            "The local model address has to start with `http://` or `https://`.".to_string(),
        );
    }
    if base.ends_with("/chat/completions") {
        return Ok(base.to_string());
    }
    Ok(format!("{base}/chat/completions"))
}

/// A service that is not running is the usual reason this fails, and the bare
/// connection error does not say so.
fn explain(endpoint: &str, error: String) -> String {
    const PREFIX: &str = "Could not reach the local model: ";
    match error.strip_prefix(PREFIX) {
        Some(detail) => format!(
            "Nothing answered at {endpoint}. Start the local model service - for Ollama that is \
             `ollama serve`, and `ollama pull <model>` once beforehand - then check the address \
             and the model name in Settings. ({detail})"
        ),
        None => error,
    }
}

pub async fn translate(
    client: &reqwest::Client,
    configured_endpoint: &str,
    model: &str,
    text: &str,
    source: &str,
    target: &str,
    kind: Kind,
) -> Result<TranslationResult, String> {
    let endpoint = endpoint_of(configured_endpoint)?;
    let model = model.trim();
    if model.is_empty() {
        return Err(
            "The local model needs a model name. Fill one in under Settings - `ollama list` \
             prints the ones you have."
                .to_string(),
        );
    }

    chat::translate(
        client,
        &chat::local(endpoint.clone(), model.to_string()),
        text,
        source,
        target,
        kind,
        "",
    )
    .await
    .map_err(|error| explain(&endpoint, error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_the_chat_path_to_a_base_url() {
        assert_eq!(
            endpoint_of("http://127.0.0.1:11434/v1").expect("a base URL is enough"),
            "http://127.0.0.1:11434/v1/chat/completions"
        );
    }

    #[test]
    fn keeps_the_chat_path_when_it_is_already_there() {
        assert_eq!(
            endpoint_of("http://127.0.0.1:8080/v1/chat/completions/").expect("the full URL works"),
            "http://127.0.0.1:8080/v1/chat/completions"
        );
    }

    #[test]
    fn rejects_an_address_that_is_not_a_url() {
        assert!(endpoint_of("127.0.0.1:11434").is_err());
        assert!(endpoint_of("   ").is_err());
    }
}
