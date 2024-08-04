use std::collections::HashMap;
use serde_json::Value;
use crate::logoi::input::tool::FunctionParameter;
use super::parse_param::insert_param;



pub fn parse_obj(
    fields: &Vec<FunctionParameter>
) -> Result<serde_json::Map<String, Value>, String> {
    let (fields_map, required_params) = match parse_fields(fields) {
        Ok((fields_map, required_params)) => (fields_map, required_params),
        Err(e) => return Err(e)
    };

    let mut obj = serde_json::Map::new();
    obj.insert("type".to_string(), serde_json::Value::String("object".to_string()));
    obj.insert("properties".to_string(), serde_json::Value::Object(fields_map));

    if !required_params.is_empty() {
        obj.insert("required".to_string(), serde_json::Value::Array(required_params.into_iter().map(serde_json::Value::String).collect()));
    }

    return Ok(obj)
}

fn parse_fields(fields: &Vec<FunctionParameter>) -> Result<(serde_json::Map<String, Value>, Vec<String>), String> {
    let mut fields_map = serde_json::Map::new();
    let mut required_params = Vec::new();
    for param in fields {
        match insert_param(&mut fields_map, &mut required_params, param) {
            Ok(_) => (),
            Err(e) => return Err(e)
        };
    }

    Ok((fields_map, required_params))
}