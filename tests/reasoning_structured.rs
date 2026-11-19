//! Wire-format + parsing tests for reasoning-model / structured-output support
//! (`response_format: json_schema`, `reasoning_effort`, `max_completion_tokens`,
//! reasoning / cached token usage, refusals).

use open_ai_rust::{
    AiMsgResponse, ChatMessage, EnumValues, FunctionCall, FunctionParameter, FunctionType,
    FunctionVariant, OpenAiModel, PayLoadBuilder, PayLoadTemplates, QuickChatTemplate,
    QuickFunctionCallTemplate, ReasoningEffort, ResponseFormat, StrictSchemaError,
    StructuredOutputError,
};
use serde::Deserialize;
use serde_json::{json, Value};

const SAMPLING_FIELDS: [&str; 4] = ["temperature", "top_p", "logprobs", "top_logprobs"];

fn quiz_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "question": { "type": "string" } },
        "required": ["question"],
        "additionalProperties": false
    })
}

fn sol_payload() -> Value {
    let payload = PayLoadBuilder::new(OpenAiModel::Custom("sol-deployment".into()))
        .messages(vec![ChatMessage::user("Make a quiz")])
        .response_format(ResponseFormat::json_schema("quiz", quiz_schema()))
        .reasoning_effort(ReasoningEffort::XHigh)
        .max_completion_tokens(4000)
        .build();
    serde_json::to_value(&payload).unwrap()
}

// ── request serialisation ──────────────────────────────────────────────────

#[test]
fn response_format_json_schema_serialises_exactly() {
    let v = sol_payload();
    assert_eq!(
        v["response_format"],
        json!({
            "type": "json_schema",
            "json_schema": { "name": "quiz", "strict": true, "schema": quiz_schema() }
        })
    );
}

#[test]
fn response_format_json_schema_with_description() {
    let rf = ResponseFormat::json_schema("quiz", quiz_schema()).with_description("A quiz");
    assert_eq!(
        serde_json::to_value(rf).unwrap(),
        json!({
            "type": "json_schema",
            "json_schema": {
                "name": "quiz", "description": "A quiz", "strict": true, "schema": quiz_schema()
            }
        })
    );
}

#[test]
fn response_format_text_and_json_object_unchanged() {
    assert_eq!(
        serde_json::to_value(ResponseFormat::Text).unwrap(),
        json!({ "type": "text" })
    );
    assert_eq!(
        serde_json::to_value(ResponseFormat::JsonObject).unwrap(),
        json!({ "type": "json_object" })
    );
}

#[test]
fn reasoning_effort_all_levels() {
    for (e, s) in [
        (ReasoningEffort::Low, "low"),
        (ReasoningEffort::Medium, "medium"),
        (ReasoningEffort::High, "high"),
        (ReasoningEffort::XHigh, "xhigh"),
    ] {
        assert_eq!(serde_json::to_value(e).unwrap(), s);
        assert_eq!(
            serde_json::from_value::<ReasoningEffort>(json!(s)).unwrap(),
            e
        );
    }
    assert_eq!(sol_payload()["reasoning_effort"], "xhigh");
}

#[test]
fn max_completion_tokens_serialises_and_max_tokens_absent() {
    let v = sol_payload();
    assert_eq!(v["max_completion_tokens"], 4000);
    assert!(v.get("max_tokens").is_none());

    let legacy = serde_json::to_value(
        PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(vec![ChatMessage::user("hi")])
            .max_tokens(50)
            .build(),
    )
    .unwrap();
    assert_eq!(legacy["max_tokens"], 50);
    assert!(legacy.get("max_completion_tokens").is_none());
}

#[test]
fn sol_payload_has_exactly_the_expected_keys() {
    let v = sol_payload();
    let mut keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "max_completion_tokens",
            "messages",
            "model",
            "reasoning_effort",
            "response_format"
        ]
    );
    assert_eq!(v["model"], "sol-deployment");
}

