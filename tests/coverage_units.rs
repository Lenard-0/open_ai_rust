//! Pure unit tests covering the constructors, builders, enum-arms, `From` impls, and
//! display impls that wiremock fixtures don't naturally exercise. No HTTP, no I/O.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use open_ai_rust::logoi::input::tool::raw_macro::FunctionCallable;
use open_ai_rust::logoi::input::tool::{EnumValues, FunctionCall, FunctionParameter, FunctionType};
use open_ai_rust::responses::input::{ResponseInputContent, ResponseInputContentPart};
use open_ai_rust::responses::output::ResponseFunctionCall;
use open_ai_rust::responses::{
    ResponseInputItem, ResponseObject, ResponseOutputContentPart, ResponseOutputItem,
    ResponseStatus, ResponseTool, ResponseToolChoice,
};
use open_ai_rust::{
    AiResponseMessage, ChatContent, ChatMessage, ChatMessageRole, ChatPayLoad, ChatToolChoice,
    ChatToolChoiceFunction, ChatToolChoiceType, Choice, CompletionTokensDetails, ContentPart,
    FunctionCallRes, ImageUrlSpec, JsonSchemaSpec, MessageToolCallType, OpenAiError, OpenAiModel,
    PayLoadBuilder, PromptTokensDetails, ReasoningEffort, ResponseFormat, StreamOptions, ToolCall,
    ToolCallFunction, ToolCallRes, Usage,
};

// =============================================================================
// raw_macro::FunctionCallable — primitive + container impls
// =============================================================================

#[test]
fn fn_callable_string() {
    assert_eq!("x".to_string().to_fn_type(), FunctionType::String);
    assert_eq!(
        <String as FunctionCallable>::schema_type(),
        FunctionType::String
    );
    let _ = "x".to_string().to_fn_call(); // exercise new()
}

#[test]
fn fn_callable_str_static() {
    let s: &'static str = "abc";
    assert_eq!(s.to_fn_type(), FunctionType::String);
    assert_eq!(
        <&'static str as FunctionCallable>::schema_type(),
        FunctionType::String
    );
    let _ = s.to_fn_call();
}

#[test]
fn fn_callable_cow_static_str() {
    let c: Cow<'static, str> = Cow::Borrowed("abc");
    assert_eq!(c.to_fn_type(), FunctionType::String);
    assert_eq!(
        <Cow<'static, str> as FunctionCallable>::schema_type(),
        FunctionType::String
    );
    let _ = c.to_fn_call();
}

#[test]
fn fn_callable_bool() {
    assert_eq!(true.to_fn_type(), FunctionType::Boolean);
    assert_eq!(
        <bool as FunctionCallable>::schema_type(),
        FunctionType::Boolean
    );
    let _ = false.to_fn_call();
}

#[test]
fn fn_callable_every_numeric() {
    macro_rules! cov {
        ($($t:ty: $v:expr),+ $(,)?) => {
            $({
                let v: $t = $v;
                assert_eq!(v.to_fn_type(), FunctionType::Number);
                assert_eq!(<$t as FunctionCallable>::schema_type(), FunctionType::Number);
                let _ = v.to_fn_call();
            })+
        }
    }
    cov!(
        i8: -1, i16: -2, i32: -3, i64: -4, i128: -5, isize: -6,
        u8: 1, u16: 2, u32: 3, u64: 4, u128: 5, usize: 6,
        f32: 1.0, f64: 2.0,
    );
}

#[test]
fn fn_callable_vec_string_with_elements() {
    let v = vec!["a".to_string(), "b".to_string()];
    assert_eq!(
        v.to_fn_type(),
        FunctionType::Array(Box::new(FunctionType::String))
    );
    assert_eq!(
        <Vec<String> as FunctionCallable>::schema_type(),
        FunctionType::Array(Box::new(FunctionType::String))
    );
    let _ = v.to_fn_call();
}

#[test]
fn fn_callable_vec_empty_falls_back_to_schema_type() {
    let v: Vec<i32> = vec![];
    assert_eq!(
        v.to_fn_type(),
        FunctionType::Array(Box::new(FunctionType::Number))
    );
}

#[test]
fn fn_callable_array_fixed_size() {
    let arr: [bool; 3] = [true, false, true];
    assert_eq!(
        arr.to_fn_type(),
        FunctionType::Array(Box::new(FunctionType::Boolean))
    );
    assert_eq!(
        <[bool; 3] as FunctionCallable>::schema_type(),
        FunctionType::Array(Box::new(FunctionType::Boolean))
    );
    let _ = arr.to_fn_call();
    let empty: [u8; 0] = [];
    assert_eq!(
        empty.to_fn_type(),
        FunctionType::Array(Box::new(FunctionType::Number))
    );
}

#[test]
fn fn_callable_option_some_and_none() {
    let s: Option<String> = Some("x".into());
    assert_eq!(
        s.to_fn_type(),
        FunctionType::Option(Box::new(FunctionType::String))
    );
    let n: Option<String> = None;
    assert_eq!(
        n.to_fn_type(),
        FunctionType::Option(Box::new(FunctionType::String))
    );
    assert_eq!(
        <Option<String> as FunctionCallable>::schema_type(),
        FunctionType::Option(Box::new(FunctionType::String))
    );
    let _ = s.to_fn_call();
}

#[test]
fn fn_callable_hashmap_with_and_without_entries() {
    let mut m: HashMap<String, i32> = HashMap::new();
    assert_eq!(
        m.to_fn_type(),
        FunctionType::Map(Box::new(FunctionType::Number))
    );
    m.insert("k".into(), 1);
    assert_eq!(
        m.to_fn_type(),
        FunctionType::Map(Box::new(FunctionType::Number))
    );
    assert_eq!(
        <HashMap<String, i32> as FunctionCallable>::schema_type(),
        FunctionType::Map(Box::new(FunctionType::Number))
    );
    let _ = m.to_fn_call();
}

