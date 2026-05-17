//! Images API — generation, editing, variations. Supports `dall-e-2`, `dall-e-3`, and
//! `gpt-image-1`.

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::Result;

/// Accessor for the Images API. Obtained via [`Client::images`].
pub struct Images<'a> {
    client: &'a Client,
}

impl<'a> Images<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// `POST /images/generations` — synthesise one or more images from a text prompt.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "images.generate"))
    )]
    pub async fn generate(&self, req: ImageGenerationRequest) -> Result<ImageResponse> {
        super::post_json(self.client, "/images/generations", &req).await
    }

    /// `POST /images/edits` — inpaint or modify an existing image. `mask` (when supplied)
    /// marks the editable region; transparent pixels are repainted from the prompt.
    /// `dall-e-2` only at the time of writing.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "images.edit"))
    )]
    pub async fn edit(&self, req: ImageEditRequest) -> Result<ImageResponse> {
        let form = build_edit_form(req)?;
        super::post_multipart(self.client, "/images/edits", form).await
    }

    /// `POST /images/variations` — produce stylistic variations of an existing image
    /// (no text prompt). `dall-e-2` only.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "images.variations"))
    )]
    pub async fn variations(&self, req: ImageVariationRequest) -> Result<ImageResponse> {
        let form = build_variation_form(req)?;
        super::post_multipart(self.client, "/images/variations", form).await
    }
}

// --- generations ------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

pub struct ImageGenerationBuilder {
    inner: ImageGenerationRequest,
}

impl ImageGenerationBuilder {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            inner: ImageGenerationRequest {
                prompt: prompt.into(),
                model: None,
                n: None,
                size: None,
                quality: None,
                style: None,
                response_format: None,
                background: None,
                output_format: None,
                output_compression: None,
                moderation: None,
                user: None,
            },
        }
    }
    pub fn model(mut self, m: impl Into<String>) -> Self {
        self.inner.model = Some(m.into());
        self
    }
    pub fn n(mut self, v: u32) -> Self {
        self.inner.n = Some(v);
        self
    }
    pub fn size(mut self, v: impl Into<String>) -> Self {
        self.inner.size = Some(v.into());
        self
    }
    pub fn quality(mut self, v: impl Into<String>) -> Self {
        self.inner.quality = Some(v.into());
        self
    }
    pub fn style(mut self, v: impl Into<String>) -> Self {
        self.inner.style = Some(v.into());
        self
    }
    pub fn response_format(mut self, v: impl Into<String>) -> Self {
        self.inner.response_format = Some(v.into());
        self
    }
    pub fn background(mut self, v: impl Into<String>) -> Self {
        self.inner.background = Some(v.into());
        self
    }
    pub fn output_format(mut self, v: impl Into<String>) -> Self {
        self.inner.output_format = Some(v.into());
        self
    }
    pub fn output_compression(mut self, v: u32) -> Self {
        self.inner.output_compression = Some(v);
        self
    }
    pub fn moderation(mut self, v: impl Into<String>) -> Self {
        self.inner.moderation = Some(v.into());
        self
    }
    pub fn user(mut self, v: impl Into<String>) -> Self {
        self.inner.user = Some(v.into());
        self
    }
    pub fn build(self) -> ImageGenerationRequest {
        self.inner
    }
}

// --- edits / variations -----------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ImageEditRequest {
    pub image: Vec<u8>,
    pub image_name: String,
    pub mask: Option<Vec<u8>>,
    pub mask_name: Option<String>,
    pub prompt: String,
    pub model: Option<String>,
    pub n: Option<u32>,
    pub size: Option<String>,
    pub response_format: Option<String>,
    pub user: Option<String>,
}

fn build_edit_form(req: ImageEditRequest) -> Result<reqwest::multipart::Form> {
    let image_part = reqwest::multipart::Part::bytes(req.image)
        .file_name(req.image_name)
        .mime_str("image/png")
        .map_err(|e| crate::error::OpenAiError::config(format!("bad mime: {e}")))?;
    let mut form = reqwest::multipart::Form::new()
        .text("prompt", req.prompt)
        .part("image", image_part);
    if let (Some(mask), Some(name)) = (req.mask, req.mask_name) {
        let mask_part = reqwest::multipart::Part::bytes(mask)
            .file_name(name)
            .mime_str("image/png")
            .map_err(|e| crate::error::OpenAiError::config(format!("bad mime: {e}")))?;
        form = form.part("mask", mask_part);
    }
    if let Some(m) = req.model {
        form = form.text("model", m);
    }
    if let Some(n) = req.n {
        form = form.text("n", n.to_string());
    }
    if let Some(s) = req.size {
        form = form.text("size", s);
    }
    if let Some(r) = req.response_format {
        form = form.text("response_format", r);
    }
    if let Some(u) = req.user {
        form = form.text("user", u);
    }
    Ok(form)
}

#[derive(Debug, Clone, Default)]
pub struct ImageVariationRequest {
    pub image: Vec<u8>,
    pub image_name: String,
    pub model: Option<String>,
    pub n: Option<u32>,
    pub size: Option<String>,
    pub response_format: Option<String>,
    pub user: Option<String>,
}

fn build_variation_form(req: ImageVariationRequest) -> Result<reqwest::multipart::Form> {
    let image_part = reqwest::multipart::Part::bytes(req.image)
        .file_name(req.image_name)
        .mime_str("image/png")
        .map_err(|e| crate::error::OpenAiError::config(format!("bad mime: {e}")))?;
    let mut form = reqwest::multipart::Form::new().part("image", image_part);
    if let Some(m) = req.model {
        form = form.text("model", m);
    }
    if let Some(n) = req.n {
        form = form.text("n", n.to_string());
    }
    if let Some(s) = req.size {
        form = form.text("size", s);
    }
    if let Some(r) = req.response_format {
        form = form.text("response_format", r);
    }
    if let Some(u) = req.user {
        form = form.text("user", u);
    }
    Ok(form)
}

// --- response ---------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageResponse {
    pub created: i64,
    pub data: Vec<ImageData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}
