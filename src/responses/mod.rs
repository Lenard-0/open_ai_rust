//! Responses API — OpenAI's stateful agentic flagship endpoint (`/v1/responses`).
//!
//! Mirrors the official SDK's `client.responses` namespace.

pub mod input;
pub mod output;
pub mod request;
pub mod resource;
pub mod stream;
pub mod tool;

pub use input::{ResponseInput, ResponseInputContent, ResponseInputItem};
pub use output::{ResponseObject, ResponseOutputContentPart, ResponseOutputItem, ResponseStatus};
pub use request::{ReasoningConfig, ResponseRequest, ResponseRequestBuilder, TextConfig};
pub use resource::Responses;
pub use stream::ResponseStreamEvent;
pub use tool::{ResponseTool, ResponseToolChoice};
