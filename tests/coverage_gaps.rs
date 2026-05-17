//! Mops up specific remaining coverage gaps after the main coverage_units / wiremock
//! tests: Chat::completions() flavour of the chat API, images.edit with mask, responses
//! resource error branches, files resource error branches, chat_stream_ext multi-choice
//! collector, etc.

use open_ai_rust::resources::chat_stream_ext::{collect_chat_stream, CollectedChatStream};
use open_ai_rust::{ChatMessage, Client, OpenAiModel, PayLoadBuilder};
use serde_json::json;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .build_unchecked()
}

// =============================================================================
// Chat::completions() flavour — non-stream + stream
// =============================================================================

#[tokio::test]
async fn chat_completions_create_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "x", "object": "chat.completion", "created": 1, "model": "m",
            "choices": [{ "finish_reason": "stop", "index": 0, "message": { "role": "assistant", "content": "ok" } }],
            "usage": { "prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2 },
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let r = c.chat().completions().create(payload).await.unwrap();
    assert_eq!(r.get_last_msg_text().as_deref(), Some("ok"));
}

#[tokio::test]
async fn chat_completions_create_stream_path() {
    let body = "data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"m\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\ndata: [DONE]\n\n";
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.as_bytes().to_vec(), "text/event-stream"),
        )
        .mount(&server)
        .await;
    let c = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let stream = c.chat().completions().create_stream(payload).await.unwrap();
    let collected = collect_chat_stream(stream).await.unwrap();
    assert_eq!(collected.content, "hi");
}

// =============================================================================
// images.edit WITH a mask (separate code path inside build_edit_form)
// =============================================================================

#[tokio::test]
async fn images_edit_with_mask_includes_mask_part() {
    use open_ai_rust::resources::images::ImageEditRequest;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/images/edits"))
        .and(header_exists("content-type"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1,
            "data": [{ "url": "https://example.com/edited.png" }],
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = ImageEditRequest {
        image: vec![0x89, 0x50, 0x4E, 0x47],
        image_name: "src.png".into(),
        mask: Some(vec![0xFF, 0xD8]), // some bytes for a mask
        mask_name: Some("mask.png".into()),
        prompt: "edit".into(),
        model: None,
        n: None,
        size: None,
        response_format: None,
        user: Some("u".into()),
    };
    let resp = c.images().edit(req).await.unwrap();
    assert_eq!(resp.data.len(), 1);
}

#[tokio::test]
async fn images_variations_with_user_field_included() {
    use open_ai_rust::resources::images::ImageVariationRequest;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/images/variations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1,
            "data": [{ "b64_json": "..." }],
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = ImageVariationRequest {
        image: vec![0x89, 0x50, 0x4E, 0x47],
        image_name: "src.png".into(),
        model: Some("dall-e-2".into()),
        n: Some(1),
        size: Some("256x256".into()),
        response_format: Some("b64_json".into()),
        user: Some("u".into()),
    };
    c.images().variations(req).await.unwrap();
}

// =============================================================================
// audio.translations: with optional fields (language/prompt/format/temp/granularities)
// =============================================================================

#[tokio::test]
async fn audio_translations_with_all_optional_fields() {
    use open_ai_rust::resources::audio::{TranscriptionFormat, TranscriptionRequestBuilder};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/translations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "text": "ok" })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = TranscriptionRequestBuilder::new("whisper-1")
        .file_bytes(b"x".to_vec(), "x.mp3")
        .mime_type("audio/mpeg")
        .language("en")
        .prompt("be brief")
        .response_format(TranscriptionFormat::VerboseJson)
        .temperature(0.0)
        .timestamp_granularities(vec!["segment".into()])
        .build();
    c.audio().translations().create(req).await.unwrap();
}

#[tokio::test]
async fn audio_transcription_create_text_error_path() {
    use open_ai_rust::resources::audio::TranscriptionRequestBuilder;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = TranscriptionRequestBuilder::new("whisper-1")
        .file_bytes(b"x".to_vec(), "x.mp3")
        .build();
    let err = c
        .audio()
        .transcriptions()
        .create_text(req)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("400"));
}