#[test]
fn sampling_fields_absent_when_unset() {
    let builder = serde_json::to_value(
        PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(vec![ChatMessage::user("hi")])
            .build(),
    )
    .unwrap();
    let chat_tpl = serde_json::to_value(
        PayLoadTemplates::Chat(QuickChatTemplate::default(vec![ChatMessage::user("hi")]))
            .to_payload(),
    )
    .unwrap();
    let fn_tpl = serde_json::to_value(
        PayLoadTemplates::FunctionCall(QuickFunctionCallTemplate::default(
            vec![ChatMessage::user("hi")],
            vec![FunctionCall {
                name: "f".into(),
                description: None,
                parameters: vec![],
            }],
            None,
        ))
        .to_payload(),
    )
    .unwrap();
    for v in [&builder, &chat_tpl, &fn_tpl] {
        for field in SAMPLING_FIELDS {
            assert!(v.get(field).is_none(), "{field} serialised when unset: {v}");
        }
        // No other optional field should leak out as `null`.
        for (k, val) in v.as_object().unwrap() {
            assert!(!val.is_null(), "{k} serialised as null: {v}");
        }
    }
}

#[test]
fn sampling_fields_present_when_set() {
    let v = serde_json::to_value(
        PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(vec![ChatMessage::user("hi")])
            .temperature(0.5)
            .top_p(0.9)
            .logprobs(true)
            .top_logprobs(3)
            .build(),
    )
    .unwrap();
    for field in SAMPLING_FIELDS {
        assert!(v.get(field).is_some(), "{field} missing");
    }
}

// ── strict schema from FunctionCall ────────────────────────────────────────

fn quiz_fn() -> FunctionCall {
    FunctionCall {
        name: "quiz".into(),
        description: Some("Generate a quiz".into()),
        parameters: vec![
            FunctionParameter::new("title", FunctionType::String).description("Quiz title"),
            FunctionParameter::new(
                "difficulty",
                FunctionType::Enum(EnumValues::String(vec!["easy".into(), "hard".into()])),
            )
            .required(false),
            FunctionParameter::new(
                "notes",
                FunctionType::Option(Box::new(FunctionType::String)),
            ),
            FunctionParameter::new(
                "questions",
                FunctionType::Array(Box::new(FunctionType::Object(vec![
                    FunctionParameter::new("prompt", FunctionType::String),
                    FunctionParameter::new("points", FunctionType::Number).required(false),
                ]))),
            ),
            FunctionParameter::new(
                "meta",
                FunctionType::Option(Box::new(FunctionType::Object(vec![
                    FunctionParameter::new("author", FunctionType::String),
                ]))),
            ),
        ],
    }
}

#[test]
fn strict_schema_from_function_call() {
    let schema = quiz_fn().to_strict_json_schema().unwrap();
    assert_eq!(
        schema,
        json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "Quiz title" },
                "difficulty": { "type": ["string", "null"], "enum": ["easy", "hard", null] },
                "notes": { "type": ["string", "null"] },
                "questions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "prompt": { "type": "string" },
                            "points": { "type": ["number", "null"] }
                        },
                        "required": ["prompt", "points"],
                        "additionalProperties": false
                    }
                },
                "meta": {
                    "type": ["object", "null"],
                    "properties": { "author": { "type": "string" } },
                    "required": ["author"],
                    "additionalProperties": false
                }
            },
            "required": ["title", "difficulty", "notes", "questions", "meta"],
            "additionalProperties": false
        })
    );
}

#[test]
fn response_format_from_function_serialises_exactly() {
    let rf = ResponseFormat::from_function(&quiz_fn()).unwrap();
    assert_eq!(
        serde_json::to_value(rf).unwrap(),
        json!({
            "type": "json_schema",
            "json_schema": {
                "name": "quiz",
                "description": "Generate a quiz",
                "strict": true,
                "schema": quiz_fn().to_strict_json_schema().unwrap()
            }
        })
    );
}

#[test]
fn tool_schema_unchanged_by_strict_helper() {
    // The function-tool serialisation stays non-strict (optional params omitted from required).
    let v = serde_json::to_value(quiz_fn()).unwrap();
    assert_eq!(v["parameters"]["required"], json!(["title", "questions"]));
    assert!(v["parameters"].get("additionalProperties").is_none());
}

#[test]
fn strict_schema_oneof_becomes_anyof() {
    let f = FunctionCall {
        name: "shape".into(),
        description: None,
        parameters: vec![FunctionParameter::new(
            "shape",
            FunctionType::OneOf(vec![
                FunctionVariant {
                    name: "Dot".into(),
                    description: None,
                    parameters: vec![],
                },
                FunctionVariant {
                    name: "Circle".into(),
                    description: None,
                    parameters: vec![FunctionParameter::new("r", FunctionType::Number)],
                },
            ]),
        )
        .required(false)],
    };
    let s = f.to_strict_json_schema().unwrap();
    assert_eq!(
        s["properties"]["shape"],
        json!({ "anyOf": [
            { "type": "string", "enum": ["Dot"], "title": "Dot" },
            {
                "type": "object",
                "properties": { "r": { "type": "number" } },
                "required": ["r"],
                "additionalProperties": false,
                "title": "Circle"
            },
            { "type": "null" }
        ]})
    );
}

