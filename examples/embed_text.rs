//! Run with: `OPENAI_API_KEY=sk-... cargo run --example embed_text`
use open_ai_rust::Client;

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let client = Client::from_env()?;

    let v = client
        .embeddings()
        .create_one("Hello, embeddings!", "text-embedding-3-small")
        .await?;
    println!("dim={} sample={:?}", v.len(), &v[..5.min(v.len())]);
    Ok(())
}
