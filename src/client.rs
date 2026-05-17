use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::error::{OpenAiError, Result};
use crate::request_options::RequestOptions;

pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Debug, Clone)]
pub enum ApiKind {
    OpenAi,
    /// Azure OpenAI requires `api-key` header (not Bearer) and `api-version` query param.
    /// `endpoint` should be the bare resource URL e.g. `https://my-resource.openai.azure.com`.
    Azure {
        api_version: String,
    },
}

#[derive(Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientInner>,
    pub(crate) per_request: RequestOptions,
}

pub(crate) struct ClientInner {
    pub http: reqwest::Client,
    pub api_key: String,
    pub base_url: String,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub default_headers: HeaderMap,
    pub api_kind: ApiKind,
    pub max_retries: u32,
    /// Azure deployment fallback used when caller hits a chat/embeddings endpoint and
    /// `base_url` looks like an Azure resource root.
    pub azure_deployment: Option<String>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.inner.base_url)
            .field("organization", &self.inner.organization)
            .field("project", &self.inner.project)
            .field("api_kind", &self.inner.api_kind)
            .finish()
    }
}

impl Client {
    /// Construct a client from a raw API key. Uses the default OpenAI base URL.
    ///
    /// ```
    /// use open_ai_rust::Client;
    /// let client = Client::new("sk-test");
    /// assert_eq!(client.base_url(), "https://api.openai.com/v1");
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder().api_key(api_key).build_unchecked()
    }

    /// Begin a [`ClientBuilder`] for fully-customised construction (custom base URL,
    /// timeout, retries, default headers, org/project, etc.).
    ///
    /// ```
    /// use std::time::Duration;
    /// use open_ai_rust::Client;
    ///
    /// let client = Client::builder()
    ///     .api_key("sk-test")
    ///     .timeout(Duration::from_secs(30))
    ///     .max_retries(3)
    ///     .default_header("x-app", "my-cli")
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(client.api_key(), "sk-test");
    /// ```
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Build from env vars: `OPENAI_API_KEY` (required), `OPENAI_BASE_URL`,
    /// `OPENAI_ORG_ID`, `OPENAI_PROJECT_ID`.
    ///
    /// ```no_run
    /// use open_ai_rust::Client;
    /// # fn run() -> open_ai_rust::Result<()> {
    /// let client = Client::from_env()?;
    /// # let _ = client; Ok(())
    /// # }
    /// ```
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| OpenAiError::config("OPENAI_API_KEY env var not set"))?;
        let mut builder = ClientBuilder::default().api_key(api_key);
        if let Ok(v) = std::env::var("OPENAI_BASE_URL") {
            builder = builder.base_url(v);
        }
        if let Ok(v) = std::env::var("OPENAI_ORG_ID") {
            builder = builder.organization(v);
        }
        if let Ok(v) = std::env::var("OPENAI_PROJECT_ID") {
            builder = builder.project(v);
        }
        builder.build()
    }

    /// Construct a client targeting Azure OpenAI.
    /// `endpoint` is the resource URL (e.g. `https://my-resource.openai.azure.com`).
    ///
    /// Azure auth is `api-key` (not `Bearer`), and the deployment + `api-version` query
    /// string are appended automatically.
    ///
    /// ```
    /// use open_ai_rust::Client;
    /// let client = Client::azure(
    ///     "az-key",
    ///     "https://my-resource.openai.azure.com",
    ///     "gpt-4o-deployment",
    ///     "2024-10-01-preview",
    /// );
    /// assert!(client.base_url().contains("openai.azure.com"));
    /// ```
    pub fn azure(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        deployment: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self {
        Self::builder()
            .api_key(api_key)
            .base_url(endpoint)
            .azure(deployment, api_version)
            .build_unchecked()
    }

    pub fn api_key(&self) -> &str {
        &self.inner.api_key
    }

    pub fn base_url(&self) -> &str {
        &self.inner.base_url
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }

    /// Return a clone of this client with the given per-request options merged in.
    /// Subsequent `.chat()` / `.responses()` / etc. calls on the returned client honour
    /// the overrides.
    ///
    /// ```
    /// use std::time::Duration;
    /// use open_ai_rust::{Client, RequestOptions};
    ///
    /// let base = Client::new("sk-test");
    /// let scoped = base.with_options(
    ///     RequestOptions::new()
    ///         .timeout(Duration::from_secs(10))
    ///         .max_retries(5)
    ///         .idempotency_key("evt-42")
    /// );
    /// assert_eq!(scoped.api_key(), "sk-test");
    /// ```
    pub fn with_options(&self, opts: RequestOptions) -> Self {
        let mut merged = self.per_request.clone();
        if opts.timeout.is_some() {
            merged.timeout = opts.timeout;
        }
        if opts.max_retries.is_some() {
            merged.max_retries = opts.max_retries;
        }
        if opts.idempotency_key.is_some() {
            merged.idempotency_key = opts.idempotency_key;
        }
        for (k, v) in opts.extra_headers.iter() {
            merged.extra_headers.insert(k.clone(), v.clone());
        }
        Self {
            inner: self.inner.clone(),
            per_request: merged,
        }
    }

    /// Shorthand for [`Self::with_options`] setting just `timeout`.
    pub fn with_timeout(&self, t: Duration) -> Self {
        self.with_options(RequestOptions::new().timeout(t))
    }

    /// Shorthand for [`Self::with_options`] setting just `max_retries`.
    pub fn with_max_retries(&self, n: u32) -> Self {
        self.with_options(RequestOptions::new().max_retries(n))
    }

    /// Shorthand for [`Self::with_options`] setting just `idempotency_key`.
    pub fn with_idempotency_key(&self, k: impl Into<String>) -> Self {
        self.with_options(RequestOptions::new().idempotency_key(k))
    }

    /// Shorthand for [`Self::with_options`] adding a single header.
    pub fn with_header(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.with_options(RequestOptions::new().header(name, value))
    }

    pub(crate) fn effective_timeout(&self) -> Option<Duration> {
        self.per_request.timeout
    }

    pub(crate) fn effective_max_retries(&self) -> u32 {
        self.per_request
            .max_retries
            .unwrap_or(self.inner.max_retries)
    }

    pub(crate) fn extra_request_headers(&self) -> HeaderMap {
        let mut h = self.per_request.extra_headers.clone();
        if let Some(key) = &self.per_request.idempotency_key {
            if let Ok(v) = HeaderValue::from_str(key) {
                h.insert(HeaderName::from_static("idempotency-key"), v);
            }
        }
        h
    }

    /// Build the URL for a given path (path may or may not start with `/`).
    /// For OpenAI: `{base_url}{path}`. For Azure: `{base_url}/openai/deployments/{dep}{path}?api-version=...`.
    pub(crate) fn build_url(&self, path: &str) -> Result<String> {
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        match &self.inner.api_kind {
            ApiKind::OpenAi => Ok(format!(
                "{}{}",
                self.inner.base_url.trim_end_matches('/'),
                path
            )),
            ApiKind::Azure { api_version } => {
                let dep =
                    self.inner.azure_deployment.as_ref().ok_or_else(|| {
                        OpenAiError::config("Azure client requires deployment name")
                    })?;
                Ok(format!(
                    "{}/openai/deployments/{}{}?api-version={}",
                    self.inner.base_url.trim_end_matches('/'),
                    dep,
                    path,
                    api_version
                ))
            }
        }
    }

    pub(crate) fn auth_headers(&self) -> HeaderMap {
        let mut headers = self.inner.default_headers.clone();
        match &self.inner.api_kind {
            ApiKind::OpenAi => {
                if let Ok(v) = HeaderValue::from_str(&format!("Bearer {}", self.inner.api_key)) {
                    headers.insert(reqwest::header::AUTHORIZATION, v);
                }
                if let Some(org) = &self.inner.organization {
                    if let Ok(v) = HeaderValue::from_str(org) {
                        headers.insert(HeaderName::from_static("openai-organization"), v);
                    }
                }
                if let Some(proj) = &self.inner.project {
                    if let Ok(v) = HeaderValue::from_str(proj) {
                        headers.insert(HeaderName::from_static("openai-project"), v);
                    }
                }
            }
            ApiKind::Azure { .. } => {
                if let Ok(v) = HeaderValue::from_str(&self.inner.api_key) {
                    headers.insert(HeaderName::from_static("api-key"), v);
                }
            }
        }
        for (k, v) in self.extra_request_headers().iter() {
            headers.insert(k.clone(), v.clone());
        }
        headers
    }

    /// Access the Chat Completions API (`/v1/chat/completions`).
    pub fn chat(&self) -> crate::resources::chat::Chat<'_> {
        crate::resources::chat::Chat::new(self)
    }

    /// Access the Embeddings API (`/v1/embeddings`).
    pub fn embeddings(&self) -> crate::resources::embeddings::Embeddings<'_> {
        crate::resources::embeddings::Embeddings::new(self)
    }

    /// Access the Audio APIs — transcriptions, translations, and speech (TTS).
    pub fn audio(&self) -> crate::resources::audio::Audio<'_> {
        crate::resources::audio::Audio::new(self)
    }

    /// Access the Responses API (`/v1/responses`) — OpenAI's stateful agentic flagship.
    pub fn responses(&self) -> crate::responses::resource::Responses<'_> {
        crate::responses::resource::Responses::new(self)
    }

    /// Access the Images API — `generate`, `edit`, `variations`.
    pub fn images(&self) -> crate::resources::images::Images<'_> {
        crate::resources::images::Images::new(self)
    }

    /// Access the Moderations API (`/v1/moderations`).
    pub fn moderations(&self) -> crate::resources::moderations::Moderations<'_> {
        crate::resources::moderations::Moderations::new(self)
    }

    /// Access the Files API (`/v1/files`) — multipart upload, list, retrieve, delete, download.
    pub fn files(&self) -> crate::resources::files::Files<'_> {
        crate::resources::files::Files::new(self)
    }

    /// Access the Models API (`/v1/models`) — list / retrieve / delete fine-tuned models.
    pub fn models(&self) -> crate::resources::models::Models<'_> {
        crate::resources::models::Models::new(self)
    }

    /// Access the Batches API (`/v1/batches`) — async, discounted batch processing.
    pub fn batches(&self) -> crate::resources::batches::Batches<'_> {
        crate::resources::batches::Batches::new(self)
    }

    /// Access the Vector Stores API (`/v1/vector_stores`) and nested `.files(store_id)`.
    pub fn vector_stores(&self) -> crate::resources::vector_stores::VectorStores<'_> {
        crate::resources::vector_stores::VectorStores::new(self)
    }

    /// Access the Fine-Tuning API. Use `.jobs()` for job CRUD + events + checkpoints.
    pub fn fine_tuning(&self) -> crate::resources::fine_tuning::FineTuning<'_> {
        crate::resources::fine_tuning::FineTuning::new(self)
    }

    /// Access the Uploads API (`/v1/uploads`) — resumable multipart uploads for files >512 MB.
    pub fn uploads(&self) -> crate::resources::uploads::Uploads<'_> {
        crate::resources::uploads::Uploads::new(self)
    }
}

