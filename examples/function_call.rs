//! Run with: `OPENAI_API_KEY=sk-... cargo run --example function_call`
use open_ai_rust::{
    ChatMessage, Client, FunctionCall, FunctionParameter, FunctionType, OpenAiModel, PayLoadBuilder,
};

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    dotenv::dotenv().ok();
    let client = Client::from_env()?;

    let tool = FunctionCall {
        name: "get_weather".to_string(),
        description: Some("Get current weather for a city".to_string()),
        parameters: vec![FunctionParameter {
            name: "city".to_string(),
            _type: FunctionType::String,
            description: Some("Name of the city".to_string()),
            required: true,
        }],
    };

    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("What's the weather in Sydney?")])
        .tools(vec![tool])
        .build();

    let resp = client.chat().create(payload).await?;
    for tc in resp.get_tool_calls() {
        println!("Function: {}\nArgs: {}", tc.name, tc.arguments);
    }
    Ok(())
}
