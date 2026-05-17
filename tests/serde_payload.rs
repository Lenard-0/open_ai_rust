//! Offline tests verifying that wire payloads match OpenAI's expected JSON shape.

#[cfg(test)]
mod tests {
    use open_ai_rust::{
        logoi::input::payload::ChatToolChoice, ChatContent, ChatMessage, ChatMessageRole,
        ContentPart, ImageUrlSpec, OpenAiModel, PayLoadBuilder, ReasoningEffort, ResponseFormat,
        ToolCall, ToolCallFunction,
    };
    use serde_json::json;

    #[test]
    fn chat_message_serializes_string_content_as_plain_string() {
        let m = ChatMessage::user("hi");
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(
            v,
            json!({
                "role": "user",
                "content": "hi",
            })
        );
    }

    #[test]
    fn chat_message_serializes_multi_part_content_as_array() {
        let m = ChatMessage {
            role: ChatMessageRole::User,
            content: ChatContent::Parts(vec![
                ContentPart::Text {
                    text: "look at this".into(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrlSpec {
                        url: "https://example.com/a.png".into(),
                        detail: Some("low".into()),
                    },
                },
            ]),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            refusal: None,
        };
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(
            v,
            json!({
                "role": "user",
                "content": [
                    { "type": "text", "text": "look at this" },
                    { "type": "image_url", "image_url": { "url": "https://example.com/a.png", "detail": "low" } }
                ]
            })
        );
    }

    #[test]
    fn tool_role_message_includes_tool_call_id() {
        let m = ChatMessage::tool("call_123", "result");
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(
            v,
            json!({ "role": "tool", "content": "result", "tool_call_id": "call_123" })
        );
    }

    #[test]
    fn assistant_message_with_tool_calls() {
        let m = ChatMessage::assistant("").with_tool_calls(vec![ToolCall {
            id: "call_1".into(),
            type_: open_ai_rust::MessageToolCallType::Function,
            function: ToolCallFunction {
                name: "get_weather".into(),
                arguments: r#"{"city":"Sydney"}"#.into(),
            },
        }]);
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(
            v,
            json!({
                "role": "assistant",
                "content": "",
                "tool_calls": [
                    {
                        "id": "call_1",
                        "type": "function",
                        "function": { "name": "get_weather", "arguments": "{\"city\":\"Sydney\"}" }
                    }
                ]
            })
        );
    }

    #[test]
    fn payload_emits_only_set_fields() {
        let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
            .messages(vec![ChatMessage::user("hi")])
            .temperature(0.2)
            .seed(7)
            .reasoning_effort(ReasoningEffort::Medium)
            .max_completion_tokens(256)
            .tool_choice(ChatToolChoice::auto())
            .build();
        let v = serde_json::to_value(&payload).unwrap();
        let map = v.as_object().unwrap();
        assert!(map.contains_key("model"));
        assert!(map.contains_key("messages"));
        assert!((map["temperature"].as_f64().unwrap() - 0.2_f64).abs() < 1e-5);
        assert_eq!(map["seed"], json!(7));
        assert_eq!(map["reasoning_effort"], json!("medium"));
        assert_eq!(map["max_completion_tokens"], json!(256));
        assert_eq!(map["tool_choice"], json!("auto"));
        // un-set fields should not appear:
        assert!(!map.contains_key("top_p"));
        assert!(!map.contains_key("frequency_penalty"));
        assert!(!map.contains_key("stop"));
    }

    #[test]
    fn tool_choice_function_serializes_as_object() {
        let tc = ChatToolChoice::function("submit_form");
        let v = serde_json::to_value(&tc).unwrap();
        assert_eq!(
            v,
            json!({ "type": "function", "function": { "name": "submit_form" } })
        );
    }

    #[test]
    fn logit_bias_serializes_as_map() {
        let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
            .messages(vec![ChatMessage::user("hi")])
            .logit_bias_entry("50256", -100)
            .logit_bias_entry("100", 50)
            .build();
        let v = serde_json::to_value(&payload).unwrap();
        let lb = v["logit_bias"].as_object().unwrap();
        assert_eq!(lb["50256"], json!(-100));
        assert_eq!(lb["100"], json!(50));
    }

    #[test]
    fn response_format_json_schema_shape() {
        let rf = ResponseFormat::json_schema(
            "person",
            json!({ "type": "object", "properties": { "name": { "type": "string" } } }),
        );
        let v = serde_json::to_value(&rf).unwrap();
        assert_eq!(v["type"], "json_schema");
        assert_eq!(v["json_schema"]["name"], "person");
        assert_eq!(v["json_schema"]["strict"], true);
    }

    #[test]
    fn stream_chunk_parses() {
        let raw = json!({
            "id": "chatcmpl-abc",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "gpt-4o-mini",
            "choices": [{
                "index": 0,
                "delta": { "role": "assistant", "content": "Hel" },
                "finish_reason": null
            }]
        });
        let chunk: open_ai_rust::ChatCompletionChunk = serde_json::from_value(raw).unwrap();
        assert_eq!(chunk.delta_text(), "Hel");
    }

    #[test]
    fn model_enum_serializes_as_bare_string() {
        assert_eq!(
            serde_json::to_value(&OpenAiModel::GPT4o).unwrap(),
            json!("gpt-4o")
        );
        assert_eq!(
            serde_json::to_value(&OpenAiModel::O3Mini).unwrap(),
            json!("o3-mini")
        );
        assert_eq!(
            serde_json::to_value(OpenAiModel::Custom("gpt-4o-2024-08-06".into())).unwrap(),
            json!("gpt-4o-2024-08-06")
        );
    }

    #[test]
    fn response_input_string_form() {
        use open_ai_rust::responses::ResponseInput;
        let v = serde_json::to_value(ResponseInput::Text("hi".into())).unwrap();
        assert_eq!(v, json!("hi"));
    }

    #[test]
    fn response_request_serializes_minimal_fields_only() {
        use open_ai_rust::responses::ResponseRequestBuilder;
        let req = ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "hi")
            .instructions("be brief")
            .build();
        let v = serde_json::to_value(&req).unwrap();
        let map = v.as_object().unwrap();
        assert_eq!(map["model"], json!("gpt-4.1-mini"));
        assert_eq!(map["input"], json!("hi"));
        assert_eq!(map["instructions"], json!("be brief"));
        assert!(!map.contains_key("temperature"));
        assert!(!map.contains_key("tools"));
    }
}
