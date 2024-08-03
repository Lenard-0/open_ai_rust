use std::collections::HashMap;
use insert_param::insert_param;
use serde::{ser::SerializeMap, Serialize, Serializer};
use super::FunctionCall;

pub mod insert_type;
pub mod insert_param;


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
            match insert_param(&mut params_map, &mut required_params, param) {
                Ok(_) => (),
                Err(e) => return Err(serde::ser::Error::custom(e))
            }
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

