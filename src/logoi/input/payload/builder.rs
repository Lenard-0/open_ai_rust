use crate::logoi::{input::tool::{ToolChoice, ToolType}, message::ChatMessage};
use super::{FunctionCall, OpenAiModel, ResponseFormatInput, ChatPayLoad};

pub struct PayLoadBuilder {
    model: OpenAiModel,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<ToolChoice>>,
    tool_choice: Option<String>,
    frequency_penalty: Option<f32>,
    logprobs: Option<bool>,
    top_logprobs: Option<i32>,
    max_tokens: Option<i32>,
    n: Option<i32>,
    presence_penalty: Option<f32>,
    response_format: Option<ResponseFormatInput>,
    seed: Option<i32>,
    service_tier: Option<String>,
    stop: Option<Vec<String>>,
    stream: Option<bool>,
    stream_options: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    user: Option<String>,
}

impl PayLoadBuilder {
    pub fn new(model: OpenAiModel) -> Self {
        PayLoadBuilder {
            model,
            messages: Vec::new(),
            tools: None,
            tool_choice: None,
            frequency_penalty: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            n: None,
            presence_penalty: None,
            response_format: None,
            seed: None,
            service_tier: None,
            stop: None,
            stream: None,
            stream_options: None,
            temperature: None,
            top_p: None,
            user: None,
        }
    }

    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages = messages;
        self
    }

    pub fn tools(mut self, tools: Vec<FunctionCall>) -> Self {
        let mut formatted_tools = vec![];
        for t in tools {
            formatted_tools.push(ToolChoice {
                function: t,
                _type: ToolType::Function
            });
        }
        self.tools = Some(formatted_tools);
        self
    }

    pub fn tool_choice(mut self, tool_choice: String) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    pub fn logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    pub fn top_logprobs(mut self, top_logprobs: i32) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }

    pub fn max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn n(mut self, n: i32) -> Self {
        self.n = Some(n);
        self
    }

    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    pub fn response_format(mut self, response_format: ResponseFormatInput) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn service_tier(mut self, service_tier: String) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn stream_options(mut self, stream_options: bool) -> Self {
        self.stream_options = Some(stream_options);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn build(self) -> ChatPayLoad {
        ChatPayLoad {
            model: self.model,
            messages: self.messages.iter().map(|message| {
                ChatMessage {
                    content: message.content.clone(),
                    role: message.role.clone(),
                    name: message.name.clone(),
                }
            }).collect(),
            tools: self.tools,
            tool_choice: self.tool_choice,
            frequency_penalty: self.frequency_penalty,
            logprobs: self.logprobs,
            top_logprobs: self.top_logprobs,
            max_tokens: self.max_tokens,
            n: self.n,
            presence_penalty: self.presence_penalty,
            response_format: self.response_format,
            seed: self.seed,
            service_tier: self.service_tier,
            stop: self.stop,
            stream: self.stream,
            stream_options: self.stream_options,
            temperature: self.temperature,
            top_p: self.top_p,
            user: self.user,
        }
    }
}
