//! Per-request overrides applied on top of [`Client`](crate::Client) defaults.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// Optional per-request overrides. Apply via [`Client::with_options`](crate::Client::with_options)
/// (or shorthand methods `Client::with_timeout` / `Client::with_max_retries` /
/// `Client::with_idempotency_key` / `Client::with_header`).
///
/// Fields set on `RequestOptions` win over `Client` defaults; un-set fields fall through
/// to the client.
///
/// # Scope notes
///
/// - **`timeout`** applies to every HTTP code path (JSON, multipart, streaming, raw bytes).
/// - **`max_retries`** applies **only to JSON requests**. Multipart uploads, streaming SSE
///   responses, and raw-bytes endpoints are single-shot — bodies aren't replayable and a
///   stream can't be resumed mid-event.
/// - **`extra_headers`** and **`idempotency_key`** are attached to every request shape.
///
/// ```ignore
/// let resp = client
///     .with_timeout(std::time::Duration::from_secs(30))
///     .with_max_retries(5)
///     .chat()
///     .create(payload)
///     .await?;
/// ```
#[derive(Default, Clone, Debug)]
pub struct RequestOptions {
    pub timeout: Option<Duration>,
    pub max_retries: Option<u32>,
    pub extra_headers: HeaderMap,
    pub idempotency_key: Option<String>,
}

impl RequestOptions {
    /// Construct an empty `RequestOptions`.
    ///
    /// ```
    /// use std::time::Duration;
    /// use open_ai_rust::RequestOptions;
    /// let opts = RequestOptions::new()
    ///     .timeout(Duration::from_secs(30))
    ///     .max_retries(5)
    ///     .header("x-trace-id", "abc-123")
    ///     .idempotency_key("evt-1");
    /// assert_eq!(opts.max_retries, Some(5));
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = Some(t);
        self
    }

    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = Some(n);
        self
    }

    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(n), Ok(v)) = (
            HeaderName::from_bytes(name.as_ref().as_bytes()),
            HeaderValue::from_str(value.as_ref()),
        ) {
            self.extra_headers.insert(n, v);
        }
        self
    }
}
