use serde_json::{json, Value};

use crate::logoi::input::tool::{EnumValues, FunctionType, FunctionVariant};

use super::parse_obj::{insert_obj_into_param_map, parse_obj};

pub fn insert_type(param_map: &mut serde_json::Map<String, serde_json::Value>, _type: &FunctionType) {
    match _type {
        FunctionType::Option(_type) => insert_type(param_map, _type.as_ref()),
        FunctionType::Enum(values) => insert_enum_type(param_map, values),
        FunctionType::Object(obj) => insert_obj_into_param_map(param_map, obj),
        FunctionType::Array(items) => parse_array(param_map, items),
        FunctionType::Map(value) => parse_map(param_map, value),
        FunctionType::OneOf(variants) => parse_oneof(param_map, variants),
        _type => {
            param_map.insert(
                "type".to_string(),
                serde_json::Value::String(_type.to_string()),
            );
        }
    }
}

fn insert_enum_type(
    param_map: &mut serde_json::Map<String, serde_json::Value>,
    values: &EnumValues,
) {
    let (type_str, enum_values) = match values {
        EnumValues::String(v) => (
            "string",
            v.iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        ),
        EnumValues::Int(v) => (
            "number",
            v.iter()
                .map(|n| serde_json::Value::Number(serde_json::Number::from(*n)))
                .collect(),
        ),
        EnumValues::Float(v) => (
            "number",
            v.iter()
                .filter_map(|n| serde_json::Number::from_f64(*n))
                .map(serde_json::Value::Number)
                .collect(),
        ),
    };
    param_map.insert(
        "type".to_string(),
        serde_json::Value::String(type_str.to_string()),
    );
    param_map.insert("enum".to_string(), serde_json::Value::Array(enum_values));
}

fn parse_array(param_map: &mut serde_json::Map<String, serde_json::Value>, items: &FunctionType) {
    param_map.insert("type".to_string(), json!("array".to_string()));
    let mut items_map = serde_json::Map::new();
    insert_type(&mut items_map, items);
    param_map.insert("items".to_string(), serde_json::Value::Object(items_map));
}

fn parse_map(param_map: &mut serde_json::Map<String, serde_json::Value>, value: &FunctionType) {
    param_map.insert("type".to_string(), json!("object"));
    let mut additional = serde_json::Map::new();
    insert_type(&mut additional, value);
    param_map.insert(
        "additionalProperties".to_string(),
        serde_json::Value::Object(additional),
    );
}

fn parse_oneof(
    param_map: &mut serde_json::Map<String, serde_json::Value>,
    variants: &[FunctionVariant],
) {
    let mut arr: Vec<Value> = Vec::with_capacity(variants.len());
    for v in variants {
        let mut variant_schema = if v.parameters.is_empty() {
            // unit variant → string const equivalent
            let mut m = serde_json::Map::new();
            m.insert("type".into(), json!("string"));
            m.insert("enum".into(), json!([v.name]));
            m
        } else {
            parse_obj(&v.parameters)
        };
        variant_schema.insert("title".into(), json!(v.name));
        if let Some(desc) = &v.description {
            variant_schema.insert("description".into(), json!(desc));
        }
        arr.push(serde_json::Value::Object(variant_schema));
    }
    param_map.insert("oneOf".into(), serde_json::Value::Array(arr));
}
