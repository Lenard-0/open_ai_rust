use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<FunctionParameter>,
}

#[derive(Serialize, Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub _type: FunctionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl<'de> Deserialize<'de> for FunctionParameter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        let name = value.get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| de::Error::missing_field("name"))?
            .to_string();

        let _type_value = value.get("_type")
            .ok_or_else(|| de::Error::missing_field("_type"))?;

        let _type: FunctionType = FunctionType::deserialize(_type_value)
            .map_err(de::Error::custom)?;

        let description = value.get("description")
            .and_then(Value::as_str)
            .map(|s| s.to_string());

        Ok(FunctionParameter {
            name,
            _type,
            description,
        })
    }
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