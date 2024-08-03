use std::{collections::HashMap, fmt::Display};

use serde::{de, ser::{SerializeMap, SerializeStruct}, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};

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
    #[serde(rename = "type")]
    pub _type: FunctionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_required: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FunctionType {
    String,
    Number,
    Boolean,
    Array,
    Object { required: Vec<String> },
    Null,
    Enum(Vec<String>),
}

impl Serialize for FunctionCall {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the fields of FunctionCall
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("name", &self.name)?;
        if let Some(ref desc) = self.description {
            map.serialize_entry("description", desc)?;
        }

        // Serialize arguments with their names as keys
        let mut params_map = HashMap::new();
        let mut required_params = Vec::new();
        for param in &self.parameters {
            if param.is_required {
                required_params.push(param.name.clone());
            }
            let mut param_map = serde_json::Map::new();
            param_map.insert("type".to_string(), serde_json::Value::String(param._type.to_string()));
            if let Some(ref description) = param.description {
                param_map.insert("description".to_string(), serde_json::Value::String(description.clone()));
            }
            params_map.insert(param.name.clone(), serde_json::Value::Object(param_map));
        }

        // Add the required list
        let mut arguments_map = serde_json::Map::new();
        for (key, value) in params_map {
            arguments_map.insert(key, value);
        }

        let mut parameters_outer_wrapper = serde_json::Map::new();
        parameters_outer_wrapper.insert("type".to_string(), serde_json::Value::String("object".to_string()));
        parameters_outer_wrapper.insert("properties".to_string(), serde_json::Value::Object(arguments_map));

        if !required_params.is_empty() {
            parameters_outer_wrapper.insert("required".to_string(), serde_json::Value::Array(required_params.into_iter().map(serde_json::Value::String).collect()));
        }

        map.serialize_entry("parameters", &serde_json::Value::Object(parameters_outer_wrapper))?;
        map.end()
    }
}
impl Serialize for FunctionParameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self._type.to_string())?;
        if let Some(ref description) = self.description {
            map.serialize_entry("description", description)?;
        }
        map.end()
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionType::String => write!(f, "string"),
            FunctionType::Number => write!(f, "number"),
            FunctionType::Boolean => write!(f, "boolean"),
            FunctionType::Array => write!(f, "array"),
            FunctionType::Object { .. } => write!(f, "object"),
            FunctionType::Null => write!(f, "null"),
            FunctionType::Enum(values) => {
                write!(f, "enum[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            }
        }
    }
}