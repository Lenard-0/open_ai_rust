// WARNING:
// KEEP IN MIND TESTS SOMETIMES CAN RANDOMLY FAIL EVEN THOUGH SAME SEED AND 0 TEMP
// IF A TEST USING THE FUNCTION CALL DOES FAIL TRY RUNNING IT AGAIN BEFORE DEBUGGING
//
// These hit the live OpenAI API and are `#[ignore]`-gated. Run with:
//   OPENAI_API_KEY=sk-... cargo test --test function_calling -- --ignored
#[cfg(test)]
mod tests {
    use open_ai_rust::{
        ChatMessage, Client, EnumValues, FunctionCall, FunctionParameter, FunctionType,
        OpenAiModel, PayLoadBuilder,
    };

    #[tokio::test]
    #[ignore = "hits the real OpenAI API"]
    async fn can_do_function_call_simple() {
        dotenv::dotenv().ok();
        let client = Client::from_env().unwrap();

        let messages = vec![
            ChatMessage::system(
                "You are part of a test in a Rust program. Follow the user's request to complete the function/tool call.",
            ),
            ChatMessage::user("Turn on light!"),
        ];

        let functions = vec![FunctionCall {
            name: "change_light".to_string(),
            description: Some("Change the light in the room.".to_string()),
            parameters: vec![FunctionParameter {
                name: "turn_on_light".to_string(),
                _type: FunctionType::Boolean,
                description: Some("True turns on the light and false turns it off".to_string()),
                required: true,
            }],
        }];

        let payload = PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(messages)
            .tools(functions)
            .seed(0)
            .build();

        let response = client.chat().create(payload).await.unwrap();
        let tool_calls = response.get_tool_calls();
        assert_eq!(tool_calls.len(), 1);
        let tool_call = &tool_calls[0];
        assert_eq!(tool_call.name, "change_light");
        let arguments = tool_call.arguments.as_object().unwrap();
        assert!(arguments.get("turn_on_light").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    #[ignore = "hits the real OpenAI API"]
    async fn can_do_weather_function_call_including_enums_and_strings_multiple_calls_and_params() {
        dotenv::dotenv().ok();
        let client = Client::from_env().unwrap();

        let messages = vec![
            ChatMessage::system(
                "You are a weather bot. Use the provided functions to answer questions. If calling both functions make sure to do them in order of rain probability first.",
            ),
            ChatMessage::user(
                "What's the weather in San Francisco today and the likelihood it'll rain?",
            ),
        ];

        let functions = vec![
            FunctionCall {
                name: "get_current_temperature".to_string(),
                description: Some(
                    "Get the current temperature for a specific location".to_string(),
                ),
                parameters: vec![
                    FunctionParameter {
                        name: "location".to_string(),
                        _type: FunctionType::String,
                        description: Some(
                            "The city and state, e.g., San Francisco, CA".to_string(),
                        ),
                        required: true,
                    },
                    FunctionParameter {
                        name: "unit".to_string(),
                        _type: FunctionType::Enum(EnumValues::String(vec![
                            "Fahrenheight".to_string(),
                            "Celcius".to_string(),
                        ])),
                        description: Some(
                            "The temperature unit to use. Infer this from the user's location."
                                .to_string(),
                        ),
                        required: true,
                    },
                ],
            },
            FunctionCall {
                name: "get_rain_probability".to_string(),
                description: Some(
                    "Get the probability of rain for a specific location".to_string(),
                ),
                parameters: vec![FunctionParameter {
                    name: "location".to_string(),
                    _type: FunctionType::String,
                    description: Some("The city and state, e.g., San Francisco, CA".to_string()),
                    required: true,
                }],
            },
        ];

        let payload = PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(messages)
            .tools(functions)
            .temperature(0.0)
            .seed(0)
            .build();

        let response = client.chat().create(payload).await.unwrap();
        let tool_calls = response.get_tool_calls();
        assert_eq!(tool_calls.len(), 2);

        let tool_call_1 = &tool_calls[0];
        assert_eq!(tool_call_1.name, "get_rain_probability");
        let arguments_1 = tool_call_1.arguments.as_object().unwrap();
        assert_eq!(
            arguments_1.get("location").unwrap().as_str().unwrap(),
            "San Francisco, CA"
        );

        let tool_call_2 = &tool_calls[1];
        assert_eq!(tool_call_2.name, "get_current_temperature");
        let arguments_2 = tool_call_2.arguments.as_object().unwrap();
        assert_eq!(
            arguments_2.get("location").unwrap().as_str().unwrap(),
            "San Francisco, CA"
        );
        assert_eq!(
            arguments_2.get("unit").unwrap().as_str().unwrap(),
            "Fahrenheight"
        );
    }
}
