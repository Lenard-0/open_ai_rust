//! Snapshot tests of canonical request/response JSON shapes.
//!
//! These exist to catch *unintended* wire-format drift on PR — if a `#[serde(rename)]`
//! changes by accident, the snapshot diff makes it obvious. Committed `.snap` files in
//! `tests/snapshots/` are the source of truth; regenerate with `cargo insta review`
//! after intentional changes.

use open_ai_rust::logoi::input::tool::{
    EnumValues, FunctionCall, FunctionParameter, FunctionType, FunctionVariant,
};
use open_ai_rust::responses::{ResponseInputItem, ResponseRequestBuilder, ResponseTool};
use open_ai_rust::{
    ChatContent, ChatMessage, ChatMessageRole, ChatToolChoice, ContentPart, ImageUrlSpec,
    OpenAiModel, PayLoadBuilder, ReasoningEffort, ResponseFormat, ToolCall, ToolCallFunction,
};
use serde_json::json;

// ---------------------------------------------------------------------------
// Chat completions payloads
// ---------------------------------------------------------------------------

#[test]
fn snapshot_chat_minimal() {
    let p = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![ChatMessage::user("hi")])
        .build();
    insta::assert_json_snapshot!("chat_minimal", &p);
}

#[test]
fn snapshot_chat_kitchen_sink() {
    let p = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
        .messages(vec![
            ChatMessage::system("You are helpful."),
            ChatMessage::user("Find me a coffee."),
        ])
        .temperature(0.3)
        .top_p(0.95)
        .max_completion_tokens(2048)
        .reasoning_effort(ReasoningEffort::Medium)
        .seed(7)
        .frequency_penalty(0.1)
        .presence_penalty(0.2)
        .n(1)
        .logprobs(true)
        .top_logprobs(5)
        .logit_bias_entry("50256", -100)
        .stop(vec!["END".into()])
        .service_tier("default")
        .parallel_tool_calls(true)
        .tool_choice(ChatToolChoice::auto())
        .user("user-42")
        .store(true)
        .metadata_entry("session", "abc-123")
        .include_usage(true)
        .build();
    insta::assert_json_snapshot!("chat_kitchen_sink", &p);
}

#[test]
fn snapshot_chat_multipart_message_with_image() {
    let m = ChatMessage {
        role: ChatMessageRole::User,
        content: ChatContent::Parts(vec![
            ContentPart::Text {
                text: "Describe this image:".into(),
            },
            ContentPart::ImageUrl {
                image_url: ImageUrlSpec {
                    url: "https://example.com/cat.png".into(),
                    detail: Some("high".into()),
                },
            },
        ]),
        name: None,
        tool_call_id: None,
        tool_calls: None,
        refusal: None,
    };
    insta::assert_json_snapshot!("message_multipart_image", &m);
}

#[test]
fn snapshot_chat_assistant_with_tool_calls() {
    let m = ChatMessage::assistant("").with_tool_calls(vec![ToolCall {
        id: "call_abc".into(),
        type_: open_ai_rust::MessageToolCallType::Function,
        function: ToolCallFunction {
            name: "get_weather".into(),
            arguments: r#"{"city":"Sydney"}"#.into(),
        },
    }]);
    insta::assert_json_snapshot!("message_assistant_tool_calls", &m);
}

#[test]
fn snapshot_chat_tool_role_reply() {
    let m = ChatMessage::tool("call_abc", r#"{"temperature":22}"#);
    insta::assert_json_snapshot!("message_tool_role", &m);
}

#[test]
fn snapshot_chat_response_format_json_schema() {
    let rf = ResponseFormat::json_schema(
        "city_info",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "population": { "type": "integer" },
            },
            "required": ["name", "population"],
            "additionalProperties": false,
        }),
    );
    insta::assert_json_snapshot!("response_format_json_schema", &rf);
}

#[test]
fn snapshot_chat_tool_choice_variants() {
    let v = json!({
        "auto": ChatToolChoice::auto(),
        "none": ChatToolChoice::none(),
        "required": ChatToolChoice::required(),
        "function": ChatToolChoice::function("submit"),
    });
    insta::assert_json_snapshot!("chat_tool_choice_variants", &v);
}

// ---------------------------------------------------------------------------
// Function tools — JSON-Schema serialisation
// ---------------------------------------------------------------------------

