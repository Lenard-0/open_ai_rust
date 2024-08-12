use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::message::ChatMessage;


    //     "choices": Array [
    //         Object {
    //             "finish_reason": String("stop"),
    //             "index": Number(0),
    //             "logprobs": Null,
    //             "message": Object {
    //                 "content": Null,
    //                 "role": String("assistant"),
    //                 "tool_calls": Array [
    //                     Object {
    //                         "function": Object {
    //                             "arguments": String("{\"questions\":[{\"question\":\"Which of the following is true about the formatting style used in lecture notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Headings are used to organize content, followed by dot points for details.\",\"correct\":true},{\"answer\":\"Dot points mark major topics, followed by additional details.\",\"correct\":false},{\"answer\":\"Excerpts are used to enumerate the primary points.\",\"correct\":false},{\"answer\":\"Headings contain all content without any additional formatting.\",\"correct\":false}],\"answer_explanation\":\"Headings are used to organize the main topics of the lecture notes, while dot points are used to provide more detailed information related to those topics.\"}},{\"question\":\"What is the role of excerpts provided in the lecture notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Excerpts summarize the entire lecture.\",\"correct\":false},{\"answer\":\"Excerpts provide verbatim transcript sections for context.\",\"correct\":true},{\"answer\":\"Excerpts highlight the main points of each section.\",\"correct\":false},{\"answer\":\"Excerpts are used to list secondary details.\",\"correct\":false}],\"answer_explanation\":\"Excerpts in the lecture notes consist of verbatim sections from transcripts to give context and support understanding of the topic.\"}},{\"question\":\"How are main ideas emphasized in the lecture notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Main ideas are summarized in a paragraph at the end of the notes.\",\"correct\":false},{\"answer\":\"Main ideas are written in headings to distinguish them from supporting details.\",\"correct\":true},{\"answer\":\"Main ideas are italicized within the dot points.\",\"correct\":false},{\"answer\":\"Main ideas are highlighted within excerpts.\",\"correct\":false}],\"answer_explanation\":\"Main ideas are emphasized through the use of headings, which differentiate them from the supporting details listed beneath as dot points.\"}},{\"question\":\"In the provided formatting, what is the primary function of the dot points?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"To summarize the main points at the end of the lecture.\",\"correct\":false},{\"answer\":\"To elaborate on the content introduced in the headings.\",\"correct\":true},{\"answer\":\"To provide unrelated additional information.\",\"correct\":false},{\"answer\":\"To repeat the information given in the excerpts.\",\"correct\":false}],\"answer_explanation\":\"Dot points serve to elaborate on and provide additional details for the content introduced by the headings.\"}},{\"question\":\"In what way are the excerpts from transcripts useful?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"They list all the important points concisely.\",\"correct\":false},{\"answer\":\"They provide a factual summary of lecture content.\",\"correct\":false},{\"answer\":\"They offer context by presenting verbatim sections of the transcript.\",\"correct\":true},{\"answer\":\"They serve as section headers.\",\"correct\":false}],\"answer_explanation\":\"Excerpts are useful because they provide context and specific examples by including verbatim sections of the transcript.\"}},{\"question\":\"Which method is NOT used for organizing content in the notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Dot points for detailed information.\",\"correct\":false},{\"answer\":\"Headings for main topics.\",\"correct\":false},{\"answer\":\"Excerpts for verbatim transcript sections.\",\"correct\":false},{\"answer\":\"Highlighting key information in red text.\",\"correct\":true}],\"answer_explanation\":\"Highlighting key information in red text is not mentioned as a method for organizing the content in the provided notes.\"}},{\"question\":\"What distinguishes the 'Headings' and 'Dot points' sections in the lecture notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Headings include verbatim extracts from transcripts, dot points are summaries of these extracts.\",\"correct\":false},{\"answer\":\"Headings present the main topics and ideas, dot points list supporting details.\",\"correct\":true},{\"answer\":\"Headings summarize the lecture, dot points repeat the same information.\",\"correct\":false},{\"answer\":\"Headings and dot points both serve to summarize the main ideas.\",\"correct\":false}],\"answer_explanation\":\"Headings present the main topics and ideas of the lecture, while dot points are used to list supporting details and elaborate on the main points.\"}},{\"question\":\"What is the purpose of using 'Headings' in the lecture notes?\",\"short_answer\":{\"answer_explanation\":\"The purpose of using 'Headings' is to organize and clearly label the main topics and ideas, making it easier for students to navigate and understand the content.\"}},{\"question\":\"Explain the significance of 'Excerpts' in the context of lecture notes.\",\"short_answer\":{\"answer_explanation\":\"Excerpts provide context and examples by including verbatim sections of the transcript, which helps in illustrating and reinforcing the main points.\"}},{\"question\":\"Discuss how 'Dot points' structure contributes to the overall understanding of the lecture material.\",\"short_answer\":{\"answer_explanation\":\"Dot points structure contributes by breaking down detailed information into manageable and clear points, elaborating on the main ideas introduced by headings, and aiding in the retention and comprehension of the material.\"}}]}"),
    //                             "name": String("create_quiz_questions"),
    //                         },
    //                         "id": String("call_K1tUuxI0TJCD3yIT1iky7yDm"),
    //                         "type": String("function"),
    //                     },
    //                 ],
    //             },
    //         },
    //     ],
    //     "created": Number(1722508789),
    //     "id": String("chatcmpl-9rNVlNhlmKsuEvmN7j5vbGvi58YSr"),
    //     "model": String("gpt-4o-2024-05-13"),
    //     "object": String("chat.completion"),
    //     "system_fingerprint": String("fp_4e2b2da518"),
    //     "usage": Object {
    //         "completion_tokens": Number(1006),
    //         "prompt_tokens": Number(956),
    //         "total_tokens": Number(1962),
    //     },
    // }
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AiMsgResponse {
    pub choices: Vec<Choice>,
    pub created: i64,
    pub id: String,
    pub model: String,
    pub object: String,
    pub usage: Usage,
    pub system_fingerprint: String,
}

