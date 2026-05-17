//! Server-sent events emitted by the Responses API when `stream: true`.

use serde::{Deserialize, Serialize};

use crate::responses::output::{ResponseObject, ResponseOutputContentPart, ResponseOutputItem};

/// Streaming event from `POST /v1/responses` with `stream: true`.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ResponseStreamEvent {
    #[serde(rename = "response.created")]
    Created {
        response: ResponseObject,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.in_progress")]
    InProgress {
        response: ResponseObject,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.completed")]
    Completed {
        response: ResponseObject,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.failed")]
    Failed {
        response: ResponseObject,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.incomplete")]
    Incomplete {
        response: ResponseObject,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded {
        output_index: i32,
        item: ResponseOutputItem,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.output_item.done")]
    OutputItemDone {
        output_index: i32,
        item: ResponseOutputItem,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded {
        item_id: String,
        output_index: i32,
        content_index: i32,
        part: ResponseOutputContentPart,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.content_part.done")]
    ContentPartDone {
        item_id: String,
        output_index: i32,
        content_index: i32,
        part: ResponseOutputContentPart,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta {
        item_id: String,
        output_index: i32,
        content_index: i32,
        delta: String,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.output_text.done")]
    OutputTextDone {
        item_id: String,
        output_index: i32,
        content_index: i32,
        text: String,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.refusal.delta")]
    RefusalDelta {
        item_id: String,
        output_index: i32,
        content_index: i32,
        delta: String,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.refusal.done")]
    RefusalDone {
        item_id: String,
        output_index: i32,
        content_index: i32,
        refusal: String,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta {
        item_id: String,
        output_index: i32,
        delta: String,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone {
        item_id: String,
        output_index: i32,
        arguments: String,
        #[serde(default)]
        sequence_number: Option<i64>,
    },
    #[serde(rename = "error")]
    Error {
        code: Option<String>,
        message: String,
        #[serde(default)]
        param: Option<String>,
    },
    /// Any future event type we don't yet model is captured here as raw JSON.
    #[serde(other)]
    Other,
}
