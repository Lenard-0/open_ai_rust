//! HTTP fixture tests using `wiremock`. Verifies each endpoint sends the right request
//! shape AND parses real-looking response bodies without ever touching the real OpenAI API.

use std::time::Duration;

use open_ai_rust::responses::{ResponseInput, ResponseRequestBuilder};
use open_ai_rust::{
    ChatMessage, Client, OpenAiModel, PayLoadBuilder, RequestOptions, ResponseFormat,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("test-key")
        .base_url(server.uri())
        .build_unchecked()
}

// --- chat completions -------------------------------------------------------

#[tokio::test]
async fn chat_completion_sends_bearer_and_parses_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .and(header("content-type", "application/json"))
        .and(body_json(json!({
            "model": "gpt-4o-mini",
            "messages": [{ "role": "user", "content": "hi" }],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-abc",
            "object": "chat.completion",
            "created": 1700000000,
            "model": "gpt-4o-mini",
            "choices": [{
                "finish_reason": "stop",
                "index": 0,
                "message": { "role": "assistant", "content": "hello back" },
            }],
            "usage": { "prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7 },
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let resp = client.chat().create(payload).await.unwrap();
    assert_eq!(resp.id, "chatcmpl-abc");
    assert_eq!(resp.get_last_msg_text().as_deref(), Some("hello back"));
}

#[tokio::test]
async fn chat_completion_json_schema_is_sent_as_object() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_json(json!({
            "model": "gpt-4o-mini",
            "messages": [{ "role": "user", "content": "hi" }],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "city",
                    "schema": { "type": "object" },
                    "strict": true,
                },
            },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "x",
            "object": "chat.completion",
            "created": 1,
            "model": "gpt-4o-mini",
            "choices": [{
                "finish_reason": "stop",
                "index": 0,
                "message": { "role": "assistant", "content": "{}" },
            }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 },
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .response_format(ResponseFormat::json_schema(
            "city",
            json!({ "type": "object" }),
        ))
        .build();
    client.chat().create(payload).await.unwrap();
}

#[tokio::test]
async fn chat_completion_5xx_then_200_retries() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(503).set_body_string("transient"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "x", "object": "chat.completion", "created": 1, "model": "m",
            "choices": [{ "finish_reason": "stop", "index": 0, "message": { "role": "assistant", "content": "ok" } }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 },
        })))
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("test")
        .base_url(server.uri())
        .max_retries(3)
        .build_unchecked();
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let resp = client.chat().create(payload).await.unwrap();
    assert_eq!(resp.get_last_msg_text().as_deref(), Some("ok"));
}

#[tokio::test]
async fn chat_completion_404_does_not_retry() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": { "message": "not found", "type": "invalid_request_error", "code": "not_found" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("test")
        .base_url(server.uri())
        .max_retries(3)
        .build_unchecked();
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let err = client.chat().create(payload).await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("404"), "got: {msg}");
    assert!(msg.contains("not found"), "got: {msg}");
}

// --- embeddings -------------------------------------------------------------

#[tokio::test]
async fn embeddings_sends_input_and_parses_vector() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .and(body_json(json!({
            "model": "text-embedding-3-small",
            "input": "hello",
            "encoding_format": "float",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "model": "text-embedding-3-small",
            "data": [{ "object": "embedding", "index": 0, "embedding": [0.1, 0.2, 0.3] }],
            "usage": { "prompt_tokens": 1, "total_tokens": 1 },
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let v = client
        .embeddings()
        .create_one("hello", "text-embedding-3-small")
        .await
        .unwrap();
    assert_eq!(v, vec![0.1, 0.2, 0.3]);
}

// --- responses --------------------------------------------------------------

#[tokio::test]
async fn responses_create_returns_output_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_1",
            "object": "response",
            "created_at": 1,
            "model": "gpt-4.1-mini",
            "status": "completed",
            "output": [{
                "type": "message",
                "id": "msg_1",
                "status": "completed",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "haiku", "annotations": [] }],
            }],
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "hi").build();
    let resp = client.responses().create(req).await.unwrap();
    assert_eq!(resp.output_text(), "haiku");
}