#[derive(Default)]
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    organization: Option<String>,
    project: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    default_headers: HeaderMap,
    api_kind: Option<ApiKind>,
    azure_deployment: Option<String>,
    http: Option<reqwest::Client>,
}

impl ClientBuilder {
    /// API key. Required for [`Self::build`]; optional for [`Self::build_unchecked`].
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Override the API base URL. Defaults to [`DEFAULT_BASE_URL`]. For Azure pass the resource
    /// root (e.g. `https://my-resource.openai.azure.com`) and call [`Self::azure`] as well.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the `OpenAI-Organization` header sent with every request.
    pub fn organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Set the `OpenAI-Project` header sent with every request.
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Default per-request HTTP timeout. Overridable per call via
    /// [`Client::with_timeout`].
    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = Some(t);
        self
    }

    /// Default number of retries on 429 / 5xx / connect errors. Applies only to JSON POSTs —
    /// multipart and streaming are single-shot. Overridable per call via
    /// [`Client::with_max_retries`].
    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = Some(n);
        self
    }

    /// Add a header sent on every request. Invalid header names / values are silently dropped.
    pub fn default_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(n), Ok(v)) = (
            HeaderName::from_bytes(name.as_ref().as_bytes()),
            HeaderValue::from_str(value.as_ref()),
        ) {
            self.default_headers.insert(n, v);
        }
        self
    }

    /// Configure this client for Azure OpenAI. Switches auth from `Authorization: Bearer` to
    /// `api-key`, and appends `?api-version=...` plus `/openai/deployments/{deployment}` to
    /// every request URL.
    pub fn azure(mut self, deployment: impl Into<String>, api_version: impl Into<String>) -> Self {
        self.api_kind = Some(ApiKind::Azure {
            api_version: api_version.into(),
        });
        self.azure_deployment = Some(deployment.into());
        self
    }

    /// Provide a pre-built `reqwest::Client`. Useful for sharing a connection pool, applying
    /// proxy / TLS settings, or injecting a mock for testing. If supplied, `timeout` on this
    /// builder is ignored — set it on the `reqwest::Client` instead.
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    /// Finalise the builder. Returns [`OpenAiError::Config`] if `api_key` is missing.
    pub fn build(self) -> Result<Client> {
        if self.api_key.is_none() {
            return Err(OpenAiError::config("api_key required"));
        }
        Ok(self.build_unchecked())
    }

    /// Like [`Self::build`] but does not require an `api_key`. Mostly useful for tests against
    /// a local mock server.
    pub fn build_unchecked(self) -> Client {
        let http = self.http.unwrap_or_else(|| {
            let mut b = reqwest::Client::builder();
            if let Some(t) = self.timeout {
                b = b.timeout(t);
            }
            b.build().unwrap_or_else(|_| reqwest::Client::new())
        });

        let base_url = self
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Client {
            inner: Arc::new(ClientInner {
                http,
                api_key: self.api_key.unwrap_or_default(),
                base_url,
                organization: self.organization,
                project: self.project,
                default_headers: self.default_headers,
                api_kind: self.api_kind.unwrap_or(ApiKind::OpenAi),
                max_retries: self.max_retries.unwrap_or(0),
                azure_deployment: self.azure_deployment,
            }),
            per_request: RequestOptions::default(),
        }
    }
}
