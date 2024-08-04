use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Value;



pub mod serialise;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ToolChoice {
    pub function: FunctionCall,
    #[serde(rename = "type")]
    pub _type: ToolType,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Vec<FunctionParameter>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function
}

#[derive(Deserialize, Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub _type: FunctionType,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum FunctionType {
    String,
    Number,
    Boolean,
    Array(Box<FunctionType>),
    Object(Vec<FunctionParameter>),
    Null,
    Enum(Vec<Value>), // MUST ALL BE SAME TYPE
    Option(Box<FunctionType>),
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionType::String => write!(f, "string"),
            FunctionType::Number => write!(f, "number"),
            FunctionType::Boolean => write!(f, "boolean"),
            FunctionType::Array(_type) => write!(f, "array<{}>", _type),
            FunctionType::Object { .. } => write!(f, "object"),
            FunctionType::Null => write!(f, "null"),
            FunctionType::Enum(values) => {
                write!(f, "Enum[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            },
            FunctionType::Option(value) => write!(f, "Option<{}>", value),
        }
    }
}