//! Files API (`/v1/files`). For files larger than 512 MB, use
//! [`Uploads`](crate::resources::uploads::Uploads) instead.

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::{OpenAiError, Result};

/// Accessor for the Files API. Obtained via [`Client::files`].
pub struct Files<'a> {
    client: &'a Client,
}

impl<'a> Files<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /files` — multipart upload of a single file. `file_name` includes the extension
    /// and is what the API reports back as `filename`. The `purpose` controls which downstream
    /// APIs can consume the file.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "files.create"))
    )]
    pub async fn create(
        &self,
        bytes: Vec<u8>,
        file_name: impl Into<String>,
        purpose: FilePurpose,
    ) -> Result<FileObject> {
        let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name.into());
        let form = reqwest::multipart::Form::new()
            .text("purpose", purpose.as_str().to_string())
            .part("file", part);
        super::post_multipart(self.client, "/files", form).await
    }

    /// `GET /files` — list every file uploaded under this API key.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "files.list"))
    )]
    pub async fn list(&self) -> Result<FileList> {
        let url = self.client.build_url("/files")?;
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

    /// `GET /files/{id}` — fetch metadata for a single file.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "files.retrieve"))
    )]
    pub async fn retrieve(&self, id: &str) -> Result<FileObject> {
        let url = self.client.build_url(&format!("/files/{}", id))?;
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

    /// `DELETE /files/{id}` — delete a file. Idempotent.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "files.delete"))
    )]
    pub async fn delete(&self, id: &str) -> Result<DeletedFile> {
        let url = self.client.build_url(&format!("/files/{}", id))?;
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

    /// `GET /files/{id}/content` — download the raw file bytes.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "files.content"))
    )]
    pub async fn content(&self, id: &str) -> Result<bytes::Bytes> {
        let url = self.client.build_url(&format!("/files/{}/content", id))?;
        let resp = self
            .client
            .http()
            .get(url)
            .headers(self.client.auth_headers())
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OpenAiError::from_response_body(status.as_u16(), &body));
        }
        Ok(resp.bytes().await?)
    }
}

/// Declared use-case for an uploaded file. The API rejects requests that mix a file with the
/// wrong purpose (e.g. trying to use a `Batch` file for fine-tuning).
#[derive(Debug, Clone, Copy)]
pub enum FilePurpose {
    /// Assistants API knowledge files.
    Assistants,
    /// JSONL input for the Batches API.
    Batch,
    /// JSONL training data for fine-tuning.
    FineTune,
    /// Image inputs for vision models.
    Vision,
    /// Generic user data (legal-evidence-style retention).
    UserData,
    /// Eval datasets.
    Evals,
}

impl FilePurpose {
    /// Wire representation expected by the OpenAI API.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assistants => "assistants",
            Self::Batch => "batch",
            Self::FineTune => "fine-tune",
            Self::Vision => "vision",
            Self::UserData => "user_data",
            Self::Evals => "evals",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileObject {
    pub id: String,
    pub object: String,
    pub bytes: u64,
    pub created_at: i64,
    pub filename: String,
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_details: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileList {
    pub object: String,
    pub data: Vec<FileObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeletedFile {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}
