//! Input shape for `POST /v1/responses`. Accepts either a plain string or an array of
//! typed items.

use serde::{Deserialize, Serialize};

/// Top-level `input` field: either a bare string or a list of items.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseInput {
    Text(String),
    Items(Vec<ResponseInputItem>),
}

impl From<String> for ResponseInput {
    fn from(s: String) -> Self {
        ResponseInput::Text(s)
    }
}
impl From<&str> for ResponseInput {
    fn from(s: &str) -> Self {
        ResponseInput::Text(s.to_string())
    }
}
impl From<Vec<ResponseInputItem>> for ResponseInput {
    fn from(v: Vec<ResponseInputItem>) -> Self {
        ResponseInput::Items(v)
    }
}

/// One element of an `input` items array.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseInputItem {
    /// User/assistant/system/developer message with typed content parts.
    Message {
        role: String,
        content: ResponseInputContent,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// Output of a function call (response to a prior `function_call` output item).
    FunctionCallOutput { call_id: String, output: String },
    /// File search result (rare — usually returned by tools, not user-supplied).
    ItemReference { id: String },
}

impl ResponseInputItem {
    pub fn user(text: impl Into<String>) -> Self {
        ResponseInputItem::Message {
            role: "user".into(),
            content: ResponseInputContent::Text(text.into()),
            name: None,
        }
    }
    pub fn system(text: impl Into<String>) -> Self {
        ResponseInputItem::Message {
            role: "system".into(),
            content: ResponseInputContent::Text(text.into()),
            name: None,
        }
    }
    pub fn developer(text: impl Into<String>) -> Self {
        ResponseInputItem::Message {
            role: "developer".into(),
            content: ResponseInputContent::Text(text.into()),
            name: None,
        }
    }
    pub fn assistant(text: impl Into<String>) -> Self {
        ResponseInputItem::Message {
            role: "assistant".into(),
            content: ResponseInputContent::Text(text.into()),
            name: None,
        }
    }
    pub fn function_call_output(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        ResponseInputItem::FunctionCallOutput {
            call_id: call_id.into(),
            output: output.into(),
        }
    }
}

/// Either a string or a list of typed parts.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseInputContent {
    Text(String),
    Parts(Vec<ResponseInputContentPart>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseInputContentPart {
    InputText {
        text: String,
    },
    InputImage {
        #[serde(skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    InputFile {
        #[serde(skip_serializing_if = "Option::is_none")]
        file_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_name: Option<String>,
    },
    InputAudio {
        input_audio: InputAudio,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InputAudio {
    pub data: String,
    pub format: String,
}
