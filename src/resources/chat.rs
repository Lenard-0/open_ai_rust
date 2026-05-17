use std::pin::Pin;

use futures_core::Stream;

use crate::client::Client;
use crate::error::Result;
use crate::logoi::input::payload::ChatPayLoad;
use crate::logoi::output::stream::ChatCompletionChunk;
use crate::logoi::output::AiMsgResponse;

/// Accessor for the Chat Completions API (`/chat/completions`).
///
/// Obtained via [`Client::chat`].
pub struct Chat<'a> {
    client: &'a Client,
}

impl<'a> Chat<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /chat/completions` — create a chat completion.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "chat.completions", model = %payload.model))
    )]
    pub async fn create(&self, payload: ChatPayLoad) -> Result<AiMsgResponse> {
        super::post_json(self.client, "/chat/completions", &payload).await
    }

    /// Streaming variant — sets `stream: true` and returns an SSE stream of [`ChatCompletionChunk`]s.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "chat.completions.stream", model = %payload.model))
    )]
    pub async fn create_stream(
        &self,
        mut payload: ChatPayLoad,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>> {
        payload.stream = Some(true);
        let body = serde_json::to_value(&payload)?;
        super::stream::post_sse_stream(self.client, "/chat/completions", body).await
    }

    /// Mirror the official SDK shape: `client.chat().completions().create(...)`.
    /// Equivalent to [`Self::create`] / [`Self::create_stream`] directly on [`Chat`].
    pub fn completions(&self) -> Completions<'a> {
        Completions {
            client: self.client,
        }
    }
}

/// Alias for [`Chat`] that mirrors `client.chat.completions` in the official OpenAI SDK.
pub struct Completions<'a> {
    client: &'a Client,
}

impl<'a> Completions<'a> {
    /// `POST /chat/completions` — see [`Chat::create`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "chat.completions", model = %payload.model))
    )]
    pub async fn create(&self, payload: ChatPayLoad) -> Result<AiMsgResponse> {
        super::post_json(self.client, "/chat/completions", &payload).await
    }

    /// Streaming variant — see [`Chat::create_stream`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "chat.completions.stream", model = %payload.model))
    )]
    pub async fn create_stream(
        &self,
        mut payload: ChatPayLoad,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>> {
        payload.stream = Some(true);
        let body = serde_json::to_value(&payload)?;
        super::stream::post_sse_stream(self.client, "/chat/completions", body).await
    }
}
