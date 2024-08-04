use serde_json::Map;
use crate::logoi::input::tool::{FunctionParameter, FunctionType};
use super::parse_type::insert_type;

pub fn insert_param(
    params_map: &mut Map<String, serde_json::Value>,
    required_params: &mut Vec<String>,
    param: &FunctionParameter
) -> Result<(), String> {
    match &param._type {
        FunctionType::Option(_type) => (),
        _ => required_params.push(param.name.clone())
    };

    let mut param_map = serde_json::Map::new();
    insert_type(&mut param_map, &param._type)?;
    if let Some(ref description) = param.description {
        param_map.insert("description".to_string(), serde_json::Value::String(description.clone()));
    }

    params_map.insert(param.name.clone(), serde_json::Value::Object(param_map));

    Ok(())
}