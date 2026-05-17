//! Resumable multipart upload API (`/v1/uploads`) for files larger than 512 MB.

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::{OpenAiError, Result};

/// Accessor for the Uploads API. Obtained via [`Client::uploads`].
pub struct Uploads<'a> {
    client: &'a Client,
}

impl<'a> Uploads<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /uploads` — declare a new resumable upload, returning an `id` to use with
    /// [`Self::add_part`] / [`Self::complete`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "uploads.create"))
    )]
    pub async fn create(&self, req: UploadCreateRequest) -> Result<Upload> {
        super::post_json(self.client, "/uploads", &req).await
    }

    /// `POST /uploads/{id}/parts` — upload one chunk. Capture the returned [`UploadPart::id`]
    /// — you'll need every part id in the same order when calling [`Self::complete`].
    /// Each part must be ≤ 64 MB.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "uploads.add_part"))
    )]
    pub async fn add_part(&self, upload_id: &str, data: Vec<u8>) -> Result<UploadPart> {
        let part = reqwest::multipart::Part::bytes(data).file_name("part");
        let form = reqwest::multipart::Form::new().part("data", part);
        super::post_multipart(self.client, &format!("/uploads/{}/parts", upload_id), form).await
    }

    /// `POST /uploads/{id}/complete` — assemble all uploaded parts (in the order given by
    /// `part_ids`) into the final file. After this, the resulting file appears in
    /// [`Files::list`](crate::resources::files::Files::list).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "uploads.complete"))
    )]
    pub async fn complete(&self, upload_id: &str, part_ids: Vec<String>) -> Result<Upload> {
        let body = serde_json::json!({ "part_ids": part_ids });
        super::post_json(
            self.client,
            &format!("/uploads/{}/complete", upload_id),
            &body,
        )
        .await
    }

    /// `POST /uploads/{id}/cancel` — abandon an in-progress upload. Already-uploaded parts are
    /// garbage-collected on the server.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "uploads.cancel"))
    )]
    pub async fn cancel(&self, upload_id: &str) -> Result<Upload> {
        let url = self
            .client
            .build_url(&format!("/uploads/{}/cancel", upload_id))?;
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
}

#[derive(Debug, Clone, Serialize)]
pub struct UploadCreateRequest {
    pub filename: String,
    pub purpose: String,
    pub bytes: u64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Upload {
    pub id: String,
    pub object: String,
    pub bytes: u64,
    pub created_at: i64,
    pub filename: String,
    pub purpose: String,
    pub status: String,
    pub expires_at: i64,
    #[serde(default)]
    pub file: Option<crate::resources::files::FileObject>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadPart {
    pub id: String,
    pub object: String,
    pub upload_id: String,
    pub created_at: i64,
}