// =============================================================================
// Resource error paths — verify each resource's error branch fires.
// =============================================================================

#[tokio::test]
async fn files_list_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c.files().list().await.unwrap_err();
    assert!(err.to_string().contains("500"));
}

#[tokio::test]
async fn files_retrieve_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/abc"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c.files().retrieve("abc").await.unwrap_err();
    assert!(err.to_string().contains("404"));
}

#[tokio::test]
async fn files_delete_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/files/abc"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c.files().delete("abc").await.unwrap_err();
    assert!(err.to_string().contains("404"));
}

#[tokio::test]
async fn files_content_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/abc/content"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c.files().content("abc").await.unwrap_err();
    assert!(err.to_string().contains("404"));
}

#[tokio::test]
async fn responses_retrieve_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/responses/x"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c.responses().retrieve("x").await.unwrap_err();
    assert!(err.to_string().contains("500"));
}

#[tokio::test]
async fn responses_cancel_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses/x/cancel"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c.responses().cancel("x").await.unwrap_err();
    assert!(err.to_string().contains("500"));
}

#[tokio::test]
async fn responses_delete_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/responses/x"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c.responses().delete("x").await.unwrap_err();
    assert!(err.to_string().contains("500"));
}

#[tokio::test]
async fn models_retrieve_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models/x"))
        .respond_with(ResponseTemplate::new(404).set_body_string("no"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c.models().retrieve("x").await.unwrap_err();
    assert!(err.to_string().contains("404"));
}

#[tokio::test]
async fn models_list_and_delete_error_paths() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/models/x"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let _ = c.models().list().await.unwrap_err();
    let _ = c.models().delete("x").await.unwrap_err();
}

#[tokio::test]
async fn batches_retrieve_cancel_list_error_paths() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/batches/x"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/batches/x/cancel"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/batches"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let _ = c.batches().retrieve("x").await.unwrap_err();
    let _ = c.batches().cancel("x").await.unwrap_err();
    let _ = c.batches().list().await.unwrap_err();
}

#[tokio::test]
async fn vector_stores_error_paths_all_verbs() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/vector_stores"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/vector_stores/x"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/vector_stores/x"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let _ = c.vector_stores().list().await.unwrap_err();
    let _ = c.vector_stores().retrieve("x").await.unwrap_err();
    let _ = c.vector_stores().delete("x").await.unwrap_err();
}

#[tokio::test]
async fn fine_tuning_jobs_cancel_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/fine_tuning/jobs/x/cancel"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let _ = c.fine_tuning().jobs().cancel("x").await.unwrap_err();
}

#[tokio::test]
async fn uploads_cancel_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/uploads/x/cancel"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let _ = c.uploads().cancel("x").await.unwrap_err();
}

// =============================================================================
// chat_stream_ext::collect_chat_stream — multi-choice + refusal collection
// =============================================================================

#[tokio::test]
async fn collect_chat_stream_handles_refusal_delta() {
    let body = "data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"m\",\"choices\":[{\"index\":0,\"delta\":{\"refusal\":\"sorry, can't\"},\"finish_reason\":null}]}\n\ndata: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"m\",\"choices\":[{\"index\":0,\"delta\":{\"refusal\":\" not allowed\"},\"finish_reason\":\"content_filter\"}]}\n\ndata: [DONE]\n\n";
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.as_bytes().to_vec(), "text/event-stream"),
        )
        .mount(&server)
        .await;
    let c = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let stream = c.chat().create_stream(payload).await.unwrap();
    let collected: CollectedChatStream = collect_chat_stream(stream).await.unwrap();
    assert_eq!(
        collected.refusal.as_deref(),
        Some("sorry, can't not allowed")
    );
    assert_eq!(collected.finish_reason.as_deref(), Some("content_filter"));
}

