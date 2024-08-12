

#[cfg(test)]
mod tests {
    use open_ai_rust::logoi::input::tool::{FunctionCall, FunctionParameter, FunctionType};
    use open_ai_rust_fn_call_extension::FunctionCall;
    use open_ai_rust::logoi::input::tool::raw_macro::FunctionCallable;

    #[test]
    pub fn can_parse_basic_struct_just_name() {
        #[derive(FunctionCall)]
        struct JustName {}
        impl JustName { fn new() -> Self { JustName {  }} }

        let expected_fn_call = FunctionCall {
            name: "JustName".to_string(),
            description: None,
            parameters: vec![]
        };

        assert_eq!(JustName::new().to_fn_call(), expected_fn_call);
    }

    #[test]
    pub fn can_parse_simple_struct_primitive_types() {
        #[derive(FunctionCall)]
        struct CreateNpc {
            name: String,
            male: bool,
            age: i32
        }

        impl CreateNpc {
            pub fn new() -> Self { Self { name: String::new(), male: true, age: 0 } }
        }

        let expected_fn_call = FunctionCall {
            name: "CreateNpc".to_string(),
            description: None,
            parameters: vec![
                FunctionParameter {
                    name: "name".to_string(),
                    _type: FunctionType::String,
                    description: None
                },
                FunctionParameter {
                    name: "male".to_string(),
                    _type: FunctionType::Boolean,
                    description: None
                },
                FunctionParameter {
                    name: "age".to_string(),
                    _type: FunctionType::Number,
                    description: None
                },
            ]
        };
        let fn_call = CreateNpc::new().to_fn_call();
        assert_eq!(fn_call, expected_fn_call);
    }

    #[test]
    pub fn can_parse_struct_with_vec_of_primitive() {
        #[derive(FunctionCall)]
        struct MakeNotes {
            heading: String,
            notes: Vec<String>,
            difficulty: u8
        }

        impl MakeNotes {
            pub fn for_fn_call() -> Self { Self { heading: "".to_string(), notes: vec!["".to_string()], difficulty: 1 } }
        }

        let expected_fn_call = FunctionCall {
            name: "MakeNotes".to_string(),
            description: None,
            parameters: vec![
                FunctionParameter {
                    name: "heading".to_string(),
                    _type: FunctionType::String,
                    description: None
                },
                FunctionParameter {
                    name: "notes".to_string(),
                    _type: FunctionType::Array(Box::new(FunctionType::String)),
                    description: None
                },
                FunctionParameter {
                    name: "difficulty".to_string(),
                    _type: FunctionType::Number,
                    description: None
                },
            ]
        };

        assert_eq!(MakeNotes::for_fn_call().to_fn_call(), expected_fn_call);
    }

    #[test]
    fn test_parse_struct_w_hashmap() {
        #[derive(FunctionCall)]
        struct OuterStruct {
            inner: InnerStruct
        }
        impl OuterStruct {
            fn new() -> Self { Self { inner: InnerStruct::new() }}
        }
        #[derive(FunctionCall)]
        struct InnerStruct {
            value: String
        }
        impl InnerStruct {
            fn new() -> Self { Self { value: String::new() }}
        }

        let expected_fn_call = FunctionCall {
            name: "OuterStruct".to_string(),
            description: None,
            parameters: vec![
                FunctionParameter {
                    name: "inner".to_string(),
                    _type: FunctionType::Object(vec![FunctionParameter {
                        name: "value".to_string(),
                        _type: FunctionType::String,
                        description: None
                    }]),
                    description: None
                }
            ]
        };
        assert_eq!(OuterStruct::new().to_fn_call(), expected_fn_call);
    }

    #[test]
    fn test_parse_struct_w_vec_wrapping_strict() {
        #[derive(FunctionCall)]
        struct OuterStruct {
            inner: Vec<InnerStruct>
        }
        impl OuterStruct {
            fn for_fn_call() -> Self { Self { inner: vec![InnerStruct::new()] }}
        }
        #[derive(FunctionCall)]
        struct InnerStruct {
            value: String
        }
        impl InnerStruct {
            fn new() -> Self { Self { value: String::new() }}
        }

        let expected_fn_call = FunctionCall {
            name: "OuterStruct".to_string(),
            description: None,
            parameters: vec![
                FunctionParameter {
                    name: "inner".to_string(),
                    _type: FunctionType::Array(Box::new(FunctionType::Object(vec![FunctionParameter {
                        name: "value".to_string(),
                        _type: FunctionType::String,
                        description: None
                    }]))),
                    description: None
                }
            ]
        };
        assert_eq!(OuterStruct::for_fn_call().to_fn_call(), expected_fn_call);
    }
}