#[test]
fn snapshot_function_call_with_all_types() {
    let fc = FunctionCall {
        name: "do_everything".into(),
        description: Some("exercises every FunctionType variant".into()),
        parameters: vec![
            FunctionParameter::new("a_string", FunctionType::String).description("a string"),
            FunctionParameter::new("a_number", FunctionType::Number),
            FunctionParameter::new("a_bool", FunctionType::Boolean),
            FunctionParameter::new(
                "an_enum",
                FunctionType::Enum(EnumValues::String(vec!["red".into(), "green".into()])),
            ),
            FunctionParameter::new(
                "an_array",
                FunctionType::Array(Box::new(FunctionType::String)),
            ),
            FunctionParameter::new("a_map", FunctionType::Map(Box::new(FunctionType::Number))),
            FunctionParameter::new(
                "an_object",
                FunctionType::Object(vec![
                    FunctionParameter::new("inner_str", FunctionType::String),
                    FunctionParameter::new("inner_num", FunctionType::Number).required(false),
                ]),
            ),
            FunctionParameter::new(
                "a_oneof",
                FunctionType::OneOf(vec![
                    FunctionVariant {
                        name: "Circle".into(),
                        description: Some("a circle".into()),
                        parameters: vec![FunctionParameter::new("radius", FunctionType::Number)],
                    },
                    FunctionVariant {
                        name: "Square".into(),
                        description: None,
                        parameters: vec![],
                    },
                ]),
            ),
            FunctionParameter::new(
                "optional_legacy",
                FunctionType::Option(Box::new(FunctionType::String)),
            ),
            FunctionParameter::new("optional_via_flag", FunctionType::String).required(false),
        ],
    };
    insta::assert_json_snapshot!("function_call_all_types", &fc);
}

// ---------------------------------------------------------------------------
// Responses API
// ---------------------------------------------------------------------------

#[test]
fn snapshot_responses_minimal() {
    let r = ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "hi").build();
    insta::assert_json_snapshot!("responses_minimal", &r);
}

#[test]
fn snapshot_responses_with_items_input() {
    let r = ResponseRequestBuilder::new(
        OpenAiModel::GPT41Mini,
        vec![
            ResponseInputItem::system("be brief"),
            ResponseInputItem::user("hello"),
            ResponseInputItem::function_call_output("call_1", r#"{"weather":"sunny"}"#),
        ],
    )
    .instructions("be brief")
    .max_output_tokens(500)
    .temperature(0.4)
    .parallel_tool_calls(true)
    .build();
    insta::assert_json_snapshot!("responses_with_items_input", &r);
}

#[test]
fn snapshot_responses_with_tools() {
    let r = ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "find me a paper on graphs")
        .tools(vec![
            ResponseTool::Function {
                name: "search_papers".into(),
                description: Some("Search Arxiv".into()),
                parameters: json!({
                    "type": "object",
                    "properties": { "query": { "type": "string" } },
                    "required": ["query"]
                }),
                strict: Some(true),
            },
            ResponseTool::FileSearch {
                vector_store_ids: vec!["vs_1".into()],
                max_num_results: Some(5),
                ranking_options: None,
                filters: None,
            },
            ResponseTool::WebSearchPreview {
                search_context_size: Some("medium".into()),
                user_location: None,
            },
        ])
        .build();
    insta::assert_json_snapshot!("responses_with_tools", &r);
}

#[test]
fn snapshot_responses_with_reasoning_and_text_format() {
    use open_ai_rust::responses::{ReasoningConfig, TextConfig};
    let r = ResponseRequestBuilder::new(OpenAiModel::O3Mini, "solve this")
        .reasoning(ReasoningConfig {
            effort: Some(ReasoningEffort::High),
            summary: Some("detailed".into()),
        })
        .text(TextConfig::json_schema(
            "answer",
            json!({
                "type": "object",
                "properties": { "value": { "type": "number" } },
                "required": ["value"]
            }),
        ))
        .build();
    insta::assert_json_snapshot!("responses_reasoning_and_text", &r);
}

// ---------------------------------------------------------------------------
// Embeddings
// ---------------------------------------------------------------------------

#[test]
fn snapshot_embeddings_request_string_input() {
    use open_ai_rust::resources::embeddings::EmbeddingRequestBuilder;
    let req = EmbeddingRequestBuilder::new("text-embedding-3-small")
        .input("hello")
        .encoding_format("float")
        .build();
    insta::assert_json_snapshot!("embeddings_string_input", &req);
}

#[test]
fn snapshot_embeddings_request_array_input_with_dimensions() {
    use open_ai_rust::resources::embeddings::EmbeddingRequestBuilder;
    let req = EmbeddingRequestBuilder::new("text-embedding-3-large")
        .input(vec!["foo".to_string(), "bar".to_string()])
        .dimensions(256)
        .user("user-42")
        .build();
    insta::assert_json_snapshot!("embeddings_array_input_with_dims", &req);
}