#[tokio::test]
async fn collect_chat_stream_with_no_choices_in_chunk() {
    // A chunk with `choices: []` (e.g. final usage-only chunk when include_usage=true)
    // should not error.
    let body = r#"data: {"id":"x","object":"chat.completion.chunk","created":1,"model":"m","choices":[]}

data: {"id":"x","object":"chat.completion.chunk","created":1,"model":"m","choices":[{"index":0,"delta":{"content":"ok"},"finish_reason":"stop"}]}

data: [DONE]

"#;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.as_bytes().to_vec(), "text/event-stream"),
        )
        .mount(&server)
        .await;
    let c = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let stream = c.chat().create_stream(payload).await.unwrap();
    let collected = collect_chat_stream(stream).await.unwrap();
    assert_eq!(collected.content, "ok");
    assert_eq!(collected.finish_reason.as_deref(), Some("stop"));
}

// =============================================================================
// stream::post_sse_stream — successful empty stream (zero events, just [DONE])
// =============================================================================

#[tokio::test]
async fn stream_only_done_terminator_yields_zero_chunks() {
    use futures_util::StreamExt;
    let body = "data: [DONE]\n\n";
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.as_bytes().to_vec(), "text/event-stream"),
        )
        .mount(&server)
        .await;
    let c = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let mut s = c.chat().create_stream(payload).await.unwrap();
    assert!(s.next().await.is_none());
}

// =============================================================================
// Responses streaming
// =============================================================================

// =============================================================================
// Connection-refused → OpenAiError::Reqwest, exercises is_retryable Reqwest arm
// and the retry-then-fail loop in post_json.
// =============================================================================

#[tokio::test]
async fn connection_refused_retries_then_surfaces_as_reqwest_error() {
    // 127.0.0.1:1 is the discard port; nothing listens.
    let c = Client::builder()
        .api_key("k")
        .base_url("http://127.0.0.1:1")
        .max_retries(1)
        .timeout(std::time::Duration::from_millis(200))
        .build_unchecked();
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let err = c.chat().create(payload).await.unwrap_err();
    match err {
        open_ai_rust::OpenAiError::Reqwest(_) => {}
        e => panic!("expected Reqwest, got {e:?}"),
    }
}

// =============================================================================
// post_json_bytes error path — `/audio/speech` returning 4xx
// =============================================================================

#[tokio::test]
async fn audio_speech_4xx_surfaces_as_api_error() {
    use open_ai_rust::resources::audio::SpeechRequestBuilder;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/speech"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": { "message": "bad speech params" }
        })))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .build_unchecked();
    let req = SpeechRequestBuilder::new("gpt-4o-mini-tts", "alloy", "hi").build();
    let err = c.audio().speech().create(req).await.unwrap_err();
    assert!(err.to_string().contains("400"));
}

// =============================================================================
// embeddings empty-data branch (when API returns valid envelope but data: [])
// =============================================================================

#[tokio::test]
async fn embeddings_create_one_empty_data_surfaces_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "model": "text-embedding-3-small",
            "data": [],
            "usage": { "prompt_tokens": 0, "total_tokens": 0 }
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let err = c
        .embeddings()
        .create_one("hi", "text-embedding-3-small")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("empty embeddings"));
}

// =============================================================================
// fine_tuning::list_checkpoints success + list_events on a non-default fixture
// =============================================================================

#[tokio::test]
async fn fine_tuning_list_checkpoints_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs/x/checkpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list", "data": [], "has_more": false,
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let v = c.fine_tuning().jobs().list_checkpoints("x").await.unwrap();
    assert_eq!(v["object"], "list");
}

#[tokio::test]
async fn fine_tuning_list_events_error_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs/x/events"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let _ = c.fine_tuning().jobs().list_events("x").await.unwrap_err();
}

// =============================================================================
// vector_stores::files.create + list error paths
// =============================================================================

#[tokio::test]
async fn vector_store_files_error_paths() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/vector_stores/vs/files"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/vector_stores/vs/files/f"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/vector_stores/vs/files/f"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e"))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let _ = c.vector_stores().files("vs").list().await.unwrap_err();
    let _ = c
        .vector_stores()
        .files("vs")
        .retrieve("f")
        .await
        .unwrap_err();
    let _ = c.vector_stores().files("vs").delete("f").await.unwrap_err();
}

// =============================================================================
// chat_stream_ext::collect_chat_stream — stream that errors mid-way bubbles
// =============================================================================

