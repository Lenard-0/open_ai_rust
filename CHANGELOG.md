# Changelog

## 1.0.0 — unreleased

First stable release. A near-total rewrite of the public API: a real `Client` type, every documented OpenAI endpoint covered, streaming, typed errors, multi-modal chat content, the new Responses API, and an opt-in tool-dispatch registry. **The pre-1.0 global-state free functions and helpers have been removed outright.**

**Breaking change** vs. published `0.2.16`: see [`MIGRATION.md`](./MIGRATION.md) for the codemod cheat sheet.

### Removed (was deprecated in 0.3 working tree, never released)
- `set_key`, `set_ai_msg_endpoint`, `set_ai_msg_endpoint_default`, `set_embeddings_endpoint`, `set_embeddings_endpoint_default` global setters.
- `open_ai_msg` / `embed` free functions and their `requests` module.
- `helpers::{get_key, get_url}` and the `RequestType` enum.
- `ResponseFormatInput` deprecated alias.
- `ChatMessageRole::Function` deprecated variant (use `Tool`).

### Added

#### Client
- `Client` struct with `Client::new(api_key)`, `Client::from_env()`, `Client::azure(...)`, `Client::builder()`.
- Per-request overrides via `Client::with_options(RequestOptions)` and shortcuts `with_timeout` / `with_max_retries` / `with_idempotency_key` / `with_header`.
- Configurable retry policy (429 / 5xx / connection errors) with exponential backoff + jitter on JSON paths.
- Per-call timeout on every HTTP shape (JSON, multipart, streaming, raw bytes).
- Typed error `OpenAiError` w/ `From<reqwest::Error> + From<serde_json::Error> + From<std::io::Error>`; parses OpenAI's `{ error: { code, message, type, param } }` envelope. `From<OpenAiError> for String` retained for backwards compat.

#### Chat Completions
- New fields: `reasoning_effort`, `max_completion_tokens`, `metadata`, `store`, `parallel_tool_calls`, proper struct `stream_options`, expanded `response_format` (`Text` / `JsonObject` / `JsonSchema { name, schema, strict }`).
- Typed `ChatToolChoice` (still accepts `"auto"`/`"required"`/`"none"` via `From<&str>`).
- Streaming via `client.chat().create_stream(...)` returning a `Stream<Item = Result<ChatCompletionChunk>>`. `collect_chat_stream` helper assembles deltas back into a single response.
- Multi-part `ChatContent` (text + image + audio + file parts), tool-role messages with `tool_call_id`, `Developer` role, `refusal` field.

#### Responses API (new)
- `client.responses().create / create_stream / retrieve / cancel / delete`.
- Typed `ResponseStreamEvent` enum covering every documented event variant.
- Input items API (`ResponseInputItem::user/system/developer/assistant/function_call_output`).
- Hosted tools: `function`, `file_search`, `web_search_preview`, `computer_use_preview`. Force a hosted tool via `ResponseToolChoice::Hosted { type }`.

#### Other endpoints
- Audio: transcriptions (whisper / gpt-4o-transcribe), translations, speech (TTS). Multi-part upload + raw-bytes return for speech.
- Images: `generate`, `edit`, `variations`. Supports `gpt-image-1` parameters (`background`, `output_format`, `moderation`, etc.).
- Moderations: text + image-URL inputs.
- Files: `create / list / retrieve / delete / content`.
- Models: `list / retrieve / delete`.
- Batches: `create / retrieve / cancel / list`.
- Vector stores: `create / list / retrieve / delete` + nested `.files(id)`.
- Fine-tuning: `jobs().{create,list,retrieve,cancel,list_events,list_checkpoints}`.
- Uploads (large multipart): `create / add_part / complete / cancel`.

#### Models enum
- Reasoning: `O1`, `O1Mini`, `O1Pro`, `O3`, `O3Mini`, `O3Pro`, `O4Mini`.
- GPT-5: `GPT5`, `GPT5Mini`, `GPT5Nano`.
- GPT-4.1 / 4.5: `GPT41`, `GPT41Mini`, `GPT41Nano`, `GPT45Preview`.
- Audio / image: `GPT4oTranscribe`, `GPT4oMiniTranscribe`, `GPT4oMiniTTS`, `GPT4oAudioPreview`, `GPT4oMiniAudioPreview`, `GPTImage1`.
- Moderation: `OmniModerationLatest`.

