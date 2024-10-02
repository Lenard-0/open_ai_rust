
#[cfg(test)]
mod tests {
    use open_ai_rust::logoi::output::{AiMsgResponse, AiResponseMessage, Choice, FunctionCallRes, ToolCallRes, Usage};
    use serde::{Deserialize, Serialize};
    use serde_json::json;


    #[test]
    fn can_convert_tool_res_to_json_then_into_struct() {
        let msg_res = AiMsgResponse {
            choices: vec![Choice {
                finish_reason: "".to_string(),
                index: 0,
                message: AiResponseMessage {
                    content: None,
                    role: "".to_string(),
                    tool_calls: Some(vec![ToolCallRes { function: FunctionCallRes {
                        name: "fn_name".to_string(),
                        arguments: json!({
                            "location": "San Francisco, CA",
                        })
                    }}])
                },
                logprobs: None
            }],
            created: 0,
            id: "".to_string(),
            model: "".to_string(),
            object: "".to_string(),
            usage: Usage { completion_tokens: 0, prompt_tokens: 0, total_tokens: 0 },
            system_fingerprint: "".to_string(),
        };

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Location {
            location: String
        }
        let fn_call_args: serde_json::Value = msg_res.get_first_tool_call_args().unwrap();
        let location: Location = serde_json::from_value(fn_call_args).unwrap();
        assert_eq!(location, Location { location: "San Francisco, CA".to_string() })
    }
}