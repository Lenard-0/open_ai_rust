use std::collections::HashMap;

use crate::logoi::{
    input::tool::{ToolChoice, ToolType},
    message::ChatMessage,
};

use super::{
    ChatPayLoad, ChatToolChoice, FunctionCall, OpenAiModel, ReasoningEffort, ResponseFormat,
    StreamOptions,
};

pub struct PayLoadBuilder {
    model: OpenAiModel,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<ToolChoice>>,
    tool_choice: Option<ChatToolChoice>,
    parallel_tool_calls: Option<bool>,
    frequency_penalty: Option<f32>,
    logit_bias: Option<HashMap<String, i32>>,
    logprobs: Option<bool>,
    top_logprobs: Option<i32>,
    max_tokens: Option<i32>,
    max_completion_tokens: Option<i32>,
    reasoning_effort: Option<ReasoningEffort>,
    n: Option<i32>,
    presence_penalty: Option<f32>,
    response_format: Option<ResponseFormat>,
    seed: Option<i32>,
    service_tier: Option<String>,
    stop: Option<Vec<String>>,
    stream: Option<bool>,
    stream_options: Option<StreamOptions>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    metadata: Option<HashMap<String, String>>,
    store: Option<bool>,
    user: Option<String>,
}

impl PayLoadBuilder {
    /// Begin a chat-completion payload for the given model.
    ///
    /// ```
    /// use open_ai_rust::{ChatMessage, OpenAiModel, PayLoadBuilder};
    /// let payload = PayLoadBuilder::new(OpenAiModel::GPT4oMini)
    ///     .messages(vec![ChatMessage::user("hi")])
    ///     .temperature(0.2)
    ///     .seed(7)
    ///     .build();
    /// assert_eq!(payload.temperature, Some(0.2));
    /// assert_eq!(payload.seed, Some(7));
    /// ```
    pub fn new(model: OpenAiModel) -> Self {
        PayLoadBuilder {
            model,
            messages: Vec::new(),
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            max_completion_tokens: None,
            reasoning_effort: None,
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
            metadata: None,
            store: None,
            user: None,
        }
    }

