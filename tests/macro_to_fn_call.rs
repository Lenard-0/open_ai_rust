

#[cfg(test)]
mod tests {
    use open_ai_rust::logoi::input::tool::{raw_macro::FunctionCallRaw, FunctionCall, FunctionParameter, FunctionType};
    use open_ai_rust_fn_call_extension::function_call;


    #[function_call("This function turns on or off the light in their room")]
    fn change_light(turn_on_light: bool, hex_color: String, brightness: i64, pulse_rate: f64) {
        if turn_on_light {
            // light on
        } else {
            // light off
        }
    }

    #[test]
    fn can_correctly_parse_function_definition_name_and_description() {
        let expected_function_call = FunctionCall {
            name: "change_light".to_string(),
            description: Some("This function turns on or off the light in their room".to_string()),
            parameters: vec![],
        };

        let converted_function_call = CHANGE_LIGHT.to_fn_call().unwrap();

        assert_eq!(converted_function_call.name, expected_function_call.name);
        assert_eq!(converted_function_call.description, expected_function_call.description);
    }

    #[test]
    fn can_correctly_parse_function_definition_primitive_parameter_types() {
        let expected_function_call = FunctionCall {
            name: "change_light".to_string(),
            description: Some("This function turns on or off the light in their room".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "turn_on_light".to_string(),
                    _type: FunctionType::Boolean,
                    description: None,
                },
                FunctionParameter {
                    name: "hex_color".to_string(),
                    _type: FunctionType::String,
                    description: None,
                },
                FunctionParameter {
                    name: "brightness".to_string(),
                    _type: FunctionType::Number,
                    description: None,
                },
                FunctionParameter {
                    name: "pulse_rate".to_string(),
                    _type: FunctionType::Number,
                    description: None,
                },
            ],
        };

        let converted_function_call = CHANGE_LIGHT.to_fn_call().unwrap();

        assert_eq!(converted_function_call.parameters, expected_function_call.parameters);
    }
}