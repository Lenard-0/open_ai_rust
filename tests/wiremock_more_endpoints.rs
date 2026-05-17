//! wiremock fixtures for every endpoint not covered in `wiremock_endpoints.rs`:
//! audio (transcriptions/translations/speech), images (edit/variations), batches,
//! vector_stores, fine_tuning, uploads, and the Responses-API retrieve/cancel/delete paths.

use open_ai_rust::{Client, OpenAiModel};
use serde_json::json;
use wiremock::matchers::{header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(server: &MockServer) -> Client {
    Client::builder()
        .api_key("test-key")
        .base_url(server.uri())
        .build_unchecked()
}

// =============================================================================
// audio
// =============================================================================

#[tokio::test]
async fn audio_transcriptions_create_parses_json_response() {
    use open_ai_rust::resources::audio::TranscriptionRequestBuilder;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .and(header_exists("content-type"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "text": "Hello, world.",
            "language": "english",
            "duration": 1.5,
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = TranscriptionRequestBuilder::new("whisper-1")
        .file_bytes(b"fake mp3 bytes".to_vec(), "test.mp3")
        .mime_type("audio/mpeg")
        .build();
    let resp = client.audio().transcriptions().create(req).await.unwrap();
    assert_eq!(resp.text, "Hello, world.");
    assert_eq!(resp.language.as_deref(), Some("english"));
    assert_eq!(resp.duration, Some(1.5));
}

#[tokio::test]
async fn audio_transcriptions_create_text_returns_plain_body() {
    use open_ai_rust::resources::audio::{TranscriptionFormat, TranscriptionRequestBuilder};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("just plain text"))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = TranscriptionRequestBuilder::new("whisper-1")
        .file_bytes(b"x".to_vec(), "x.mp3")
        .response_format(TranscriptionFormat::Text)
        .build();
    let text = client
        .audio()
        .transcriptions()
        .create_text(req)
        .await
        .unwrap();
    assert_eq!(text, "just plain text");
}

#[tokio::test]
async fn audio_translations_create() {
    use open_ai_rust::resources::audio::TranscriptionRequestBuilder;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/translations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "text": "translated" })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = TranscriptionRequestBuilder::new("whisper-1")
        .file_bytes(b"x".to_vec(), "x.mp3")
        .build();
    let resp = client.audio().translations().create(req).await.unwrap();
    assert_eq!(resp.text, "translated");
}

#[tokio::test]
async fn audio_speech_create_returns_raw_bytes() {
    use open_ai_rust::resources::audio::SpeechRequestBuilder;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/speech"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "audio/mpeg")
                .set_body_bytes(b"\xFFid3\x03\x00\x00\x00fakeMP3"),
        )
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = SpeechRequestBuilder::new("gpt-4o-mini-tts", "alloy", "say something")
        .response_format("mp3")
        .build();
    let bytes = client.audio().speech().create(req).await.unwrap();
    assert!(bytes.starts_with(b"\xFFid3"));
    assert!(bytes.len() > 5);
}

// =============================================================================
// images — edit + variations
// =============================================================================

#[tokio::test]
async fn images_edit_uses_multipart_with_prompt_and_image() {
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

    let client = client_for(&server);
    let req = ImageEditRequest {
        image: vec![0x89, 0x50, 0x4E, 0x47], // PNG magic
        image_name: "src.png".into(),
        mask: None,
        mask_name: None,
        prompt: "make it gold".into(),
        model: Some("gpt-image-1".into()),
        n: Some(1),
        size: Some("1024x1024".into()),
        response_format: None,
        user: None,
    };
    let resp = client.images().edit(req).await.unwrap();
    assert_eq!(resp.data.len(), 1);
    assert_eq!(
        resp.data[0].url.as_deref(),
        Some("https://example.com/edited.png")
    );
}

#[tokio::test]
async fn images_variations_returns_b64() {
    use open_ai_rust::resources::images::ImageVariationRequest;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/images/variations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1,
            "data": [{ "b64_json": "iVBORw0KG..." }],
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let req = ImageVariationRequest {
        image: vec![0x89, 0x50, 0x4E, 0x47],
        image_name: "src.png".into(),
        model: None,
        n: Some(2),
        size: None,
        response_format: Some("b64_json".into()),
        user: None,
    };
    let resp = client.images().variations(req).await.unwrap();
    assert_eq!(resp.data[0].b64_json.as_deref(), Some("iVBORw0KG..."));
}

// =============================================================================
// batches
// =============================================================================

