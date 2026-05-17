//! Synthesise speech and write the MP3 to disk.
//!
//! Usage: `OPENAI_API_KEY=sk-... cargo run --example tts -- "Hello, world." out.mp3`
use std::io::Write;

use open_ai_rust::resources::audio::SpeechRequestBuilder;
use open_ai_rust::Client;

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let text = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Hello from open_ai_rust.".to_string());
    let out_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "out.mp3".to_string());

    let client = Client::from_env()?;
    let req = SpeechRequestBuilder::new("gpt-4o-mini-tts", "alloy", text)
        .response_format("mp3")
        .build();

    let bytes = client.audio().speech().create(req).await?;
    std::fs::File::create(&out_path)?.write_all(&bytes)?;
    println!("wrote {} bytes → {}", bytes.len(), out_path);
    Ok(())
}
