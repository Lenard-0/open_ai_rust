//! Run with: `OPENAI_API_KEY=sk-... cargo run --example chat_stream`
use std::io::Write;

use futures_util::StreamExt;
use open_ai_rust::{ChatMessage, Client, OpenAiModel, PayLoadBuilder};

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let client = Client::from_env()?;

    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user(
            "Tell me a 2-sentence joke about Rust.",
        )])
        .include_usage(true)
        .build();

    let mut stream = client.chat().create_stream(payload).await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        print!("{}", chunk.delta_text());
        std::io::stdout().flush().ok();
    }
    println!();
    Ok(())
}
