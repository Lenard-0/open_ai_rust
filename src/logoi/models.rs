use std::fmt::Display;


pub enum OpenAiModel {
    GPT,
    GPT2,
    GPT3,
    GPT4,
    GPT4o
}

impl Display for OpenAiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpenAiModel::GPT => write!(f, "GPT"),
            OpenAiModel::GPT2 => write!(f, "GPT2"),
            OpenAiModel::GPT3 => write!(f, "GPT3"),
            OpenAiModel::GPT4 => write!(f, "GPT4"),
            OpenAiModel::GPT4o => write!(f, "GPT4o"),
        }
    }
}