#[test]
fn fn_callable_btreemap_with_and_without_entries() {
    let mut m: BTreeMap<String, bool> = BTreeMap::new();
    assert_eq!(
        m.to_fn_type(),
        FunctionType::Map(Box::new(FunctionType::Boolean))
    );
    m.insert("k".into(), true);
    assert_eq!(
        m.to_fn_type(),
        FunctionType::Map(Box::new(FunctionType::Boolean))
    );
    assert_eq!(
        <BTreeMap<String, bool> as FunctionCallable>::schema_type(),
        FunctionType::Map(Box::new(FunctionType::Boolean))
    );
    let _ = m.to_fn_call();
}

#[test]
#[should_panic]
fn fn_callable_fn_schema_default_panics_on_primitive() {
    let _ = <String as FunctionCallable>::fn_schema();
}

#[test]
#[should_panic]
fn fn_callable_schema_type_default_unreachable_for_primitive_override() {
    // This proves the default impl is never actually called for primitives — all
    // primitive impls override `schema_type`. But the default impl itself panics; we
    // call it via a fake type that doesn't override it.
    struct Bare;
    impl FunctionCallable for Bare {
        fn to_fn_call(&self) -> FunctionCall {
            FunctionCall::new()
        }
        fn to_fn_type(&self) -> FunctionType {
            FunctionType::Null
        }
        // Don't override schema_type / fn_schema — exercise default impls.
    }
    let _ = <Bare as FunctionCallable>::schema_type();
}

#[test]
#[should_panic]
fn fn_callable_fn_schema_default_panics_for_user_type_that_forgot_to_override() {
    struct Bare;
    impl FunctionCallable for Bare {
        fn to_fn_call(&self) -> FunctionCall {
            FunctionCall::new()
        }
        fn to_fn_type(&self) -> FunctionType {
            FunctionType::Null
        }
    }
    let _ = <Bare as FunctionCallable>::fn_schema();
}

// =============================================================================
// raw_macro::fn_macro::FunctionCallRaw
// =============================================================================

#[test]
fn function_call_raw_to_fn_call_happy_path() {
    use open_ai_rust::logoi::input::tool::raw_macro::fn_macro::FunctionCallRaw;
    let mut params = [""; 100];
    params[0] = "turn_on_light: bool";
    params[1] = "name: string";
    params[2] = "age: i64";
    params[3] = "weight: f64";
    let raw = FunctionCallRaw {
        name: "change_light",
        description: "Toggle a light",
        parameters: params,
    };
    let fc = raw.to_fn_call().unwrap();
    assert_eq!(fc.name, "change_light");
    assert_eq!(fc.description.as_deref(), Some("Toggle a light"));
    assert_eq!(fc.parameters.len(), 4);
    assert_eq!(fc.parameters[0].name, "turn_on_light");
    assert_eq!(fc.parameters[0]._type, FunctionType::Boolean);
    assert_eq!(fc.parameters[1]._type, FunctionType::String);
    assert_eq!(fc.parameters[2]._type, FunctionType::Number);
    assert_eq!(fc.parameters[3]._type, FunctionType::Number);
}

#[test]
fn function_call_raw_empty_description_becomes_none() {
    use open_ai_rust::logoi::input::tool::raw_macro::fn_macro::FunctionCallRaw;
    let raw = FunctionCallRaw {
        name: "x",
        description: "",
        parameters: [""; 100],
    };
    let fc = raw.to_fn_call().unwrap();
    assert!(fc.description.is_none());
}

#[test]
fn function_call_raw_returns_err_on_unsupported_type() {
    use open_ai_rust::logoi::input::tool::raw_macro::fn_macro::FunctionCallRaw;
    let mut params = [""; 100];
    params[0] = "weird: vec<Foo>";
    let raw = FunctionCallRaw {
        name: "x",
        description: "",
        parameters: params,
    };
    let err = raw.to_fn_call().unwrap_err();
    assert!(err.contains("Not yet supported"));
}

#[test]
fn function_call_raw_skips_params_without_colon() {
    use open_ai_rust::logoi::input::tool::raw_macro::fn_macro::FunctionCallRaw;
    let mut params = [""; 100];
    params[0] = "no-colon-here";
    params[1] = "x: bool";
    let raw = FunctionCallRaw {
        name: "x",
        description: "",
        parameters: params,
    };
    let fc = raw.to_fn_call().unwrap();
    assert_eq!(fc.parameters.len(), 1);
    assert_eq!(fc.parameters[0].name, "x");
}

// =============================================================================
// FunctionCall / FunctionParameter / FunctionType / EnumValues — constructors + display
// =============================================================================

#[test]
fn function_call_new_and_default() {
    let fc = FunctionCall::new();
    assert_eq!(fc.name, "");
    assert!(fc.description.is_none());
    assert!(fc.parameters.is_empty());
    let d: FunctionCall = Default::default();
    assert_eq!(d.name, fc.name);
}

#[test]
fn function_parameter_new_chains_description_required() {
    let p = FunctionParameter::new("x", FunctionType::String)
        .description("desc")
        .required(false);
    assert_eq!(p.name, "x");
    assert_eq!(p._type, FunctionType::String);
    assert_eq!(p.description.as_deref(), Some("desc"));
    assert!(!p.required);
    let d: FunctionParameter = Default::default();
    assert_eq!(d._type, FunctionType::String);
    assert!(d.required);
}

