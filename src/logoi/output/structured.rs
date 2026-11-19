//! Helpers for reading a structured-output (`response_format: json_schema`) reply.

use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

use super::AiMsgResponse;

/// Why a structured-output reply could not be turned into JSON.
///
/// Returned by [`AiMsgResponse::structured_json`] and [`AiMsgResponse::parse_structured`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StructuredOutputError {
    /// The response had no `choices`.
    #[error("structured output: response has no choices")]
    NoChoices,

    /// The model refused (`choices[0].message.refusal` was set).
    #[error("structured output: model refused: {0}")]
    Refusal(String),

    /// `finish_reason` was `"length"`: the output hit `max_completion_tokens` (or the context
    /// limit) and the JSON is cut off. Raise the limit or lower `reasoning_effort`.
    #[error("structured output: truncated (finish_reason = \"length\"); content: {content:?}")]
    Truncated { content: Option<String> },

    /// `finish_reason` was `"content_filter"`.
    #[error("structured output: blocked by content filter")]
    ContentFilter,

    /// `choices[0].message.content` was missing or empty.
    #[error("structured output: message content is empty (finish_reason = {finish_reason:?})")]
    EmptyContent { finish_reason: String },

    /// The content was not valid JSON, or did not deserialise into the requested type.
    #[error("structured output: invalid JSON: {source}; content: {content:?}")]
    InvalidJson {
        #[source]
        source: serde_json::Error,
        content: String,
    },
}

impl From<StructuredOutputError> for String {
    fn from(e: StructuredOutputError) -> String {
        e.to_string()
    }
}

impl AiMsgResponse {
    /// The raw `choices[0].message.content` of a structured-output reply, after checking for a
    /// refusal, `finish_reason: "length"` / `"content_filter"`, and empty content.
    pub fn structured_content(&self) -> Result<&str, StructuredOutputError> {
        let choice = self
            .choices
            .first()
            .ok_or(StructuredOutputError::NoChoices)?;
        if let Some(refusal) = &choice.message.refusal {
            return Err(StructuredOutputError::Refusal(refusal.clone()));
        }
        match choice.finish_reason.as_str() {
            "length" => {
                return Err(StructuredOutputError::Truncated {
                    content: choice.message.content.clone(),
                })
            }
            "content_filter" => return Err(StructuredOutputError::ContentFilter),
            _ => {}
        }
        match choice.message.content.as_deref() {
            Some(c) if !c.trim().is_empty() => Ok(c),
            _ => Err(StructuredOutputError::EmptyContent {
                finish_reason: choice.finish_reason.clone(),
            }),
        }
    }

    /// Parse `choices[0].message.content` as JSON. See [`Self::structured_content`] for the
    /// checks run first.
    ///
    /// ```
    /// use open_ai_rust::AiMsgResponse;
    /// let resp: AiMsgResponse = serde_json::from_value(serde_json::json!({
    ///     "id": "x", "object": "chat.completion", "created": 0, "model": "m",
    ///     "choices": [{ "index": 0, "finish_reason": "stop",
    ///         "message": { "role": "assistant", "content": "{\"ok\":true}" } }],
    ///     "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 }
    /// })).unwrap();
    /// assert_eq!(resp.structured_json().unwrap()["ok"], true);
    /// ```
    pub fn structured_json(&self) -> Result<Value, StructuredOutputError> {
        self.parse_structured()
    }

    /// Deserialise `choices[0].message.content` into `T`. See [`Self::structured_content`] for
    /// the checks run first.
    pub fn parse_structured<T: DeserializeOwned>(&self) -> Result<T, StructuredOutputError> {
        let content = self.structured_content()?;
        serde_json::from_str(content).map_err(|source| StructuredOutputError::InvalidJson {
            source,
            content: content.to_string(),
        })
    }
}
