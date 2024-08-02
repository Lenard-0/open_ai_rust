use crate::logoi::{input::tool::FunctionCall, message::ChatMessage, models::OpenAiModel};

use super::ChatPayLoad;


/// This is a bunch of shorthand templates since you likely will not want to manually write out every setting,
/// but still want error handling to make sure you don't miss out on crucial features.
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
            Self::FunctionCall(template) => ChatPayLoad {
                model: template.model,
                messages: template.messages,
                tools: Some(template.tools),
                tool_choice: Some("auto".to_string()),
                frequency_penalty: None,
                logprobs: None,
                top_logprobs: None,
                max_tokens: None,
                n: None,
                presence_penalty: None,
                response_format: None,
                seed: None,
                service_tier: None,
                stop: None,
                stream: None,
                stream_options: None,
                temperature: None,
                top_p: None,
                user: None,
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
            model: OpenAiModel::GPT4o,
            messages
        }
    }

    pub fn to_payload(self) -> crate::logoi::input::payload::ChatPayLoad {
        ChatPayLoad {
            model: self.model,
            messages: self.messages,
            frequency_penalty: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            n: None,
            presence_penalty: None,
            response_format: None,
            seed: None,
            service_tier: None,
            stop: None,
            stream: None,
            stream_options: None,
            temperature: None,
            top_p: None,
            tools: None,
            user: None,
            tool_choice: None,
        }
    }
}

// let payload = json!({
//     "model": MODEL,
//     "messages": messages,
//     "tools": processed_functions,
//     "tool_choice": match tool_choice_name {
//         Some(tool_choice_name) => json!({
//             "type": "function",
//             "function": {
//                 "name": tool_choice_name
//             }
//         }),
//         None => json!("auto".to_string())
//     }
// });
pub struct QuickFunctionCallTemplate {
    pub model: OpenAiModel,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<FunctionCall>,
    pub tool_choice: Option<FunctionCall>,
}

impl QuickFunctionCallTemplate {
    pub fn default(messages: Vec<ChatMessage>, tools: Vec<FunctionCall>, tool_choice: Option<FunctionCall>) -> Self {
        Self {
            model: OpenAiModel::GPT4o,
            messages,
            tools,
            tool_choice
        }
    }
}