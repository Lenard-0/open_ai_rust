# Migrating from `0.2.x` to `1.0.0`

`1.0` is a large modernisation and the crate's exit from pre-1.0: a real `Client` type, every endpoint covered, streaming, typed errors, multi-part chat content, the new Responses API, an opt-in tool-dispatch registry, and clean `cargo fmt`/`clippy` hygiene.

> **Hard break.** The 0.2.x global-state API (`set_key`, `set_ai_msg_endpoint`, `set_embeddings_endpoint`, `open_ai_msg`, `embed`, the `RequestType` enum, the `ResponseFormatInput` alias, the `ChatMessageRole::Function` variant) has been fully removed. You must move to `Client` to upgrade.

This page is a codemod-style cheat sheet. Each "before → after" block shows the minimum diff.

---

## 1. Client construction

### Before

```rust
use open_ai_rust::{requests::open_ai_msg, set_key};

set_key(std::env::var("OPENAI_SK")?);
let resp = open_ai_msg(payload).await?;
```

### After

```rust
use open_ai_rust::Client;

let client = Client::from_env()?; // reads OPENAI_API_KEY
let resp = client.chat().create(payload).await?;
```

Or explicit:

```rust
let client = Client::new(std::env::var("OPENAI_API_KEY")?);
```

The old `set_key` / `open_ai_msg` / `embed` still work but emit deprecation warnings.

---

## 2. Azure

### Before

```rust
set_key(azure_key);
set_ai_msg_endpoint("https://my-resource.openai.azure.com/openai/deployments/my-deploy/chat/completions?api-version=...");
```

### After

```rust
let client = Client::azure(
    azure_key,
    "https://my-resource.openai.azure.com",
    "my-deploy",
    "2024-10-01-preview",
);
```

`Client::azure` constructs the deployment path + `api-version` query string + sends `api-key` (not `Bearer`) automatically.

---

## 3. `ChatMessage` content type

`content: String` is now `content: ChatContent` (untagged enum: either string or array of typed parts).

### Before

```rust
ChatMessage {
    role: ChatMessageRole::User,
    content: "hello".to_string(),
    name: None,
}
```

### After — pick one

Helper constructor (recommended):

```rust
ChatMessage::user("hello")
```

Or struct literal with `.into()`:

```rust
ChatMessage {
    role: ChatMessageRole::User,
    content: "hello".into(),
    name: None,
    tool_call_id: None,
    tool_calls: None,
    refusal: None,
}
```

Multi-content (image input):

```rust
ChatMessage::user("Describe this image:")
    .with_image("https://example.com/cat.png")
```

Tool reply:

```rust
ChatMessage::tool("call_123", r#"{"temperature": 22}"#)
```

---

## 4. `ChatPayLoad` field changes

| Field | Before | After |
|-------|--------|-------|
| `tool_choice` | `Option<String>` | `Option<ChatToolChoice>` (still accepts `"auto"` / `"required"` / `"none"` via `From<&str>`) |
| `stream_options` | `Option<bool>` (wire-format wrong) | `Option<StreamOptions>` (`{ include_usage: bool }`) |
| `response_format` | `Option<ResponseFormatInput>` (only `Text` / `JsonObject`) | `Option<ResponseFormat>` (adds `JsonSchema { name, schema, strict }`) |

```rust
// Before
.tool_choice("auto".to_string())
.stream_options(true)  // ← was sending wrong shape

// After
.tool_choice(ChatToolChoice::auto())
.include_usage(true)   // shorthand
```

Structured outputs via JSON Schema:

```rust
.response_format(ResponseFormat::json_schema("city", json!({
    "type": "object",
    "properties": { "name": { "type": "string" } },
    "required": ["name"],
    "additionalProperties": false
})))
```

New fields on `ChatPayLoad`: `max_completion_tokens`, `reasoning_effort`, `metadata`, `store`, `parallel_tool_calls`. Required for reasoning models (`o1`, `o3`, `gpt-5`).

---

## 5. `FunctionParameter` adds `required` field

```rust
FunctionParameter {
    name: "city".into(),
    _type: FunctionType::String,
    description: None,
    required: true,        // ← new — defaults to true
}
```

Use `FunctionParameter::new(name, _type)` for the builder path:

```rust
FunctionParameter::new("city", FunctionType::String)
    .description("The city name")
    .required(false)
```

Optionality model: a parameter ends up in JSON-Schema `required[]` iff `param.required == true && !matches!(_type, FunctionType::Option(_))`.

New `FunctionType` variants you can now use:

- `FunctionType::Map(Box<...>)` — `HashMap<String, V>` / `BTreeMap<String, V>` → `{ "type": "object", "additionalProperties": ... }`
- `FunctionType::OneOf(Vec<FunctionVariant>)` — Rust enums with data variants → JSON Schema `oneOf`