    /// Replace the full message list. Order matters — earlier messages are earlier in the
    /// conversation context.
    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages = messages;
        self
    }

    /// Push a single message onto the end of the conversation.
    pub fn add_message(mut self, message: ChatMessage) -> Self {
        self.messages.push(message);
        self
    }

    /// Set the function tools available for the model to call. Each [`FunctionCall`] is wrapped
    /// in a `{"type": "function", "function": ...}` envelope automatically.
    pub fn tools(mut self, tools: Vec<FunctionCall>) -> Self {
        let formatted: Vec<ToolChoice> = tools
            .into_iter()
            .map(|t| ToolChoice {
                function: t,
                _type: ToolType::Function,
            })
            .collect();
        self.tools = Some(formatted);
        self
    }

    /// Constrain how the model selects tools. Accepts the `"auto"` / `"none"` / `"required"`
    /// presets or a [`ChatToolChoiceFunction`](crate::ChatToolChoiceFunction) for a specific tool.
    pub fn tool_choice(mut self, tool_choice: impl Into<ChatToolChoice>) -> Self {
        self.tool_choice = Some(tool_choice.into());
        self
    }

    /// Enable / disable parallel tool calls (`parallel_tool_calls`). Default at the API is `true`.
    pub fn parallel_tool_calls(mut self, v: bool) -> Self {
        self.parallel_tool_calls = Some(v);
        self
    }

    /// Penalise tokens by how often they have already appeared. Valid range `-2.0..=2.0`.
    /// Positive values reduce repetition.
    pub fn frequency_penalty(mut self, v: f32) -> Self {
        self.frequency_penalty = Some(v);
        self
    }

    /// Replace the entire `logit_bias` map.
    pub fn logit_bias(mut self, map: HashMap<String, i32>) -> Self {
        self.logit_bias = Some(map);
        self
    }

    /// Insert a single `(token_id, bias)` entry into the `logit_bias` map.
    /// Bias must be in `-100..=100`.
    pub fn logit_bias_entry(mut self, token_id: impl Into<String>, bias: i32) -> Self {
        self.logit_bias
            .get_or_insert_with(HashMap::new)
            .insert(token_id.into(), bias);
        self
    }

    /// Whether the response should include log-probabilities for each output token.
    pub fn logprobs(mut self, v: bool) -> Self {
        self.logprobs = Some(v);
        self
    }

    /// Number of top alternative tokens to return per position when `logprobs` is enabled.
    /// Valid range `0..=20`. Requires `.logprobs(true)`.
    pub fn top_logprobs(mut self, v: i32) -> Self {
        self.top_logprobs = Some(v);
        self
    }

    /// Maximum tokens generated in the completion. Legacy field — newer models prefer
    /// [`Self::max_completion_tokens`]. Some recent models reject `max_tokens` entirely.
    pub fn max_tokens(mut self, v: i32) -> Self {
        self.max_tokens = Some(v);
        self
    }

    /// Maximum tokens generated in the completion, including reasoning tokens for `o*` models.
    /// Preferred over [`Self::max_tokens`] for GPT-4o / GPT-4.1 / o-series / GPT-5.
    pub fn max_completion_tokens(mut self, v: i32) -> Self {
        self.max_completion_tokens = Some(v);
        self
    }

    /// Effort level for reasoning models (`o1`, `o3`, `o4-mini`, GPT-5 family). Higher levels
    /// produce more reasoning tokens before the final answer.
    pub fn reasoning_effort(mut self, v: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(v);
        self
    }

    /// Number of completions to generate per prompt. Defaults to `1`. Billing scales linearly.
    pub fn n(mut self, v: i32) -> Self {
        self.n = Some(v);
        self
    }

    /// Penalise tokens that have already appeared at all (regardless of frequency). Valid range
    /// `-2.0..=2.0`. Positive values encourage new topics.
    pub fn presence_penalty(mut self, v: f32) -> Self {
        self.presence_penalty = Some(v);
        self
    }

    /// Force the model's output into a specific format. Use [`ResponseFormat::JsonObject`]
    /// for free-form JSON or [`ResponseFormat::json_schema`] for strict-schema JSON.
    pub fn response_format(mut self, v: ResponseFormat) -> Self {
        self.response_format = Some(v);
        self
    }

    /// Best-effort determinism seed. Two calls with the same `seed` + payload *may* produce
    /// identical outputs; not guaranteed across model versions.
    pub fn seed(mut self, v: i32) -> Self {
        self.seed = Some(v);
        self
    }

    /// Service tier for the request, e.g. `"auto"`, `"default"`, `"flex"`. Enterprise feature.
    pub fn service_tier(mut self, v: impl Into<String>) -> Self {
        self.service_tier = Some(v.into());
        self
    }

    /// Up to four strings that, if generated, stop the completion. The stop strings are not
    /// included in the output.
    pub fn stop(mut self, v: Vec<String>) -> Self {
        self.stop = Some(v);
        self
    }

    /// Manually toggle SSE streaming. [`Client::chat().create_stream`](crate::Client::chat)
    /// sets this for you — only call this if you need to drive serialisation yourself.
    pub fn stream(mut self, v: bool) -> Self {
        self.stream = Some(v);
        self
    }

    /// Replace the entire `stream_options` object. Prefer [`Self::include_usage`] for the common
    /// case of just toggling `include_usage`.
    pub fn stream_options(mut self, v: StreamOptions) -> Self {
        self.stream_options = Some(v);
        self
    }

    /// Convenience: set `stream_options.include_usage` — emit a final usage chunk at the end of
    /// the stream so token counts are available without a follow-up call.
    pub fn include_usage(mut self, v: bool) -> Self {
        self.stream_options = Some(StreamOptions {
            include_usage: Some(v),
        });
        self
    }

    /// Sampling temperature in `0.0..=2.0`. Higher = more random. Default at the API is `1.0`.
    /// Generally set either `temperature` or [`Self::top_p`], not both.
    pub fn temperature(mut self, v: f32) -> Self {
        self.temperature = Some(v);
        self
    }

    /// Nucleus sampling cutoff in `0.0..=1.0`. Lower = more conservative.
    /// Generally set either [`Self::temperature`] or `top_p`, not both.
    pub fn top_p(mut self, v: f32) -> Self {
        self.top_p = Some(v);
        self
    }

    /// Replace the entire metadata map. Use [`Self::metadata_entry`] to add a single entry.
    /// Up to 16 key-value pairs, each 64-char key / 512-char value.
    pub fn metadata(mut self, v: HashMap<String, String>) -> Self {
        self.metadata = Some(v);
        self
    }

    /// Insert a single key into the metadata map (creating it if absent).
    pub fn metadata_entry(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.metadata
            .get_or_insert_with(HashMap::new)
            .insert(k.into(), v.into());
        self
    }

    /// Whether to persist this completion to your OpenAI Storage tenant (for later retrieval).
    pub fn store(mut self, v: bool) -> Self {
        self.store = Some(v);
        self
    }

    /// End-user identifier for abuse-tracking. Opaque to OpenAI; not sent back in the response.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Finalise the builder into a [`ChatPayLoad`] ready to send.
    pub fn build(self) -> ChatPayLoad {
        ChatPayLoad {
            model: self.model,
            messages: self.messages,
            tools: self.tools,
            tool_choice: self.tool_choice,
            parallel_tool_calls: self.parallel_tool_calls,
            frequency_penalty: self.frequency_penalty,
            logit_bias: self.logit_bias,
            logprobs: self.logprobs,
            top_logprobs: self.top_logprobs,
            max_tokens: self.max_tokens,
            max_completion_tokens: self.max_completion_tokens,
            reasoning_effort: self.reasoning_effort,
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
            metadata: self.metadata,
            store: self.store,
            user: self.user,
        }
    }
}
