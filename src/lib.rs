//! Comprehensive, idiomatic Rust SDK for the OpenAI API.
//!
//! # Quick start
//!
//! ```no_run
//! use open_ai_rust::{ChatMessage, Client, OpenAiModel, PayLoadBuilder};
//!
//! # async fn run() -> open_ai_rust::Result<()> {
//! let client = Client::from_env()?;            // reads OPENAI_API_KEY
//! let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
//!     .messages(vec![
//!         ChatMessage::system("You are helpful."),
//!         ChatMessage::user("Say hi."),
//!     ])
//!     .temperature(0.2)
//!     .build();
//!
//! let resp = client.chat().create(payload).await?;
//! println!("{}", resp.get_last_msg_text().unwrap_or_default());
//! # Ok(())
//! # }
//! ```
//!
//! # Resources
//!
//! The [`Client`] type exposes every OpenAI endpoint via short accessors:
//!
//! - [`Client::chat`] — Chat Completions (sync + streaming)
//! - [`Client::responses`] — Responses API (sync + streaming, with tool dispatch)
//! - [`Client::embeddings`] — `text-embedding-3-{small,large}` + legacy ada-002
//! - [`Client::audio`] — `transcriptions`, `translations`, `speech` (whisper / gpt-4o-tts)
//! - [`Client::images`] — `generations`, `edits`, `variations` (dall-e + gpt-image-1)
//! - [`Client::moderations`] — text + image moderation
//! - [`Client::files`] — multipart upload + retrieve/delete
//! - [`Client::models`] — list/retrieve/delete
//! - [`Client::batches`] — async batch API
//! - [`Client::vector_stores`] — vector store + nested files
//! - [`Client::fine_tuning`] — fine-tuning jobs + events + checkpoints
//! - [`Client::uploads`] — resumable uploads for files > 512 MB
//!
//! # Features
//!
//! - `rustls-tls` *(default)* — TLS via rustls
//! - `native-tls` — TLS via system OpenSSL
//! - `stream` *(default)* — streaming helpers
//! - `tool_registry` — `linkme`-backed dispatch slice for the `#[tool]` macro
//! - `tracing` — `debug!`/`warn!` on every HTTP request + retry, spans on each call
//! - `utoipa` — derive `ToSchema` on enums
//!
//! Azure is supported without any feature flag — use [`Client::azure`].
//!
//! # Migration
//!
//! See [`MIGRATION.md`](https://github.com/Lenard-0/open_ai_rust/blob/main/MIGRATION.md) for
//! 0.2 → 1.0 codemods.

pub mod client;
pub mod error;
pub mod logoi;
pub mod request_options;
pub mod resources;
pub mod responses;

#[cfg(feature = "tool_registry")]
pub mod tool_registry;

#[doc(hidden)]
pub mod __macro_support;

pub use client::{ApiKind, Client, ClientBuilder, DEFAULT_BASE_URL};
pub use error::{OpenAiError, Result};
pub use request_options::RequestOptions;

pub use logoi::input::payload::builder::PayLoadBuilder;
pub use logoi::input::payload::templates::{
    PayLoadTemplates, QuickChatTemplate, QuickFunctionCallTemplate,
};
pub use logoi::input::payload::{
    ChatPayLoad, ChatToolChoice, ChatToolChoiceFunction, ChatToolChoiceType, JsonSchemaSpec,
    ReasoningEffort, ResponseFormat, StreamOptions,
};
pub use logoi::input::tool::raw_macro::fn_macro::{FunctionCallRaw, FunctionParamRaw};
pub use logoi::input::tool::raw_macro::FunctionCallable;
pub use logoi::input::tool::{
    EnumValues, FunctionCall, FunctionParameter, FunctionType, FunctionVariant, ToolChoice,
    ToolType,
};
pub use logoi::message::{
    ChatContent, ChatMessage, ChatMessageRole, ContentPart, FileRefSpec, ImageUrlSpec,
    InputAudioSpec, ToolCall, ToolCallFunction, ToolCallType as MessageToolCallType,
};
pub use logoi::models::OpenAiModel;
pub use logoi::output::{
    AiMsgResponse, AiResponseMessage, AssistantDelta, ChatCompletionChunk, Choice, ChunkChoice,
    CompletionTokensDetails, FunctionCallRes, FunctionDelta, PromptTokensDetails, ToolCallDelta,
    ToolCallRes, Usage,
};
pub use resources::chat_stream_ext::{collect_chat_stream, CollectedChatStream, CollectedToolCall};
