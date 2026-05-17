//! Audio APIs — transcription (speech → text), translation (speech → English text), and speech
//! synthesis (text → audio).

use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::error::Result;

/// Accessor for the Audio APIs. Obtained via [`Client::audio`].
pub struct Audio<'a> {
    client: &'a Client,
}

impl<'a> Audio<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Speech → text. Whisper or `gpt-4o-transcribe` family.
    pub fn transcriptions(&self) -> Transcriptions<'a> {
        Transcriptions {
            client: self.client,
        }
    }

    /// Speech → English text. Same Whisper model but always translates into English.
    pub fn translations(&self) -> Translations<'a> {
        Translations {
            client: self.client,
        }
    }

    /// Text → audio (TTS). `tts-1`, `tts-1-hd`, or `gpt-4o-mini-tts`.
    pub fn speech(&self) -> Speech<'a> {
        Speech {
            client: self.client,
        }
    }
}

// --- transcription ----------------------------------------------------------

/// `POST /audio/transcriptions` — speech to text.
pub struct Transcriptions<'a> {
    client: &'a Client,
}

/// Request body for transcription / translation. Use [`TranscriptionRequestBuilder`] for
/// ergonomic construction.
#[derive(Debug, Clone, Default)]
pub struct TranscriptionRequest {
    /// Audio file bytes.
    pub file: Vec<u8>,
    /// Filename to send in the multipart upload — file extension matters for some models.
    pub file_name: String,
    /// MIME type, e.g. `"audio/mpeg"`. Defaults to `audio/mpeg` if unset.
    pub mime_type: Option<String>,
    /// Model ID, e.g. `"whisper-1"` or `"gpt-4o-transcribe"`.
    pub model: String,
    /// Optional ISO-639-1 language hint. Improves accuracy & latency.
    pub language: Option<String>,
    /// Optional bias prompt for the transcriber (e.g. acronyms, proper nouns).
    pub prompt: Option<String>,
    /// Override the response format. Use [`Transcriptions::create_text`] for non-JSON formats.
    pub response_format: Option<TranscriptionFormat>,
    /// Sampling temperature `0.0..=1.0`.
    pub temperature: Option<f32>,
    /// Either `["word"]`, `["segment"]`, or both. Only honoured with `VerboseJson`.
    pub timestamp_granularities: Option<Vec<String>>,
}

/// Available response formats for transcription. `Text` / `Srt` / `Vtt` are raw text and require
/// [`Transcriptions::create_text`].
#[derive(Debug, Clone, Copy)]
pub enum TranscriptionFormat {
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

impl TranscriptionFormat {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Text => "text",
            Self::Srt => "srt",
            Self::VerboseJson => "verbose_json",
            Self::Vtt => "vtt",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranscriptionResponse {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub segments: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub words: Vec<serde_json::Value>,
}

/// Fluent builder for [`TranscriptionRequest`].
pub struct TranscriptionRequestBuilder {
    inner: TranscriptionRequest,
}

impl TranscriptionRequestBuilder {
    /// Start a builder for the given transcription model ID.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            inner: TranscriptionRequest {
                model: model.into(),
                ..Default::default()
            },
        }
    }
    /// Provide the audio bytes plus a filename (extension affects detection).
    pub fn file_bytes(mut self, bytes: Vec<u8>, file_name: impl Into<String>) -> Self {
        self.inner.file = bytes;
        self.inner.file_name = file_name.into();
        self
    }
    /// Override the MIME type. Defaults to `"audio/mpeg"`.
    pub fn mime_type(mut self, m: impl Into<String>) -> Self {
        self.inner.mime_type = Some(m.into());
        self
    }
    /// ISO-639-1 language hint, e.g. `"en"`.
    pub fn language(mut self, l: impl Into<String>) -> Self {
        self.inner.language = Some(l.into());
        self
    }
    /// Bias prompt — acronyms / proper nouns the model might otherwise mishear.
    pub fn prompt(mut self, p: impl Into<String>) -> Self {
        self.inner.prompt = Some(p.into());
        self
    }
    /// Response format. Use [`Transcriptions::create_text`] for `Text`/`Srt`/`Vtt`.
    pub fn response_format(mut self, f: TranscriptionFormat) -> Self {
        self.inner.response_format = Some(f);
        self
    }
    /// Sampling temperature `0.0..=1.0`.
    pub fn temperature(mut self, t: f32) -> Self {
        self.inner.temperature = Some(t);
        self
    }
    /// `["word"]`, `["segment"]`, or both. Requires `response_format = VerboseJson`.
    pub fn timestamp_granularities(mut self, g: Vec<String>) -> Self {
        self.inner.timestamp_granularities = Some(g);
        self
    }
    /// Finalise the builder.
    pub fn build(self) -> TranscriptionRequest {
        self.inner
    }
}

