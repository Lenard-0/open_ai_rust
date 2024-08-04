use serde_json::json;

use crate::logoi::input::tool::FunctionType;

use super::parse_obj::parse_obj;

pub fn insert_type(
    param_map: &mut serde_json::Map<String, serde_json::Value>,
    _type: &FunctionType
) -> Result<(), String> {
    match _type {
        FunctionType::Option(_type) => return insert_type(param_map, _type.as_ref()),
        FunctionType::Enum(values) => insert_enum_type(param_map, values)?,
        FunctionType::Object(obj) => {
            let obj = parse_obj(obj)?;
            for (key, value) in obj {
                param_map.insert(key, value);
            }
        },
        FunctionType::Array(items) => {
            param_map.insert("type".to_string(), json!("array".to_string()));
            param_map.insert("items".to_string(), serde_json::Value::Object({
                let mut items_map = serde_json::Map::new();
                insert_type(&mut items_map, items)?;
                items_map
            }));
        },
        _type => { param_map.insert("type".to_string(), serde_json::Value::String(_type.to_string())); }
    };
    Ok(())
}

fn insert_enum_type(
    param_map: &mut serde_json::Map<String, serde_json::Value>,
    values: &Vec<serde_json::Value>
) -> Result<(), String> {
    make_sure_all_values_same_type(values)?;
    let _type = match values.first() {
        Some(value) => json_type_to_string(value)?,
        None => return Err("Enum values must not be empty".to_string())
    };
    param_map.insert("type".to_string(), serde_json::Value::String(_type.to_string()));
    param_map.insert("enum".to_string(), serde_json::Value::Array(values.clone()));
    Ok(())
}

fn json_type_to_string(_type: &serde_json::Value) -> Result<String, String> {
    match _type {
        serde_json::Value::String(_) => Ok("string".to_string()),
        serde_json::Value::Number(_) => Ok("number".to_string()),
        serde_json::Value::Bool(_) => Ok("boolean".to_string()),
        _ => Err("Invalid type".to_string())
    }
}

fn make_sure_all_values_same_type(values: &Vec<serde_json::Value>) -> Result<(), String> {
    let first_type = match values.first() {
        Some(value) => json_type_to_string(value)?,
        None => return Err("Enum values must not be empty".to_string())
    };

    for value in values {
        let value_type = json_type_to_string(value)?;
        if value_type != first_type {
            return Err("All values in Enum must be of the same type".to_string())
        }
    }
    Ok(())
}