#[test]
fn function_type_display_all_variants() {
    assert_eq!(FunctionType::String.to_string(), "string");
    assert_eq!(FunctionType::Number.to_string(), "number");
    assert_eq!(FunctionType::Boolean.to_string(), "boolean");
    assert_eq!(FunctionType::Null.to_string(), "null");
    assert_eq!(
        FunctionType::Array(Box::new(FunctionType::String)).to_string(),
        "array<string>"
    );
    assert_eq!(
        FunctionType::Option(Box::new(FunctionType::Number)).to_string(),
        "Option<number>"
    );
    assert_eq!(
        FunctionType::Map(Box::new(FunctionType::Boolean)).to_string(),
        "map<boolean>"
    );
    assert_eq!(
        FunctionType::OneOf(vec![
            open_ai_rust::FunctionVariant {
                name: "A".into(),
                description: None,
                parameters: vec![]
            },
            open_ai_rust::FunctionVariant {
                name: "B".into(),
                description: None,
                parameters: vec![]
            },
        ])
        .to_string(),
        "oneOf<A|B>"
    );
    assert_eq!(FunctionType::Object(vec![]).to_string(), "object");
    // FunctionType::Display uses `join(", ")` for strings, and `iter+to_string` for nums.
    assert_eq!(
        FunctionType::Enum(EnumValues::String(vec!["red".into(), "blue".into()])).to_string(),
        "enum<red, blue>"
    );
    assert_eq!(
        FunctionType::Enum(EnumValues::Int(vec![1, 2])).to_string(),
        "enum<1, 2>"
    );
    assert_eq!(
        FunctionType::Enum(EnumValues::Float(vec![1.5])).to_string(),
        "enum<1.5>"
    );
}

#[test]
fn enum_values_display_branches() {
    assert_eq!(EnumValues::String(vec!["x".into()]).to_string(), r#"["x"]"#);
    assert_eq!(EnumValues::Int(vec![1]).to_string(), "[1]");
    assert_eq!(EnumValues::Float(vec![1.25]).to_string(), "[1.25]");
}

#[test]
fn enum_values_int_serialises_as_number_type_with_enum_array() {
    let p = FunctionCall {
        name: "x".into(),
        description: None,
        parameters: vec![FunctionParameter::new(
            "n",
            FunctionType::Enum(EnumValues::Int(vec![1, 2, 3])),
        )],
    };
    let v = serde_json::to_value(&p).unwrap();
    assert_eq!(v["parameters"]["properties"]["n"]["type"], "number");
    assert_eq!(
        v["parameters"]["properties"]["n"]["enum"],
        serde_json::json!([1, 2, 3])
    );
}

#[test]
fn enum_values_float_serialises_as_number_type() {
    let p = FunctionCall {
        name: "x".into(),
        description: None,
        parameters: vec![FunctionParameter::new(
            "n",
            FunctionType::Enum(EnumValues::Float(vec![1.5, 2.5])),
        )],
    };
    let v = serde_json::to_value(&p).unwrap();
    assert_eq!(v["parameters"]["properties"]["n"]["type"], "number");
    assert_eq!(
        v["parameters"]["properties"]["n"]["enum"],
        serde_json::json!([1.5, 2.5])
    );
}

#[test]
fn function_type_null_serialises_as_null_type() {
    let p = FunctionCall {
        name: "x".into(),
        description: None,
        parameters: vec![FunctionParameter::new("n", FunctionType::Null)],
    };
    let v = serde_json::to_value(&p).unwrap();
    assert_eq!(v["parameters"]["properties"]["n"]["type"], "null");
}

// =============================================================================
// ChatToolChoice / ResponseFormat / StreamOptions / ReasoningEffort
// =============================================================================

#[test]
fn chat_tool_choice_none_and_required_and_from_string() {
    assert_eq!(
        serde_json::to_value(ChatToolChoice::none()).unwrap(),
        "none"
    );
    assert_eq!(
        serde_json::to_value(ChatToolChoice::required()).unwrap(),
        "required"
    );
    let from_string: ChatToolChoice = String::from("auto").into();
    assert_eq!(serde_json::to_value(&from_string).unwrap(), "auto");
    let from_str: ChatToolChoice = "auto".into();
    assert_eq!(serde_json::to_value(&from_str).unwrap(), "auto");
}

#[test]
fn chat_tool_choice_struct_variant_round_trips() {
    let tc = ChatToolChoice::Function {
        type_: ChatToolChoiceType::Function,
        function: ChatToolChoiceFunction { name: "x".into() },
    };
    let s = serde_json::to_string(&tc).unwrap();
    let parsed: ChatToolChoice = serde_json::from_str(&s).unwrap();
    if let ChatToolChoice::Function { function, .. } = parsed {
        assert_eq!(function.name, "x");
    } else {
        panic!("not Function");
    }
}

#[test]
fn response_format_text_and_json_object_shapes() {
    let v = serde_json::to_value(ResponseFormat::Text).unwrap();
    assert_eq!(v, serde_json::json!({ "type": "text" }));
    let v = serde_json::to_value(ResponseFormat::JsonObject).unwrap();
    assert_eq!(v, serde_json::json!({ "type": "json_object" }));
}

#[test]
fn json_schema_spec_with_description() {
    let rf = ResponseFormat::JsonSchema {
        json_schema: JsonSchemaSpec {
            name: "x".into(),
            description: Some("desc".into()),
            schema: serde_json::json!({ "type": "object" }),
            strict: Some(false),
        },
    };
    let v = serde_json::to_value(&rf).unwrap();
    assert_eq!(v["json_schema"]["description"], "desc");
    assert_eq!(v["json_schema"]["strict"], false);
}

#[test]
fn stream_options_default_is_empty_object() {
    let s: StreamOptions = StreamOptions::default();
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v, serde_json::json!({}));
}

#[test]
fn reasoning_effort_all_variants_serialise() {
    assert_eq!(serde_json::to_value(ReasoningEffort::Low).unwrap(), "low");
    assert_eq!(
        serde_json::to_value(ReasoningEffort::Medium).unwrap(),
        "medium"
    );
    assert_eq!(serde_json::to_value(ReasoningEffort::High).unwrap(), "high");
}

// =============================================================================
// PayLoadBuilder — remaining setters
// =============================================================================

