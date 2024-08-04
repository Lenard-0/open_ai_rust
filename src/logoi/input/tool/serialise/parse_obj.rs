use std::collections::HashMap;
use serde_json::Value;
use crate::logoi::input::tool::FunctionParameter;
use super::parse_param::insert_param;



pub fn parse_obj(
    parameters: &Vec<FunctionParameter>
) -> Result< serde_json::Map<String, Value>, String> {
    let mut params_map = HashMap::new();
    let mut required_params = Vec::new();
    for param in parameters {
        match insert_param(&mut params_map, &mut required_params, param) {
            Ok(_) => (),
            Err(e) => return Err(e)
        };
    }

    let mut arguments_map = serde_json::Map::new();
    for (key, value) in params_map {
        arguments_map.insert(key, value);
    }

    let mut obj = serde_json::Map::new();
    obj.insert("type".to_string(), serde_json::Value::String("object".to_string()));
    obj.insert("properties".to_string(), serde_json::Value::Object(arguments_map));

    if !required_params.is_empty() {
        obj.insert("required".to_string(), serde_json::Value::Array(required_params.into_iter().map(serde_json::Value::String).collect()));
    }

    return Ok(obj)
}