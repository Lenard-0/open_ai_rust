//! Response object returned by the Responses API.

use serde::{Deserialize, Serialize};

use crate::logoi::output::Usage;
use crate::responses::tool::ResponseTool;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseObject {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub model: String,
    pub status: ResponseStatus,
    #[serde(default)]
    pub output: Vec<ResponseOutputItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ResponseTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl ResponseObject {
    /// Concatenate all assistant text segments across all `message` output items.
    pub fn output_text(&self) -> String {
        let mut out = String::new();
        for item in &self.output {
            if let ResponseOutputItem::Message { content, .. } = item {
                for part in content {
                    if let ResponseOutputContentPart::OutputText { text, .. } = part {
                        out.push_str(text);
                    }
                }
            }
        }
        out
    }

    /// Collect every `function_call` output item.
    pub fn function_calls(&self) -> Vec<&ResponseFunctionCall> {
        self.output
            .iter()
            .filter_map(|i| {
                if let ResponseOutputItem::FunctionCall(fc) = i {
                    Some(fc)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Queued,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Incomplete,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseError {
    pub code: String,
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseOutputItem {
    Message {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        role: String,
        #[serde(default)]
        content: Vec<ResponseOutputContentPart>,
    },
    #[serde(rename = "function_call")]
    FunctionCall(ResponseFunctionCall),
    #[serde(rename = "function_call_output")]
    FunctionCallOutput {
        id: String,
        call_id: String,
        output: String,
    },
    Reasoning {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(default)]
        summary: Vec<serde_json::Value>,
        #[serde(default)]
        encrypted_content: Option<String>,
    },
    FileSearchCall {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(default)]
        queries: Vec<String>,
        #[serde(default)]
        results: Vec<serde_json::Value>,
    },
    WebSearchCall {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    /// Catch-all for tool-call variants not (yet) modelled here.
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseFunctionCall {
    pub id: String,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseOutputContentPart {
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<serde_json::Value>,
    },
    Refusal {
        refusal: String,
    },
    #[serde(other)]
    Other,
}
