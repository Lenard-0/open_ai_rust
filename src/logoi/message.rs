use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatMessage {
    /// The role of the author of this message.
    pub role: ChatMessageRole,
    /// The contents of the message
    ///
    /// This is always required for all messages, except for when ChatGPT calls
    /// a function.
    pub content: String,
    /// The name of the user in a multi-user chat
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ChatMessageRole {
    System,
    User,
    Assistant,
    Function,
}

impl From<String> for ChatMessageRole {
    fn from(role: String) -> Self {
        match role.as_str() {
            "system" => ChatMessageRole::System,
            "user" => ChatMessageRole::User,
            "assistant" => ChatMessageRole::Assistant,
            "function" => ChatMessageRole::Function,
            _ => ChatMessageRole::User,
        }
    }
}