#[test]
fn payload_builder_setters_round_trip() {
    let mut metadata = HashMap::new();
    metadata.insert("k".into(), "v".into());
    let mut logit = HashMap::new();
    logit.insert("100".to_string(), 50);

    let p = PayLoadBuilder::new(OpenAiModel::GPT4o)
        .add_message(ChatMessage::user("hi"))
        .add_message(ChatMessage::assistant("ack"))
        .frequency_penalty(0.1)
        .logprobs(true)
        .top_logprobs(3)
        .max_tokens(100)
        .n(2)
        .presence_penalty(0.05)
        .response_format(ResponseFormat::Text)
        .seed(7)
        .service_tier("scale")
        .stop(vec!["END".into()])
        .stream(true)
        .stream_options(StreamOptions {
            include_usage: Some(false),
        })
        .top_p(0.5)
        .metadata(metadata)
        .metadata_entry("k2", "v2")
        .store(false)
        .user("usr")
        .logit_bias(logit)
        .parallel_tool_calls(false)
        .build();
    assert_eq!(p.messages.len(), 2);
    assert_eq!(p.frequency_penalty, Some(0.1));
    assert_eq!(p.max_tokens, Some(100));
    assert_eq!(p.n, Some(2));
    assert_eq!(p.user.as_deref(), Some("usr"));
    assert_eq!(p.store, Some(false));
    assert_eq!(p.metadata.unwrap().len(), 2);
    assert_eq!(p.logit_bias.unwrap()["100"], 50);
    assert_eq!(p.stream_options.unwrap().include_usage, Some(false));
}

#[test]
fn payload_default_constructor_via_new() {
    let p = ChatPayLoad::new(OpenAiModel::GPT4oMini, vec![ChatMessage::user("hi")]);
    assert_eq!(p.messages.len(), 1);
    assert!(p.tools.is_none());
}

// =============================================================================
// Templates — PayLoadTemplates / Quick*
// =============================================================================

#[test]
fn payload_templates_chat_default() {
    use open_ai_rust::PayLoadTemplates;
    let p = PayLoadTemplates::default(vec![ChatMessage::user("hi")]).to_payload();
    assert_eq!(p.messages.len(), 1);
}

#[test]
fn payload_templates_function_call_with_tool_choice() {
    use open_ai_rust::{PayLoadTemplates, QuickFunctionCallTemplate};
    let tc = FunctionCall {
        name: "do_thing".into(),
        description: None,
        parameters: vec![],
    };
    let template = QuickFunctionCallTemplate::default(
        vec![ChatMessage::user("x")],
        vec![tc.clone()],
        Some(tc.clone()),
    );
    let p = PayLoadTemplates::FunctionCall(template).to_payload();
    let choice = p.tool_choice.unwrap();
    let v = serde_json::to_value(&choice).unwrap();
    assert_eq!(v["function"]["name"], "do_thing");
}

#[test]
fn payload_templates_function_call_default_tool_choice_auto() {
    use open_ai_rust::{PayLoadTemplates, QuickFunctionCallTemplate};
    let tc = FunctionCall {
        name: "n".into(),
        description: None,
        parameters: vec![],
    };
    let template = QuickFunctionCallTemplate::default(vec![ChatMessage::user("x")], vec![tc], None);
    let p = PayLoadTemplates::FunctionCall(template).to_payload();
    assert_eq!(
        serde_json::to_value(p.tool_choice.unwrap()).unwrap(),
        "auto"
    );
}

// =============================================================================
// OpenAiModel — every variant
// =============================================================================

#[test]
fn open_ai_model_every_variant_as_str() {
    let pairs: &[(OpenAiModel, &str)] = &[
        (OpenAiModel::O1, "o1"),
        (OpenAiModel::O1Mini, "o1-mini"),
        (OpenAiModel::O1Pro, "o1-pro"),
        (OpenAiModel::O3, "o3"),
        (OpenAiModel::O3Mini, "o3-mini"),
        (OpenAiModel::O3Pro, "o3-pro"),
        (OpenAiModel::O4Mini, "o4-mini"),
        (OpenAiModel::GPT5, "gpt-5"),
        (OpenAiModel::GPT5Mini, "gpt-5-mini"),
        (OpenAiModel::GPT5Nano, "gpt-5-nano"),
        (OpenAiModel::GPT45Preview, "gpt-4.5-preview"),
        (OpenAiModel::GPT41, "gpt-4.1"),
        (OpenAiModel::GPT41Mini, "gpt-4.1-mini"),
        (OpenAiModel::GPT41Nano, "gpt-4.1-nano"),
        (OpenAiModel::GPT4o, "gpt-4o"),
        (OpenAiModel::GPT4oMini, "gpt-4o-mini"),
        (OpenAiModel::GPT4oAudioPreview, "gpt-4o-audio-preview"),
        (
            OpenAiModel::GPT4oMiniAudioPreview,
            "gpt-4o-mini-audio-preview",
        ),
        (OpenAiModel::GPT4oTranscribe, "gpt-4o-transcribe"),
        (OpenAiModel::GPT4oMiniTranscribe, "gpt-4o-mini-transcribe"),
        (OpenAiModel::GPT4oMiniTTS, "gpt-4o-mini-tts"),
        (OpenAiModel::GPT4Turbo, "gpt-4-turbo"),
        (OpenAiModel::GPT4, "gpt-4"),
        (OpenAiModel::GPT35Turbo, "gpt-3.5-turbo"),
        (OpenAiModel::GPTImage1, "gpt-image-1"),
        (OpenAiModel::DALLE3, "dall-e-3"),
        (OpenAiModel::DALLE2, "dall-e-2"),
        (OpenAiModel::TTS1, "tts-1"),
        (OpenAiModel::TTS1HD, "tts-1-hd"),
        (OpenAiModel::Whisper1, "whisper-1"),
        (OpenAiModel::TextEmbedding3Large, "text-embedding-3-large"),
        (OpenAiModel::TextEmbedding3Small, "text-embedding-3-small"),
        (OpenAiModel::TextEmbeddingAda002, "text-embedding-ada-002"),
        (OpenAiModel::OmniModerationLatest, "omni-moderation-latest"),
        (OpenAiModel::TextModerationLatest, "text-moderation-latest"),
        (OpenAiModel::TextModerationStable, "text-moderation-stable"),
        (OpenAiModel::TextModeration007, "text-moderation-007"),
        (OpenAiModel::Babbage002, "babbage-002"),
        (OpenAiModel::Davinci002, "davinci-002"),
        (
            OpenAiModel::Custom("gpt-4o-2024-08-06".into()),
            "gpt-4o-2024-08-06",
        ),
    ];
    for (m, s) in pairs {
        assert_eq!(m.as_str(), *s);
        assert_eq!(m.to_string(), *s);
        assert_eq!(
            serde_json::to_value(m).unwrap(),
            serde_json::Value::String(s.to_string())
        );
    }
}

