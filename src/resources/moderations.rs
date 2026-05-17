//! Moderations API (`/v1/moderations`) — content-safety classification.

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::Result;

/// Accessor for the Moderations API. Obtained via [`Client::moderations`].
pub struct Moderations<'a> {
    client: &'a Client,
}

impl<'a> Moderations<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /moderations` — classify text or image content against OpenAI's safety taxonomy.
    /// Per-category scores live in [`ModerationResult::categories`] and
    /// [`ModerationResult::category_scores`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "moderations"))
    )]
    pub async fn create(&self, req: ModerationRequest) -> Result<ModerationResponse> {
        super::post_json(self.client, "/moderations", &req).await
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModerationRequest {
    pub input: ModerationInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ModerationInput {
    Text(String),
    Texts(Vec<String>),
    Items(Vec<ModerationItem>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModerationItem {
    Text { text: String },
    ImageUrl { image_url: ImageUrlContent },
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageUrlContent {
    pub url: String,
}

impl From<&str> for ModerationInput {
    fn from(s: &str) -> Self {
        ModerationInput::Text(s.to_string())
    }
}
impl From<String> for ModerationInput {
    fn from(s: String) -> Self {
        ModerationInput::Text(s)
    }
}
impl From<Vec<String>> for ModerationInput {
    fn from(s: Vec<String>) -> Self {
        ModerationInput::Texts(s)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModerationResponse {
    pub id: String,
    pub model: String,
    pub results: Vec<ModerationResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModerationResult {
    pub flagged: bool,
    pub categories: serde_json::Value,
    pub category_scores: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_applied_input_types: Option<serde_json::Value>,
}