// =============================================================================
// PayLoadBuilder::tools chain (was untested)
// =============================================================================

#[test]
fn payload_builder_tools_setter() {
    use open_ai_rust::{FunctionCall, FunctionParameter, FunctionType};
    let p = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .tools(vec![FunctionCall {
            name: "f".into(),
            description: None,
            parameters: vec![FunctionParameter::new("x", FunctionType::String)],
        }])
        .build();
    assert!(p.tools.is_some());
}

// =============================================================================
// FunctionParameter default_required deserialization path
// =============================================================================

#[test]
fn function_parameter_deserialize_without_required_defaults_true() {
    use open_ai_rust::FunctionParameter;
    let json = r#"{"name":"x","_type":"String","description":null}"#;
    let p: FunctionParameter = serde_json::from_str(json).unwrap();
    assert!(p.required);
}

// =============================================================================
// images.generate (was untested — only edit/variations covered)
// =============================================================================

#[test]
fn chat_message_text_on_text_variant() {
    let m = ChatMessage::user("hello world");
    assert_eq!(m.text(), "hello world");
}

#[tokio::test]
async fn images_variations_without_response_format_field() {
    use open_ai_rust::resources::images::ImageVariationRequest;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/images/variations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1,
            "data": [{ "url": "https://example.com/v.png" }]
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = ImageVariationRequest {
        image: vec![0x89, 0x50, 0x4E, 0x47],
        image_name: "x.png".into(),
        model: None,
        n: None,
        size: None,
        response_format: None,
        user: None,
    };
    c.images().variations(req).await.unwrap();
}

#[tokio::test]
async fn images_edit_without_size_response_format_or_user() {
    use open_ai_rust::resources::images::ImageEditRequest;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/images/edits"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1,
            "data": [{ "url": "https://example.com/e.png" }]
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = ImageEditRequest {
        image: vec![0x89, 0x50, 0x4E, 0x47],
        image_name: "x.png".into(),
        mask: None,
        mask_name: None,
        prompt: "edit".into(),
        model: None,
        n: None,
        size: None,
        response_format: None,
        user: None,
    };
    c.images().edit(req).await.unwrap();
}

#[tokio::test]
async fn images_generate_basic() {
    use open_ai_rust::resources::images::ImageGenerationBuilder;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/images/generations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1,
            "data": [{ "url": "https://example.com/x.png" }]
        })))
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = ImageGenerationBuilder::new("a kitten")
        .model("gpt-image-1")
        .build();
    let resp = c.images().generate(req).await.unwrap();
    assert_eq!(
        resp.data[0].url.as_deref(),
        Some("https://example.com/x.png")
    );
}

// =============================================================================
// ResponseInput::From<String> (only From<&str> was tested before)
// =============================================================================

#[test]
fn response_input_from_owned_string() {
    use open_ai_rust::responses::ResponseInput;
    let s = String::from("hi");
    let r: ResponseInput = s.into();
    assert_eq!(serde_json::to_value(&r).unwrap(), "hi");
}

// =============================================================================
// TranscriptionFormat::as_str — every variant via create() so build_audio_form runs
// =============================================================================

#[tokio::test]
async fn transcription_format_as_str_for_every_variant() {
    use open_ai_rust::resources::audio::{TranscriptionFormat, TranscriptionRequestBuilder};
    for f in [
        TranscriptionFormat::Json,
        TranscriptionFormat::Text,
        TranscriptionFormat::Srt,
        TranscriptionFormat::VerboseJson,
        TranscriptionFormat::Vtt,
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "text": "x" })))
            .mount(&server)
            .await;
        let c = client_for(&server);
        let req = TranscriptionRequestBuilder::new("whisper-1")
            .file_bytes(b"x".to_vec(), "x.mp3")
            .response_format(f)
            .build();
        // create() will fail for non-Json formats (server returns JSON but client tries
        // to deserialize a TranscriptionResponse); use create_text for the non-json ones.
        if matches!(
            f,
            TranscriptionFormat::Json | TranscriptionFormat::VerboseJson
        ) {
            c.audio().transcriptions().create(req).await.unwrap();
        } else {
            // text/srt/vtt — server still returns JSON but we just need build_audio_form
            // to run with each variant. create_text returns plain string from body.
            c.audio().transcriptions().create_text(req).await.unwrap();
        }
    }
}

