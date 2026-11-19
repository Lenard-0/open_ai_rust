//! Strict-mode JSON Schema generation for structured outputs.
//!
//! OpenAI / Azure strict mode (`"strict": true`) requires every object to set
//! `additionalProperties: false` and to list **every** property in `required`. Optional fields
//! are expressed as nullable (`"type": ["string", "null"]`) instead of being left out of
//! `required`. Strict mode does not support `oneOf` (we emit `anyOf`), nor free-form maps.

use serde_json::{json, Map, Value};
use thiserror::Error;

use crate::logoi::input::tool::{EnumValues, FunctionCall, FunctionParameter, FunctionType};

/// A [`FunctionCall`] schema that can't be expressed in strict mode.
#[derive(Debug, Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum StrictSchemaError {
    /// `FunctionType::Map` (`HashMap<String, V>` / `BTreeMap<String, V>`) needs
    /// `additionalProperties: <schema>`, which strict mode forbids. Use a `Vec` of key/value
    /// structs instead, or send with `strict: false`.
    #[error("strict schema: map type at `{path}` is not supported in strict mode (additionalProperties must be false)")]
    UnsupportedMap { path: String },
}

impl FunctionCall {
    /// Build a strict-mode JSON Schema (the `schema` value of a `json_schema` response format)
    /// from this function's parameters.
    ///
    /// - every object gets `"additionalProperties": false`
    /// - every property is listed in `required`
    /// - parameters with `required: false` or `FunctionType::Option(_)` become nullable
    /// - `OneOf` is emitted as `anyOf`
    ///
    /// ```
    /// use open_ai_rust::{FunctionCall, FunctionParameter, FunctionType};
    /// use serde_json::json;
    ///
    /// let f = FunctionCall {
    ///     name: "city".into(),
    ///     description: None,
    ///     parameters: vec![
    ///         FunctionParameter::new("name", FunctionType::String),
    ///         FunctionParameter::new("population", FunctionType::Number).required(false),
    ///     ],
    /// };
    /// assert_eq!(f.to_strict_json_schema().unwrap(), json!({
    ///     "type": "object",
    ///     "properties": {
    ///         "name": { "type": "string" },
    ///         "population": { "type": ["number", "null"] }
    ///     },
    ///     "required": ["name", "population"],
    ///     "additionalProperties": false
    /// }));
    /// ```
    pub fn to_strict_json_schema(&self) -> Result<Value, StrictSchemaError> {
        Ok(Value::Object(strict_obj(&self.parameters, "$")?))
    }
}

fn strict_obj(
    params: &[FunctionParameter],
    path: &str,
) -> Result<Map<String, Value>, StrictSchemaError> {
    let mut properties = Map::new();
    let mut required = Vec::with_capacity(params.len());
    for p in params {
        let child_path = format!("{path}.{}", p.name);
        let mut schema = strict_type(&p._type, &child_path)?;
        if !p.required && !matches!(p._type, FunctionType::Option(_)) {
            schema = make_nullable(schema);
        }
        if let Some(desc) = &p.description {
            schema.insert("description".into(), json!(desc));
        }
        properties.insert(p.name.clone(), Value::Object(schema));
        required.push(json!(p.name));
    }
    let mut out = Map::new();
    out.insert("type".into(), json!("object"));
    out.insert("properties".into(), Value::Object(properties));
    out.insert("required".into(), Value::Array(required));
    out.insert("additionalProperties".into(), json!(false));
    Ok(out)
}

fn typed(t: &str) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("type".into(), json!(t));
    m
}

fn strict_type(t: &FunctionType, path: &str) -> Result<Map<String, Value>, StrictSchemaError> {
    Ok(match t {
        FunctionType::String => typed("string"),
        FunctionType::Number => typed("number"),
        FunctionType::Boolean => typed("boolean"),
        FunctionType::Null => typed("null"),
        FunctionType::Option(inner) => make_nullable(strict_type(inner, path)?),
        FunctionType::Enum(values) => {
            let (ty, vals) = match values {
                EnumValues::String(v) => ("string", json!(v)),
                EnumValues::Int(v) => ("number", json!(v)),
                EnumValues::Float(v) => ("number", json!(v)),
            };
            let mut m = typed(ty);
            m.insert("enum".into(), vals);
            m
        }
        FunctionType::Object(params) => strict_obj(params, path)?,
        FunctionType::Array(items) => {
            let mut m = typed("array");
            let items = strict_type(items, &format!("{path}[]"))?;
            m.insert("items".into(), Value::Object(items));
            m
        }
        FunctionType::Map(_) => {
            return Err(StrictSchemaError::UnsupportedMap {
                path: path.to_string(),
            })
        }
        FunctionType::OneOf(variants) => {
            let mut any_of = Vec::with_capacity(variants.len());
            for v in variants {
                let mut schema = if v.parameters.is_empty() {
                    let mut m = typed("string");
                    m.insert("enum".into(), json!([v.name]));
                    m
                } else {
                    strict_obj(&v.parameters, &format!("{path}<{}>", v.name))?
                };
                schema.insert("title".into(), json!(v.name));
                if let Some(desc) = &v.description {
                    schema.insert("description".into(), json!(desc));
                }
                any_of.push(Value::Object(schema));
            }
            let mut m = Map::new();
            m.insert("anyOf".into(), Value::Array(any_of));
            m
        }
    })
}

/// Allow `null` in addition to whatever `schema` already accepts. Every schema built by
/// [`strict_type`] has either `anyOf` or `type`.
fn make_nullable(mut m: Map<String, Value>) -> Map<String, Value> {
    let null = json!({ "type": "null" });
    if let Some(Value::Array(any_of)) = m.get_mut("anyOf") {
        if !any_of.contains(&null) {
            any_of.push(null);
        }
        return m;
    }
    let mut types = match m.remove("type") {
        Some(Value::Array(ts)) => ts,
        Some(t) => vec![t],
        None => vec![],
    };
    if !types.contains(&json!("null")) {
        types.push(json!("null"));
    }
    m.insert(
        "type".into(),
        if types.len() == 1 {
            types.remove(0)
        } else {
            Value::Array(types)
        },
    );
    if let Some(Value::Array(values)) = m.get_mut("enum") {
        if !values.contains(&Value::Null) {
            values.push(Value::Null);
        }
    }
    m
}
