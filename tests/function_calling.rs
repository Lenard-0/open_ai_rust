
#[cfg(test)]
mod tests {
    use open_ai_rust::{logoi::{input::{payload::builder::PayLoadBuilder, tool::{FunctionCall, FunctionParameter, FunctionType, ToolType}}, message::{ChatMessage, ChatMessageRole}, models::OpenAiModel}, requests::open_ai_msg, set_key};

    #[test]
    fn can_correctly_parse_function_definition() {
        let function_def = FunctionCall {
            name: "change_light".to_string(),
            description: Some("Change the light in the room.".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "turn_on_light".to_string(),
                    _type: FunctionType::Boolean,
                    description: Some("True turns on the light and false turns it off".to_string()),
                    is_required: true
                }
            ]
        };

        let function_def_json = serde_json::to_value(&function_def).unwrap();
        println!("function_def_json: {:#?}", function_def_json);
        // expecting:
        // {
        //     "description": String("Change the light in the room."),
        //     "name": String("change_light"),
        //     "arguments": Object {
        //         "turn_on_light": Object {
        //             "description": String("True turns on the light and false turns it off"),
        //             "type": String("boolean"),
        //         },
        //         "required": ["turn_on_light"]
        //     },
        // }

        let arguments = function_def_json.get("arguments").unwrap().as_object().unwrap();
        let turn_on_light = arguments.get("turn_on_light").unwrap().as_object().unwrap();
        assert_eq!(turn_on_light.get("type").unwrap().as_str().unwrap(), "boolean");
        assert_eq!(turn_on_light.get("description").unwrap().as_str().unwrap(), "True turns on the light and false turns it off");

        let required = arguments.get("required").unwrap().as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].as_str().unwrap(), "turn_on_light");

    }

    #[tokio::test]
    async fn can_do_function_call_simple() {
        dotenv::dotenv().ok();
        set_key(std::env::var("OPENAI_SK").unwrap()); // Set the OpenAI API key from the environment variable

        let system_msg = ChatMessage {
            role: ChatMessageRole::System,
            content: "You are part of a test in a Rust program. Follow the user's request to complete the function/tool call.".to_string(),
            name: None
        };

        let user_msg = ChatMessage {
            role: ChatMessageRole::User,
            content: "Turn on light!".to_string(),
            name: None
        };

        let functions = vec![
            FunctionCall {
                name: "change_light".to_string(),
                description: Some("Change the light in the room.".to_string()),
                parameters: vec![
                    FunctionParameter {
                        name: "turn_on_light".to_string(),
                        _type: FunctionType::Boolean,
                        description: Some("True turns on the light and false turns it off".to_string()),
                        is_required: true
                    }
                ],
            }
        ];

        let payload = PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(vec![system_msg, user_msg])
            .tools(functions)
            .build();

        let response = open_ai_msg(payload).await.unwrap();

        let tool_calls = response.get_tool_calls();

        assert_eq!(tool_calls.len(), 1);

        let tool_call = &tool_calls[0];

        assert_eq!(tool_call.name, "change_light");

        println!("tool_call: {:#?}", tool_call);

        let arguments = tool_call.arguments.as_object().unwrap();

        println!("arguments: {:#?}", arguments);

        assert_eq!(arguments.get("turn_on_light").unwrap().as_bool().unwrap(), true);
    }
}



