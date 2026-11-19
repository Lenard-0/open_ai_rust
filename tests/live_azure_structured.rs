//! Live structured-output check against an Azure OpenAI v1 endpoint (reasoning models such as
//! GPT-6.x Sol). Ignored by default.
//!
//! Needs:
//!   AZURE_OPENAI_API_KEY         — sent as `Authorization: Bearer`; never printed
//!   AZURE_OPENAI_ENDPOINT        — e.g. https://my-resource.openai.azure.com
//!   AZURE_OPENAI_CHAT_DEPLOYMENT — deployment name, sent as `model`
//!
//! Run: `cargo test --test live_azure_structured -- --ignored --nocapture`

use open_ai_rust::{
    ChatMessage, Client, FunctionCall, FunctionParameter, FunctionType, OpenAiModel,
    PayLoadBuilder, ReasoningEffort, ResponseFormat,
};
use serde::Deserialize;

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} not set"))
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Quiz {
    title: String,
    questions: Vec<Question>,
    notes: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Question {
    prompt: String,
    answer: String,
}

fn quiz_fn() -> FunctionCall {
    FunctionCall {
        name: "quiz".into(),
        description: Some("A short quiz".into()),
        parameters: vec![
            FunctionParameter::new("title", FunctionType::String),
            FunctionParameter::new(
                "questions",
                FunctionType::Array(Box::new(FunctionType::Object(vec![
                    FunctionParameter::new("prompt", FunctionType::String),
                    FunctionParameter::new("answer", FunctionType::String),
                ]))),
            ),
            FunctionParameter::new(
                "notes",
                FunctionType::Option(Box::new(FunctionType::String)),
            ),
        ],
    }
}

#[tokio::test]
#[ignore = "hits a real Azure OpenAI deployment"]
async fn live_azure_structured_output() {
    dotenv::dotenv().ok();
    let endpoint = env("AZURE_OPENAI_ENDPOINT");
    let deployment = env("AZURE_OPENAI_CHAT_DEPLOYMENT");
    let client = Client::builder()
        .api_key(env("AZURE_OPENAI_API_KEY"))
        .base_url(format!("{}/openai/v1", endpoint.trim_end_matches('/')))
        .build()
        .expect("client");

    let payload = PayLoadBuilder::new(OpenAiModel::Custom(deployment))
        .messages(vec![
            ChatMessage::system("You write short quizzes."),
            ChatMessage::user("Write a 2-question quiz about Australian capitals."),
        ])
        .response_format(ResponseFormat::from_function(&quiz_fn()).unwrap())
        .reasoning_effort(ReasoningEffort::Low)
        .max_completion_tokens(4000)
        .build();

    let resp = client.chat().create(payload).await.expect("request failed");
    let quiz: Quiz = resp.parse_structured().expect("structured output");
    assert!(!quiz.title.is_empty());
    assert!(!quiz.questions.is_empty());
    assert!(quiz
        .questions
        .iter()
        .all(|q| !q.prompt.is_empty() && !q.answer.is_empty()));
    let _ = quiz.notes;
    println!(
        "ok: {} questions; usage: {:?}",
        quiz.questions.len(),
        resp.usage.completion_tokens_details
    );
}
