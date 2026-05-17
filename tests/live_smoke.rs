//! Minimal smoke tests against the real OpenAI API. All gated `#[ignore]` so they don't
//! run on regular `cargo test`. Catches upstream API drift (deprecated models, response
//! shape changes, new required fields).
//!
//! Run manually:
//!   `OPENAI_API_KEY=sk-... cargo test --test live_smoke -- --ignored`
//!
//! Run from CI:
//!   The nightly `live-smoke` workflow runs them with a repository secret.

use open_ai_rust::{ChatMessage, Client, OpenAiModel, PayLoadBuilder};

fn client() -> Client {
    dotenv::dotenv().ok();
    Client::from_env().expect("OPENAI_API_KEY missing — set it to run live tests")
}

#[tokio::test]
#[ignore = "hits the real API"]
async fn live_chat_completion_basic() {
    let resp = client()
        .chat()
        .create(
            PayLoadBuilder::new(OpenAiModel::GPT4oMini)
                .messages(vec![
                    ChatMessage::system("Respond with exactly the word `pong`."),
                    ChatMessage::user("ping"),
                ])
                .temperature(0.0)
                .max_completion_tokens(20)
                .build(),
        )
        .await
        .expect("chat completion failed");

    let text = resp.get_last_msg_text().unwrap_or_default();
    assert!(
        text.to_lowercase().contains("pong"),
        "expected 'pong', got: {text:?}"
    );
    // Sanity: usage is reported.
    assert!(resp.usage.prompt_tokens > 0);
    assert!(resp.usage.total_tokens > 0);
}

#[tokio::test]
#[ignore = "hits the real API"]
async fn live_chat_streaming_returns_text() {
    use futures_util::StreamExt;

    let mut stream = client()
        .chat()
        .create_stream(
            PayLoadBuilder::new(OpenAiModel::GPT4oMini)
                .messages(vec![ChatMessage::user("Say `hi`.")])
                .temperature(0.0)
                .max_completion_tokens(20)
                .build(),
        )
        .await
        .expect("create_stream failed");

    let mut text = String::new();
    let mut chunks = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("stream chunk");
        text.push_str(&chunk.delta_text());
        chunks += 1;
    }
    assert!(chunks > 0, "expected at least one chunk");
    assert!(!text.is_empty(), "empty streamed text");
}

#[tokio::test]
#[ignore = "hits the real API"]
async fn live_embeddings_returns_vector_of_known_dim() {
    let v = client()
        .embeddings()
        .create_one("hello", "text-embedding-3-small")
        .await
        .expect("embeddings failed");
    assert_eq!(v.len(), 1536, "text-embedding-3-small dim drift");
}

#[tokio::test]
#[ignore = "hits the real API"]
async fn live_moderations_classifies_clean_input() {
    use open_ai_rust::resources::moderations::ModerationRequest;

    let resp = client()
        .moderations()
        .create(ModerationRequest {
            input: "hello world".into(),
            model: Some("omni-moderation-latest".into()),
        })
        .await
        .expect("moderations failed");
    assert!(!resp.results.is_empty());
    assert!(
        !resp.results[0].flagged,
        "clean input flagged: {:?}",
        resp.results
    );
}

#[tokio::test]
#[ignore = "hits the real API"]
async fn live_models_list_includes_gpt_4o_mini() {
    let list = client().models().list().await.expect("models.list failed");
    assert!(
        list.data.iter().any(|m| m.id == "gpt-4o-mini"),
        "gpt-4o-mini not in models list — has it been deprecated?"
    );
}

#[tokio::test]
#[ignore = "hits the real API"]
async fn live_responses_api_returns_output_text() {
    use open_ai_rust::responses::ResponseRequestBuilder;

    let resp = client()
        .responses()
        .create(
            ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "Say `pong`.")
                .instructions("Respond with the word pong.")
                .max_output_tokens(20)
                .build(),
        )
        .await
        .expect("responses.create failed");

    let text = resp.output_text();
    assert!(
        text.to_lowercase().contains("pong"),
        "expected 'pong', got: {text:?}"
    );
}
