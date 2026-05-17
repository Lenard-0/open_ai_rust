//! Run with: `OPENAI_API_KEY=sk-... cargo run --example responses_basic`
use open_ai_rust::responses::ResponseRequestBuilder;
use open_ai_rust::{Client, OpenAiModel};

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let client = Client::from_env()?;

    let req = ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "Write a haiku about Rust.")
        .instructions("You are a haiku poet.")
        .build();

    let resp = client.responses().create(req).await?;
    println!("status={:?}\n{}", resp.status, resp.output_text());
    Ok(())
}
