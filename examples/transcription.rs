//! Transcribe an audio file with whisper / gpt-4o-transcribe.
//!
//! Usage: `OPENAI_API_KEY=sk-... cargo run --example transcription -- path/to/audio.mp3`
use open_ai_rust::resources::audio::TranscriptionRequestBuilder;
use open_ai_rust::Client;

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/speech_test_audio.mp3".to_string());
    let bytes = std::fs::read(&path)?;
    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("audio.mp3")
        .to_string();

    let client = Client::from_env()?;
    let req = TranscriptionRequestBuilder::new("gpt-4o-mini-transcribe")
        .file_bytes(bytes, file_name)
        .mime_type("audio/mpeg")
        .build();

    let resp = client.audio().transcriptions().create(req).await?;
    println!("{}", resp.text);
    Ok(())
}
