

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use open_ai_rust::logoi::input::tool::{raw_macro::struct_macro::struct_to_fn_call, FunctionCall, FunctionParameter, FunctionType};
    use open_ai_rust_fn_call_extension::FunctionCall;
    use open_ai_rust::logoi::input::tool::raw_macro::FunctionCallable;

    #[test]
    pub fn can_parse_basic_struct_just_name() {
        #[derive(FunctionCall)]
        struct _JustName {}
        impl _JustName { fn new() -> Self { _JustName {  }} }

        let expected_fn_call = FunctionCall {
            name: "_JustName".to_string(),
            description: None,
            parameters: vec![]
        };

        assert_eq!(_JustName::new().to_fn_call(), expected_fn_call);
    }

    #[test]
    pub fn can_parse_simple_struct_primitive_types() {
        #[derive(FunctionCall)]
        struct CreateNpc {
            // name: String,
            // male: bool,
            // age: i32
        }

        impl CreateNpc {
            // pub fn new() -> Self { Self { name: String::new(), }} //male: true, age: 0 } }
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
        // let fn_call = CreateNpc::new().to_fn_call();//struct_to_fn_call().unwrap();

        // println!("{:#?}", fn_call);
        // assert_eq!(fn_call, expected_fn_call);
    }

    // #[test]
    // pub fn can_parse_struct_with_vec_of_primitive() {
    //     #[derive(FunctionCallable)]
    //     struct MakeNotes {
    //         heading: String,
    //         notes: Vec<String>,
    //         difficulty: u8
    //     }

    //     let expected_fn_call = FunctionCall {
    //         name: "make_notes".to_string(),
    //         description: None,
    //         parameters: vec![
    //             FunctionParameter {
    //                 name: "heading".to_string(),
    //                 _type: FunctionType::String,
    //                 description: None
    //             },
    //             FunctionParameter {
    //                 name: "notes".to_string(),
    //                 _type: FunctionType::Array(Box::new(FunctionType::String)),
    //                 description: None
    //             },
    //             FunctionParameter {
    //                 name: "difficulty".to_string(),
    //                 _type: FunctionType::Number,
    //                 description: None
    //             },
    //         ]
    //     };

    //     // println!("{}", CREATENPC);
    //     let fn_call = struct_to_fn_call(MAKENOTES).unwrap();

    //     println!("{:#?}", fn_call);
    //     assert_eq!(fn_call, expected_fn_call);
    // }

    // // #[test]
    // // fn test_parse_struct_w_hashmap() {
    // //     #[derive(FunctionCallType)]
    // //     struct _HashMapStruct {
    // //         obj: HashMap<String, String>
    // //     }

    // //     let fn_call = struct_to_fn_call(_HASHMAPSTRUCT).unwrap();
    // //     let expected_fn_call = FunctionCall {
    // //         name: "make_notes".to_string(),
    // //         description: None,
    // //         parameters: vec![
    // //             FunctionParameter {
    // //                 name: "obj".to_string(),
    // //                 _type: FunctionType::Object(()),
    // //                 description: None
    // //             },
    // //             FunctionParameter {
    // //                 name: "notes".to_string(),
    // //                 _type: FunctionType::Array(Box::new(FunctionType::String)),
    // //                 description: None
    // //             },
    // //             FunctionParameter {
    // //                 name: "difficulty".to_string(),
    // //                 _type: FunctionType::Number,
    // //                 description: None
    // //             },
    // //         ]
    // //     };
    // //     println!("HASHMAPSTRUCT {:?}", _HASHMAPSTRUCT);
    // //     assert_eq!(_HASHMAPSTRUCT, expected_output);
    // // }

    // // #[test]
    // // fn test_parse_struct_w_vec_wrapping_hashmap() {
    // //     #[derive(FunctionCallType)]
    // //     struct _VecHashMapStruct {
    // //         objs: Vec<HashMap<String, String>>
    // //     }

    // //     // Define the expected output
    // //     let expected_output = r#"_VecHashMapStruct { objs : Vec < HashMap < String, String > >"#;

    // //     // Assert that the parsed code matches the expected output
    // //     println!("HASHMAPSTRUCT {:?}", _VECHASHMAPSTRUCT);
    // //     assert_eq!(_VECHASHMAPSTRUCT, expected_output);
    // // }
}