#[test]
fn open_ai_model_from_string_and_str() {
    let a: OpenAiModel = String::from("custom-model-id").into();
    assert_eq!(a.as_str(), "custom-model-id");
    let b: OpenAiModel = "another".into();
    assert_eq!(b.as_str(), "another");
}

// =============================================================================
// ChatMessage / ChatMessageRole / ChatContent / ContentPart
// =============================================================================

#[test]
fn chat_message_developer_and_with_name() {
    let m = ChatMessage::developer("instructions").with_name("alice");
    assert_eq!(m.role, ChatMessageRole::Developer);
    assert_eq!(m.name.as_deref(), Some("alice"));
}

#[test]
fn chat_message_text_with_parts_includes_text_and_refusal() {
    let m = ChatMessage {
        role: ChatMessageRole::Assistant,
        content: ChatContent::Parts(vec![
            ContentPart::Text { text: "hi ".into() },
            ContentPart::ImageUrl {
                image_url: ImageUrlSpec {
                    url: "u".into(),
                    detail: None,
                },
            },
            ContentPart::Refusal {
                refusal: "no".into(),
            },
        ]),
        name: None,
        tool_call_id: None,
        tool_calls: None,
        refusal: None,
    };
    assert_eq!(m.text(), "hi no");
}

#[test]
fn chat_message_with_image_promotes_text_to_parts() {
    let m = ChatMessage::user("look:").with_image("https://x");
    match m.content {
        ChatContent::Parts(parts) => assert_eq!(parts.len(), 2),
        _ => panic!("expected Parts"),
    }
}

#[test]
fn chat_message_with_image_on_empty_user_promotes_to_single_image_part() {
    let m = ChatMessage::user("").with_image("https://x");
    if let ChatContent::Parts(parts) = m.content {
        assert_eq!(parts.len(), 1);
    } else {
        panic!("expected Parts");
    }
}

#[test]
fn chat_message_with_image_on_existing_parts_appends() {
    let mut m = ChatMessage::user("");
    m.content = ChatContent::Parts(vec![ContentPart::Text { text: "x".into() }]);
    let m = m.with_image("u");
    if let ChatContent::Parts(parts) = m.content {
        assert_eq!(parts.len(), 2);
    } else {
        panic!("expected Parts");
    }
}

#[test]
fn chat_message_role_from_string_every_case() {
    assert_eq!(
        ChatMessageRole::from(String::from("system")),
        ChatMessageRole::System
    );
    assert_eq!(
        ChatMessageRole::from(String::from("user")),
        ChatMessageRole::User
    );
    assert_eq!(
        ChatMessageRole::from(String::from("assistant")),
        ChatMessageRole::Assistant
    );
    assert_eq!(
        ChatMessageRole::from(String::from("tool")),
        ChatMessageRole::Tool
    );
    assert_eq!(
        ChatMessageRole::from(String::from("developer")),
        ChatMessageRole::Developer
    );
    // Unknown falls back to User.
    assert_eq!(
        ChatMessageRole::from(String::from("???")),
        ChatMessageRole::User
    );
}

#[test]
fn chat_message_role_display_every_case() {
    assert_eq!(ChatMessageRole::System.to_string(), "system");
    assert_eq!(ChatMessageRole::User.to_string(), "user");
    assert_eq!(ChatMessageRole::Assistant.to_string(), "assistant");
    assert_eq!(ChatMessageRole::Tool.to_string(), "tool");
    assert_eq!(ChatMessageRole::Developer.to_string(), "developer");
}

#[test]
fn chat_content_default_is_empty_text() {
    let d: ChatContent = Default::default();
    if let ChatContent::Text(s) = d {
        assert!(s.is_empty());
    } else {
        panic!("default is Text");
    }
}

#[test]
fn chat_content_from_parts_vector() {
    let parts = vec![ContentPart::Text { text: "x".into() }];
    let c: ChatContent = parts.into();
    assert!(matches!(c, ChatContent::Parts(_)));
}

// =============================================================================
// AiMsgResponse / get_messages with tool_calls + refusal
// =============================================================================

#[test]
fn ai_msg_response_get_messages_round_trips_tool_calls_and_refusal() {
    use open_ai_rust::AiMsgResponse;
    let r = AiMsgResponse {
        choices: vec![Choice {
            finish_reason: "stop".into(),
            index: 0,
            message: AiResponseMessage {
                content: Some("hi".into()),
                role: "assistant".into(),
                refusal: Some("nope".into()),
                audio: None,
                tool_calls: Some(vec![ToolCallRes {
                    id: Some("call_1".into()),
                    type_: Some("function".into()),
                    function: FunctionCallRes {
                        name: "fn".into(),
                        arguments: serde_json::json!({"k":"v"}),
                    },
                }]),
            },
            logprobs: None,
        }],
        created: 1,
        id: "x".into(),
        model: "m".into(),
        object: "chat.completion".into(),
        usage: Usage {
            completion_tokens: Some(1),
            prompt_tokens: 1,
            total_tokens: 2,
            prompt_tokens_details: Some(PromptTokensDetails::default()),
            completion_tokens_details: Some(CompletionTokensDetails::default()),
        },
        system_fingerprint: None,
        service_tier: None,
    };

    let msgs = r.get_messages();
    assert_eq!(msgs.len(), 1);
    let m = &msgs[0];
    assert_eq!(m.role, ChatMessageRole::Assistant);
    assert!(m.tool_calls.is_some());
    let tcs = m.tool_calls.as_ref().unwrap();
    assert_eq!(tcs[0].id, "call_1");
    assert!(matches!(tcs[0].type_, MessageToolCallType::Function));
    assert_eq!(m.refusal.as_deref(), Some("nope"));

    assert_eq!(r.get_last_msg_text(), Some("hi".into()));
}