#[tokio::test]
async fn responses_create_accepts_items_input() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_json(json!({
            "model": "gpt-4.1-mini",
            "input": [{
                "type": "message",
                "role": "user",
                "content": "hi",
            }],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "r", "object": "response", "created_at": 1, "model": "m",
            "status": "completed", "output": []
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    use open_ai_rust::responses::ResponseInputItem;
    let req = ResponseRequestBuilder::new(
        OpenAiModel::GPT41Mini,
        ResponseInput::Items(vec![ResponseInputItem::user("hi")]),
    )
    .build();
    client.responses().create(req).await.unwrap();
}

// --- moderations ------------------------------------------------------------

#[tokio::test]
async fn moderations_sends_and_parses() {
    use open_ai_rust::resources::moderations::ModerationRequest;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/moderations"))
        .and(body_json(json!({
            "input": "naughty",
            "model": "omni-moderation-latest",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "modr_1",
            "model": "omni-moderation-latest",
            "results": [{
                "flagged": false,
                "categories": { "harassment": false },
                "category_scores": { "harassment": 0.0 },
            }],
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = ModerationRequest {
        input: "naughty".into(),
        model: Some("omni-moderation-latest".to_string()),
    };
    let resp = client.moderations().create(req).await.unwrap();
    assert_eq!(resp.id, "modr_1");
    assert!(!resp.results[0].flagged);
}

// --- models -----------------------------------------------------------------

#[tokio::test]
async fn models_list_parses() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [
                { "id": "gpt-4o-mini", "object": "model", "created": 1, "owned_by": "openai" },
                { "id": "gpt-4.1", "object": "model", "created": 2, "owned_by": "openai" }
            ]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let list = client.models().list().await.unwrap();
    assert_eq!(list.data.len(), 2);
    assert_eq!(list.data[0].id, "gpt-4o-mini");
}

// --- files ------------------------------------------------------------------

#[tokio::test]
async fn files_upload_uses_multipart() {
    use open_ai_rust::resources::files::FilePurpose;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .and(header_exists("content-type"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "file_1",
            "object": "file",
            "bytes": 5,
            "created_at": 1,
            "filename": "data.txt",
            "purpose": "user_data",
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .files()
        .create(b"hello".to_vec(), "data.txt", FilePurpose::UserData)
        .await
        .unwrap();
    assert_eq!(resp.id, "file_1");
    assert_eq!(resp.filename, "data.txt");
}

// --- request options --------------------------------------------------------

#[tokio::test]
async fn idempotency_key_header_is_sent_when_set() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("idempotency-key", "my-key-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "x", "object": "chat.completion", "created": 1, "model": "m",
            "choices": [{ "finish_reason": "stop", "index": 0, "message": { "role": "assistant", "content": "ok" } }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 },
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let resp = client
        .with_idempotency_key("my-key-123")
        .chat()
        .create(payload)
        .await
        .unwrap();
    assert_eq!(resp.get_last_msg_text().as_deref(), Some("ok"));
}

#[tokio::test]
async fn extra_header_via_request_options_is_sent() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("x-trace-id", "abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "x", "object": "chat.completion", "created": 1, "model": "m",
            "choices": [{ "finish_reason": "stop", "index": 0, "message": { "role": "assistant", "content": "ok" } }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 },
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let opts = RequestOptions::new().header("X-Trace-Id", "abc-123");
    client
        .with_options(opts)
        .chat()
        .create(payload)
        .await
        .unwrap();
}

#[tokio::test]
async fn per_request_timeout_aborts_slow_endpoint() {
    let server = MockServer::start().await;
    // Server delays 2s; our per-request timeout is 200ms.
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2))
                .set_body_json(json!({
                    "id": "x", "object": "chat.completion", "created": 1, "model": "m",
                    "choices": [], "usage": { "prompt_tokens": 0, "total_tokens": 0 }
                })),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let start = std::time::Instant::now();
    let result = client
        .with_timeout(Duration::from_millis(200))
        .chat()
        .create(payload)
        .await;
    let elapsed = start.elapsed();
    assert!(result.is_err(), "expected timeout error, got Ok");
    // The mock delays 2s; if our timeout fired we should be well under that.
    assert!(
        elapsed < Duration::from_millis(1_500),
        "per-request timeout did not fire: elapsed={:?}",
        elapsed
    );
}

// --- azure --------------------------------------------------------------

#[tokio::test]
async fn azure_client_uses_api_key_header_and_deployment_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/openai/deployments/my-deploy/chat/completions"))
        .and(header("api-key", "az-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "x", "object": "chat.completion", "created": 1, "model": "m",
            "choices": [{ "finish_reason": "stop", "index": 0, "message": { "role": "assistant", "content": "ok" } }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 },
        })))
        .mount(&server)
        .await;

    let client = Client::azure("az-key", server.uri(), "my-deploy", "2024-10-01-preview");
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4o)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let resp = client.chat().create(payload).await.unwrap();
    assert_eq!(resp.get_last_msg_text().as_deref(), Some("ok"));
}

// --- streaming SSE ----------------------------------------------------------

/// Compose a raw SSE body that the chat-completions streaming code path should parse
/// end-to-end into a series of `ChatCompletionChunk`s, terminating on `data: [DONE]`.
fn build_sse_body(chunks: &[serde_json::Value]) -> String {
    let mut out = String::new();
    for c in chunks {
        out.push_str("data: ");
        out.push_str(&serde_json::to_string(c).unwrap());
        out.push_str("\n\n");
    }
    out.push_str("data: [DONE]\n\n");
    out
}

#[tokio::test]
async fn streaming_chat_parses_deltas_and_terminates_on_done() {
    use futures_util::StreamExt;

    let body = build_sse_body(&[
        json!({
            "id": "chatcmpl-1", "object": "chat.completion.chunk",
            "created": 1, "model": "gpt-4o-mini",
            "choices": [{ "index": 0, "delta": { "role": "assistant", "content": "Hel" }, "finish_reason": null }]
        }),
        json!({
            "id": "chatcmpl-1", "object": "chat.completion.chunk",
            "created": 1, "model": "gpt-4o-mini",
            "choices": [{ "index": 0, "delta": { "content": "lo " }, "finish_reason": null }]
        }),
        json!({
            "id": "chatcmpl-1", "object": "chat.completion.chunk",
            "created": 1, "model": "gpt-4o-mini",
            "choices": [{ "index": 0, "delta": { "content": "world" }, "finish_reason": null }]
        }),
        json!({
            "id": "chatcmpl-1", "object": "chat.completion.chunk",
            "created": 1, "model": "gpt-4o-mini",
            "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }]
        }),
    ]);

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.into_bytes(), "text/event-stream"),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let mut stream = client.chat().create_stream(payload).await.unwrap();

    let mut text = String::new();
    let mut final_finish: Option<String> = None;
    let mut count = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        count += 1;
        text.push_str(&chunk.delta_text());
        if let Some(choice) = chunk.choices.first() {
            if let Some(fr) = &choice.finish_reason {
                final_finish = Some(fr.clone());
            }
        }
    }
    assert_eq!(count, 4, "expected 4 chunks before [DONE]");
    assert_eq!(text, "Hello world");
    assert_eq!(final_finish.as_deref(), Some("stop"));
}

