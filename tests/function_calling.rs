
// WARNING:
// KEEP IN MIND TESTS SOMETIMES CAN RANDOMLY FAIL EVEN THOUGH SAME SEED AND 0 TEMP
// IF A TEST USING THE FUNCTION CALL DOES FAIL TRY RUNNING IT AGAIN BEFORE DEBUGGING
#[cfg(test)]
mod tests {
    use open_ai_rust::{logoi::{input::{payload::builder::PayLoadBuilder, tool::{FunctionCall, FunctionParameter, FunctionType}}, message::{ChatMessage, ChatMessageRole}, models::OpenAiModel}, requests::open_ai_msg, set_key};
    use serde_json::{json, Value};

    #[test]
    fn can_correctly_parse_simplest_function_definition() {
        let function_def = FunctionCall {
            name: "change_light".to_string(),
            description: Some("Change the light in the room.".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "turn_on_light".to_string(),
                    _type: FunctionType::Boolean,
                    description: Some("True turns on the light and false turns it off".to_string()),
                }
            ]
        };

        let function_def_json = serde_json::to_value(&function_def).unwrap();
        assert_eq!(function_def_json, json!({
            "description": "Change the light in the room.",
            "name": "change_light",
            "parameters": {
                "properties": {
                    "turn_on_light": {
                        "description": "True turns on the light and false turns it off",
                        "type": "boolean",
                    },
                },
                "required": [
                    "turn_on_light",
                ],
                "type": "object",
            },
        }));
    }

    #[test]
    fn can_correctly_parse_function_definition_with_enum_parameter() {
        let function_def = FunctionCall {
            name: "get_temp".to_string(),
            description: Some("Get's the current temperature.".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "unit".to_string(),
                    _type: FunctionType::Enum(json!(["Fahrenheight", "Celcius"]).as_array().unwrap().to_vec()),
                    description: Some("The temperature unit to use.".to_string()),
                }
            ]
        };

        let function_def_json = serde_json::to_value(&function_def).unwrap();
        assert_eq!(function_def_json, json!({
            "description": "Get's the current temperature.",
            "name": "get_temp",
            "parameters": {
                "properties": {
                    "unit": {
                        "description": "The temperature unit to use.",
                        "type": "string",
                        "enum": ["Fahrenheight", "Celcius"],
                    },
                },
                "required": [
                    "unit",
                ],
                "type": "object",
            },
        }));
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
                    }
                ],
            }
        ];

        let payload = PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(vec![system_msg, user_msg])
            .tools(functions)
            .seed(0)
            .build();

        let response = open_ai_msg(payload).await.unwrap();
        let tool_calls = response.get_tool_calls();
        assert_eq!(tool_calls.len(), 1);
        let tool_call = &tool_calls[0];
        assert_eq!(tool_call.name, "change_light");
        let arguments = tool_call.arguments.as_object().unwrap();
        assert_eq!(arguments.get("turn_on_light").unwrap().as_bool().unwrap(), true);
    }

    #[tokio::test]
    async fn can_do_weather_function_calls() {
        dotenv::dotenv().ok();
        set_key(std::env::var("OPENAI_SK").unwrap()); // Set the OpenAI API key from the environment variable

        let system_msg = ChatMessage {
            role: ChatMessageRole::System,
            content: "You are a weather bot. Use the provided functions to answer questions. If calling both functions make sure to do them in order of rain probability first.".to_string(),
            name: None
        };

        let user_msg = ChatMessage {
            role: ChatMessageRole::User,
            content: "What's the weather in San Francisco today and the likelihood it'll rain?".to_string(),
            name: None
        };

        let functions = vec![
            FunctionCall {
                name: "get_current_temperature".to_string(),
                description: Some("Get the current temperature for a specific location".to_string()),
                parameters: vec![
                    FunctionParameter {
                        name: "location".to_string(),
                        _type: FunctionType::String,
                        description: Some("The city and state, e.g., San Francisco, CA".to_string()),
                    },
                    FunctionParameter {
                        name: "unit".to_string(),
                        _type: FunctionType::Enum(vec![
                            Value::String("Fahrenheight".to_string()),
                            Value::String("Celcius".to_string())
                        ]),
                        description: Some("The temperature unit to use. Infer this from the user's location.".to_string()),
                    }
                ],
            },
            FunctionCall {
                name: "get_rain_probability".to_string(),
                description: Some("Get the probability of rain for a specific location".to_string()),
                parameters: vec![
                    FunctionParameter {
                        name: "location".to_string(),
                        _type: FunctionType::String,
                        description: Some("The city and state, e.g., San Francisco, CA".to_string()),
                    }
                ],
            }
        ];

        let payload = PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(vec![system_msg, user_msg])
            .tools(functions)
            .temperature(0.0)
            .seed(0)
            .build();

        let response = open_ai_msg(payload).await.unwrap();
        let tool_calls = response.get_tool_calls();
        assert_eq!(tool_calls.len(), 2);

        // Check the first tool call
        let tool_call_1 = &tool_calls[0];
        assert_eq!(tool_call_1.name, "get_rain_probability");
        let arguments_1 = tool_call_1.arguments.as_object().unwrap();
        assert_eq!(arguments_1.get("location").unwrap().as_str().unwrap(), "San Francisco, CA");

        // Check the second tool call
        let tool_call_2 = &tool_calls[1];
        assert_eq!(tool_call_2.name, "get_current_temperature");
        let arguments_2 = tool_call_2.arguments.as_object().unwrap();
        assert_eq!(arguments_2.get("location").unwrap().as_str().unwrap(), "San Francisco, CA");
        assert_eq!(arguments_2.get("unit").unwrap().as_str().unwrap(), "Fahrenheight");
    }

}



