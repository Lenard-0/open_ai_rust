use super::parse_obj::parse_obj;
use crate::logoi::input::tool::FunctionCall;
use serde::{Serialize, Serializer};
use serde_json::Value;

impl Serialize for FunctionCall {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = init_parsing_fn(self);
        map.insert(
            "parameters".to_string(),
            serde_json::Value::Object(parse_obj(&self.parameters)),
        );
        map.serialize(serializer)
    }
}

fn init_parsing_fn(fn_def: &FunctionCall) -> serde_json::Map<String, Value> {
    let mut json = serde_json::Map::new();
    json.insert("name".to_string(), Value::String(fn_def.name.clone()));
    if let Some(ref desc) = fn_def.description {
        json.insert("description".to_string(), Value::String(desc.clone()));
    }

    json
}