#[test]
fn ai_msg_response_get_tool_calls_flattens_across_choices() {
    use open_ai_rust::AiMsgResponse;
    let r = AiMsgResponse {
        choices: vec![
            Choice {
                finish_reason: "stop".into(),
                index: 0,
                message: AiResponseMessage {
                    content: None,
                    role: "assistant".into(),
                    refusal: None,
                    audio: None,
                    tool_calls: Some(vec![ToolCallRes {
                        id: None,
                        type_: None,
                        function: FunctionCallRes {
                            name: "a".into(),
                            arguments: serde_json::json!({}),
                        },
                    }]),
                },
                logprobs: None,
            },
            Choice {
                finish_reason: "stop".into(),
                index: 1,
                message: AiResponseMessage {
                    content: None,
                    role: "assistant".into(),
                    refusal: None,
                    audio: None,
                    tool_calls: None,
                },
                logprobs: None,
            },
        ],
        created: 1,
        id: "x".into(),
        model: "m".into(),
        object: "chat.completion".into(),
        usage: Usage {
            completion_tokens: Some(0),
            prompt_tokens: 0,
            total_tokens: 0,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        },
        system_fingerprint: None,
        service_tier: None,
    };
    let tcs = r.get_tool_calls();
    assert_eq!(tcs.len(), 1);
    assert_eq!(tcs[0].name, "a");
}

#[test]
fn ai_msg_response_get_first_tool_call_args_error_cases() {
    use open_ai_rust::AiMsgResponse;
    let r_no_choices = AiMsgResponse {
        choices: vec![],
        created: 1,
        id: "x".into(),
        model: "m".into(),
        object: "x".into(),
        usage: Usage {
            completion_tokens: None,
            prompt_tokens: 0,
            total_tokens: 0,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        },
        system_fingerprint: None,
        service_tier: None,
    };
    assert!(r_no_choices.get_first_tool_call_args().is_err());

    let r_no_tools = AiMsgResponse {
        choices: vec![Choice {
            finish_reason: "stop".into(),
            index: 0,
            message: AiResponseMessage {
                content: None,
                role: "assistant".into(),
                refusal: None,
                audio: None,
                tool_calls: None,
            },
            logprobs: None,
        }],
        ..r_no_choices
    };
    assert!(r_no_tools.get_first_tool_call_args().is_err());

    let r_empty_tools = AiMsgResponse {
        choices: vec![Choice {
            finish_reason: "stop".into(),
            index: 0,
            message: AiResponseMessage {
                content: None,
                role: "assistant".into(),
                refusal: None,
                audio: None,
                tool_calls: Some(vec![]),
            },
            logprobs: None,
        }],
        ..r_no_tools
    };
    assert!(r_empty_tools.get_first_tool_call_args().is_err());
}

#[test]
fn ai_msg_response_get_last_msg_text_none_when_no_choices() {
    use open_ai_rust::AiMsgResponse;
    let r = AiMsgResponse {
        choices: vec![],
        created: 0,
        id: "".into(),
        model: "".into(),
        object: "".into(),
        usage: Usage {
            completion_tokens: None,
            prompt_tokens: 0,
            total_tokens: 0,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        },
        system_fingerprint: None,
        service_tier: None,
    };
    assert!(r.get_last_msg_text().is_none());
}

#[test]
fn tool_call_res_get_args() {
    let tc = ToolCallRes {
        id: None,
        type_: None,
        function: FunctionCallRes {
            name: "f".into(),
            arguments: serde_json::json!({"x":1}),
        },
    };
    let v = tc.get_args();
    assert_eq!(v["x"], 1);
}

#[test]
fn function_call_res_arguments_accepts_object_form_too() {
    // Custom deserializer accepts both string (canonical) and already-decoded object.
    let s = r#"{"name":"f","arguments":{"k":"v"}}"#;
    let f: FunctionCallRes = serde_json::from_str(s).unwrap();
    assert_eq!(f.name, "f");
    assert_eq!(f.arguments["k"], "v");
}

// =============================================================================
// Responses input/output/tool/request
// =============================================================================

#[test]
fn response_input_item_helpers_emit_correct_shapes() {
    use serde_json::json;
    let u = ResponseInputItem::user("hi");
    let s = ResponseInputItem::system("sys");
    let d = ResponseInputItem::developer("dev");
    let a = ResponseInputItem::assistant("ack");
    let fco = ResponseInputItem::function_call_output("call_1", "result");

    assert_eq!(serde_json::to_value(&u).unwrap()["role"], "user");
    assert_eq!(serde_json::to_value(&s).unwrap()["role"], "system");
    assert_eq!(serde_json::to_value(&d).unwrap()["role"], "developer");
    assert_eq!(serde_json::to_value(&a).unwrap()["role"], "assistant");
    assert_eq!(
        serde_json::to_value(&fco).unwrap(),
        json!({ "type": "function_call_output", "call_id": "call_1", "output": "result" })
    );
}

#[test]
fn response_input_content_parts_round_trip() {
    let p = ResponseInputContent::Parts(vec![
        ResponseInputContentPart::InputText { text: "x".into() },
        ResponseInputContentPart::InputImage {
            image_url: Some("http://x".into()),
            file_id: None,
            detail: Some("high".into()),
        },
        ResponseInputContentPart::InputFile {
            file_id: Some("f".into()),
            file_data: None,
            file_name: None,
        },
        ResponseInputContentPart::InputAudio {
            input_audio: open_ai_rust::responses::input::InputAudio {
                data: "AAA".into(),
                format: "mp3".into(),
            },
        },
    ]);
    let s = serde_json::to_string(&p).unwrap();
    let _: ResponseInputContent = serde_json::from_str(&s).unwrap();
}

