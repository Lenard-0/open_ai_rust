//! Stream a Responses-API answer to stdout as text deltas arrive.
//!
//! Run with: `OPENAI_API_KEY=sk-... cargo run --example responses_stream`
use std::io::Write;

use futures_util::StreamExt;
use open_ai_rust::responses::{ResponseRequestBuilder, ResponseStreamEvent};
use open_ai_rust::{Client, OpenAiModel};

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let client = Client::from_env()?;

    let req = ResponseRequestBuilder::new(
        OpenAiModel::GPT41Mini,
        "Write a short 3-sentence story about a Rust crab.",
    )
    .instructions("Be terse and whimsical.")
    .build();

    let mut events = client.responses().create_stream(req).await?;
    while let Some(event) = events.next().await {
        match event? {
            ResponseStreamEvent::OutputTextDelta { delta, .. } => {
                print!("{delta}");
                std::io::stdout().flush().ok();
            }
            ResponseStreamEvent::Completed { response, .. } => {
                println!("\n\n[completed, status={:?}]", response.status);
            }
            ResponseStreamEvent::Failed { response, .. } => {
                eprintln!("\n[failed: {:?}]", response.error);
            }
            _ => {}
        }
    }
    Ok(())
}
