//! Models API (`/v1/models`) — list / retrieve / delete (fine-tuned) models.

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::{OpenAiError, Result};

/// Accessor for the Models API. Obtained via [`Client::models`].
pub struct Models<'a> {
    client: &'a Client,
}

impl<'a> Models<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `GET /models` — list every model your API key has access to (built-in + fine-tuned).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "models.list"))
    )]
    pub async fn list(&self) -> Result<ModelList> {
        let url = self.client.build_url("/models")?;
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

    /// `GET /models/{id}` — fetch metadata for a single model.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "models.retrieve"))
    )]
    pub async fn retrieve(&self, id: &str) -> Result<ModelInfo> {
        let url = self.client.build_url(&format!("/models/{}", id))?;
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

    /// `DELETE /models/{id}` — delete a fine-tuned model. Only works on models your org owns;
    /// built-in models (`gpt-4o`, etc.) return 403.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "models.delete"))
    )]
    pub async fn delete(&self, id: &str) -> Result<DeletedModel> {
        let url = self.client.build_url(&format!("/models/{}", id))?;
        let resp = self
            .client
            .http()
            .delete(url)
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeletedModel {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}