#[test]
fn response_tool_choice_constructors_and_from_impls() {
    assert_eq!(
        serde_json::to_value(ResponseToolChoice::auto()).unwrap(),
        "auto"
    );
    assert_eq!(
        serde_json::to_value(ResponseToolChoice::none()).unwrap(),
        "none"
    );
    assert_eq!(
        serde_json::to_value(ResponseToolChoice::required()).unwrap(),
        "required"
    );
    let f = ResponseToolChoice::function("doit");
    let v = serde_json::to_value(&f).unwrap();
    assert_eq!(v["type"], "function");
    assert_eq!(v["name"], "doit");
    let from_string: ResponseToolChoice = "auto".to_string().into();
    let from_str: ResponseToolChoice = "auto".into();
    let _ = (from_string, from_str);
    let hosted = ResponseToolChoice::Hosted {
        type_: "web_search_preview".into(),
    };
    let v = serde_json::to_value(&hosted).unwrap();
    assert_eq!(v["type"], "web_search_preview");
}

#[test]
fn response_request_builder_setters() {
    use open_ai_rust::responses::{ReasoningConfig, ResponseRequestBuilder, TextConfig};
    let req = ResponseRequestBuilder::new(OpenAiModel::GPT41Mini, "hi")
        .instructions("inst")
        .previous_response_id("prev")
        .tools(vec![ResponseTool::Function {
            name: "f".into(),
            description: None,
            parameters: serde_json::json!({}),
            strict: None,
        }])
        .tool_choice(ResponseToolChoice::auto())
        .parallel_tool_calls(true)
        .max_output_tokens(64)
        .temperature(0.1)
        .top_p(0.9)
        .user("u")
        .metadata({
            let mut m = HashMap::new();
            m.insert("k".into(), "v".into());
            m
        })
        .store(true)
        .truncation("auto")
        .service_tier("default")
        .include(vec!["file_search.results".into()])
        .text(TextConfig::json_schema(
            "name",
            serde_json::json!({ "type": "object" }),
        ))
        .reasoning(ReasoningConfig {
            effort: Some(ReasoningEffort::Low),
            summary: Some("auto".into()),
        })
        .reasoning_effort(ReasoningEffort::Medium)
        .build();
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["instructions"], "inst");
    assert_eq!(v["previous_response_id"], "prev");
    assert_eq!(v["truncation"], "auto");
    assert_eq!(v["service_tier"], "default");
    assert!(v["text"]["format"]["type"].is_string());
    // reasoning_effort chained after reasoning(); should keep the summary.
    assert_eq!(v["reasoning"]["summary"], "auto");
    assert_eq!(v["reasoning"]["effort"], "medium");
}

#[test]
fn response_object_function_calls_filter_and_output_text_aggregation() {
    let obj = ResponseObject {
        id: "r".into(),
        object: "response".into(),
        created_at: 1,
        model: "m".into(),
        status: ResponseStatus::Completed,
        output: vec![
            ResponseOutputItem::Message {
                id: "m1".into(),
                status: None,
                role: "assistant".into(),
                content: vec![
                    ResponseOutputContentPart::OutputText {
                        text: "hello ".into(),
                        annotations: vec![],
                    },
                    ResponseOutputContentPart::OutputText {
                        text: "world".into(),
                        annotations: vec![],
                    },
                    ResponseOutputContentPart::Refusal {
                        refusal: "nope".into(),
                    },
                ],
            },
            ResponseOutputItem::FunctionCall(ResponseFunctionCall {
                id: "fc1".into(),
                call_id: "c".into(),
                name: "do".into(),
                arguments: "{}".into(),
                status: None,
            }),
            ResponseOutputItem::Reasoning {
                id: "r1".into(),
                status: None,
                summary: vec![],
                encrypted_content: None,
            },
        ],
        usage: None,
        error: None,
        incomplete_details: None,
        previous_response_id: None,
        instructions: None,
        temperature: None,
        top_p: None,
        max_output_tokens: None,
        parallel_tool_calls: None,
        reasoning: None,
        text: None,
        tools: vec![],
        metadata: None,
    };
    assert_eq!(obj.output_text(), "hello world");
    let fcs = obj.function_calls();
    assert_eq!(fcs.len(), 1);
    assert_eq!(fcs[0].name, "do");
}

#[test]
fn response_status_all_variants_round_trip() {
    let states = [
        ResponseStatus::Queued,
        ResponseStatus::InProgress,
        ResponseStatus::Completed,
        ResponseStatus::Failed,
        ResponseStatus::Cancelled,
        ResponseStatus::Incomplete,
    ];
    for s in states {
        let v = serde_json::to_value(&s).unwrap();
        let back: ResponseStatus = serde_json::from_value(v).unwrap();
        assert_eq!(back, s);
    }
}

// =============================================================================
// resources::files::FilePurpose
// =============================================================================

#[test]
fn file_purpose_all_variants_as_str() {
    use open_ai_rust::resources::files::FilePurpose;
    assert_eq!(FilePurpose::Assistants.as_str(), "assistants");
    assert_eq!(FilePurpose::Batch.as_str(), "batch");
    assert_eq!(FilePurpose::FineTune.as_str(), "fine-tune");
    assert_eq!(FilePurpose::Vision.as_str(), "vision");
    assert_eq!(FilePurpose::UserData.as_str(), "user_data");
    assert_eq!(FilePurpose::Evals.as_str(), "evals");
}

// =============================================================================
// Image / audio / moderation builders
// =============================================================================

