
#[cfg(test)]
mod tests {
    use open_ai_rust::{logoi::{input::{payload::builder::PayLoadBuilder, tool::{FunctionCall, FunctionParameter, FunctionType}}, message::{ChatMessage, ChatMessageRole}, models::OpenAiModel}, requests::open_ai_msg};



    #[tokio::test]
    async fn can_tool_call_using_azure() {
        dotenv::dotenv().ok();

        open_ai_rust::set_key(std::env::var("AZURE_AI_SK").unwrap());
        open_ai_rust::set_ai_msg_endpoint(std::env::var("AZURE_AI_ENDPOINT").unwrap());

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
        let tool_call = response.get_first_tool_call_args().unwrap();
        println!("Tool call: {:#?}", tool_call);
    }

    #[tokio::test]
    async fn can_tool_call_using_azure_complex() {
        dotenv::dotenv().ok();

        open_ai_rust::set_key(std::env::var("AZURE_AI_SK").unwrap());
        open_ai_rust::set_ai_msg_endpoint(std::env::var("AZURE_AI_ENDPOINT").unwrap());

        let system_msg = ChatMessage {
            role: ChatMessageRole::System,
            content: "You are an advanced Tutor AI bot.
Create a key concept from the last minute of the following transcript.
The key concept should be concise and cover the main idea.
A key concept should summarise, condense, & extract the main idea from the last minute of the lecture.
You will be also given additional context. Do not summarise this content, this is supporting context for you to
better understand the main idea.
The key concept should not be a matter-of-fact statement, but a summarised version of the content.
The heading should be 2 - 5 words in length.
1 sentences MAX.
Explain it in simple terms so that a 15 year old can understand it. More aligned with a definition.
Never repeat the transcript verbatim. You must summarise the content.
You must NEVER respond with the heading, or the verbatim transcript. If you are confused because you lack context or something, please tell the user this.
You can never give a response that is not summarising a key concept. Summarisation IS your job

YOU MUST RETURN A TOOL CALL USING THE PROVIDED TOOL CALL SCHEMA
Even if the user prompt is not clear, empty, broken etc you must return a tool call using the provided schema else everything will break.
In this case do not hallucinate but explain to the user that you are confused and need more context.
".to_string(),
            name: None
        };

        let user_msg = ChatMessage {
            role: ChatMessageRole::User,
            content: "".to_string(),
            name: None
        };

        let functions = vec![
            FunctionCall {
                name: "CreateKeyConcept".to_string(),
                description: None,
                parameters: vec![
                    FunctionParameter {
                        name: "heading".to_string(),
                        _type: FunctionType::String,
                        description: None,
                    },
                    FunctionParameter {
                        name: "content".to_string(),
                        _type: FunctionType::String,
                        description: None,
                    },
                ],
            }
        ];

        let payload = PayLoadBuilder::new(OpenAiModel::GPT4o)
            .messages(vec![system_msg, user_msg])
            .tools(functions)
            .seed(0)
            .build();

        let response = open_ai_msg(payload).await.unwrap();
        let tool_call = response.get_first_tool_call_args().unwrap();
        println!("Tool call: {:#?}", tool_call);
    }

}