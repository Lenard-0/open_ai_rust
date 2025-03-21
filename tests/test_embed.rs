
#[cfg(test)]
mod tests {
    use open_ai_rust::{requests::embed::embed, set_key};


    #[tokio::test]
    async fn can_embed_text() {
        dotenv::dotenv().ok();
        set_key(std::env::var("OPENAI_SK").unwrap()); // Set the OpenAI API key from the environment variable
        let text = "Hello, world!".to_string();
        let embedding = embed(text, None).await.unwrap();
        assert_eq!(embedding.len(), 1536);
    }
}