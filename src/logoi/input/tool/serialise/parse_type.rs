use serde_json::json;

use crate::logoi::input::tool::{EnumValues, FunctionType};

use super::parse_obj::insert_obj_into_param_map;

pub fn insert_type(
    param_map: &mut serde_json::Map<String, serde_json::Value>,
    _type: &FunctionType
) -> Result<(), String> {
    match _type {
        FunctionType::Option(_type) => return insert_type(param_map, _type.as_ref()),
        FunctionType::Enum(values) => insert_enum_type(param_map, values)?,
        FunctionType::Object(obj) => insert_obj_into_param_map(param_map, obj)?,
        FunctionType::Array(items) => parse_array(param_map, items)?,
        _type => { param_map.insert("type".to_string(), serde_json::Value::String(_type.to_string())); }
    };

    Ok(())
}

fn insert_enum_type(
    param_map: &mut serde_json::Map<String, serde_json::Value>,
    values: &EnumValues
) -> Result<(), String> {
    param_map.insert("type".to_string(), serde_json::Value::String(enum_value_to_string(values)?));
    param_map.insert("enum".to_string(), serde_json::Value::Array(match values {
        EnumValues::String(values) => values.iter().map(|v| serde_json::Value::String(v.to_string())).collect(),
        EnumValues::Int(values) => values.iter().map(|v| serde_json::Value::Number(serde_json::Number::from(*v))).collect(),
        EnumValues::Float(values) => values.iter().map(|v| serde_json::Value::Number(serde_json::Number::from_f64(*v).unwrap())).collect(),
    }));

    Ok(())
}

fn enum_value_to_string(_type: &EnumValues) -> Result<String, String> {
    match _type {
        EnumValues::String(_) => Ok("string".to_string()),
        EnumValues::Int(_) => Ok("number".to_string()),
        EnumValues::Float(_) => Ok("number".to_string()),
    }
}

fn parse_array(param_map: &mut serde_json::Map<String, serde_json::Value>, items: &FunctionType) -> Result<(), String> {
    param_map.insert("type".to_string(), json!("array".to_string()));
    param_map.insert("items".to_string(), serde_json::Value::Object({
        let mut items_map = serde_json::Map::new();
        insert_type(&mut items_map, items)?;
        items_map
    }));

    Ok(())
}