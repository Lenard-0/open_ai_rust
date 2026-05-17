//! Batches API (`/v1/batches`) — async, 24-hour, discounted bulk requests.
//!
//! Typical flow: upload a JSONL file via [`Files::create`](crate::resources::files::Files::create)
//! with `purpose = FilePurpose::Batch`, then `create()` a batch pointing at the file ID, then
//! `retrieve()` until `status == "completed"`, then download the output file.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::{OpenAiError, Result};

/// Accessor for the Batches API. Obtained via [`Client::batches`].
pub struct Batches<'a> {
    client: &'a Client,
}

impl<'a> Batches<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /batches` — schedule a new batch job over a previously-uploaded input file.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "batches.create"))
    )]
    pub async fn create(&self, req: BatchCreateRequest) -> Result<Batch> {
        super::post_json(self.client, "/batches", &req).await
    }

    /// `GET /batches/{id}` — poll a batch for status / output file IDs.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "batches.retrieve"))
    )]
    pub async fn retrieve(&self, id: &str) -> Result<Batch> {
        let url = self.client.build_url(&format!("/batches/{}", id))?;
        let resp = self
            .client
            .http()
            .get(url)
            .headers(self.client.auth_headers())
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(OpenAiError::from_response_body(status.as_u16(), &body));
        }
        Ok(serde_json::from_str(&body)?)
    }

    /// `POST /batches/{id}/cancel` — request cancellation. In-flight items may still complete.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "batches.cancel"))
    )]
    pub async fn cancel(&self, id: &str) -> Result<Batch> {
        let url = self.client.build_url(&format!("/batches/{}/cancel", id))?;
        let resp = self
            .client
            .http()
            .post(url)
            .headers(self.client.auth_headers())
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(OpenAiError::from_response_body(status.as_u16(), &body));
        }
        Ok(serde_json::from_str(&body)?)
    }

    /// `GET /batches` — list batches in reverse chronological order.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "batches.list"))
    )]
    pub async fn list(&self) -> Result<BatchList> {
        let url = self.client.build_url("/batches")?;
        let resp = self
            .client
            .http()
            .get(url)
            .headers(self.client.auth_headers())
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(OpenAiError::from_response_body(status.as_u16(), &body));
        }
        Ok(serde_json::from_str(&body)?)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateRequest {
    pub input_file_id: String,
    pub endpoint: String,
    pub completion_window: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Batch {
    pub id: String,
    pub object: String,
    pub endpoint: String,
    pub input_file_id: String,
    pub completion_window: String,
    pub status: String,
    pub created_at: i64,
    #[serde(default)]
    pub output_file_id: Option<String>,
    #[serde(default)]
    pub error_file_id: Option<String>,
    #[serde(default)]
    pub in_progress_at: Option<i64>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub finalizing_at: Option<i64>,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub failed_at: Option<i64>,
    #[serde(default)]
    pub expired_at: Option<i64>,
    #[serde(default)]
    pub cancelling_at: Option<i64>,
    #[serde(default)]
    pub cancelled_at: Option<i64>,
    #[serde(default)]
    pub request_counts: Option<serde_json::Value>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(default)]
    pub errors: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchList {
    pub object: String,
    pub data: Vec<Batch>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}
