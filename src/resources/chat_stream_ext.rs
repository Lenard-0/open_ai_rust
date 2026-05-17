//! Helpers for working with a chat completion stream.

use std::pin::Pin;

use futures_core::Stream;
use futures_util::StreamExt;

use crate::error::Result;
use crate::logoi::output::stream::ChatCompletionChunk;

/// Collected, assembled output of a streamed chat completion.
#[derive(Debug, Clone, Default)]
pub struct CollectedChatStream {
    pub content: String,
    pub refusal: Option<String>,
    pub tool_calls: Vec<CollectedToolCall>,
    pub finish_reason: Option<String>,
    pub model: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CollectedToolCall {
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments: String,
}

/// Extension methods on a `Stream` of [`ChatCompletionChunk`]s.
pub async fn collect_chat_stream(
    mut stream: Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>>,
) -> Result<CollectedChatStream> {
    let mut out = CollectedChatStream::default();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        out.id.get_or_insert(chunk.id.clone());
        out.model.get_or_insert(chunk.model.clone());

        if let Some(choice) = chunk.choices.first() {
            if let Some(text) = &choice.delta.content {
                out.content.push_str(text);
            }
            if let Some(r) = &choice.delta.refusal {
                out.refusal.get_or_insert_with(String::new).push_str(r);
            }
            if let Some(deltas) = &choice.delta.tool_calls {
                for d in deltas {
                    let idx = d.index as usize;
                    while out.tool_calls.len() <= idx {
                        out.tool_calls.push(CollectedToolCall::default());
                    }
                    let slot = &mut out.tool_calls[idx];
                    if let Some(id) = &d.id {
                        slot.id = Some(id.clone());
                    }
                    if let Some(f) = &d.function {
                        if let Some(name) = &f.name {
                            slot.name = Some(name.clone());
                        }
                        if let Some(args) = &f.arguments {
                            slot.arguments.push_str(args);
                        }
                    }
                }
            }
            if let Some(fr) = &choice.finish_reason {
                out.finish_reason = Some(fr.clone());
            }
        }
    }
    Ok(out)
}