impl<'a> Transcriptions<'a> {
    /// `POST /audio/transcriptions` — transcribe audio into the source language. Returns a
    /// JSON [`TranscriptionResponse`]; use [`Self::create_text`] for `text`/`srt`/`vtt`.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "audio.transcriptions"))
    )]
    pub async fn create(&self, req: TranscriptionRequest) -> Result<TranscriptionResponse> {
        let form = build_audio_form(&req)?;
        super::post_multipart(self.client, "/audio/transcriptions", form).await
    }

    /// `POST /audio/transcriptions` returning the raw response body as a string. Use this for
    /// non-JSON response formats (`Text` / `Srt` / `Vtt`) where the API does not wrap the
    /// transcript in a JSON envelope.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip_all,
            fields(endpoint = "audio.transcriptions.text")
        )
    )]
    pub async fn create_text(&self, req: TranscriptionRequest) -> Result<String> {
        let url = self.client.build_url("/audio/transcriptions")?;
        let form = build_audio_form(&req)?;
        let resp = self
            .client
            .http()
            .post(url)
            .headers(self.client.auth_headers())
            .multipart(form)
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(crate::error::OpenAiError::from_response_body(
                status.as_u16(),
                &body,
            ));
        }
        Ok(body)
    }
}

fn build_audio_form(req: &TranscriptionRequest) -> Result<reqwest::multipart::Form> {
    let mut part =
        reqwest::multipart::Part::bytes(req.file.clone()).file_name(req.file_name.clone());
    if let Some(m) = &req.mime_type {
        part = part
            .mime_str(m)
            .map_err(|e| crate::error::OpenAiError::config(format!("bad mime: {e}")))?;
    } else {
        part = part
            .mime_str("audio/mpeg")
            .map_err(|e| crate::error::OpenAiError::config(format!("bad mime: {e}")))?;
    }
    let mut form = reqwest::multipart::Form::new()
        .text("model", req.model.clone())
        .part("file", part);
    if let Some(l) = &req.language {
        form = form.text("language", l.clone());
    }
    if let Some(p) = &req.prompt {
        form = form.text("prompt", p.clone());
    }
    if let Some(f) = req.response_format {
        form = form.text("response_format", f.as_str());
    }
    if let Some(t) = req.temperature {
        form = form.text("temperature", t.to_string());
    }
    if let Some(g) = &req.timestamp_granularities {
        for v in g {
            form = form.text("timestamp_granularities[]", v.clone());
        }
    }
    Ok(form)
}

// --- translation ------------------------------------------------------------

/// `POST /audio/translations` — translate audio into English text.
pub struct Translations<'a> {
    client: &'a Client,
}

impl<'a> Translations<'a> {
    /// `POST /audio/translations` — same multipart shape as transcription but always outputs
    /// English regardless of the source language.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "audio.translations"))
    )]
    pub async fn create(&self, req: TranscriptionRequest) -> Result<TranscriptionResponse> {
        let form = build_audio_form(&req)?;
        super::post_multipart(self.client, "/audio/translations", form).await
    }
}

// --- speech (TTS) -----------------------------------------------------------

/// `POST /audio/speech` — text-to-speech synthesis.
pub struct Speech<'a> {
    client: &'a Client,
}

/// Request body for `/v1/audio/speech`. Use [`SpeechRequestBuilder`] for ergonomic construction.
#[derive(Debug, Clone, Serialize)]
pub struct SpeechRequest {
    /// TTS model ID: `"tts-1"`, `"tts-1-hd"`, or `"gpt-4o-mini-tts"`.
    pub model: String,
    /// Text to synthesise (max ~4096 chars).
    pub input: String,
    /// Voice ID, e.g. `"alloy"`, `"echo"`, `"fable"`, `"onyx"`, `"nova"`, `"shimmer"`.
    pub voice: String,
    /// `"mp3"` (default), `"opus"`, `"aac"`, `"flac"`, `"wav"`, `"pcm"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// Playback speed, `0.25..=4.0`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    /// `gpt-4o-mini-tts` only — natural-language voice direction (tone, accent, pacing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Fluent builder for [`SpeechRequest`].
pub struct SpeechRequestBuilder {
    inner: SpeechRequest,
}

impl SpeechRequestBuilder {
    /// Required tuple: model, voice ID, input text.
    pub fn new(
        model: impl Into<String>,
        voice: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            inner: SpeechRequest {
                model: model.into(),
                voice: voice.into(),
                input: input.into(),
                response_format: None,
                speed: None,
                instructions: None,
            },
        }
    }
    /// `"mp3"` / `"opus"` / `"aac"` / `"flac"` / `"wav"` / `"pcm"`. Default `"mp3"`.
    pub fn response_format(mut self, f: impl Into<String>) -> Self {
        self.inner.response_format = Some(f.into());
        self
    }
    /// Playback speed, `0.25..=4.0`.
    pub fn speed(mut self, s: f32) -> Self {
        self.inner.speed = Some(s);
        self
    }
    /// Voice-direction prompt for `gpt-4o-mini-tts`. Ignored by `tts-1` / `tts-1-hd`.
    pub fn instructions(mut self, i: impl Into<String>) -> Self {
        self.inner.instructions = Some(i.into());
        self
    }
    /// Finalise the builder.
    pub fn build(self) -> SpeechRequest {
        self.inner
    }
}

impl<'a> Speech<'a> {
    /// `POST /audio/speech` — returns the raw audio bytes encoded per `response_format`.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip_all, fields(endpoint = "audio.speech"))
    )]
    pub async fn create(&self, req: SpeechRequest) -> Result<bytes::Bytes> {
        super::post_json_bytes(self.client, "/audio/speech", &req).await
    }
}
