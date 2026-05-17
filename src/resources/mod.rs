pub mod audio;
pub mod batches;
pub mod chat;
pub mod chat_stream_ext;
pub mod embeddings;
pub mod files;
pub mod fine_tuning;
pub mod images;
pub mod models;
pub mod moderations;
pub(crate) mod sse_parser;
pub mod stream;
pub mod uploads;
pub mod vector_stores;

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::client::Client;
use crate::error::{OpenAiError, Result};

/// Internal helper: perform a JSON POST and decode the response. Retries on transient
/// failures (HTTP 429 / 5xx / connection errors) per the client's effective max_retries.
pub(crate) async fn post_json<B: Serialize, R: DeserializeOwned>(
    client: &Client,
    path: &str,
    body: &B,
) -> Result<R> {
    let url = client.build_url(path)?;
    let body_value = serde_json::to_value(body)?;
    let max_attempts = client.effective_max_retries().saturating_add(1);
    let per_req_timeout = client.effective_timeout();

    #[cfg(feature = "tracing")]
    tracing::debug!(target: "open_ai_rust", method = "POST", %url, "request");

    let mut last_err: Option<OpenAiError> = None;
    for attempt in 0..max_attempts {
        let attempt_result = async {
            let mut req = client
                .http()
                .post(&url)
                .headers(client.auth_headers())
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .json(&body_value);
            if let Some(t) = per_req_timeout {
                req = req.timeout(t);
            }
            let resp = req.send().await?;
            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(OpenAiError::from_response_body(status.as_u16(), &text));
            }
            let text = resp.text().await?;
            serde_json::from_str::<R>(&text).map_err(OpenAiError::from)
        }
        .await;

        match attempt_result {
            Ok(v) => return Ok(v),
            Err(e) if is_retryable(&e) && attempt + 1 < max_attempts => {
                #[cfg(feature = "tracing")]
                tracing::warn!(target: "open_ai_rust", attempt = attempt + 1, error = %e, "retrying");
                last_err = Some(e);
                tokio::time::sleep(backoff_delay(attempt)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or_else(|| OpenAiError::config("retry budget exhausted")))
}

fn is_retryable(err: &OpenAiError) -> bool {
    match err {
        OpenAiError::Reqwest(e) => e.is_connect() || e.is_timeout() || e.is_request(),
        OpenAiError::Api { status, .. } => *status == 429 || (500..=599).contains(status),
        _ => false,
    }
}

fn backoff_delay(attempt: u32) -> Duration {
    // Exponential w/ small jitter: 500ms, 1s, 2s, 4s … capped at 30s.
    let base_ms: u64 = 500u64.saturating_mul(1u64 << attempt.min(6));
    let capped = base_ms.min(30_000);
    // Cheap deterministic jitter via attempt nibble (avoids adding rand dep).
    let jitter = (attempt as u64 * 137) % 250;
    Duration::from_millis(capped + jitter)
}

pub(crate) async fn post_multipart<R: DeserializeOwned>(
    client: &Client,
    path: &str,
    form: reqwest::multipart::Form,
) -> Result<R> {
    let url = client.build_url(path)?;
    let mut req = client
        .http()
        .post(url)
        .headers(client.auth_headers())
        .multipart(form);
    if let Some(t) = client.effective_timeout() {
        req = req.timeout(t);
    }
    let resp = req.send().await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(OpenAiError::from_response_body(status.as_u16(), &text));
    }

    let text = resp.text().await?;
    serde_json::from_str(&text).map_err(OpenAiError::from)
}

pub(crate) async fn post_json_bytes<B: Serialize>(
    client: &Client,
    path: &str,
    body: &B,
) -> Result<bytes::Bytes> {
    let url = client.build_url(path)?;
    let mut req = client
        .http()
        .post(url)
        .headers(client.auth_headers())
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(body);
    if let Some(t) = client.effective_timeout() {
        req = req.timeout(t);
    }
    let resp = req.send().await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(OpenAiError::from_response_body(status.as_u16(), &text));
    }

    Ok(resp.bytes().await?)
}
