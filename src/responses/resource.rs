use std::pin::Pin;

use futures_core::Stream;

use crate::client::Client;
use crate::error::{OpenAiError, Result};
use crate::responses::output::ResponseObject;
use crate::responses::request::ResponseRequest;
use crate::responses::stream::ResponseStreamEvent;

/// Accessor for the Responses API. Obtained via [`Client::responses`].
pub struct Responses<'a> {
    client: &'a Client,
}

impl<'a> Responses<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /responses` — create a new stateful response. Use
    /// [`ResponseRequestBuilder`](crate::responses::ResponseRequestBuilder) to construct the
    /// request body.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "responses.create"))
    )]
    pub async fn create(&self, req: ResponseRequest) -> Result<ResponseObject> {
        crate::resources::post_json(self.client, "/responses", &req).await
    }

    /// Streaming variant — sets `stream: true` and yields typed [`ResponseStreamEvent`]
    /// variants, one per server-sent event.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "responses.create_stream")
        )
    )]
    pub async fn create_stream(
        &self,
        mut req: ResponseRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ResponseStreamEvent>> + Send>>> {
        req.stream = Some(true);
        let body = serde_json::to_value(&req)?;
        crate::resources::stream::post_sse_stream(self.client, "/responses", body).await
    }

    /// `GET /responses/{id}` — fetch a previously-created response. Only works when the
    /// originating call had `store: true` (the default for the Responses API).
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "responses.retrieve"))
    )]
    pub async fn retrieve(&self, id: &str) -> Result<ResponseObject> {
        let url = self.client.build_url(&format!("/responses/{}", id))?;
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

    /// `DELETE /responses/{id}` — permanently remove a stored response.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "responses.delete"))
    )]
    pub async fn delete(&self, id: &str) -> Result<()> {
        let url = self.client.build_url(&format!("/responses/{}", id))?;
        let resp = self
            .client
            .http()
            .delete(url)
            .headers(self.client.auth_headers())
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OpenAiError::from_response_body(status.as_u16(), &body));
        }
        Ok(())
    }

    /// `POST /responses/{id}/cancel` — cancel a still-running response (e.g. one issued
    /// async with `background: true`). In-flight tool calls may still complete.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "responses.cancel"))
    )]
    pub async fn cancel(&self, id: &str) -> Result<ResponseObject> {
        let url = self
            .client
            .build_url(&format!("/responses/{}/cancel", id))?;
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
