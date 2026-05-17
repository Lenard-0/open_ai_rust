//! End-to-end registry test. Doesn't depend on the sister macro — emulates exactly
//! what the macro is expected to emit.

#![cfg(feature = "tool_registry")]

use open_ai_rust::logoi::input::tool::{FunctionCall, FunctionParameter, FunctionType};
use open_ai_rust::tool_registry::{
    find_tool, invoke_tool, registered_tool_schemas, ToolEntry, TOOLS,
};
use serde_json::json;

#[linkme::distributed_slice(TOOLS)]
static ECHO_TOOL: ToolEntry = ToolEntry {
    name: "echo",
    description: Some("Echoes the input string"),
    schema: || FunctionCall {
        name: "echo".to_string(),
        description: Some("Echoes the input string".to_string()),
        parameters: vec![FunctionParameter {
            name: "text".to_string(),
            _type: FunctionType::String,
            description: None,
            required: true,
        }],
    },
    dispatch: |args| {
        Box::pin(async move {
            let s = args
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing 'text'".to_string())?
                .to_string();
            Ok(serde_json::json!({ "echoed": s }))
        })
    },
};

#[linkme::distributed_slice(TOOLS)]
static ADD_TOOL: ToolEntry = ToolEntry {
    name: "add",
    description: None,
    schema: || FunctionCall {
        name: "add".to_string(),
        description: None,
        parameters: vec![
            FunctionParameter {
                name: "a".to_string(),
                _type: FunctionType::Number,
                description: None,
                required: true,
            },
            FunctionParameter {
                name: "b".to_string(),
                _type: FunctionType::Number,
                description: None,
                required: true,
            },
        ],
    },
    dispatch: |args| {
        Box::pin(async move {
            let a = args.get("a").and_then(|v| v.as_f64()).ok_or("missing a")?;
            let b = args.get("b").and_then(|v| v.as_f64()).ok_or("missing b")?;
            Ok(serde_json::json!({ "sum": a + b }))
        })
    },
};

#[test]
fn registry_contains_both_tools() {
    let names: Vec<&str> = registered_tool_schemas()
        .iter()
        .map(|s| s.name.as_str())
        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &str)
        .collect();
    assert!(names.contains(&"echo"));
    assert!(names.contains(&"add"));
}

#[test]
fn find_tool_works() {
    assert!(find_tool("echo").is_some());
    assert!(find_tool("nonexistent").is_none());
}

#[tokio::test]
async fn invoke_echo() {
    let out = invoke_tool("echo", json!({ "text": "hi" })).await.unwrap();
    assert_eq!(out, json!({ "echoed": "hi" }));
}

#[tokio::test]
async fn invoke_add() {
    let out = invoke_tool("add", json!({ "a": 2, "b": 3 })).await.unwrap();
    assert_eq!(out["sum"], 5.0);
}

#[tokio::test]
async fn invoke_unknown_tool_errors() {
    let err = invoke_tool("missing", json!({})).await.unwrap_err();
    assert!(err.to_string().contains("unknown tool"));
}

#[tokio::test]
async fn invoke_all_dispatches_pairs_and_preserves_call_ids() {
    use open_ai_rust::tool_registry::invoke_all;

    let calls = vec![
        ("call_1".into(), "echo".into(), json!({ "text": "first" })),
        ("call_2".into(), "add".into(), json!({ "a": 10, "b": 5 })),
        ("call_3".into(), "echo".into(), json!({ "text": "third" })),
        ("call_4".into(), "missing".into(), json!({})),
    ];
    let results = invoke_all(calls).await;
    assert_eq!(results.len(), 4);
    assert_eq!(results[0].0, "call_1");
    assert_eq!(results[0].1.as_ref().unwrap()["echoed"], "first");
    assert_eq!(results[1].0, "call_2");
    assert_eq!(results[1].1.as_ref().unwrap()["sum"], 15.0);
    assert_eq!(results[2].0, "call_3");
    assert_eq!(results[2].1.as_ref().unwrap()["echoed"], "third");
    assert_eq!(results[3].0, "call_4");
    assert!(results[3].1.is_err());
}

#[tokio::test]
async fn invoke_tool_returns_config_error_when_user_fn_errors() {
    // The `echo` tool returns Err("missing 'text'") when invoked without `text`.
    let err = invoke_tool("echo", json!({})).await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("missing 'text'"), "got: {msg}");
}

#[test]
fn registered_tools_slice_accessor() {
    use open_ai_rust::tool_registry::registered_tools;
    let s = registered_tools();
    assert!(s.iter().any(|t| t.name == "echo"));
}
