//! Run with: `OPENAI_API_KEY=sk-... cargo run --example structured_output`
use open_ai_rust::{ChatMessage, Client, OpenAiModel, PayLoadBuilder, ResponseFormat};
use serde_json::json;

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let client = Client::from_env()?;

    let schema = json!({
        "type": "object",
        "properties": {
            "city": { "type": "string" },
            "country": { "type": "string" },
            "population": { "type": "integer" }
        },
        "required": ["city", "country", "population"],
        "additionalProperties": false
    });

    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("Give me info on Sydney as JSON.")])
        .response_format(ResponseFormat::json_schema("city_info", schema))
        .build();

    let resp = client.chat().create(payload).await?;
    println!("{}", resp.get_last_msg_text().unwrap_or_default());
    Ok(())
}
