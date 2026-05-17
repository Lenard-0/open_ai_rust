use core::fmt;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// A single message in the chat conversation.
///
/// Mirrors the OpenAI Chat Completions message schema:
/// - `role` — `system` / `user` / `assistant` / `tool` / `developer` / `function` (deprecated).
/// - `content` — either a plain string or a list of typed content parts (text/image/audio/file).
/// - `tool_call_id` — set when `role == Tool` to indicate which assistant tool call this responds to.
/// - `tool_calls` — set on an assistant message that is *calling* tools.
/// - `refusal` — assistant refusal text (when the model declines).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatMessageRole,
    pub content: ChatContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

impl ChatMessage {
    /// Construct a message with an arbitrary role + content.
    ///
    /// ```
    /// use open_ai_rust::{ChatMessage, ChatMessageRole};
    /// let m = ChatMessage::new(ChatMessageRole::User, "hi");
    /// assert_eq!(m.text(), "hi");
    /// ```
    pub fn new(role: ChatMessageRole, content: impl Into<ChatContent>) -> Self {
        Self {
            role,
            content: content.into(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            refusal: None,
        }
    }

    pub fn system(content: impl Into<ChatContent>) -> Self {
        Self::new(ChatMessageRole::System, content)
    }

    pub fn developer(content: impl Into<ChatContent>) -> Self {
        Self::new(ChatMessageRole::Developer, content)
    }

    /// User-role message.
    ///
    /// ```
    /// # use open_ai_rust::ChatMessage;
    /// let m = ChatMessage::user("hi");
    /// assert_eq!(m.text(), "hi");
    /// ```
    pub fn user(content: impl Into<ChatContent>) -> Self {
        Self::new(ChatMessageRole::User, content)
    }

    pub fn assistant(content: impl Into<ChatContent>) -> Self {
        Self::new(ChatMessageRole::Assistant, content)
    }

    /// Build a `tool` role message responding to a previous tool call.
    ///
    /// ```
    /// # use open_ai_rust::ChatMessage;
    /// let m = ChatMessage::tool("call_abc", r#"{"weather":"sunny"}"#);
    /// assert_eq!(m.tool_call_id.as_deref(), Some("call_abc"));
    /// ```
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<ChatContent>) -> Self {
        let mut m = Self::new(ChatMessageRole::Tool, content);
        m.tool_call_id = Some(tool_call_id.into());
        m
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_tool_calls(mut self, calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(calls);
        self
    }

    /// Append an `image_url` content part to this message, converting `content` into a
    /// multi-part array if needed.
    ///
    /// ```
    /// # use open_ai_rust::{ChatMessage, ChatContent};
    /// let m = ChatMessage::user("Describe this:")
    ///     .with_image("https://example.com/cat.png");
    /// assert!(matches!(m.content, ChatContent::Parts(_)));
    /// ```
    pub fn with_image(mut self, url: impl Into<String>) -> Self {
        let new_part = ContentPart::ImageUrl {
            image_url: ImageUrlSpec {
                url: url.into(),
                detail: None,
            },
        };
        let parts = match std::mem::replace(&mut self.content, ChatContent::Text(String::new())) {
            ChatContent::Text(t) if !t.is_empty() => vec![ContentPart::Text { text: t }, new_part],
            ChatContent::Text(_) => vec![new_part],
            ChatContent::Parts(mut parts) => {
                parts.push(new_part);
                parts
            }
        };
        self.content = ChatContent::Parts(parts);
        self
    }

    /// Convenience to get plain text content (joins parts if multi-part text-only).
    pub fn text(&self) -> String {
        match &self.content {
            ChatContent::Text(s) => s.clone(),
            ChatContent::Parts(parts) => parts
                .iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.clone()),
                    ContentPart::Refusal { refusal } => Some(refusal.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

/// Either a plain string or a list of typed parts. Serialised as the bare string if a
/// `Text` variant, or as an array if `Parts`.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ChatContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl Default for ChatContent {
    fn default() -> Self {
        ChatContent::Text(String::new())
    }
}

impl From<String> for ChatContent {
    fn from(s: String) -> Self {
        ChatContent::Text(s)
    }
}
impl From<&str> for ChatContent {
    fn from(s: &str) -> Self {
        ChatContent::Text(s.to_string())
    }
}
impl From<Vec<ContentPart>> for ChatContent {
    fn from(p: Vec<ContentPart>) -> Self {
        ChatContent::Parts(p)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    #[serde(rename = "image_url")]
    ImageUrl {
        image_url: ImageUrlSpec,
    },
    InputAudio {
        input_audio: InputAudioSpec,
    },
    File {
        file: FileRefSpec,
    },
    Refusal {
        refusal: String,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ImageUrlSpec {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InputAudioSpec {
    pub data: String,
    pub format: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileRefSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum ChatMessageRole {
    System,
    User,
    Assistant,
    Tool,
    Developer,
}

impl From<String> for ChatMessageRole {
    fn from(role: String) -> Self {
        match role.as_str() {
            "system" => ChatMessageRole::System,
            "user" => ChatMessageRole::User,
            "assistant" => ChatMessageRole::Assistant,
            "tool" => ChatMessageRole::Tool,
            "developer" => ChatMessageRole::Developer,
            _ => ChatMessageRole::User,
        }
    }
}

impl Display for ChatMessageRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ChatMessageRole::System => "system",
            ChatMessageRole::User => "user",
            ChatMessageRole::Assistant => "assistant",
            ChatMessageRole::Tool => "tool",
            ChatMessageRole::Developer => "developer",
        };
        f.write_str(s)
    }
}

/// Tool call emitted in an assistant message (input form — matches output form).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: ToolCallType,
    pub function: ToolCallFunction,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ToolCallType {
    Function,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ToolCallFunction {
    pub name: String,
    /// Arguments as raw JSON string (per OpenAI spec).
    pub arguments: String,
}
