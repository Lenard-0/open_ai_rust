use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod raw_macro;
pub mod serialise;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ToolChoice {
    pub function: FunctionCall,
    #[serde(rename = "type")]
    pub _type: ToolType,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Vec<FunctionParameter>,
}

impl Default for FunctionCall {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionCall {
    pub fn new() -> Self {
        FunctionCall {
            name: "".to_string(),
            description: None,
            parameters: vec![],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub _type: FunctionType,
    pub description: Option<String>,
    /// Whether this parameter is required. Defaults to `true`.
    ///
    /// Note: a parameter is treated as optional at the JSON-schema `required` array level
    /// when either `required == false` OR the inner `_type` is `FunctionType::Option(_)`.
    /// Generally prefer setting `required: false` over wrapping in `Option`; the OpenAI
    /// strict-schema convention is to list optionality in `required`, not via nullability.
    #[serde(default = "default_required")]
    pub required: bool,
}

fn default_required() -> bool {
    true
}

impl Default for FunctionParameter {
    fn default() -> Self {
        Self {
            name: String::new(),
            _type: FunctionType::String,
            description: None,
            required: true,
        }
    }
}

impl FunctionParameter {
    /// Builder-style constructor for a required string parameter; chain `.required(false)` / `.description(...)`.
    pub fn new(name: impl Into<String>, _type: FunctionType) -> Self {
        Self {
            name: name.into(),
            _type,
            description: None,
            required: true,
        }
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn required(mut self, v: bool) -> Self {
        self.required = v;
        self
    }
}

// impl ToTokens for FunctionParameter {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         let name = &self.name;
//         let description = &self.description;
//         let _type = &self._type;
//         tokens.extend(quote! {
//             FunctionCall {
//                 name: #name,
//                 description: #description,
//                 _type: #_type,
//             }
//         });
//     }
// }

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum FunctionType {
    String,
    Number,
    Boolean,
    Array(Box<FunctionType>),
    Object(Vec<FunctionParameter>),
    Null,
    Enum(EnumValues),
    Option(Box<FunctionType>),
    /// `HashMap<String, V>` / `BTreeMap<String, V>` — emitted as `{ "type": "object",
    /// "additionalProperties": <V schema> }`.
    Map(Box<FunctionType>),
    /// Tagged union from a Rust enum with data variants — emitted as JSON-Schema `oneOf`.
    OneOf(Vec<FunctionVariant>),
}

/// One variant of a `OneOf` (JSON-Schema `oneOf`) — corresponds to one variant of a
/// Rust enum carried by `#[derive(FunctionCall)]`.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionVariant {
    /// Discriminator value (the Rust variant name unless overridden).
    pub name: String,
    pub description: Option<String>,
    /// Parameters carried by this variant (empty for unit variants).
    pub parameters: Vec<FunctionParameter>,
}

// impl ToTokens for FunctionType {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//                 match self {
//             FunctionType::String => {
//                 tokens.extend(quote! {
//                     FunctionType::String
//                 });
//             },
//             FunctionType::Number => {
//                 tokens.extend(quote! {
//                     FunctionType::Number
//                 });
//             },
//             FunctionType::Boolean => {
//                 tokens.extend(quote! {
//                     FunctionType::Boolean
//                 });
//             },
//             FunctionType::Array(_type) => {
//                 tokens.extend(quote! {
//                     FunctionType::Array(Box::new(#_type))
//                 });
//             },
//             FunctionType::Object(parameters) => {
//                 for parameter in parameters {
//                     parameter.to_tokens(tokens);
//                 }
//             },
//             FunctionType::Null => {
//                 tokens.extend(quote! {
//                     FunctionType::Null
//                 });
//             },
//             FunctionType::Enum(values) => {
//                 match values {
//                     EnumValues::String(values) => {
//                         for value in values {
//                             tokens.extend(quote! {
//                                 EnumValue::String(#value)
//                             });
//                         }
//                     },
//                     EnumValues::Int(values) => {
//                         for value in values {
//                             tokens.extend(quote! {
//                                 EnumValue::Int(#value)
//                             });
//                         }
//                     },
//                     EnumValues::Float(values) => {
//                         for value in values {
//                             tokens.extend(quote! {
//                                 EnumValue::Float(#value)
//                             });
//                         }
//                     }
//                 }
//             },
//             FunctionType::Option(value) => {
//                 tokens.extend(quote! {
//                     FunctionType::Option(Box::new(#value))
//                 });
//             },
//         }
//     }
// }

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionType::String => write!(f, "string"),
            FunctionType::Number => write!(f, "number"),
            FunctionType::Boolean => write!(f, "boolean"),
            FunctionType::Array(_type) => write!(f, "array<{}>", _type),
            FunctionType::Object { .. } => write!(f, "object"),
            FunctionType::Null => write!(f, "null"),
            FunctionType::Enum(values) => match values {
                EnumValues::String(values) => {
                    write!(f, "enum<{}>", values.join(", "))
                }
                EnumValues::Int(values) => {
                    write!(
                        f,
                        "enum<{}>",
                        values
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
                EnumValues::Float(values) => {
                    write!(
                        f,
                        "enum<{}>",
                        values
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
            },
            FunctionType::Option(value) => write!(f, "Option<{}>", value),
            FunctionType::Map(value) => write!(f, "map<{}>", value),
            FunctionType::OneOf(variants) => {
                let names: Vec<_> = variants.iter().map(|v| v.name.as_str()).collect();
                write!(f, "oneOf<{}>", names.join("|"))
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum EnumValues {
    String(Vec<String>),
    Int(Vec<i64>),
    Float(Vec<f64>),
}

impl Display for EnumValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => write!(f, "{:?}", value),
            Self::Int(value) => write!(f, "{:?}", value),
            Self::Float(value) => write!(f, "{:?}", value),
        }
    }
}

// impl ToTokens for EnumValues {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         match self {
//             Self::String(value) => {
//                 for v in value {
//                     tokens.extend(quote! {
//                         EnumValue::String(#v)
//                     });
//                 }
//             },
//             Self::Int(value) => {
//                 for v in value {
//                     tokens.extend(quote! {
//                         EnumValue::Int(#v)
//                     });
//                 }
//             },
//             Self::Float(value) => {
//                 for v in value {
//                     tokens.extend(quote! {
//                         EnumValue::Float(#v)
//                     });
//                 }
//             },
//         }
//     }
// }
