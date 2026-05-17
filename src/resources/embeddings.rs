//! Embeddings — `/v1/embeddings`.

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::Result;
use crate::logoi::output::Usage;

/// Accessor for the Embeddings API. Obtained via [`Client::embeddings`].
pub struct Embeddings<'a> {
    client: &'a Client,
}

/// Request body for `/v1/embeddings`. Use [`EmbeddingRequestBuilder`] for ergonomic construction.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingRequest {
    /// Embedding model ID, e.g. `"text-embedding-3-small"`.
    pub model: String,
    /// One or more inputs to embed.
    pub input: EmbeddingInput,
    /// `"float"` (default) or `"base64"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    /// Truncated embedding dimensionality. Only supported by `text-embedding-3-*` models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    /// End-user identifier for abuse-tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Polymorphic embedding input: single string, batch of strings, or pre-tokenised IDs.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    String(String),
    Strings(Vec<String>),
    TokenIds(Vec<i64>),
    TokenIdGroups(Vec<Vec<i64>>),
}

impl From<String> for EmbeddingInput {
    fn from(s: String) -> Self {
        EmbeddingInput::String(s)
    }
}
impl From<&str> for EmbeddingInput {
    fn from(s: &str) -> Self {
        EmbeddingInput::String(s.to_string())
    }
}
impl From<Vec<String>> for EmbeddingInput {
    fn from(v: Vec<String>) -> Self {
        EmbeddingInput::Strings(v)
    }
}

/// Response from `/v1/embeddings`. `data` is parallel-indexed to the request `input`.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
    pub usage: Usage,
}

/// One embedding vector + its position in the request batch.
#[derive(Debug, Clone, Deserialize)]
pub struct Embedding {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u32,
}

/// Fluent builder for [`EmbeddingRequest`].
pub struct EmbeddingRequestBuilder {
    model: String,
    input: Option<EmbeddingInput>,
    encoding_format: Option<String>,
    dimensions: Option<u32>,
    user: Option<String>,
}

impl EmbeddingRequestBuilder {
    /// Start a builder for the given embedding model ID.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: None,
            encoding_format: None,
            dimensions: None,
            user: None,
        }
    }
    /// Set the input — anything convertible into [`EmbeddingInput`] (string, `Vec<String>`, etc.).
    pub fn input(mut self, input: impl Into<EmbeddingInput>) -> Self {
        self.input = Some(input.into());
        self
    }
    /// `"float"` (default) or `"base64"`.
    pub fn encoding_format(mut self, f: impl Into<String>) -> Self {
        self.encoding_format = Some(f.into());
        self
    }
    /// Truncate the embedding to `d` dimensions. Only `text-embedding-3-*` supports this.
    pub fn dimensions(mut self, d: u32) -> Self {
        self.dimensions = Some(d);
        self
    }
    /// End-user identifier for abuse-tracking.
    pub fn user(mut self, u: impl Into<String>) -> Self {
        self.user = Some(u.into());
        self
    }
    /// Finalise. If `input` is unset, defaults to an empty string (the API will reject).
    pub fn build(self) -> EmbeddingRequest {
        EmbeddingRequest {
            model: self.model,
            input: self.input.unwrap_or(EmbeddingInput::String(String::new())),
            encoding_format: self.encoding_format,
            dimensions: self.dimensions,
            user: self.user,
        }
    }
}

impl<'a> Embeddings<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /embeddings` — embed one or more inputs and return the full batched response.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "embeddings", model = %req.model))
    )]
    pub async fn create(&self, req: EmbeddingRequest) -> Result<EmbeddingResponse> {
        super::post_json(self.client, "/embeddings", &req).await
    }

    /// Convenience: embed a single string with `encoding_format=float` and return the raw vector.
    ///
    /// Returns [`OpenAiError::Stream`](crate::OpenAiError::Stream) if the response contains no
    /// embeddings (which OpenAI should never do, but the guard is here defensively).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "embeddings.one"))
    )]
    pub async fn create_one(
        &self,
        text: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Vec<f32>> {
        let req = EmbeddingRequestBuilder::new(model)
            .input(text.into())
            .encoding_format("float")
            .build();
        let resp = self.create(req).await?;
        resp.data
            .into_iter()
            .next()
            .map(|e| e.embedding)
            .ok_or_else(|| crate::error::OpenAiError::Stream("empty embeddings response".into()))
    }
}
