//! Compile-time tool-schema representation emitted by
//! [`open_ai_rust_fn_call_extension`](https://docs.rs/open_ai_rust_fn_call_extension)
//! macros (`#[function_call]` / `#[tool]`).
//!
//! Each macro emits a `const FN_NAME: FunctionCallRaw<'static> = ...;` literal
//! next to the user's function. Call [`FunctionCallRaw::to_fn_call`] to lift
//! it to the runtime `FunctionCall` schema understood by the rest of the SDK.

use crate::logoi::input::tool::{FunctionCall, FunctionParameter, FunctionType};

/// One parameter inside [`FunctionCallRaw::parameters`]. All fields are
/// `&'static str` so the whole struct can live in a `const`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FunctionParamRaw<'a> {
    /// Parameter name as written in the Rust function signature (or as
    /// overridden by `#[function_call(rename = "...")]`).
    pub name: &'a str,
    /// Rendered type — generic parameters preserved
    /// (e.g. `"Option<String>"`, `"Vec<u8>"`).
    pub ty: &'a str,
    /// Optional human-readable description. Empty string == none.
    pub description: &'a str,
}

/// Compile-time schema literal emitted by `#[function_call]` / `#[tool]`.
///
/// The shape mirrors a JSON-tool `FunctionCall` so the macro can build it
/// entirely at expansion time without invoking any runtime code. Call
/// [`Self::to_fn_call`] to convert.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct FunctionCallRaw<'a> {
    pub name: &'a str,
    /// Empty string == no description.
    pub description: &'a str,
    pub parameters: &'a [FunctionParamRaw<'a>],
}

impl<'a> FunctionCallRaw<'a> {
    /// Lift the compile-time literal into the runtime [`FunctionCall`]
    /// representation used elsewhere in the SDK.
    ///
    /// # Errors
    /// Returns the original parameter-type string if it doesn't map to a
    /// supported [`FunctionType`] variant. See [`parse_raw_type`] for the
    /// list of accepted spellings.
    pub fn to_fn_call(self) -> Result<FunctionCall, String> {
        let name = self.name.to_string();
        let description = if self.description.is_empty() {
            None
        } else {
            Some(self.description.to_string())
        };

        let mut parameters = Vec::with_capacity(self.parameters.len());
        for raw in self.parameters {
            parameters.push(FunctionParameter {
                name: raw.name.to_string(),
                _type: parse_raw_type(raw.ty)?,
                description: if raw.description.is_empty() {
                    None
                } else {
                    Some(raw.description.to_string())
                },
                // Parameter required by default; `Option<T>` types are
                // detected and marked non-required at macro-expansion time
                // in the derive emission (not here).
                required: !is_option_type_str(raw.ty),
            });
        }

        Ok(FunctionCall {
            name,
            description,
            parameters,
        })
    }
}

/// Quick heuristic: a parameter is non-required when its rendered type is
/// `Option<...>`. The macro preserves the original Rust type spelling, so a
/// substring check is sufficient and avoids parsing the whole grammar here.
fn is_option_type_str(ty: &str) -> bool {
    let t = ty.trim();
    t.starts_with("Option<") || t.starts_with("Option <") || t == "Option"
}

/// Map a rendered Rust type back to a [`FunctionType`].
///
/// Supports the canonical Rust primitive spellings (`bool`, `String`, `i64`,
/// `u32`, `f64`, …) plus the lower-cased JSON-Schema words (`string`,
/// `number`, `boolean`). `Option<T>`, `Vec<T>` and `HashMap<K, V>` are
/// unwrapped recursively. Unknown types return an `Err` containing the
/// original spelling for the macro consumer to surface.
fn parse_raw_type(_type: &str) -> Result<FunctionType, String> {
    let t = _type.trim();

    // Option<Inner> → recurse on Inner.
    if let Some(inner) = strip_wrapper(t, "Option") {
        return Ok(FunctionType::Option(Box::new(parse_raw_type(inner)?)));
    }
    // Vec<Inner> → array of Inner.
    if let Some(inner) = strip_wrapper(t, "Vec") {
        return Ok(FunctionType::Array(Box::new(parse_raw_type(inner)?)));
    }
    // HashMap<_, V> / BTreeMap<_, V> → Map(V).
    if let Some(inner) = strip_wrapper(t, "HashMap").or_else(|| strip_wrapper(t, "BTreeMap")) {
        // Take everything after the first comma at top level.
        if let Some(idx) = top_level_comma(inner) {
            let v_ty = inner[idx + 1..].trim();
            return Ok(FunctionType::Map(Box::new(parse_raw_type(v_ty)?)));
        }
    }

    match t {
        // Idiomatic Rust spellings.
        "String" | "&str" | "&'static str" | "Cow<'static, str>" | "string" => {
            Ok(FunctionType::String)
        }
        "bool" | "boolean" => Ok(FunctionType::Boolean),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" | "number" => Ok(FunctionType::Number),
        other => Err(format!(
            "raw schema: type `{other}` is not yet supported by FunctionCallRaw::to_fn_call"
        )),
    }
}

/// If `t` is `Name<...>`, return the content between the outermost `<>`.
fn strip_wrapper<'a>(t: &'a str, name: &str) -> Option<&'a str> {
    let rest = t.strip_prefix(name)?.trim_start();
    let rest = rest.strip_prefix('<')?;
    let rest = rest.strip_suffix('>')?;
    Some(rest)
}

/// Locate a comma at angle-bracket depth zero (so we don't split
/// `HashMap<String, Vec<i64>>` at the inner comma).
fn top_level_comma(s: &str) -> Option<usize> {
    let mut depth: i32 = 0;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}
