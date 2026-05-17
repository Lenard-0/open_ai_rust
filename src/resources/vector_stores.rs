//! Vector Stores API (`/v1/vector_stores`) — file-backed retrieval indices used by the
//! Assistants / Responses APIs. All endpoints send the `OpenAI-Beta: assistants=v2` header.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::{OpenAiError, Result};

/// Accessor for the Vector Stores API. Obtained via [`Client::vector_stores`].
pub struct VectorStores<'a> {
    client: &'a Client,
}

impl<'a> VectorStores<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /vector_stores` — create a vector store, optionally seeded with `file_ids`.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "vector_stores.create"))
    )]
    pub async fn create(&self, req: VectorStoreCreateRequest) -> Result<VectorStore> {
        super::post_json(self.client, "/vector_stores", &req).await
    }

    /// `GET /vector_stores` — list every vector store owned by this API key.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "vector_stores.list"))
    )]
    pub async fn list(&self) -> Result<VectorStoreList> {
        get_json(self.client, "/vector_stores").await
    }

    /// `GET /vector_stores/{id}` — fetch a single store's metadata + file counts.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "vector_stores.retrieve")
        )
    )]
    pub async fn retrieve(&self, id: &str) -> Result<VectorStore> {
        get_json(self.client, &format!("/vector_stores/{}", id)).await
    }

    /// `DELETE /vector_stores/{id}` — delete a store. Member files are detached but not deleted.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "vector_stores.delete"))
    )]
    pub async fn delete(&self, id: &str) -> Result<DeletedVectorStore> {
        delete_json(self.client, &format!("/vector_stores/{}", id)).await
    }

    /// Sub-resource for managing files inside a specific vector store.
    pub fn files(&self, store_id: &str) -> VectorStoreFiles<'a> {
        VectorStoreFiles {
            client: self.client,
            store_id: store_id.to_string(),
        }
    }
}

async fn get_json<T: serde::de::DeserializeOwned>(client: &Client, path: &str) -> Result<T> {
    let url = client.build_url(path)?;
    let resp = client
        .http()
        .get(url)
        .headers(client.auth_headers())
        .header("OpenAI-Beta", "assistants=v2")
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(OpenAiError::from_response_body(status.as_u16(), &body));
    }
    Ok(serde_json::from_str(&body)?)
}

async fn delete_json<T: serde::de::DeserializeOwned>(client: &Client, path: &str) -> Result<T> {
    let url = client.build_url(path)?;
    let resp = client
        .http()
        .delete(url)
        .headers(client.auth_headers())
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(OpenAiError::from_response_body(status.as_u16(), &body));
    }
    Ok(serde_json::from_str(&body)?)
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct VectorStoreCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_strategy: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VectorStore {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    #[serde(default)]
    pub name: Option<String>,
    pub usage_bytes: u64,
    pub file_counts: serde_json::Value,
    pub status: String,
    #[serde(default)]
    pub expires_after: Option<serde_json::Value>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub last_active_at: Option<i64>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VectorStoreList {
    pub object: String,
    pub data: Vec<VectorStore>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeletedVectorStore {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}

/// Per-store file accessor. Obtained via [`VectorStores::files`].
pub struct VectorStoreFiles<'a> {
    client: &'a Client,
    store_id: String,
}

impl<'a> VectorStoreFiles<'a> {
    /// `POST /vector_stores/{store_id}/files` — attach a previously-uploaded file to this store.
    /// The file is chunked & embedded server-side; poll [`Self::retrieve`] until
    /// `status == "completed"`.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "vector_stores.files.create")
        )
    )]
    pub async fn create(&self, file_id: impl Into<String>) -> Result<VectorStoreFile> {
        let body = serde_json::json!({ "file_id": file_id.into() });
        super::post_json(
            self.client,
            &format!("/vector_stores/{}/files", self.store_id),
            &body,
        )
        .await
    }

    /// `GET /vector_stores/{store_id}/files` — list files attached to this store.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "vector_stores.files.list")
        )
    )]
    pub async fn list(&self) -> Result<VectorStoreFileList> {
        get_json(
            self.client,
            &format!("/vector_stores/{}/files", self.store_id),
        )
        .await
    }

    /// `GET /vector_stores/{store_id}/files/{file_id}` — fetch indexing status for one file.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "vector_stores.files.retrieve")
        )
    )]
    pub async fn retrieve(&self, file_id: &str) -> Result<VectorStoreFile> {
        get_json(
            self.client,
            &format!("/vector_stores/{}/files/{}", self.store_id, file_id),
        )
        .await
    }

    /// `DELETE /vector_stores/{store_id}/files/{file_id}` — detach a file from the store.
    /// Does not delete the underlying file.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "vector_stores.files.delete")
        )
    )]
    pub async fn delete(&self, file_id: &str) -> Result<DeletedVectorStoreFile> {
        delete_json(
            self.client,
            &format!("/vector_stores/{}/files/{}", self.store_id, file_id),
        )
        .await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VectorStoreFile {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub vector_store_id: String,
    pub status: String,
    #[serde(default)]
    pub last_error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VectorStoreFileList {
    pub object: String,
    pub data: Vec<VectorStoreFile>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeletedVectorStoreFile {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}
