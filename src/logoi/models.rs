use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Curated list of OpenAI model IDs. Use [`OpenAiModel::Custom`] for anything not listed
/// (e.g. dated snapshots like `gpt-4o-2024-08-06`, or new releases).
///
/// See <https://platform.openai.com/docs/models> for the authoritative list.
#[derive(Deserialize, Debug, Clone)]
pub enum OpenAiModel {
    // Reasoning models
    O1,
    O1Mini,
    O1Pro,
    O3,
    O3Mini,
    O3Pro,
    O4Mini,

    // GPT-5 family
    GPT5,
    GPT5Mini,
    GPT5Nano,

    // GPT-4.5 / 4.1 family
    GPT45Preview,
    GPT41,
    GPT41Mini,
    GPT41Nano,

    // GPT-4o family
    GPT4o,
    GPT4oMini,
    GPT4oAudioPreview,
    GPT4oMiniAudioPreview,
    GPT4oTranscribe,
    GPT4oMiniTranscribe,
    GPT4oMiniTTS,

    // Older GPT-4
    GPT4Turbo,
    GPT4,

    // GPT-3.5
    GPT35Turbo,

    // Images
    GPTImage1,
    DALLE3,
    DALLE2,

    // Audio (Whisper / TTS)
    TTS1,
    TTS1HD,
    Whisper1,

    // Embeddings
    TextEmbedding3Large,
    TextEmbedding3Small,
    TextEmbeddingAda002,

    // Moderation
    OmniModerationLatest,
    TextModerationLatest,
    TextModerationStable,
    TextModeration007,

    // Legacy completions
    Babbage002,
    Davinci002,

    /// Escape hatch for any model not enumerated here (preferred over expanding this enum
    /// for every snapshot ID).
    Custom(String),
}

impl OpenAiModel {
    /// The wire-format model ID for this variant.
    ///
    /// ```
    /// use open_ai_rust::OpenAiModel;
    /// assert_eq!(OpenAiModel::GPT4oMini.as_str(), "gpt-4o-mini");
    /// assert_eq!(OpenAiModel::O3Mini.as_str(), "o3-mini");
    /// assert_eq!(OpenAiModel::Custom("gpt-4o-2024-08-06".into()).as_str(), "gpt-4o-2024-08-06");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            OpenAiModel::O1 => "o1",
            OpenAiModel::O1Mini => "o1-mini",
            OpenAiModel::O1Pro => "o1-pro",
            OpenAiModel::O3 => "o3",
            OpenAiModel::O3Mini => "o3-mini",
            OpenAiModel::O3Pro => "o3-pro",
            OpenAiModel::O4Mini => "o4-mini",

            OpenAiModel::GPT5 => "gpt-5",
            OpenAiModel::GPT5Mini => "gpt-5-mini",
            OpenAiModel::GPT5Nano => "gpt-5-nano",

            OpenAiModel::GPT45Preview => "gpt-4.5-preview",
            OpenAiModel::GPT41 => "gpt-4.1",
            OpenAiModel::GPT41Mini => "gpt-4.1-mini",
            OpenAiModel::GPT41Nano => "gpt-4.1-nano",

            OpenAiModel::GPT4o => "gpt-4o",
            OpenAiModel::GPT4oMini => "gpt-4o-mini",
            OpenAiModel::GPT4oAudioPreview => "gpt-4o-audio-preview",
            OpenAiModel::GPT4oMiniAudioPreview => "gpt-4o-mini-audio-preview",
            OpenAiModel::GPT4oTranscribe => "gpt-4o-transcribe",
            OpenAiModel::GPT4oMiniTranscribe => "gpt-4o-mini-transcribe",
            OpenAiModel::GPT4oMiniTTS => "gpt-4o-mini-tts",

            OpenAiModel::GPT4Turbo => "gpt-4-turbo",
            OpenAiModel::GPT4 => "gpt-4",
            OpenAiModel::GPT35Turbo => "gpt-3.5-turbo",

            OpenAiModel::GPTImage1 => "gpt-image-1",
            OpenAiModel::DALLE3 => "dall-e-3",
            OpenAiModel::DALLE2 => "dall-e-2",
            OpenAiModel::TTS1 => "tts-1",
            OpenAiModel::TTS1HD => "tts-1-hd",
            OpenAiModel::Whisper1 => "whisper-1",

            OpenAiModel::TextEmbedding3Large => "text-embedding-3-large",
            OpenAiModel::TextEmbedding3Small => "text-embedding-3-small",
            OpenAiModel::TextEmbeddingAda002 => "text-embedding-ada-002",

            OpenAiModel::OmniModerationLatest => "omni-moderation-latest",
            OpenAiModel::TextModerationLatest => "text-moderation-latest",
            OpenAiModel::TextModerationStable => "text-moderation-stable",
            OpenAiModel::TextModeration007 => "text-moderation-007",

            OpenAiModel::Babbage002 => "babbage-002",
            OpenAiModel::Davinci002 => "davinci-002",

            OpenAiModel::Custom(s) => s.as_str(),
        }
    }
}

impl From<&str> for OpenAiModel {
    fn from(s: &str) -> Self {
        OpenAiModel::Custom(s.to_string())
    }
}

impl From<String> for OpenAiModel {
    fn from(s: String) -> Self {
        OpenAiModel::Custom(s)
    }
}

impl Display for OpenAiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for OpenAiModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