#[test]
fn image_generation_builder_chains_every_setter() {
    use open_ai_rust::resources::images::ImageGenerationBuilder;
    let r = ImageGenerationBuilder::new("a cat")
        .model("gpt-image-1")
        .n(2)
        .size("1024x1024")
        .quality("high")
        .style("natural")
        .response_format("b64_json")
        .background("transparent")
        .output_format("png")
        .output_compression(80)
        .moderation("strict")
        .user("u")
        .build();
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["prompt"], "a cat");
    assert_eq!(v["model"], "gpt-image-1");
    assert_eq!(v["n"], 2);
    assert_eq!(v["size"], "1024x1024");
    assert_eq!(v["quality"], "high");
    assert_eq!(v["style"], "natural");
    assert_eq!(v["response_format"], "b64_json");
    assert_eq!(v["background"], "transparent");
    assert_eq!(v["output_format"], "png");
    assert_eq!(v["output_compression"], 80);
    assert_eq!(v["moderation"], "strict");
    assert_eq!(v["user"], "u");
}

#[test]
fn transcription_builder_chains_every_setter() {
    use open_ai_rust::resources::audio::{TranscriptionFormat, TranscriptionRequestBuilder};
    let r = TranscriptionRequestBuilder::new("whisper-1")
        .file_bytes(b"x".to_vec(), "x.mp3")
        .mime_type("audio/mpeg")
        .language("en")
        .prompt("hello")
        .response_format(TranscriptionFormat::VerboseJson)
        .temperature(0.0)
        .timestamp_granularities(vec!["segment".into(), "word".into()])
        .build();
    assert_eq!(r.model, "whisper-1");
    assert_eq!(r.language.as_deref(), Some("en"));
    assert_eq!(r.prompt.as_deref(), Some("hello"));
    assert_eq!(r.temperature, Some(0.0));
    assert!(r.timestamp_granularities.is_some());
}

#[test]
fn transcription_formats_serialise_as_strings() {
    use open_ai_rust::resources::audio::{TranscriptionFormat, TranscriptionRequestBuilder};
    // Exercise every variant's `as_str` indirectly via building.
    for f in [
        TranscriptionFormat::Json,
        TranscriptionFormat::Text,
        TranscriptionFormat::Srt,
        TranscriptionFormat::VerboseJson,
        TranscriptionFormat::Vtt,
    ] {
        let r = TranscriptionRequestBuilder::new("whisper-1")
            .file_bytes(b"x".to_vec(), "x.mp3")
            .response_format(f)
            .build();
        assert!(r.response_format.is_some());
    }
}

#[test]
fn speech_builder_chains_setters() {
    use open_ai_rust::resources::audio::SpeechRequestBuilder;
    let r = SpeechRequestBuilder::new("gpt-4o-mini-tts", "alloy", "hi")
        .response_format("opus")
        .speed(1.25)
        .instructions("whisper")
        .build();
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["voice"], "alloy");
    assert_eq!(v["response_format"], "opus");
    assert_eq!(v["speed"], 1.25);
    assert_eq!(v["instructions"], "whisper");
}

#[test]
fn moderation_input_from_impls() {
    use open_ai_rust::resources::moderations::ModerationInput;
    let _a: ModerationInput = "x".into();
    let _b: ModerationInput = String::from("x").into();
    let _c: ModerationInput = vec!["a".to_string(), "b".to_string()].into();
}

// =============================================================================
// embeddings — From impls
// =============================================================================

#[test]
fn embedding_input_from_impls() {
    use open_ai_rust::resources::embeddings::EmbeddingInput;
    let _a: EmbeddingInput = "x".into();
    let _b: EmbeddingInput = String::from("x").into();
    let _c: EmbeddingInput = vec!["x".to_string()].into();
}

// =============================================================================
// OpenAiError additional surfaces
// =============================================================================

#[test]
fn openai_error_stream_constructor_and_display() {
    let e = OpenAiError::stream("oops");
    let s = e.to_string();
    assert!(s.contains("oops"));
}

#[test]
fn openai_error_display_contains_status_and_message() {
    let e = OpenAiError::Api {
        status: 503,
        message: "boom".into(),
        code: Some("c".into()),
        type_: Some("t".into()),
        param: None,
    };
    let s = e.to_string();
    assert!(s.contains("503"));
    assert!(s.contains("boom"));
    assert!(s.contains("code=c"));
    assert!(s.contains("type=t"));
}

#[test]
fn openai_error_from_io_error() {
    let io = std::io::Error::other("io");
    let e: OpenAiError = io.into();
    assert!(matches!(e, OpenAiError::Io(_)));
}

#[test]
fn openai_error_from_serde_error() {
    let bad: serde_json::Error = serde_json::from_str::<u32>("not a num").unwrap_err();
    let e: OpenAiError = bad.into();
    assert!(matches!(e, OpenAiError::Decode(_)));
}

#[test]
fn openai_error_to_string_via_from_string() {
    let s: String = OpenAiError::config("hi").into();
    assert!(s.contains("hi"));
}

// =============================================================================
// Usage / detail structs Default impls
// =============================================================================

#[test]
fn token_detail_defaults_zero_everywhere() {
    let p = PromptTokensDetails::default();
    assert_eq!(p.cached_tokens, 0);
    assert_eq!(p.audio_tokens, 0);
    let c = CompletionTokensDetails::default();
    assert_eq!(c.reasoning_tokens, 0);
    assert_eq!(c.audio_tokens, 0);
    assert_eq!(c.accepted_prediction_tokens, 0);
    assert_eq!(c.rejected_prediction_tokens, 0);
}

// =============================================================================
// ToolCall struct round trip
// =============================================================================

#[test]
fn tool_call_round_trip() {
    let tc = ToolCall {
        id: "x".into(),
        type_: MessageToolCallType::Function,
        function: ToolCallFunction {
            name: "f".into(),
            arguments: r#"{"a":1}"#.into(),
        },
    };
    let s = serde_json::to_string(&tc).unwrap();
    let back: ToolCall = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id, "x");
}
