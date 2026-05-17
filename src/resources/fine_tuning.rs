//! Fine-Tuning API (`/v1/fine_tuning/jobs`).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::{OpenAiError, Result};

/// Accessor for the Fine-Tuning API. Obtained via [`Client::fine_tuning`].
pub struct FineTuning<'a> {
    client: &'a Client,
}

impl<'a> FineTuning<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Access fine-tuning job CRUD + event/checkpoint queries.
    pub fn jobs(&self) -> FineTuningJobs<'a> {
        FineTuningJobs {
            client: self.client,
        }
    }
}

/// Fine-tuning job sub-resource. Obtained via [`FineTuning::jobs`].
pub struct FineTuningJobs<'a> {
    client: &'a Client,
}

impl<'a> FineTuningJobs<'a> {
    /// `POST /fine_tuning/jobs` — start a new fine-tuning job. `training_file` must be a file
    /// uploaded with `purpose = FilePurpose::FineTune`.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "fine_tuning.jobs.create")
        )
    )]
    pub async fn create(&self, req: FineTuningJobRequest) -> Result<FineTuningJob> {
        super::post_json(self.client, "/fine_tuning/jobs", &req).await
    }

    /// `GET /fine_tuning/jobs` — list fine-tuning jobs.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "fine_tuning.jobs.list"))
    )]
    pub async fn list(&self) -> Result<FineTuningJobList> {
        get_json(self.client, "/fine_tuning/jobs").await
    }

    /// `GET /fine_tuning/jobs/{id}` — poll a job's status.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "fine_tuning.jobs.retrieve")
        )
    )]
    pub async fn retrieve(&self, id: &str) -> Result<FineTuningJob> {
        get_json(self.client, &format!("/fine_tuning/jobs/{}", id)).await
    }

    /// `POST /fine_tuning/jobs/{id}/cancel` — request cancellation; the job may still complete
    /// the in-progress training step.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "fine_tuning.jobs.cancel")
        )
    )]
    pub async fn cancel(&self, id: &str) -> Result<FineTuningJob> {
        let url = self
            .client
            .build_url(&format!("/fine_tuning/jobs/{}/cancel", id))?;
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

    /// `GET /fine_tuning/jobs/{id}/events` — fetch the lifecycle event log for a job. Useful for
    /// surfacing progress to a human operator.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "fine_tuning.jobs.events")
        )
    )]
    pub async fn list_events(&self, id: &str) -> Result<FineTuningEventList> {
        get_json(self.client, &format!("/fine_tuning/jobs/{}/events", id)).await
    }

    /// `GET /fine_tuning/jobs/{id}/checkpoints` — list intermediate model checkpoints emitted by
    /// the training job. Returned as raw `serde_json::Value` (response schema varies by model).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "fine_tuning.jobs.checkpoints")
        )
    )]
    pub async fn list_checkpoints(&self, id: &str) -> Result<serde_json::Value> {
        get_json(
            self.client,
            &format!("/fine_tuning/jobs/{}/checkpoints", id),
        )
        .await
    }
}

async fn get_json<T: serde::de::DeserializeOwned>(client: &Client, path: &str) -> Result<T> {
    let url = client.build_url(path)?;
    let resp = client
        .http()
        .get(url)
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

#[derive(Debug, Clone, Serialize)]
pub struct FineTuningJobRequest {
    pub model: String,
    pub training_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrations: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FineTuningJob {
    pub id: String,
    pub object: String,
    pub model: String,
    pub created_at: i64,
    pub status: String,
    #[serde(default)]
    pub finished_at: Option<i64>,
    #[serde(default)]
    pub fine_tuned_model: Option<String>,
    pub organization_id: String,
    #[serde(default)]
    pub result_files: Vec<String>,
    pub training_file: String,
    #[serde(default)]
    pub validation_file: Option<String>,
    #[serde(default)]
    pub hyperparameters: Option<serde_json::Value>,
    #[serde(default)]
    pub trained_tokens: Option<u64>,
    #[serde(default)]
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FineTuningJobList {
    pub object: String,
    pub data: Vec<FineTuningJob>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FineTuningEventList {
    pub object: String,
    pub data: Vec<serde_json::Value>,
    #[serde(default)]
    pub has_more: bool,
}
