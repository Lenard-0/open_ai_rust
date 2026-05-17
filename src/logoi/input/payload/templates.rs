use crate::logoi::{
    input::tool::{FunctionCall, ToolChoice, ToolType},
    message::ChatMessage,
    models::OpenAiModel,
};

use super::{ChatPayLoad, ChatToolChoice};

/// Shorthand templates for common request shapes. Prefer [`crate::PayLoadBuilder`] for full control.
pub enum PayLoadTemplates {
    Chat(QuickChatTemplate),
    FunctionCall(QuickFunctionCallTemplate),
}

impl PayLoadTemplates {
    pub fn default(messages: Vec<ChatMessage>) -> Self {
        Self::Chat(QuickChatTemplate::default(messages))
    }

    pub fn to_payload(self) -> ChatPayLoad {
        match self {
            Self::Chat(template) => template.to_payload(),
            Self::FunctionCall(template) => {
                let mut payload = ChatPayLoad::new(template.model, template.messages);
                payload.tools = template.tools;
                payload.tool_choice = Some(match template.tool_choice {
                    Some(fc) => ChatToolChoice::function(fc.name),
                    None => ChatToolChoice::auto(),
                });
                payload
            }
        }
    }
}

pub struct QuickChatTemplate {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessage>,
}

impl QuickChatTemplate {
    pub fn default(messages: Vec<ChatMessage>) -> Self {
        Self {
            model: OpenAiModel::GPT4oMini,
            messages,
        }
    }

    pub fn to_payload(self) -> ChatPayLoad {
        ChatPayLoad::new(self.model, self.messages)
    }
}

pub struct QuickFunctionCallTemplate {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessage>,
    pub tools: Option<Vec<ToolChoice>>,
    pub tool_choice: Option<FunctionCall>,
}

impl QuickFunctionCallTemplate {
    pub fn default(
        messages: Vec<ChatMessage>,
        functions: Vec<FunctionCall>,
        tool_choice: Option<FunctionCall>,
    ) -> Self {
        let tools: Vec<ToolChoice> = functions
            .into_iter()
            .map(|f| ToolChoice {
                function: f,
                _type: ToolType::Function,
            })
            .collect();
        Self {
            model: OpenAiModel::GPT4oMini,
            messages,
            tools: Some(tools),
            tool_choice,
        }
    }
}