// =============================================================================
// resources/mod.rs — multipart 4xx error branch + post_multipart timeout branch.
// =============================================================================

#[tokio::test]
async fn multipart_4xx_surfaces_as_api_error() {
    use open_ai_rust::resources::files::FilePurpose;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": { "message": "bad multipart" }
        })))
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked();
    let err = c
        .files()
        .create(b"x".to_vec(), "x.txt", FilePurpose::UserData)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("400"));
}

#[tokio::test]
async fn multipart_with_per_request_timeout_aborts() {
    use open_ai_rust::resources::files::FilePurpose;
    use std::time::Duration;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2))
                .set_body_json(json!({
                    "id": "f", "object": "file", "bytes": 1, "created_at": 1,
                    "filename": "x.txt", "purpose": "user_data"
                })),
        )
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .max_retries(0)
        .build_unchecked()
        .with_timeout(Duration::from_millis(200));
    let start = std::time::Instant::now();
    let result = c
        .files()
        .create(b"x".to_vec(), "x.txt", FilePurpose::UserData)
        .await;
    let elapsed = start.elapsed();
    assert!(result.is_err());
    assert!(
        elapsed < Duration::from_millis(1500),
        "timeout didn't fire: {elapsed:?}"
    );
}

// =============================================================================
// post_json_bytes with per-request timeout (audio speech path)
// =============================================================================

#[tokio::test]
async fn audio_speech_with_per_request_timeout_aborts() {
    use open_ai_rust::resources::audio::SpeechRequestBuilder;
    use std::time::Duration;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/speech"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2))
                .set_body_bytes(b"audio bytes here"),
        )
        .mount(&server)
        .await;
    let c = Client::builder()
        .api_key("k")
        .base_url(server.uri())
        .build_unchecked()
        .with_timeout(Duration::from_millis(200));
    let req = SpeechRequestBuilder::new("gpt-4o-mini-tts", "alloy", "hi").build();
    let result = c.audio().speech().create(req).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn collect_chat_stream_propagates_stream_error() {
    // Server sends an event then an invalid one; collect_chat_stream should bubble Err.
    let body = "data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"m\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\ndata: {not valid json\n\ndata: [DONE]\n\n";
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.as_bytes().to_vec(), "text/event-stream"),
        )
        .mount(&server)
        .await;
    let c = client_for(&server);
    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    let stream = c.chat().create_stream(payload).await.unwrap();
    let result = collect_chat_stream(stream).await;
    assert!(result.is_err(), "collect should propagate stream errors");
}

#[tokio::test]
async fn responses_create_stream_parses_typed_events() {
    use futures_util::StreamExt;
    use open_ai_rust::responses::{ResponseRequestBuilder, ResponseStreamEvent};

    let body = "event: response.output_text.delta\ndata: {\"type\":\"response.output_text.delta\",\"item_id\":\"i\",\"output_index\":0,\"content_index\":0,\"delta\":\"hi\"}\n\ndata: {\"type\":\"response.output_text.done\",\"item_id\":\"i\",\"output_index\":0,\"content_index\":0,\"text\":\"hi\"}\n\ndata: [DONE]\n\n";
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(body.as_bytes().to_vec(), "text/event-stream"),
        )
        .mount(&server)
        .await;
    let c = client_for(&server);
    let req = ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "hi").build();
    let mut s = c.responses().create_stream(req).await.unwrap();
    let mut got_delta = false;
    let mut got_done = false;
    while let Some(ev) = s.next().await {
        match ev.unwrap() {
            ResponseStreamEvent::OutputTextDelta { delta, .. } => {
                assert_eq!(delta, "hi");
                got_delta = true;
            }
            ResponseStreamEvent::OutputTextDone { text, .. } => {
                assert_eq!(text, "hi");
                got_done = true;
            }
            _ => {}
        }
    }
    assert!(got_delta);
    assert!(got_done);
}
