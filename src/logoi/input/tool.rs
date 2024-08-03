use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<FunctionParameter>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub _type: FunctionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum FunctionType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Null,
    Enum(Vec<String>),
}