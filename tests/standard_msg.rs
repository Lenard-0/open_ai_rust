
#[cfg(test)]
mod tests {
    use open_ai_rust::{logoi::{input::payload::templates::PayLoadTemplates, message::{ChatMessage, ChatMessageRole}}, requests::open_ai_msg, set_key};

    #[tokio::test]
    async fn can_do_regular_chat_completion() {
        dotenv::dotenv().ok();
        set_key(std::env::var("OPENAI_SK").unwrap()); // Set the OpenAI API key from the environment variable

        let system_msg = ChatMessage {
            role: ChatMessageRole::System,
            content: "You are part of a test in a Rust program. Respond with an exact copy of the user msg.".to_string(),
            name: None
        };

        let user_msg = ChatMessage {
            role: ChatMessageRole::User,
            content: "Hello world!".to_string(),
            name: None
        };

        let payload = PayLoadTemplates::default(vec![system_msg, user_msg]);

        let response = open_ai_msg(payload.to_payload()).await.unwrap();
        assert_eq!(response.choices[0].message.content, Some("Hello world!".to_string()));
        assert_eq!(response.choices.len(), 1);
    }
}