#[tokio::test]
async fn streaming_chat_collect_chat_stream_helper_assembles_correctly() {
    let body = build_sse_body(&[
        json!({
            "id": "x", "object": "chat.completion.chunk",
            "created": 1, "model": "gpt-4o-mini",
            "choices": [{ "index": 0, "delta": { "role": "assistant", "tool_calls": [
                { "index": 0, "id": "call_abc", "type": "function",
                  "function": { "name": "get_weather", "arguments": "{\"loc" } }
            ] }, "finish_reason": null }]
        }),
        json!({
            "id": "x", "object": "chat.completion.chunk",
            "created": 1, "model": "gpt-4o-mini",
            "choices": [{ "index": 0, "delta": { "tool_calls": [
                { "index": 0, "function": { "arguments": "ation\":\"Sydney\"}" } }
            ] }, "finish_reason": "tool_calls" }]
        }),
    ]);

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.into_bytes(), "text/event-stream"),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let stream = client.chat().create_stream(payload).await.unwrap();
    let collected = open_ai_rust::collect_chat_stream(stream).await.unwrap();
    assert_eq!(collected.tool_calls.len(), 1);
    let tc = &collected.tool_calls[0];
    assert_eq!(tc.id.as_deref(), Some("call_abc"));
    assert_eq!(tc.name.as_deref(), Some("get_weather"));
    assert_eq!(tc.arguments, r#"{"location":"Sydney"}"#);
    assert_eq!(collected.finish_reason.as_deref(), Some("tool_calls"));
}

#[tokio::test]
async fn streaming_chat_error_status_yields_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": { "message": "boom", "type": "server_error" }
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let result = client.chat().create_stream(payload).await;
    let err = match result {
        Ok(_) => panic!("expected error, got Ok"),
        Err(e) => e,
    };
    let msg = err.to_string();
    assert!(msg.contains("500"), "got: {msg}");
}
