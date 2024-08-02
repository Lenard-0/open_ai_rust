
#[cfg(test)]
mod tests {
    use open_ai_rust::{logoi::{input::{payload::templates::PayLoadTemplates, tool::{FunctionCall, FunctionParameter, FunctionType}}, message::{ChatMessage, ChatMessageRole}}, requests::open_ai_msg, set_key};

    #[tokio::test]
    async fn can_do_function_call_simple() {
        dotenv::dotenv().ok();
        set_key(std::env::var("OPENAI_SK").unwrap()); // Set the OpenAI API key from the environment variable

        let system_msg = ChatMessage {
            role: ChatMessageRole::System,
            content: "You are part of a test in a Rust program. Follow the user's request to complete the function/tool call.".to_string(),
            name: None
        };

        let user_msg = ChatMessage {
            role: ChatMessageRole::User,
            content: "Let there be light!".to_string(),
            name: None
        };

        let functions = vec![
            FunctionCall {
                name: "change_light".to_string(),
                description: Some("Change the light in the room.".to_string()),
                arguments: vec![
                    FunctionParameter {
                        name: "turn_on_light".to_string(),
                        _type: FunctionType::Boolean,
                        description: Some("True turns on the light and false turns it off".to_string())
                    }
                ]
            }
        ];

        let payload = PayLoadTemplates::default(vec![system_msg, user_msg]);

        let response = match open_ai_msg(payload.to_payload()).await {
            Ok(data) => data,
            Err(e) => {
                println!("Error: {}", e);
                panic!();
            }
        };
        assert_eq!(response.choices[0].message.content, Some("Hello world!".to_string()));
        assert_eq!(response.choices.len(), 1);
    }
}