#[tokio::test]
async fn batches_full_lifecycle() {
    use open_ai_rust::resources::batches::BatchCreateRequest;

    let server = MockServer::start().await;
    let batch_body = json!({
        "id": "batch_abc",
        "object": "batch",
        "endpoint": "/v1/chat/completions",
        "input_file_id": "file_in",
        "completion_window": "24h",
        "status": "in_progress",
        "created_at": 1,
    });
    Mock::given(method("POST"))
        .and(path("/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_body.clone()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/batches/batch_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_body.clone()))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/batches/batch_abc/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut b = batch_body.clone();
            b["status"] = json!("cancelling");
            b
        }))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [batch_body.clone()],
            "has_more": false,
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);

    let created = client
        .batches()
        .create(BatchCreateRequest {
            input_file_id: "file_in".into(),
            endpoint: "/v1/chat/completions".into(),
            completion_window: "24h".into(),
            metadata: None,
        })
        .await
        .unwrap();
    assert_eq!(created.id, "batch_abc");

    let retrieved = client.batches().retrieve("batch_abc").await.unwrap();
    assert_eq!(retrieved.status, "in_progress");

    let cancelled = client.batches().cancel("batch_abc").await.unwrap();
    assert_eq!(cancelled.status, "cancelling");

    let list = client.batches().list().await.unwrap();
    assert_eq!(list.data.len(), 1);
}

// =============================================================================
// vector_stores
// =============================================================================

#[tokio::test]
async fn vector_stores_full_lifecycle() {
    use open_ai_rust::resources::vector_stores::VectorStoreCreateRequest;

    let server = MockServer::start().await;
    let vs_body = json!({
        "id": "vs_1",
        "object": "vector_store",
        "created_at": 1,
        "name": "kb",
        "usage_bytes": 0,
        "file_counts": { "in_progress": 0, "completed": 0, "failed": 0, "cancelled": 0, "total": 0 },
        "status": "completed",
    });
    Mock::given(method("POST"))
        .and(path("/vector_stores"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vs_body.clone()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/vector_stores"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [vs_body.clone()],
            "has_more": false,
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/vector_stores/vs_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vs_body.clone()))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/vector_stores/vs_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "vs_1", "object": "vector_store.deleted", "deleted": true,
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let created = client
        .vector_stores()
        .create(VectorStoreCreateRequest {
            name: Some("kb".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(created.id, "vs_1");

    let list = client.vector_stores().list().await.unwrap();
    assert_eq!(list.data.len(), 1);

    let retrieved = client.vector_stores().retrieve("vs_1").await.unwrap();
    assert_eq!(retrieved.id, "vs_1");

    let deleted = client.vector_stores().delete("vs_1").await.unwrap();
    assert!(deleted.deleted);
}

#[tokio::test]
async fn vector_store_files_nested_resource() {
    let server = MockServer::start().await;
    let vsf = json!({
        "id": "vsf_1",
        "object": "vector_store.file",
        "created_at": 1,
        "vector_store_id": "vs_1",
        "status": "completed",
    });
    Mock::given(method("POST"))
        .and(path("/vector_stores/vs_1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vsf.clone()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/vector_stores/vs_1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [vsf.clone()],
            "has_more": false,
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/vector_stores/vs_1/files/vsf_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vsf.clone()))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/vector_stores/vs_1/files/vsf_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "vsf_1", "object": "vector_store.file.deleted", "deleted": true,
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let files = client.vector_stores().files("vs_1");

    let added = files.create("file_xyz").await.unwrap();
    assert_eq!(added.id, "vsf_1");

    let list = files.list().await.unwrap();
    assert_eq!(list.data.len(), 1);

    let retrieved = files.retrieve("vsf_1").await.unwrap();
    assert_eq!(retrieved.status, "completed");

    let deleted = files.delete("vsf_1").await.unwrap();
    assert!(deleted.deleted);
}

// =============================================================================
// fine_tuning
// =============================================================================

#[tokio::test]
async fn fine_tuning_jobs_lifecycle() {
    use open_ai_rust::resources::fine_tuning::FineTuningJobRequest;

    let server = MockServer::start().await;
    let job = json!({
        "id": "ft_1",
        "object": "fine_tuning.job",
        "model": "gpt-4o-mini",
        "created_at": 1,
        "status": "queued",
        "organization_id": "org_1",
        "training_file": "file_train",
        "result_files": [],
    });
    Mock::given(method("POST"))
        .and(path("/fine_tuning/jobs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(job.clone()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list", "data": [job.clone()], "has_more": false,
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs/ft_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(job.clone()))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/fine_tuning/jobs/ft_1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut j = job.clone();
            j["status"] = json!("cancelled");
            j
        }))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs/ft_1/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{ "object": "fine_tuning.job.event", "id": "ev_1", "level": "info", "message": "queued" }],
            "has_more": false,
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/fine_tuning/jobs/ft_1/checkpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list", "data": [], "has_more": false,
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let jobs = client.fine_tuning().jobs();

    let created = jobs
        .create(FineTuningJobRequest {
            model: "gpt-4o-mini".into(),
            training_file: "file_train".into(),
            validation_file: None,
            hyperparameters: None,
            suffix: None,
            seed: None,
            integrations: None,
            metadata: None,
        })
        .await
        .unwrap();
    assert_eq!(created.id, "ft_1");

    let list = jobs.list().await.unwrap();
    assert_eq!(list.data.len(), 1);

    let retrieved = jobs.retrieve("ft_1").await.unwrap();
    assert_eq!(retrieved.status, "queued");

    let cancelled = jobs.cancel("ft_1").await.unwrap();
    assert_eq!(cancelled.status, "cancelled");

    let events = jobs.list_events("ft_1").await.unwrap();
    assert_eq!(events.data.len(), 1);

    let checkpoints = jobs.list_checkpoints("ft_1").await.unwrap();
    assert_eq!(checkpoints["object"], "list");
}

