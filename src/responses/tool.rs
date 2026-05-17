//! Tools accepted by the Responses API (broader than chat — also `web_search`,
//! `file_search`, `computer_use_preview`, etc.).

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseTool {
    Function {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        parameters: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    WebSearchPreview {
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<serde_json::Value>,
    },
    FileSearch {
        vector_store_ids: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_num_results: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ranking_options: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filters: Option<serde_json::Value>,
    },
    ComputerUsePreview {
        display_width: u32,
        display_height: u32,
        environment: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseToolChoice {
    /// `"none"` | `"auto"` | `"required"`.
    Mode(String),
    /// Force a hosted tool (e.g. `{ type: "web_search_preview" }`).
    Hosted {
        #[serde(rename = "type")]
        type_: String,
    },
    /// Force a function tool by name.
    Function {
        #[serde(rename = "type")]
        type_: String,
        name: String,
    },
}

impl ResponseToolChoice {
    pub fn auto() -> Self {
        Self::Mode("auto".into())
    }
    pub fn none() -> Self {
        Self::Mode("none".into())
    }
    pub fn required() -> Self {
        Self::Mode("required".into())
    }
    pub fn function(name: impl Into<String>) -> Self {
        Self::Function {
            type_: "function".into(),
            name: name.into(),
        }
    }
}

impl From<String> for ResponseToolChoice {
    fn from(s: String) -> Self {
        ResponseToolChoice::Mode(s)
    }
}
impl From<&str> for ResponseToolChoice {
    fn from(s: &str) -> Self {
        ResponseToolChoice::Mode(s.to_string())
    }
}