#[test]
fn strict_schema_rejects_maps() {
    let f = FunctionCall {
        name: "m".into(),
        description: None,
        parameters: vec![FunctionParameter::new(
            "tags",
            FunctionType::Map(Box::new(FunctionType::String)),
        )],
    };
    assert_eq!(
        f.to_strict_json_schema().unwrap_err(),
        StrictSchemaError::UnsupportedMap {
            path: "$.tags".into()
        }
    );
    assert!(ResponseFormat::from_function(&f).is_err());
}

// ── response parsing ───────────────────────────────────────────────────────

fn response(message: Value, finish_reason: &str, usage: Value) -> AiMsgResponse {
    serde_json::from_value(json!({
        "id": "chatcmpl-1",
        "object": "chat.completion",
        "created": 1,
        "model": "sol",
        "choices": [{ "index": 0, "finish_reason": finish_reason, "message": message }],
        "usage": usage
    }))
    .unwrap()
}

fn basic_usage() -> Value {
    json!({ "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 })
}

#[test]
fn parses_reasoning_and_cached_token_usage() {
    let resp = response(
        json!({ "role": "assistant", "content": "{\"question\":\"Q?\"}" }),
        "stop",
        json!({
            "prompt_tokens": 120,
            "completion_tokens": 900,
            "total_tokens": 1020,
            "prompt_tokens_details": { "cached_tokens": 64 },
            "completion_tokens_details": { "reasoning_tokens": 832 }
        }),
    );
    assert_eq!(
        resp.usage
            .completion_tokens_details
            .as_ref()
            .unwrap()
            .reasoning_tokens,
        832
    );
    assert_eq!(
        resp.usage
            .prompt_tokens_details
            .as_ref()
            .unwrap()
            .cached_tokens,
        64
    );
}

#[test]
fn usage_details_optional() {
    let resp = response(
        json!({ "role": "assistant", "content": "{}" }),
        "stop",
        basic_usage(),
    );
    assert!(resp.usage.completion_tokens_details.is_none());
    assert!(resp.usage.prompt_tokens_details.is_none());
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
struct Quiz {
    question: String,
}

#[test]
fn parse_structured_ok() {
    let resp = response(
        json!({ "role": "assistant", "content": "{\"question\":\"Q?\"}" }),
        "stop",
        basic_usage(),
    );
    assert_eq!(
        resp.parse_structured::<Quiz>().unwrap(),
        Quiz {
            question: "Q?".into()
        }
    );
    assert_eq!(resp.structured_json().unwrap(), json!({ "question": "Q?" }));
}

#[test]
fn parse_structured_refusal() {
    let resp = response(
        json!({ "role": "assistant", "content": null, "refusal": "I can't help with that." }),
        "stop",
        basic_usage(),
    );
    match resp.structured_json() {
        Err(StructuredOutputError::Refusal(r)) => assert_eq!(r, "I can't help with that."),
        other => panic!("expected refusal, got {other:?}"),
    }
    // The refusal is also visible on the raw message.
    assert_eq!(
        resp.choices[0].message.refusal.as_deref(),
        Some("I can't help with that.")
    );
}

#[test]
fn parse_structured_truncated() {
    let resp = response(
        json!({ "role": "assistant", "content": "{\"question\":\"Q" }),
        "length",
        basic_usage(),
    );
    assert!(matches!(
        resp.structured_json(),
        Err(StructuredOutputError::Truncated { content: Some(_) })
    ));
}

#[test]
fn parse_structured_invalid_json() {
    let resp = response(
        json!({ "role": "assistant", "content": "not json" }),
        "stop",
        basic_usage(),
    );
    match resp.structured_json() {
        Err(StructuredOutputError::InvalidJson { content, .. }) => assert_eq!(content, "not json"),
        other => panic!("expected invalid json, got {other:?}"),
    }
    // Valid JSON but wrong shape is also InvalidJson.
    let resp = response(
        json!({ "role": "assistant", "content": "{\"x\":1}" }),
        "stop",
        basic_usage(),
    );
    assert!(matches!(
        resp.parse_structured::<Quiz>(),
        Err(StructuredOutputError::InvalidJson { .. })
    ));
}

#[test]
fn parse_structured_empty_and_filtered() {
    let resp = response(
        json!({ "role": "assistant", "content": "" }),
        "stop",
        basic_usage(),
    );
    assert!(matches!(
        resp.structured_json(),
        Err(StructuredOutputError::EmptyContent { .. })
    ));
    let resp = response(
        json!({ "role": "assistant", "content": null }),
        "content_filter",
        basic_usage(),
    );
    assert!(matches!(
        resp.structured_json(),
        Err(StructuredOutputError::ContentFilter)
    ));
    let resp: AiMsgResponse = serde_json::from_value(json!({
        "id": "x", "object": "chat.completion", "created": 1, "model": "m",
        "choices": [], "usage": basic_usage()
    }))
    .unwrap();
    assert!(matches!(
        resp.structured_json(),
        Err(StructuredOutputError::NoChoices)
    ));
}

#[test]
fn strict_schema_scalar_and_enum_variants() {
    let f = FunctionCall {
        name: "s".into(),
        description: None,
        parameters: vec![
            FunctionParameter::new("flag", FunctionType::Boolean),
            FunctionParameter::new("nothing", FunctionType::Null).required(false),
            FunctionParameter::new("level", FunctionType::Enum(EnumValues::Int(vec![1, 2])))
                .required(false),
            FunctionParameter::new("ratio", FunctionType::Enum(EnumValues::Float(vec![0.5]))),
            FunctionParameter::new(
                "maybe_maybe",
                FunctionType::Option(Box::new(FunctionType::Option(Box::new(
                    FunctionType::String,
                )))),
            ),
            FunctionParameter::new(
                "tags",
                FunctionType::Array(Box::new(FunctionType::Option(Box::new(
                    FunctionType::String,
                )))),
            ),
        ],
    };
    let p = &f.to_strict_json_schema().unwrap()["properties"];
    assert_eq!(p["flag"], json!({ "type": "boolean" }));
    assert_eq!(p["nothing"], json!({ "type": "null" }));
    assert_eq!(
        p["level"],
        json!({ "type": ["number", "null"], "enum": [1, 2, null] })
    );
    assert_eq!(p["ratio"], json!({ "type": "number", "enum": [0.5] }));
    assert_eq!(p["maybe_maybe"], json!({ "type": ["string", "null"] }));
    assert_eq!(
        p["tags"],
        json!({ "type": "array", "items": { "type": ["string", "null"] } })
    );
}

#[test]
fn strict_schema_oneof_descriptions_and_double_nullable() {
    let variants = vec![FunctionVariant {
        name: "A".into(),
        description: Some("variant a".into()),
        parameters: vec![],
    }];
    let f = FunctionCall {
        name: "o".into(),
        description: None,
        parameters: vec![FunctionParameter::new(
            "v",
            FunctionType::Option(Box::new(FunctionType::Option(Box::new(
                FunctionType::OneOf(variants),
            )))),
        )],
    };
    assert_eq!(
        f.to_strict_json_schema().unwrap()["properties"]["v"],
        json!({ "anyOf": [
            { "type": "string", "enum": ["A"], "title": "A", "description": "variant a" },
            { "type": "null" }
        ]})
    );
}

#[test]
fn error_display_and_string_conversion() {
    let e = StrictSchemaError::UnsupportedMap { path: "$.m".into() };
    assert!(e.to_string().contains("$.m"));

    let resp = response(
        json!({ "role": "assistant", "content": null, "refusal": "no" }),
        "stop",
        basic_usage(),
    );
    let s: String = resp.structured_json().unwrap_err().into();
    assert_eq!(s, "structured output: model refused: no");
    for (msg, fr) in [
        (json!({ "role": "assistant", "content": "{" }), "length"),
        (
            json!({ "role": "assistant", "content": null }),
            "content_filter",
        ),
        (json!({ "role": "assistant", "content": " " }), "stop"),
        (json!({ "role": "assistant", "content": "x" }), "stop"),
    ] {
        let err = response(msg, fr, basic_usage())
            .structured_json()
            .unwrap_err();
        assert!(err.to_string().starts_with("structured output:"), "{err}");
    }
}
