use super::parse_param::insert_param;
use crate::logoi::input::tool::FunctionParameter;
use serde_json::Value;

pub fn insert_obj_into_param_map(
    param_map: &mut serde_json::Map<String, serde_json::Value>,
    obj: &Vec<FunctionParameter>,
) {
    for (key, value) in parse_obj(obj) {
        param_map.insert(key, value);
    }
}

pub fn parse_obj(obj: &Vec<FunctionParameter>) -> serde_json::Map<String, Value> {
    let (fields_map, required_params) = parse_fields(obj);

    let mut out: serde_json::Map<String, Value> = serde_json::Map::new();
    out.insert(
        "type".to_string(),
        serde_json::Value::String("object".to_string()),
    );
    out.insert(
        "properties".to_string(),
        serde_json::Value::Object(fields_map),
    );
    out.insert(
        "required".to_string(),
        serde_json::Value::Array(
            required_params
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ),
    );
    out
}

fn parse_fields(
    fields: &Vec<FunctionParameter>,
) -> (serde_json::Map<String, Value>, Vec<String>) {
    let mut fields_map = serde_json::Map::new();
    let mut required_params = Vec::new();
    for param in fields {
        insert_param(&mut fields_map, &mut required_params, param);
    }
    (fields_map, required_params)
}
