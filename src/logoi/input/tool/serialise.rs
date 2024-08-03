use std::collections::HashMap;

use serde::{ser::SerializeMap, Serialize, Serializer};

use super::{FunctionCall, FunctionType};

impl Serialize for FunctionCall {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the fields of FunctionCall
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("name", &self.name)?;
        if let Some(ref desc) = self.description {
            map.serialize_entry("description", desc)?;
        }

        // Serialize arguments with their names as keys
        let mut params_map = HashMap::new();
        let mut required_params = Vec::new();
        for param in &self.parameters {
            match &param._type {
                FunctionType::Option(_type) => (),
                _ => required_params.push(param.name.clone())
            };

            let mut param_map = serde_json::Map::new();

            match insert_type(&mut param_map, &param._type) {
                Ok(_) => (),
                Err(e) => return Err(serde::ser::Error::custom(e))
            };

            if let Some(ref description) = param.description {
                param_map.insert("description".to_string(), serde_json::Value::String(description.clone()));
            }

            params_map.insert(param.name.clone(), serde_json::Value::Object(param_map));
        }

        // Add the required list
        let mut arguments_map = serde_json::Map::new();
        for (key, value) in params_map {
            arguments_map.insert(key, value);
        }

        let mut parameters_outer_wrapper = serde_json::Map::new();
        parameters_outer_wrapper.insert("type".to_string(), serde_json::Value::String("object".to_string()));
        parameters_outer_wrapper.insert("properties".to_string(), serde_json::Value::Object(arguments_map));

        if !required_params.is_empty() {
            parameters_outer_wrapper.insert("required".to_string(), serde_json::Value::Array(required_params.into_iter().map(serde_json::Value::String).collect()));
        }

        map.serialize_entry("parameters", &serde_json::Value::Object(parameters_outer_wrapper))?;
        map.end()
    }
}

fn insert_type(
    param_map: &mut serde_json::Map<String, serde_json::Value>,
    _type: &FunctionType
) -> Result<(), String> {
    match _type {
        FunctionType::Option(_type) => {
            return insert_type(param_map, _type.as_ref())
        },
        FunctionType::Enum(values) => {
                // TODO: make sure all values are same type else throw error
                // until above is impl, type will be the first type
                if values.is_empty() { // throw error as not allowed
                    return Err("Enum values must not be empty".to_string());
                }

                let _type = match values.first().unwrap() {
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::Bool(_) => "boolean",
                    _ => "null"
                };
                param_map.insert("type".to_string(), serde_json::Value::String(_type.to_string()));
                param_map.insert("enum".to_string(), serde_json::Value::Array(values.clone()))
        },
        _type => param_map.insert("type".to_string(), serde_json::Value::String(_type.to_string()))
    };
    Ok(())
}