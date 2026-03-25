
#[cfg(test)]
mod tests {
    use open_ai_rust::{requests::embed::embed, set_embeddings_endpoint, set_key};


    #[tokio::test]
    async fn can_embed_text() {
        dotenv::dotenv().ok();
        set_key(std::env::var("AZURE_AI_SK").unwrap());
        set_embeddings_endpoint(std::env::var("AZURE_EMBED_ENDPOINT").unwrap());
        let text = "Hello, world!".to_string();
        let embedding = embed(text, None).await.unwrap();
        assert_eq!(embedding.len(), 1536);
    }
}