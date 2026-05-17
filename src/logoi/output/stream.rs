//! Chat Completions streaming response (`stream: true`).

use serde::{Deserialize, Serialize};

use crate::logoi::output::Usage;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub choices: Vec<ChunkChoice>,
    /// Final chunk includes usage when `stream_options.include_usage = true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChunkChoice {
    pub index: i32,
    pub delta: AssistantDelta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AssistantDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ToolCallDelta {
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub type_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionDelta>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FunctionDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

impl ChatCompletionChunk {
    /// Concatenate `delta.content` from all choices (typically one choice).
    pub fn delta_text(&self) -> String {
        self.choices
            .iter()
            .filter_map(|c| c.delta.content.clone())
            .collect()
    }
}
