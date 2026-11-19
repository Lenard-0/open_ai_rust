//! `#[derive(FunctionCall)]` (open_ai_rust_fn_call_extension >= 0.3.1) feeds straight into a
//! strict `json_schema` response format via `ResponseFormat::for_type`. Since 0.3.1 the derive
//! emits fully qualified paths, so only the derive macro needs importing.

use open_ai_rust::ResponseFormat;
use open_ai_rust_fn_call_extension::FunctionCall;
use serde_json::json;

#[derive(FunctionCall)]
#[allow(dead_code)]
/// A quiz question.
struct Question {
    /// The question text.
    prompt: String,
    points: Option<i32>,
    tags: Vec<String>,
}

#[test]
fn derived_type_produces_strict_response_format() {
    let v = serde_json::to_value(ResponseFormat::for_type::<Question>().unwrap()).unwrap();
    let js = &v["json_schema"];
    assert_eq!(v["type"], "json_schema");
    assert_eq!(js["name"], "Question");
    assert_eq!(js["strict"], true);
    let schema = &js["schema"];
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], json!(["prompt", "points", "tags"]));
    assert_eq!(schema["properties"]["prompt"]["type"], "string");
    assert_eq!(
        schema["properties"]["points"]["type"],
        json!(["number", "null"])
    );
    assert_eq!(
        schema["properties"]["tags"],
        json!({ "type": "array", "items": { "type": "string" } })
    );
}
