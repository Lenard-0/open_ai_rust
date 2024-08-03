
#[cfg(test)]
mod tests {
    use open_ai_rust::logoi::input::tool::{FunctionCall, FunctionParameter, FunctionType};
    use serde_json::json;

    #[test]
    fn can_correctly_parse_function_definition_with_bool_parameter() {
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
    fn can_correctly_parse_function_definition_with_string_parameter() {
        let function_def = FunctionCall {
            name: "set_name".to_string(),
            description: Some("Sets the name.".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "name".to_string(),
                    _type: FunctionType::String,
                    description: Some("The name to set.".to_string()),
                }
            ]
        };

        let function_def_json = serde_json::to_value(&function_def).unwrap();
        assert_eq!(function_def_json, json!({
            "description": "Sets the name.",
            "name": "set_name",
            "parameters": {
                "properties": {
                    "name": {
                        "description": "The name to set.",
                        "type": "string",
                    },
                },
                "required": [
                    "name",
                ],
                "type": "object",
            },
        }));
    }

    #[test]
    fn can_correctly_parse_function_definition_with_number_parameter() {
        let function_def = FunctionCall {
            name: "set_age".to_string(),
            description: Some("Sets the age.".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "age".to_string(),
                    _type: FunctionType::Number,
                    description: Some("The age to set.".to_string()),
                }
            ]
        };

        let function_def_json = serde_json::to_value(&function_def).unwrap();
        assert_eq!(function_def_json, json!({
            "description": "Sets the age.",
            "name": "set_age",
            "parameters": {
                "properties": {
                    "age": {
                        "description": "The age to set.",
                        "type": "number",
                    },
                },
                "required": [
                    "age",
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

    #[test]
    fn can_correctly_parse_function_definition_with_array_parameter() {
        let function_def = FunctionCall {
            name: "set_tags".to_string(),
            description: Some("Sets multiple tags.".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "tags".to_string(),
                    _type: FunctionType::Array,
                    description: Some("The tags to set.".to_string()),
                }
            ]
        };

        let function_def_json = serde_json::to_value(&function_def).unwrap();
        assert_eq!(function_def_json, json!({
            "description": "Sets multiple tags.",
            "name": "set_tags",
            "parameters": {
                "properties": {
                    "tags": {
                        "description": "The tags to set.",
                        "type": "array",
                        "items": {
                            "type": "string",
                        }
                    },
                },
                "required": [
                    "tags",
                ],
                "type": "object",
            },
        }));
    }

    #[test]
    fn can_correctly_parse_function_definition_with_object_parameter() {
        let function_def = FunctionCall {
            name: "update_profile".to_string(),
            description: Some("Updates the user profile.".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "profile".to_string(),
                    _type: FunctionType::Object(vec![
                        FunctionParameter {
                            name: "username".to_string(),
                            _type: FunctionType::String,
                            description: Some("The username.".to_string()),
                        },
                        FunctionParameter {
                            name: "age".to_string(),
                            _type: FunctionType::Number,
                            description: Some("The age of the user.".to_string()),
                        }
                    ]),
                    description: Some("The profile details.".to_string()),
                }
            ]
        };

        let function_def_json = serde_json::to_value(&function_def).unwrap();
        assert_eq!(function_def_json, json!({
            "description": "Updates the user profile.",
            "name": "update_profile",
            "parameters": {
                "properties": {
                    "profile": {
                        "description": "The profile details.",
                        "type": "object",
                        "properties": {
                            "username": {
                                "description": "The username.",
                                "type": "string",
                            },
                            "age": {
                                "description": "The age of the user.",
                                "type": "number",
                            }
                        },
                        "required": [
                            "username",
                            "age",
                        ],
                    },
                },
                "required": [
                    "profile",
                ],
                "type": "object",
            },
        }));
    }

    #[test]
    fn can_correctly_parse_function_definition_with_option_parameter() {
        let function_def = FunctionCall {
            name: "set_optional".to_string(),
            description: Some("Sets an optional value.".to_string()),
            parameters: vec![
                FunctionParameter {
                    name: "value".to_string(),
                    _type: FunctionType::Option(Box::new(FunctionType::String)),
                    description: Some("An optional string value.".to_string()),
                }
            ]
        };

        let function_def_json = serde_json::to_value(&function_def).unwrap();
        assert_eq!(function_def_json, json!({
            "description": "Sets an optional value.",
            "name": "set_optional",
            "parameters": {
                "properties": {
                    "value": {
                        "description": "An optional string value.",
                        "type": "string",
                    },
                },
                "required": [],
                "type": "object",
            },
        }));
    }
}