---

## 6. Output / response types

- `AiMsgResponse.system_fingerprint` is now `Option<String>` (was `String`).
- `AiResponseMessage` gains `refusal: Option<String>`, `audio: Option<Value>`.
- `Usage` gains `prompt_tokens_details` and `completion_tokens_details` (both `Option`).
- `ToolCallRes` gains `id: Option<String>` and `type_: Option<String>`.

If you struct-literal `Usage { ... }` in tests, add the two new `None` fields.

---

## 7. Errors

`Result<T, String>` is the old surface. The new typed alias is:

```rust
pub type Result<T> = std::result::Result<T, OpenAiError>;
```

`OpenAiError: From<reqwest::Error> + From<serde_json::Error> + From<std::io::Error>` and `From<OpenAiError> for String` is provided — so functions that returned `Result<_, String>` still compile if you just keep using `?` on the new client methods and convert at the boundary.

Useful match arms:

```rust
match e {
    OpenAiError::Api { status: 429, .. } => /* retry yourself */,
    OpenAiError::Api { status, message, code, .. } => /* surface to user */,
    OpenAiError::Reqwest(e) if e.is_timeout() => /* tighten timeout */,
    OpenAiError::Decode(_) => /* server returned malformed JSON */,
    _ => /* … */,
}
```

---

## 8. Streaming

New in 1.0 (didn't exist before):

```rust
use futures_util::StreamExt;

let mut stream = client.chat().create_stream(payload).await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?.delta_text());
}
```

Helper to assemble a streamed response back into a single text + tool-call set:

```rust
use open_ai_rust::collect_chat_stream;
let assembled = collect_chat_stream(stream).await?;
println!("{}", assembled.content);
```

---

## 9. Responses API (new flagship)

```rust
use open_ai_rust::responses::ResponseRequestBuilder;

let req = ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "Write a haiku.")
    .instructions("You are a haiku poet.")
    .build();
let resp = client.responses().create(req).await?;
println!("{}", resp.output_text());
```

Streaming with typed events:

```rust
let mut events = client.responses().create_stream(req).await?;
while let Some(ev) = events.next().await {
    if let open_ai_rust::responses::ResponseStreamEvent::OutputTextDelta { delta, .. } = ev? {
        print!("{delta}");
    }
}
```

---

## 10. Per-request options

```rust
let resp = client
    .with_timeout(Duration::from_secs(30))
    .with_max_retries(5)
    .with_idempotency_key("user-42-msg-7")
    .with_header("x-trace-id", "abc-123")
    .chat()
    .create(payload)
    .await?;
```

Each `.with_*` returns a `Client` clone sharing the same `Arc<ClientInner>` — cheap. `max_retries` applies to JSON code paths only (streaming + multipart are single-shot).

---

## 11. Feature flags (opt-in)

| Feature        | Effect |
|----------------|--------|
| `rustls-tls`   | TLS via rustls (default) |
| `native-tls`   | TLS via system OpenSSL |
| `stream`       | (default) streaming helpers |
| `tool_registry`| `linkme`-backed dispatch slice for `#[tool]` macro |
| `tracing`      | `debug!`/`warn!` on every HTTP request + retry |
| `utoipa`       | derive `ToSchema` on enums (was unconditional in 0.2; now opt-in) |
| `macro_v2`     | enable the 2 derive-macro tests once `open_ai_rust_fn_call_extension v0.3` ships |

Azure support is now built in (no feature flag) — use [`Client::azure`].

If you used `utoipa::ToSchema` from this crate in 0.2 you'll need to add `features = ["utoipa"]` to your `Cargo.toml` dependency on `open_ai_rust` in 1.0.

---

## 12. Module re-exports (flat surface)

In addition to the existing `logoi::...` paths, every public type is now re-exported at the crate root:

```rust
use open_ai_rust::{
    ChatMessage, ChatMessageRole, ChatContent, ContentPart,
    ChatPayLoad, PayLoadBuilder, ChatToolChoice, ResponseFormat, ReasoningEffort,
    OpenAiModel, FunctionCall, FunctionParameter, FunctionType, FunctionVariant,
    AiMsgResponse, Usage, Client, ClientBuilder, OpenAiError, Result,
    RequestOptions, ChatCompletionChunk, collect_chat_stream,
};
```

---

## 13. Things removed

- `requests::audio::stt` module (entirely commented out in 0.2; superseded by `client.audio().transcriptions()`).

Nothing else has been deleted — everything else is `#[deprecated]` so a clean upgrade is a warning-fix exercise, not a rewrite.
