//! End-to-end test: `#[derive(FunctionCall)]` from
//! `open_ai_rust_fn_call_extension` against the canonical
//! `FunctionCallable` trait path.

#[cfg(test)]
mod tests {
    use open_ai_rust::logoi::input::tool::raw_macro::FunctionCallable;
    use open_ai_rust::logoi::input::tool::{FunctionCall, FunctionParameter, FunctionType};
    use open_ai_rust_fn_call_extension::FunctionCall;

    #[test]
    pub fn can_parse_basic_struct_just_name() {
        #[derive(FunctionCall)]
        struct JustName {}

        let expected = FunctionCall {
            name: "JustName".to_string(),
            description: None,
            parameters: vec![],
        };
        assert_eq!(JustName::fn_schema(), expected);
    }

    #[test]
    pub fn can_parse_simple_struct_primitive_types() {
        #[derive(FunctionCall)]
        #[allow(dead_code)]
        struct CreateNpc {
            name: String,
            male: bool,
            age: i32,
        }

        let expected = FunctionCall {
            name: "CreateNpc".to_string(),
            description: None,
            parameters: vec![
                FunctionParameter {
                    name: "name".to_string(),
                    _type: FunctionType::String,
                    description: None,
                    required: true,
                },
                FunctionParameter {
                    name: "male".to_string(),
                    _type: FunctionType::Boolean,
                    description: None,
                    required: true,
                },
                FunctionParameter {
                    name: "age".to_string(),
                    _type: FunctionType::Number,
                    description: None,
                    required: true,
                },
            ],
        };
        assert_eq!(CreateNpc::fn_schema(), expected);
    }

    #[test]
    pub fn can_parse_struct_with_vec_of_primitive() {
        #[derive(FunctionCall)]
        #[allow(dead_code)]
        struct MakeNotes {
            heading: String,
            notes: Vec<String>,
            difficulty: u8,
        }

        let expected = FunctionCall {
            name: "MakeNotes".to_string(),
            description: None,
            parameters: vec![
                FunctionParameter {
                    name: "heading".to_string(),
                    _type: FunctionType::String,
                    description: None,
                    required: true,
                },
                FunctionParameter {
                    name: "notes".to_string(),
                    _type: FunctionType::Array(Box::new(FunctionType::String)),
                    description: None,
                    required: true,
                },
                FunctionParameter {
                    name: "difficulty".to_string(),
                    _type: FunctionType::Number,
                    description: None,
                    required: true,
                },
            ],
        };
        assert_eq!(MakeNotes::fn_schema(), expected);
    }

    #[test]
    fn nested_struct_schema_unwraps_into_object() {
        #[derive(FunctionCall)]
        #[allow(dead_code)]
        struct OuterStruct {
            inner: InnerStruct,
        }
        #[derive(FunctionCall)]
        #[allow(dead_code)]
        struct InnerStruct {
            value: String,
        }

        let expected = FunctionCall {
            name: "OuterStruct".to_string(),
            description: None,
            parameters: vec![FunctionParameter {
                name: "inner".to_string(),
                _type: FunctionType::Object(vec![FunctionParameter {
                    name: "value".to_string(),
                    _type: FunctionType::String,
                    description: None,
                    required: true,
                }]),
                description: None,
                required: true,
            }],
        };
        assert_eq!(OuterStruct::fn_schema(), expected);
    }

    #[test]
    fn vec_of_user_struct_unwraps_into_array_object() {
        #[derive(FunctionCall)]
        #[allow(dead_code)]
        struct OuterStruct {
            inner: Vec<InnerStruct>,
        }
        #[derive(FunctionCall)]
        #[allow(dead_code)]
        struct InnerStruct {
            value: String,
        }

        let expected = FunctionCall {
            name: "OuterStruct".to_string(),
            description: None,
            parameters: vec![FunctionParameter {
                name: "inner".to_string(),
                _type: FunctionType::Array(Box::new(FunctionType::Object(vec![
                    FunctionParameter {
                        name: "value".to_string(),
                        _type: FunctionType::String,
                        description: None,
                        required: true,
                    },
                ]))),
                description: None,
                required: true,
            }],
        };
        assert_eq!(OuterStruct::fn_schema(), expected);
    }
}
