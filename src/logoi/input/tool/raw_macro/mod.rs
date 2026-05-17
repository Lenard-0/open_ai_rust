use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use super::{FunctionCall, FunctionType};

pub mod fn_macro;

/// Type-level schema reflection — describes how a Rust type maps to a JSON-Schema /
/// OpenAI function-tool parameter type.
///
/// `#[derive(FunctionCall)]` emits an impl for your struct/enum.
pub trait FunctionCallable {
    /// Instance method retained for backwards compatibility with the original derive
    /// emission. Prefer [`FunctionCallable::schema_type`] / [`FunctionCallable::fn_schema`].
    fn to_fn_call(&self) -> FunctionCall;

    /// Instance method retained for backwards compatibility — derives the type of a value.
    fn to_fn_type(&self) -> FunctionType;

    /// Associated (no-instance) type reflection. Preferred over `to_fn_type` for the
    /// OpenAI tool-registration flow (you describe the type before any value exists).
    fn schema_type() -> FunctionType
    where
        Self: Sized,
    {
        // Fallback: requires an instance — only sound for types that implement Default.
        // Override in the derive emission and in primitive impls below.
        panic!(
            "FunctionCallable::schema_type() not implemented for this type; \
             override in your impl or use the #[derive(FunctionCall)] macro."
        )
    }

    /// Associated (no-instance) FunctionCall schema for `#[derive(FunctionCall)]` structs.
    /// Primitive impls panic — only meaningful on user types.
    fn fn_schema() -> FunctionCall
    where
        Self: Sized,
    {
        panic!(
            "FunctionCallable::fn_schema() is only meaningful on #[derive(FunctionCall)] \
             types — not on primitives or containers."
        )
    }
}

// --- primitive impls --------------------------------------------------------

impl FunctionCallable for String {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        FunctionType::String
    }
    fn schema_type() -> FunctionType {
        FunctionType::String
    }
}

impl FunctionCallable for &'static str {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        FunctionType::String
    }
    fn schema_type() -> FunctionType {
        FunctionType::String
    }
}

impl FunctionCallable for Cow<'static, str> {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        FunctionType::String
    }
    fn schema_type() -> FunctionType {
        FunctionType::String
    }
}

impl FunctionCallable for bool {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        FunctionType::Boolean
    }
    fn schema_type() -> FunctionType {
        FunctionType::Boolean
    }
}

macro_rules! impl_numeric {
    ($($t:ty),+) => {
        $(
            impl FunctionCallable for $t {
                fn to_fn_call(&self) -> FunctionCall { FunctionCall::new() }
                fn to_fn_type(&self) -> FunctionType { FunctionType::Number }
                fn schema_type() -> FunctionType { FunctionType::Number }
            }
        )+
    };
}

impl_numeric!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

// --- container impls --------------------------------------------------------

impl<T: FunctionCallable> FunctionCallable for Vec<T> {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        if let Some(first) = self.first() {
            FunctionType::Array(Box::new(first.to_fn_type()))
        } else {
            FunctionType::Array(Box::new(T::schema_type()))
        }
    }
    fn schema_type() -> FunctionType {
        FunctionType::Array(Box::new(T::schema_type()))
    }
}

impl<T: FunctionCallable, const N: usize> FunctionCallable for [T; N] {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        if let Some(first) = self.first() {
            FunctionType::Array(Box::new(first.to_fn_type()))
        } else {
            FunctionType::Array(Box::new(T::schema_type()))
        }
    }
    fn schema_type() -> FunctionType {
        FunctionType::Array(Box::new(T::schema_type()))
    }
}

impl<T: FunctionCallable> FunctionCallable for Option<T> {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        match self {
            Some(v) => FunctionType::Option(Box::new(v.to_fn_type())),
            None => FunctionType::Option(Box::new(T::schema_type())),
        }
    }
    fn schema_type() -> FunctionType {
        FunctionType::Option(Box::new(T::schema_type()))
    }
}

impl<V: FunctionCallable> FunctionCallable for HashMap<String, V> {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        if let Some((_, v)) = self.iter().next() {
            FunctionType::Map(Box::new(v.to_fn_type()))
        } else {
            FunctionType::Map(Box::new(V::schema_type()))
        }
    }
    fn schema_type() -> FunctionType {
        FunctionType::Map(Box::new(V::schema_type()))
    }
}

impl<V: FunctionCallable> FunctionCallable for BTreeMap<String, V> {
    fn to_fn_call(&self) -> FunctionCall {
        FunctionCall::new()
    }
    fn to_fn_type(&self) -> FunctionType {
        if let Some((_, v)) = self.iter().next() {
            FunctionType::Map(Box::new(v.to_fn_type()))
        } else {
            FunctionType::Map(Box::new(V::schema_type()))
        }
    }
    fn schema_type() -> FunctionType {
        FunctionType::Map(Box::new(V::schema_type()))
    }
}