#### Tooling
- `FunctionParameter.required: bool` field (defaults `true`).
- `FunctionType::Map(Box<...>)` → JSON Schema `additionalProperties`.
- `FunctionType::OneOf(Vec<FunctionVariant>)` → JSON Schema `oneOf` (Rust enums with data variants).
- New `FunctionCallable` impls: `&'static str`, `Cow<'static, str>`, `i128`, `u128`, `[T; N]`, `HashMap<String, V>`, `BTreeMap<String, V>`.
- New `FunctionCallable::schema_type()` and `fn_schema()` associated (no-instance) methods.
- Tool registry behind `tool_registry` feature (linkme-backed): `TOOLS`, `invoke_tool`, `find_tool`, `registered_tool_schemas`.

#### Output types
- `AiMsgResponse.system_fingerprint` is now `Option<String>`; `service_tier` added.
- `Usage.prompt_tokens_details` + `Usage.completion_tokens_details` for reasoning/audio token splits.
- `AiResponseMessage.refusal` + `audio`.
- `ToolCallRes.id` + `type_`.
- `ChatCompletionChunk` + `AssistantDelta` + `ToolCallDelta` for streaming.

#### Infrastructure
- `cargo` feature flags: `rustls-tls` (default), `native-tls`, `stream` (default), `utoipa`, `tracing`, `azure`, `tool_registry`, `macro_v2`.
- `tracing` feature: `debug!` on every HTTP request, `warn!` on each retry. `#[tracing::instrument]` spans on every resource method (`chat.completions`, `responses.create`, `audio.speech`, `files.create`, …) tagged with an `endpoint` field for filtering.
- CI: fmt + clippy `-D warnings` + build matrix (stable + 1.75 MSRV) + offline tests + docs.
- 16 doctests on public API (`Client::new/from_env/builder/azure/with_options`, `ChatMessage::user/tool/with_image/new`, `PayLoadBuilder::new`, `ResponseFormat::json_schema`, `ChatToolChoice::auto/function`, `OpenAiModel::as_str`, `RequestOptions::new`).
- 50+ offline tests across serde shapes, tool schema, response parsing, wiremock HTTP fixtures (including 3 SSE streaming tests covering delta assembly + tool-call assembly via `collect_chat_stream` + error-during-stream-open), and tool registry.
- 9 examples: `chat_basic`, `chat_stream`, `structured_output`, `function_call`, `responses_basic`, `responses_stream`, `embed_text`, `transcription`, `tts`.

#### Sampling
- `ChatPayLoad.logit_bias` — `HashMap<String, i32>` of token-ID → bias (`-100..=100`). Builder shortcuts: `.logit_bias(map)` and `.logit_bias_entry(token_id, bias)`.

### Changed (breaking)
- `ChatMessage.content`: `String` → `ChatContent` (untagged enum). Use `ChatMessage::user/system/assistant/tool/developer(...)` helpers or `.into()`.
- `ChatPayLoad.tool_choice`: `Option<String>` → `Option<ChatToolChoice>` (`From<&str>` retained).
- `ChatPayLoad.stream_options`: `Option<bool>` → `Option<StreamOptions>` (was wrong on the wire before).
- `ChatPayLoad.response_format`: `Option<ResponseFormatInput>` → `Option<ResponseFormat>` (deprecated type alias retained).
- `FunctionParameter` gains `required: bool` field (defaults `true` — Add it to struct literals).
- `Usage` gains 2 new `Option` fields.
- `AiMsgResponse.system_fingerprint` is `Option<String>`.
- `ChatMessageRole::Function` now `#[deprecated]`; use `Tool`.
- `utoipa::ToSchema` derive on `ChatMessageRole` now behind the `utoipa` feature (off by default).

### Deprecated (still compile, will be removed at 1.0)
- `set_key` / `set_ai_msg_endpoint(_default)` / `set_embeddings_endpoint(_default)`.
- `requests::open_ai_msg` / `requests::embed`.
- `ResponseFormatInput` (alias for `ResponseFormat`).
- `ChatMessageRole::Function`.

### Removed
- `src/requests/audio/stt.rs` (was entirely commented out; superseded by `client.audio().transcriptions()`).

### Fixed
- `FunctionCallRes.arguments` deserializer now accepts both JSON-string-encoded args (OpenAI's wire format) and already-decoded objects.
- `stream_options` wire shape (now actually matches OpenAI's `{ include_usage: bool }` spec).
- `system_fingerprint` no longer panics on responses that omit the field (some providers do).

### Known limitations
- `tests/macro_to_fn_call.rs` and `tests/struct_macro_parsing.rs` gated behind `--features macro_v2` pending `open_ai_rust_fn_call_extension v0.3` ship (which needs to emit the new `required` field).
- Apple sandbox platforms (iOS/watchOS/tvOS) can't use the `tool_registry` feature — `linkme` requires linker support those platforms don't provide.

---

## 0.2.16 and earlier

See `git log` prior to the 1.0 work.
