//! Criterion benches for the serde-heavy hot paths users hit on every request.
//!
//! Run with: `cargo bench`
//!
//! Benches don't run in CI by default. To detect regressions, save a baseline:
//!   `cargo bench --bench hot_paths -- --save-baseline before`
//!   <make changes>
//!   `cargo bench --bench hot_paths -- --baseline before`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use open_ai_rust::logoi::input::tool::{EnumValues, FunctionCall, FunctionParameter, FunctionType};
use open_ai_rust::{AiMsgResponse, ChatCompletionChunk, ChatMessage, OpenAiModel, PayLoadBuilder};
use serde_json::json;

fn sample_payload() -> open_ai_rust::ChatPayLoad {
    PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![
            ChatMessage::system("You are helpful."),
            ChatMessage::user("Plan a 3-day trip to Sydney."),
        ])
        .temperature(0.4)
        .top_p(0.9)
        .max_completion_tokens(1024)
        .seed(7)
        .build()
}

fn sample_response_json() -> String {
    json!({
        "id": "chatcmpl-abc",
        "object": "chat.completion",
        "created": 1700000000,
        "model": "gpt-4o-mini",
        "system_fingerprint": "fp_x",
        "choices": [{
            "finish_reason": "stop",
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Day 1: arrive in Sydney. Day 2: Bondi Beach + Manly. Day 3: harbour cruise.",
            },
        }],
        "usage": {
            "prompt_tokens": 24,
            "completion_tokens": 28,
            "total_tokens": 52,
            "prompt_tokens_details": { "cached_tokens": 0 },
            "completion_tokens_details": { "reasoning_tokens": 0 },
        },
    })
    .to_string()
}

fn sample_chunk_json() -> String {
    json!({
        "id": "chatcmpl-abc",
        "object": "chat.completion.chunk",
        "created": 1700000000,
        "model": "gpt-4o-mini",
        "choices": [{
            "index": 0,
            "delta": { "role": "assistant", "content": "Hello, world." },
            "finish_reason": null
        }]
    })
    .to_string()
}

fn sample_function_call() -> FunctionCall {
    FunctionCall {
        name: "create_event".into(),
        description: Some("Create a calendar event.".into()),
        parameters: vec![
            FunctionParameter::new("title", FunctionType::String).description("Event title"),
            FunctionParameter::new(
                "attendees",
                FunctionType::Array(Box::new(FunctionType::String)),
            ),
            FunctionParameter::new(
                "details",
                FunctionType::Object(vec![
                    FunctionParameter::new("location", FunctionType::String),
                    FunctionParameter::new(
                        "category",
                        FunctionType::Enum(EnumValues::String(vec![
                            "work".into(),
                            "personal".into(),
                            "other".into(),
                        ])),
                    ),
                ]),
            ),
            FunctionParameter::new("tags", FunctionType::Map(Box::new(FunctionType::String)))
                .required(false),
        ],
    }
}

fn bench_serialize_chat_payload(c: &mut Criterion) {
    let payload = sample_payload();
    c.bench_function("serialize_chat_payload", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&payload)).unwrap();
            black_box(s);
        })
    });
}

fn bench_deserialize_chat_response(c: &mut Criterion) {
    let body = sample_response_json();
    c.bench_function("deserialize_chat_response", |b| {
        b.iter(|| {
            let r: AiMsgResponse = serde_json::from_str(black_box(&body)).unwrap();
            black_box(r);
        })
    });
}

fn bench_deserialize_chunk(c: &mut Criterion) {
    let body = sample_chunk_json();
    c.bench_function("deserialize_chat_chunk", |b| {
        b.iter(|| {
            let r: ChatCompletionChunk = serde_json::from_str(black_box(&body)).unwrap();
            black_box(r);
        })
    });
}

fn bench_serialize_function_call_schema(c: &mut Criterion) {
    // This hits the hand-rolled `Serialize` impl in src/logoi/input/tool/serialise/*.
    let fc = sample_function_call();
    c.bench_function("serialize_function_call_schema", |b| {
        b.iter(|| {
            let v = serde_json::to_value(black_box(&fc)).unwrap();
            black_box(v);
        })
    });
}

criterion_group!(
    benches,
    bench_serialize_chat_payload,
    bench_deserialize_chat_response,
    bench_deserialize_chunk,
    bench_serialize_function_call_schema,
);
criterion_main!(benches);