impl AiMsgResponse {
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        self.choices.iter().map(|choice| {
            ChatMessage {
                content: choice.message.content.clone().unwrap_or("".to_string()),
                role: choice.message.role.clone().into(),
                name: None
            }
        }).collect()
    }

    pub fn get_last_msg_text(&self) -> Option<String> {
        match self.choices.last() {
            Some(msg) => msg.message.content.clone(),
            None => None
        }
    }

    pub fn get_tool_calls(&self) -> Vec<FunctionCallRes> {
        self.choices.iter().map(|choice| {
            choice.message.tool_calls.iter().map(|tool_call| {
                tool_call.function.clone()
            }).collect::<Vec<FunctionCallRes>>()
        }).flatten().collect()
    }

    pub fn get_tool_call_args(&self) -> Result<Value, String> {
        match self.choices.first() {
            Some(choice) => match choice.message.tool_calls.first() {
                Some(tool_call) => Ok(tool_call.get_args()),
                None => return Err("No tool calls in first choice of in get tool call json".to_string())
            },
            None => return Err("No choices in get tool call json".to_string())
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Choice {
    pub finish_reason: String,
    pub index: i32,
    pub message: AiResponseMessage,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Usage {
    pub completion_tokens: i32,
    pub prompt_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AiResponseMessage {
    pub content: Option<String>,
    pub role: String,
    pub tool_calls: Vec<ToolCallRes>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ToolCallRes {
    pub function: FunctionCallRes,
}

impl ToolCallRes {
    pub fn get_args(&self) -> Value {
        return json!(self.function.arguments)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionCallRes {
    pub name: String,
    #[serde(deserialize_with = "deserialize_json_string")]
    pub arguments: Value,
}

fn deserialize_json_string<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

