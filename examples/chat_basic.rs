//! Run with: `OPENAI_API_KEY=sk-... cargo run --example chat_basic`
use open_ai_rust::{ChatMessage, Client, OpenAiModel, PayLoadBuilder};

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let client = Client::from_env()?;

    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Say hello in 5 words."),
        ])
        .build();

    let resp = client.chat().create(payload).await?;
    println!("{}", resp.get_last_msg_text().unwrap_or_default());
    Ok(())
}