// =============================================================================
// uploads
// =============================================================================

#[tokio::test]
async fn uploads_full_lifecycle() {
    use open_ai_rust::resources::uploads::UploadCreateRequest;

    let server = MockServer::start().await;
    let upload = json!({
        "id": "upload_1",
        "object": "upload",
        "bytes": 1024,
        "created_at": 1,
        "filename": "big.bin",
        "purpose": "user_data",
        "status": "pending",
        "expires_at": 9999,
    });
    Mock::given(method("POST"))
        .and(path("/uploads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(upload.clone()))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/uploads/upload_1/parts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "part_1",
            "object": "upload.part",
            "upload_id": "upload_1",
            "created_at": 2,
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/uploads/upload_1/complete"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut u = upload.clone();
            u["status"] = json!("completed");
            u
        }))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/uploads/upload_1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut u = upload.clone();
            u["status"] = json!("cancelled");
            u
        }))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let uploads = client.uploads();

    let created = uploads
        .create(UploadCreateRequest {
            filename: "big.bin".into(),
            purpose: "user_data".into(),
            bytes: 1024,
            mime_type: "application/octet-stream".into(),
        })
        .await
        .unwrap();
    assert_eq!(created.id, "upload_1");

    let part = uploads
        .add_part("upload_1", b"first chunk".to_vec())
        .await
        .unwrap();
    assert_eq!(part.id, "part_1");

    let completed = uploads
        .complete("upload_1", vec!["part_1".into()])
        .await
        .unwrap();
    assert_eq!(completed.status, "completed");

    // Cancel works on a different upload (lifecycle done, but exercise the path)
    let cancelled = uploads.cancel("upload_1").await.unwrap();
    assert_eq!(cancelled.status, "cancelled");
}

// =============================================================================
// responses — retrieve / cancel / delete
// =============================================================================

#[tokio::test]
async fn responses_retrieve_returns_stored_response() {
    use open_ai_rust::responses::ResponseStatus;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/responses/resp_x"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_x",
            "object": "response",
            "created_at": 1,
            "model": "gpt-4.1-mini",
            "status": "completed",
            "output": [],
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.responses().retrieve("resp_x").await.unwrap();
    assert_eq!(resp.id, "resp_x");
    assert_eq!(resp.status, ResponseStatus::Completed);
}

#[tokio::test]
async fn responses_cancel_returns_cancelled_response() {
    use open_ai_rust::responses::ResponseStatus;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses/resp_x/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "resp_x",
            "object": "response",
            "created_at": 1,
            "model": "gpt-4.1-mini",
            "status": "cancelled",
            "output": [],
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client.responses().cancel("resp_x").await.unwrap();
    assert_eq!(resp.status, ResponseStatus::Cancelled);
}

#[tokio::test]
async fn responses_delete_returns_ok_on_204() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/responses/resp_x"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = client_for(&server);
    client.responses().delete("resp_x").await.unwrap();
}

// =============================================================================
// models — retrieve + delete (list is in wiremock_endpoints.rs)
// =============================================================================

#[tokio::test]
async fn models_retrieve_one() {
    let _ = OpenAiModel::GPT4oMini; // suppress unused import warning if any
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models/gpt-4o-mini"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "gpt-4o-mini", "object": "model", "created": 1, "owned_by": "openai",
        })))
        .mount(&server)
        .await;
    let client = client_for(&server);
    let m = client.models().retrieve("gpt-4o-mini").await.unwrap();
    assert_eq!(m.id, "gpt-4o-mini");
}

#[tokio::test]
async fn models_delete_fine_tuned() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/models/ft:gpt-4o-mini:org::abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "ft:gpt-4o-mini:org::abc", "object": "model", "deleted": true,
        })))
        .mount(&server)
        .await;
    let client = client_for(&server);
    let resp = client
        .models()
        .delete("ft:gpt-4o-mini:org::abc")
        .await
        .unwrap();
    assert!(resp.deleted);
}

// =============================================================================
// files.list + files.content (only files.create is in wiremock_endpoints.rs)
// =============================================================================

#[tokio::test]
async fn files_list_returns_collection() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [
                { "id": "file_a", "object": "file", "bytes": 10, "created_at": 1, "filename": "a", "purpose": "user_data" },
                { "id": "file_b", "object": "file", "bytes": 20, "created_at": 2, "filename": "b", "purpose": "user_data" },
            ],
            "has_more": false,
        })))
        .mount(&server)
        .await;
    let client = client_for(&server);
    let list = client.files().list().await.unwrap();
    assert_eq!(list.data.len(), 2);
}

#[tokio::test]
async fn files_content_returns_raw_bytes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/file_a/content"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"raw file bytes"))
        .mount(&server)
        .await;
    let client = client_for(&server);
    let bytes = client.files().content("file_a").await.unwrap();
    assert_eq!(&bytes[..], b"raw file bytes");
}
