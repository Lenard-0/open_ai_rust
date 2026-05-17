# open_ai_rust

[![crates.io](https://img.shields.io/crates/v/open_ai_rust.svg)](https://crates.io/crates/open_ai_rust)
[![docs.rs](https://img.shields.io/docsrs/open_ai_rust)](https://docs.rs/open_ai_rust)
[![downloads](https://img.shields.io/crates/d/open_ai_rust.svg)](https://crates.io/crates/open_ai_rust)
[![license](https://img.shields.io/crates/l/open_ai_rust.svg)](./LICENSE)

A comprehensive, idiomatic Rust SDK for the OpenAI API.

Mirrors the official OpenAI SDK's namespacing — `client.chat().create(...)`, `client.responses().create(...)`, `client.embeddings().create(...)` — while keeping a few Rust-flavoured ergonomics that the official clients lack: typed function-call schemas derived from your structs, builder-pattern payloads, retry / timeout / idempotency-key configuration per request, and `Result<T, OpenAiError>` everywhere.

- **MSRV:** Rust 1.75
- **Async runtime:** `tokio`
- **HTTP:** `reqwest` (default `rustls`, opt-in `native-tls`)
- **License:** Apache-2.0

## Install

```toml
[dependencies]
open_ai_rust = "1"
tokio = { version = "1", features = ["full"] }
```

## Quick start

```rust,no_run
use open_ai_rust::{ChatMessage, Client, OpenAiModel, PayLoadBuilder};

#[tokio::main]
async fn main() -> open_ai_rust::Result<()> {
    let client = Client::from_env()?; // reads OPENAI_API_KEY

    let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![
            ChatMessage::system("You are helpful."),
            ChatMessage::user("Say hi."),
        ])
        .temperature(0.2)
        .build();

    let resp = client.chat().create(payload).await?;
    println!("{}", resp.get_last_msg_text().unwrap_or_default());
    Ok(())
}
```

## Coverage

| Resource           | Entry point                                                            | Notes |
| ------------------ | ---------------------------------------------------------------------- | ----- |
| Chat completions   | `client.chat().create(...)`                                            | streaming via `.create_stream(...)` |
| Responses API      | `client.responses().create(...)`                                       | flagship; streaming + `retrieve` / `cancel` / `delete` |
| Embeddings         | `client.embeddings().create(...)`                                      | `create_one(text, model)` shortcut for the common case |
| Audio              | `client.audio().{transcriptions,translations,speech}()`                | whisper, gpt-4o-transcribe, tts |
| Images             | `client.images().{generate,edit,variations}(...)`                      | dall-e + gpt-image-1 |
| Moderations        | `client.moderations().create(...)`                                     | text + image inputs |
| Files              | `client.files().{create,list,retrieve,delete,content}(...)`            | multipart upload |
| Models             | `client.models().{list,retrieve,delete}(...)`                          | |
| Batches            | `client.batches().{create,retrieve,cancel,list}(...)`                  | |
| Vector stores      | `client.vector_stores().{create,list,retrieve,delete}(...)` + `.files(id)` | |
| Fine-tuning        | `client.fine_tuning().jobs().{create,list,retrieve,cancel,list_events,list_checkpoints}(...)` | |
| Uploads            | `client.uploads().{create,add_part,complete,cancel}(...)`              | resumable, for files > 512 MB |

## Streaming

```rust,no_run
# use open_ai_rust::{ChatMessage, Client, OpenAiModel, PayLoadBuilder};
# async fn run() -> open_ai_rust::Result<()> {
# let client = Client::from_env()?;
# let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini).messages(vec![ChatMessage::user("hi")]).build();
use futures_util::StreamExt;

let mut stream = client.chat().create_stream(payload).await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?.delta_text());
}
# Ok(()) }
```

For the Responses API, `client.responses().create_stream(...)` yields typed [`ResponseStreamEvent`](https://docs.rs/open_ai_rust/latest/open_ai_rust/responses/enum.ResponseStreamEvent.html) variants (one per OpenAI server-sent event).

## Structured outputs

```rust,no_run
use open_ai_rust::{ChatMessage, OpenAiModel, PayLoadBuilder, ResponseFormat};
use serde_json::json;

let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
    .messages(vec![ChatMessage::user("Describe Sydney.")])
    .response_format(ResponseFormat::json_schema("city", json!({
        "type": "object",
        "properties": { "name": { "type": "string" }, "population": { "type": "integer" } },
        "required": ["name", "population"],
        "additionalProperties": false
    })))
    .build();
```

## Function / tool calls

Hand-written schema:

```rust,no_run
use open_ai_rust::{
    ChatMessage, Client, FunctionCall, FunctionParameter, FunctionType,
    OpenAiModel, PayLoadBuilder,
};

# async fn run() -> open_ai_rust::Result<()> {
# let client = Client::from_env()?;
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
    .messages(vec![ChatMessage::user("Weather in Sydney?")])
    .tools(vec![tool])
    .build();

let resp = client.chat().create(payload).await?;
for tc in resp.get_tool_calls() {
    println!("{}({})", tc.name, tc.arguments);
}
# Ok(()) }
```

Schema derived from a Rust struct (via the companion crate
[`open_ai_rust_fn_call_extension`](https://crates.io/crates/open_ai_rust_fn_call_extension)):

```ignore
use open_ai_rust::{ChatMessage, OpenAiModel, PayLoadBuilder};
use open_ai_rust::logoi::input::tool::raw_macro::FunctionCallable;
use open_ai_rust_fn_call_extension::FunctionCall;

#[derive(FunctionCall)]
struct GetWeather {
    /// Name of the city.
    city: String,
}

let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
    .messages(vec![ChatMessage::user("Weather in Sydney?")])
    .tools(vec![GetWeather::fn_schema()])
    .build();
```

Doc-comments on fields become parameter descriptions; `Option<T>` and `#[fc(required = false)]` mark optional parameters.

## Per-request retries, timeouts, idempotency

`Client` defaults can be overridden per logical call:

```rust,no_run
use std::time::Duration;
use open_ai_rust::{Client, RequestOptions};

# async fn run() -> open_ai_rust::Result<()> {
let client = Client::from_env()?;

// One-off override:
let resp = client
    .with_timeout(Duration::from_secs(30))
    .with_max_retries(5)
    .with_idempotency_key("evt-42")
    .chat()
    .create(/* payload */ todo!())
    .await?;

// Or compose explicitly:
let scoped = client.with_options(
    RequestOptions::new()
        .timeout(Duration::from_secs(10))
        .max_retries(3)
        .header("x-trace-id", "abc-123"),
);
# let _ = resp; let _ = scoped; Ok(()) }
```

Retries apply to JSON requests on HTTP 429 / 5xx / connection errors, with exponential backoff (500 ms → 30 s cap). Streaming and multipart uploads are single-shot — bodies / SSE streams cannot be replayed.

## Errors

Every fallible call returns [`Result<T, OpenAiError>`](https://docs.rs/open_ai_rust/latest/open_ai_rust/enum.OpenAiError.html). Variants:

| Variant   | When                                                       | Retried? |
| --------- | ---------------------------------------------------------- | -------- |
| `Api`     | OpenAI returned non-2xx with a JSON error envelope         | 429 / 5xx — yes |
| `Reqwest` | network / connect / timeout error from `reqwest`           | connect & timeout — yes |
| `Decode`  | response body did not match the expected schema            | no |
| `Stream`  | malformed SSE chunk or premature stream close              | no |
| `Config`  | misuse of the client (missing API key, bad Azure deployment) | no |
| `Io`      | local I/O failure (e.g. multipart file read)               | no |

## Azure OpenAI

```rust
use open_ai_rust::Client;

let client = Client::azure(
    "az-key",
    "https://my-resource.openai.azure.com",
    "gpt-4o-deployment",
    "2024-10-01-preview",
);
```

Uses the `api-key` header (not `Bearer`) and appends `?api-version=...` automatically.

## Feature flags

| Feature         | Default | Purpose |
| --------------- | :-----: | ------- |
| `rustls-tls`    | ✅      | TLS via rustls |
| `stream`        | ✅      | streaming helpers (`create_stream`, `collect_chat_stream`) |
| `native-tls`    |         | TLS via system OpenSSL |
| `tracing`       |         | `debug!` / `warn!` on every HTTP request + retry, spans on each call |
| `utoipa`        |         | derive `ToSchema` on enums (for OpenAPI generation) |
| `tool_registry` |         | `linkme`-backed dispatch slice for the `#[tool]` attribute macro |
| `macro_v2`      |         | enable derive-macro tests once `open_ai_rust_fn_call_extension` ships v0.3 |

## Migration from `0.2.x`

`1.0` is a breaking redesign — see [`MIGRATION.md`](./MIGRATION.md) for a full codemod-style upgrade guide. TL;DR: replace global-state helpers (`set_key`, `set_ai_msg_endpoint`, `open_ai_msg`, `embed`, …) with a `Client`, swap `ChatMessage` struct literals for `ChatMessage::user("...")` helpers, accept new `Option` fields on `Usage` / `AiMsgResponse`, and opt into `utoipa` via the feature flag if you used `ToSchema`. The legacy free functions are no longer compiled — there is no incremental path.

## Examples

Runnable examples in [`examples/`](./examples):

```sh
cargo run --example chat_basic
cargo run --example chat_stream
cargo run --example structured_output
cargo run --example function_call
cargo run --example responses_basic
cargo run --example responses_stream
cargo run --example embed_text
cargo run --example transcription -- path/to/audio.mp3
cargo run --example tts -- "Hello world" out.mp3
```

All examples expect `OPENAI_API_KEY` in the environment (or in a `.env` file).

## Comparison with `async-openai`

`async-openai` is the most established alternative; this crate occupies a slightly different niche:

- **Typed function-call schemas from your structs.** `#[derive(FunctionCall)]` emits the JSON schema automatically — `async-openai` requires you to hand-build the schema as `serde_json::Value` or via builders.
- **Builder + struct in one type.** `PayLoadBuilder` keeps required-vs-optional explicit at the type level.
- **Per-request `RequestOptions`** (retries / timeouts / idempotency keys / extra headers) without rebuilding the client.
- **Smaller surface, fewer transitive deps** (no `derive_builder` / `secrecy` / etc.).

`async-openai` has been around longer, supports more peripheral endpoints (assistants v1, threads, runs), and has a larger user base — if you need those, it's still the right choice.

## Contributing

Issues and PRs welcome at <https://github.com/Lenard-0/open_ai_rust>. Please run `cargo test` and `cargo clippy --all-targets` before submitting.

## License

Apache-2.0 — see [LICENSE](./LICENSE).
