use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::message::ChatMessage;

pub mod stream;
pub use stream::{AssistantDelta, ChatCompletionChunk, ChunkChoice, FunctionDelta, ToolCallDelta};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AiMsgResponse {
    pub choices: Vec<Choice>,
    pub created: i64,
    pub id: String,
    pub model: String,
    pub object: String,
    pub usage: Usage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
}

impl AiMsgResponse {
    /// Adapt response choices into [`ChatMessage`]s (for round-tripping into next request).
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.choices
            .iter()
            .map(|choice| {
                let mut msg = ChatMessage::new(
                    choice.message.role.clone().into(),
                    choice.message.content.clone().unwrap_or_default(),
                );
                if let Some(tcs) = &choice.message.tool_calls {
                    let calls: Vec<crate::logoi::message::ToolCall> = tcs
                        .iter()
                        .map(|tc| crate::logoi::message::ToolCall {
                            id: tc.id.clone().unwrap_or_default(),
                            type_: crate::logoi::message::ToolCallType::Function,
                            function: crate::logoi::message::ToolCallFunction {
                                name: tc.function.name.clone(),
                                // `Value::to_string` is infallible.
                            arguments: tc.function.arguments.to_string(),
                            },
                        })
                        .collect();
                    msg.tool_calls = Some(calls);
                }
                if let Some(r) = &choice.message.refusal {
                    msg.refusal = Some(r.clone());
                }
                msg
            })
            .collect()
    }

    pub fn get_last_msg_text(&self) -> Option<String> {
        self.choices.last().and_then(|c| c.message.content.clone())
    }

    pub fn get_tool_calls(&self) -> Vec<FunctionCallRes> {
        self.choices
            .iter()
            .flat_map(|choice| choice.message.tool_calls.clone().unwrap_or_default())
            .map(|tool_call| tool_call.function.clone())
            .collect()
    }

    pub fn get_first_tool_call_args(&self) -> Result<Value, String> {
        if let Some(choice) = self.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                if let Some(tool_call) = tool_calls.first() {
                    Ok(tool_call.get_args())
                } else {
                    Err("No tool calls found".to_string())
                }
            } else {
                Err("No tool calls found".to_string())
            }
        } else {
            Err("No choices found".to_string())
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Choice {
    pub finish_reason: String,
    pub index: i32,
    pub message: AiResponseMessage,
    #[serde(default)]
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Usage {
    #[serde(default)]
    pub completion_tokens: Option<i32>,
    pub prompt_tokens: i32,
    pub total_tokens: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: i32,
    #[serde(default)]
    pub audio_tokens: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct CompletionTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: i32,
    #[serde(default)]
    pub audio_tokens: i32,
    #[serde(default)]
    pub accepted_prediction_tokens: i32,
    #[serde(default)]
    pub rejected_prediction_tokens: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AiResponseMessage {
    #[serde(default)]
    pub content: Option<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRes>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ToolCallRes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub type_: Option<String>,
    pub function: FunctionCallRes,
}

impl ToolCallRes {
    pub fn get_args(&self) -> Value {
        json!(self.function.arguments)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionCallRes {
    pub name: String,
    #[serde(deserialize_with = "deserialize_json_string")]
    pub arguments: Value,
}

fn deserialize_json_string<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        Value::String(s) => serde_json::from_str(&s).map_err(D::Error::custom),
        other => Ok(other),
    }
}
