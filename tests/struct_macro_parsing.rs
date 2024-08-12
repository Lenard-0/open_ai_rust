

#[cfg(test)]
mod tests {
    use open_ai_rust::logoi::input::tool::{raw_macro::struct_macro::struct_to_fn_call, FunctionCall, FunctionParameter, FunctionType};
    use open_ai_rust_fn_call_extension::FunctionCallType;

    #[test]
    pub fn can_parse_simple_struct_primitive_types() {
        #[derive(FunctionCallType)]
        struct CreateNpc {
            name: String,
            male: bool,
            age: i64
        }

        let expected_fn_call = FunctionCall {
            name: "create_npc".to_string(),
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

        // println!("{}", CREATENPC);
        let fn_call = struct_to_fn_call(CREATENPC).unwrap();

        // println!("{:#?}", fn_call);
        assert_eq!(fn_call, expected_fn_call);
    }

    #[test]
    pub fn can_parse_struct_with_vec_of_primitive() {
        #[derive(FunctionCallType)]
        struct MakeNotes {
            heading: String,
            notes: Vec<String>,
            difficulty: u8
        }

        let expected_fn_call = FunctionCall {
            name: "make_notes".to_string(),
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

        // println!("{}", CREATENPC);
        let fn_call = struct_to_fn_call(MAKENOTES).unwrap();

        println!("{:#?}", fn_call);
        assert_eq!(fn_call, expected_fn_call);
    }
}