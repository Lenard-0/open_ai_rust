//! SSE streaming helpers shared by Chat Completions and Responses APIs.

use std::pin::Pin;

use futures_core::Stream;
use futures_util::StreamExt;
use serde::de::DeserializeOwned;

use super::sse_parser::{parse_next_event, SseEvent};
use crate::client::Client;
use crate::error::{OpenAiError, Result};

/// Open an SSE stream POST: send `body` as JSON, return a stream of parsed events of type `T`.
///
/// Stops cleanly on a `data: [DONE]` line.
pub(crate) async fn post_sse_stream<T: DeserializeOwned + Send + 'static>(
    client: &Client,
    path: &str,
    body: serde_json::Value,
) -> Result<Pin<Box<dyn Stream<Item = Result<T>> + Send>>> {
    let url = client.build_url(path)?;
    let resp = client
        .http()
        .post(url)
        .headers(client.auth_headers())
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(OpenAiError::from_response_body(status.as_u16(), &text));
    }

    let bytes_stream = resp.bytes_stream();
    let parsed = async_stream::stream! {
        let mut buffer = String::new();
        let mut stream = bytes_stream;
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(b) => b,
                Err(e) => {
                    yield Err(OpenAiError::Reqwest(e));
                    return;
                }
            };
            let s = match std::str::from_utf8(&chunk) {
                Ok(s) => s,
                Err(e) => {
                    yield Err(OpenAiError::stream(format!("non-utf8 SSE chunk: {e}")));
                    return;
                }
            };
            buffer.push_str(s);

            while let Some(event) = parse_next_event(&mut buffer) {
                match event {
                    SseEvent::Done => return,
                    SseEvent::Data(data) => {
                        match serde_json::from_str::<T>(&data) {
                            Ok(ev) => yield Ok(ev),
                            Err(e) => {
                                yield Err(OpenAiError::stream(format!(
                                    "failed to decode SSE event: {e} body={data}"
                                )));
                                return;
                            }
                        }
                    }
                }
            }
        }
    };
    Ok(Box::pin(parsed))
}
