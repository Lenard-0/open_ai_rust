//! Error type returned by every fallible call in the crate.

use serde::Deserialize;
use thiserror::Error;

/// Shorthand for `std::result::Result<T, OpenAiError>`.
pub type Result<T> = std::result::Result<T, OpenAiError>;

/// All failure modes the SDK can surface.
///
/// # Retry behaviour
///
/// JSON requests issued through [`Client`](crate::Client) retry automatically (with exponential
/// backoff) on the variants marked *retryable* below, up to
/// [`RequestOptions::max_retries`](crate::RequestOptions). Multipart uploads, raw-bytes endpoints,
/// and streaming SSE calls are single-shot and never retry — bodies and stream cursors cannot be
/// replayed.
#[derive(Debug, Error)]
pub enum OpenAiError {
    /// OpenAI returned a non-2xx HTTP response. `status`, `message`, `code`, `type_`, and `param`
    /// come from the OpenAI JSON error envelope (falling back to the raw body if it isn't JSON).
    ///
    /// **Retryable when `status == 429` or `500..=599`.** Otherwise the error is propagated.
    #[error("OpenAI API error ({status}): {message}{}{}",
        code.as_ref().map(|c| format!(" [code={}]", c)).unwrap_or_default(),
        type_.as_ref().map(|t| format!(" [type={}]", t)).unwrap_or_default()
    )]
    Api {
        status: u16,
        message: String,
        code: Option<String>,
        #[allow(non_snake_case)]
        type_: Option<String>,
        param: Option<String>,
    },

    /// Transport-level failure from `reqwest` (DNS, TLS, connect, read timeout, etc.).
    ///
    /// **Retryable when the underlying error is a connect / timeout / request-builder failure.**
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// The response body could not be deserialised into the expected type. Usually indicates an
    /// OpenAI schema change or a model returning a malformed structured output. Not retryable.
    #[error("JSON decode error: {0}")]
    Decode(#[from] serde_json::Error),

    /// A streaming response (SSE) ended prematurely or emitted a malformed event. Not retryable —
    /// the stream cursor cannot be resumed.
    #[error("Stream error: {0}")]
    Stream(String),

    /// Client misuse: missing API key, invalid base URL, Azure deployment not set, etc. Caught
    /// before any HTTP traffic. Not retryable.
    #[error("Config error: {0}")]
    Config(String),

    /// Local I/O failure (e.g. reading a file for a multipart upload). Not retryable.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl OpenAiError {
    /// Construct a [`Self::Config`] error from a string-like message.
    pub fn config(msg: impl Into<String>) -> Self {
        OpenAiError::Config(msg.into())
    }

    /// Construct a [`Self::Stream`] error from a string-like message.
    pub fn stream(msg: impl Into<String>) -> Self {
        OpenAiError::Stream(msg.into())
    }

    pub(crate) fn from_response_body(status: u16, body: &str) -> Self {
        if let Ok(envelope) = serde_json::from_str::<ApiErrorEnvelope>(body) {
            let e = envelope.error;
            return OpenAiError::Api {
                status,
                message: e.message,
                code: e.code,
                type_: e.r#type,
                param: e.param,
            };
        }
        OpenAiError::Api {
            status,
            message: body.to_string(),
            code: None,
            type_: None,
            param: None,
        }
    }
}

impl From<OpenAiError> for String {
    fn from(value: OpenAiError) -> Self {
        value.to_string()
    }
}

#[derive(Deserialize)]
struct ApiErrorEnvelope {
    error: ApiErrorBody,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    message: String,
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    param: Option<String>,
}
