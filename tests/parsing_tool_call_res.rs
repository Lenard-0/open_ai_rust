
#[cfg(test)]
mod tests {
    use open_ai_rust::logoi::output::AiMsgResponse;
    use serde_json::{json, Value};

    #[test]
    fn can_parse_complex_function_call_res() {
        let json_res = json!({
            "choices": [
                {
                    "finish_reason": "stop",
                    "index": 0,
                    "logprobs": Value::Null,
                    "message": {
                        "content": Value::Null,
                        "role": "assistant",
                        "tool_calls": [
                            {
                                "function": {
                                    "arguments": "{\"questions\":[{\"question\":\"Which of the following is true about the formatting style used in lecture notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Headings are used to organize content, followed by dot points for details.\",\"correct\":true},{\"answer\":\"Dot points mark major topics, followed by additional details.\",\"correct\":false},{\"answer\":\"Excerpts are used to enumerate the primary points.\",\"correct\":false},{\"answer\":\"Headings contain all content without any additional formatting.\",\"correct\":false}],\"answer_explanation\":\"Headings are used to organize the main topics of the lecture notes, while dot points are used to provide more detailed information related to those topics.\"}},{\"question\":\"What is the role of excerpts provided in the lecture notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Excerpts summarize the entire lecture.\",\"correct\":false},{\"answer\":\"Excerpts provide verbatim transcript sections for context.\",\"correct\":true},{\"answer\":\"Excerpts highlight the main points of each section.\",\"correct\":false},{\"answer\":\"Excerpts are used to list secondary details.\",\"correct\":false}],\"answer_explanation\":\"Excerpts in the lecture notes consist of verbatim sections from transcripts to give context and support understanding of the topic.\"}},{\"question\":\"How are main ideas emphasized in the lecture notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Main ideas are summarized in a paragraph at the end of the notes.\",\"correct\":false},{\"answer\":\"Main ideas are written in headings to distinguish them from supporting details.\",\"correct\":true},{\"answer\":\"Main ideas are italicized within the dot points.\",\"correct\":false},{\"answer\":\"Main ideas are highlighted within excerpts.\",\"correct\":false}],\"answer_explanation\":\"Main ideas are emphasized through the use of headings, which differentiate them from the supporting details listed beneath as dot points.\"}},{\"question\":\"In the provided formatting, what is the primary function of the dot points?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"To summarize the main points at the end of the lecture.\",\"correct\":false},{\"answer\":\"To elaborate on the content introduced in the headings.\",\"correct\":true},{\"answer\":\"To provide unrelated additional information.\",\"correct\":false},{\"answer\":\"To repeat the information given in the excerpts.\",\"correct\":false}],\"answer_explanation\":\"Dot points serve to elaborate on and provide additional details for the content introduced by the headings.\"}},{\"question\":\"In what way are the excerpts from transcripts useful?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"They list all the important points concisely.\",\"correct\":false},{\"answer\":\"They provide a factual summary of lecture content.\",\"correct\":false},{\"answer\":\"They offer context by presenting verbatim sections of the transcript.\",\"correct\":true},{\"answer\":\"They serve as section headers.\",\"correct\":false}],\"answer_explanation\":\"Excerpts are useful because they provide context and specific examples by including verbatim sections of the transcript.\"}},{\"question\":\"Which method is NOT used for organizing content in the notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Dot points for detailed information.\",\"correct\":false},{\"answer\":\"Headings for main topics.\",\"correct\":false},{\"answer\":\"Excerpts for verbatim transcript sections.\",\"correct\":false},{\"answer\":\"Highlighting key information in red text.\",\"correct\":true}],\"answer_explanation\":\"Highlighting key information in red text is not mentioned as a method for organizing the content in the provided notes.\"}},{\"question\":\"What distinguishes the 'Headings' and 'Dot points' sections in the lecture notes?\",\"multiple_choice\":{\"possible_answers\":[{\"answer\":\"Headings include verbatim extracts from transcripts, dot points are summaries of these extracts.\",\"correct\":false},{\"answer\":\"Headings present the main topics and ideas, dot points list supporting details.\",\"correct\":true},{\"answer\":\"Headings summarize the lecture, dot points repeat the same information.\",\"correct\":false},{\"answer\":\"Headings and dot points both serve to summarize the main ideas.\",\"correct\":false}],\"answer_explanation\":\"Headings present the main topics and ideas of the lecture, while dot points are used to list supporting details and elaborate on the main points.\"}},{\"question\":\"What is the purpose of using 'Headings' in the lecture notes?\",\"short_answer\":{\"answer_explanation\":\"The purpose of using 'Headings' is to organize and clearly label the main topics and ideas, making it easier for students to navigate and understand the content.\"}},{\"question\":\"Explain the significance of 'Excerpts' in the context of lecture notes.\",\"short_answer\":{\"answer_explanation\":\"Excerpts provide context and examples by including verbatim sections of the transcript, which helps in illustrating and reinforcing the main points.\"}},{\"question\":\"Discuss how 'Dot points' structure contributes to the overall understanding of the lecture material.\",\"short_answer\":{\"answer_explanation\":\"Dot points structure contributes by breaking down detailed information into manageable and clear points, elaborating on the main ideas introduced by headings, and aiding in the retention and comprehension of the material.\"}}]}",
                                    "name": "create_questions",
                                },
                                "id": "call_K1tUuxI0TJCD3yIT1iky7yDm",
                                "type": "function",
                            },
                        ],
                    },
                },
            ],
            "created": 1722508789,
            "id": "chatcmpl-9rNVlNhlmKsuEvmN7j5vbGvi58YSr",
            "model": "gpt-4o-2024-05-13",
            "object": "chat.completion",
            "system_fingerprint": "fp_4e2b2da518",
            "usage": {
                "completion_tokens": 1006,
                "prompt_tokens": 956,
                "total_tokens": 1962,
            }
        });

        let res: AiMsgResponse = match serde_json::from_value(json_res) {
            Ok(data) => data,
            Err(e) => {
                println!("Error: {}", e);
                panic!();
            }
        };

        let function_calls = res.get_tool_calls();
        assert_eq!(function_calls.len(), 1);
        let function_call = &function_calls[0];
        assert_eq!(function_call.name, "create_questions");

        let questions = function_call.arguments["questions"].as_array().unwrap();
        println!("questions: {:#?}", questions);
        assert_eq!(questions.len(), 10);

        let first_question = &questions[0];
        println!("first_question: {:#?}", first_question);
        assert_eq!(first_question["question"], "Which of the following is true about the formatting style used in lecture notes?");
        assert_eq!(first_question["multiple_choice"], json!({
            "possible_answers": [
                {"answer": "Headings are used to organize content, followed by dot points for details.", "correct": true},
                {"answer": "Dot points mark major topics, followed by additional details.", "correct": false},
                {"answer": "Excerpts are used to enumerate the primary points.", "correct": false},
                {"answer": "Headings contain all content without any additional formatting.", "correct": false}
            ],
            "answer_explanation": "Headings are used to organize the main topics of the lecture notes, while dot points are used to provide more detailed information related to those topics."
        }));
    }
}

