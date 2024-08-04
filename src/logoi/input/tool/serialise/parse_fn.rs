
use serde::{Serialize, Serializer};
use serde_json::Value;
use crate::logoi::input::tool::FunctionCall;
use super::parse_obj::parse_obj;

impl Serialize for FunctionCall {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = init_parsing_fn(self);

        let parameters_outer_obj = match parse_obj(&self.parameters) {
            Ok(obj) => obj,
            Err(e) => return Err(serde::ser::Error::custom(e))
        };

        map.insert("parameters".to_string(), serde_json::Value::Object(parameters_outer_obj));
        return map.serialize(serializer)
    }
}

fn init_parsing_fn(fn_def: &FunctionCall) -> serde_json::Map<String, Value> {
    let mut json = serde_json::Map::new();
    json.insert("name".to_string(), Value::String(fn_def.name.clone()));
    if let Some(ref desc) = fn_def.description {
        json.insert("description".to_string(), Value::String(desc.clone()));
    }

    return json
}