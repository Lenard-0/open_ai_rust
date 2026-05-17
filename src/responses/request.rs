use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::logoi::input::payload::ReasoningEffort;
use crate::logoi::models::OpenAiModel;
use crate::responses::input::ResponseInput;
use crate::responses::tool::{ResponseTool, ResponseToolChoice};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseRequest {
    pub model: OpenAiModel,
    pub input: ResponseInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ResponseTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ResponseToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TextConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<serde_json::Value>,
}

impl TextConfig {
    pub fn json_schema(name: impl Into<String>, schema: serde_json::Value) -> Self {
        Self {
            format: Some(serde_json::json!({
                "type": "json_schema",
                "name": name.into(),
                "schema": schema,
                "strict": true,
            })),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ReasoningConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// `"auto" | "concise" | "detailed"` etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

pub struct ResponseRequestBuilder {
    inner: ResponseRequest,
}

impl ResponseRequestBuilder {
    pub fn new(model: OpenAiModel, input: impl Into<ResponseInput>) -> Self {
        Self {
            inner: ResponseRequest {
                model,
                input: input.into(),
                instructions: None,
                previous_response_id: None,
                tools: None,
                tool_choice: None,
                parallel_tool_calls: None,
                max_output_tokens: None,
                temperature: None,
                top_p: None,
                user: None,
                metadata: None,
                store: None,
                stream: None,
                truncation: None,
                service_tier: None,
                include: None,
                text: None,
                reasoning: None,
            },
        }
    }

    pub fn instructions(mut self, v: impl Into<String>) -> Self {
        self.inner.instructions = Some(v.into());
        self
    }
    pub fn previous_response_id(mut self, v: impl Into<String>) -> Self {
        self.inner.previous_response_id = Some(v.into());
        self
    }
    pub fn tools(mut self, tools: Vec<ResponseTool>) -> Self {
        self.inner.tools = Some(tools);
        self
    }
    pub fn tool_choice(mut self, v: impl Into<ResponseToolChoice>) -> Self {
        self.inner.tool_choice = Some(v.into());
        self
    }
    pub fn parallel_tool_calls(mut self, v: bool) -> Self {
        self.inner.parallel_tool_calls = Some(v);
        self
    }
    pub fn max_output_tokens(mut self, v: i32) -> Self {
        self.inner.max_output_tokens = Some(v);
        self
    }
    pub fn temperature(mut self, v: f32) -> Self {
        self.inner.temperature = Some(v);
        self
    }
    pub fn top_p(mut self, v: f32) -> Self {
        self.inner.top_p = Some(v);
        self
    }
    pub fn user(mut self, v: impl Into<String>) -> Self {
        self.inner.user = Some(v.into());
        self
    }
    pub fn metadata(mut self, m: HashMap<String, String>) -> Self {
        self.inner.metadata = Some(m);
        self
    }
    pub fn store(mut self, v: bool) -> Self {
        self.inner.store = Some(v);
        self
    }
    pub fn truncation(mut self, v: impl Into<String>) -> Self {
        self.inner.truncation = Some(v.into());
        self
    }
    pub fn service_tier(mut self, v: impl Into<String>) -> Self {
        self.inner.service_tier = Some(v.into());
        self
    }
    pub fn include(mut self, v: Vec<String>) -> Self {
        self.inner.include = Some(v);
        self
    }
    pub fn text(mut self, v: TextConfig) -> Self {
        self.inner.text = Some(v);
        self
    }
    pub fn reasoning(mut self, v: ReasoningConfig) -> Self {
        self.inner.reasoning = Some(v);
        self
    }
    pub fn reasoning_effort(mut self, e: ReasoningEffort) -> Self {
        self.inner.reasoning = Some(ReasoningConfig {
            effort: Some(e),
            summary: self
                .inner
                .reasoning
                .as_ref()
                .and_then(|r| r.summary.clone()),
        });
        self
    }
    pub fn build(self) -> ResponseRequest {
        self.inner
    }
}
