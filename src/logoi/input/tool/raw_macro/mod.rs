//! Compile-time / runtime type-level schema reflection for OpenAI function
//! tool parameters.
//!
//! The [`FunctionCallable`] trait describes how a Rust type maps to a
//! JSON-Schema [`FunctionType`]. Implementations are provided here for every
//! primitive and standard-library container; user types should derive the
//! impl with `#[derive(open_ai_rust_fn_call_extension::FunctionCall)]`.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use super::{FunctionCall, FunctionType};

pub mod fn_macro;

/// Describes a Rust type's mapping to a JSON-Schema tool-parameter type.
///
/// `#[derive(FunctionCall)]` from `open_ai_rust_fn_call_extension` emits an
/// implementation for any struct or unit-variant enum. Built-in impls in this
/// module cover the standard primitive and container types.
///
/// Two associated methods, both static (no instance required):
///
/// * [`schema_type`](FunctionCallable::schema_type) — the JSON-Schema type of
///   the value. Always meaningful: primitives return their leaf variant
///   (`String`, `Number`, `Boolean`, ...); containers wrap the schema of
///   their element type; user types return `FunctionType::Object(...)`.
/// * [`fn_schema`](FunctionCallable::fn_schema) — a full [`FunctionCall`]
///   describing the type as a complete tool function definition. Only
///   meaningful on derived user types; primitives panic, since "describe
///   `bool` as a function" has no canonical meaning.
pub trait FunctionCallable {
    /// Static JSON-schema [`FunctionType`] for `Self`. Required.
    fn schema_type() -> FunctionType
    where
        Self: Sized;

    /// Full [`FunctionCall`] schema for `Self`. Only meaningful on user types
    /// that have derived `FunctionCall`; the default body panics so primitive
    /// container impls (`bool`, `Vec<T>`, ...) can omit it.
    fn fn_schema() -> FunctionCall
    where
        Self: Sized,
    {
        panic!(
            "FunctionCallable::fn_schema() is only meaningful on \
             #[derive(FunctionCall)] types — not on primitives or containers."
        )
    }
}

// ── primitive impls ─────────────────────────────────────────────────────────

impl FunctionCallable for String {
    fn schema_type() -> FunctionType {
        FunctionType::String
    }
}

impl FunctionCallable for &'static str {
    fn schema_type() -> FunctionType {
        FunctionType::String
    }
}

impl FunctionCallable for Cow<'static, str> {
    fn schema_type() -> FunctionType {
        FunctionType::String
    }
}

impl FunctionCallable for bool {
    fn schema_type() -> FunctionType {
        FunctionType::Boolean
    }
}

macro_rules! impl_numeric {
    ($($t:ty),+) => {
        $(
            impl FunctionCallable for $t {
                fn schema_type() -> FunctionType { FunctionType::Number }
            }
        )+
    };
}

impl_numeric!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

// ── container impls ─────────────────────────────────────────────────────────

impl<T: FunctionCallable> FunctionCallable for Vec<T> {
    fn schema_type() -> FunctionType {
        FunctionType::Array(Box::new(T::schema_type()))
    }
}

impl<T: FunctionCallable, const N: usize> FunctionCallable for [T; N] {
    fn schema_type() -> FunctionType {
        FunctionType::Array(Box::new(T::schema_type()))
    }
}

impl<T: FunctionCallable> FunctionCallable for Option<T> {
    fn schema_type() -> FunctionType {
        FunctionType::Option(Box::new(T::schema_type()))
    }
}

impl<V: FunctionCallable> FunctionCallable for HashMap<String, V> {
    fn schema_type() -> FunctionType {
        FunctionType::Map(Box::new(V::schema_type()))
    }
}

impl<V: FunctionCallable> FunctionCallable for BTreeMap<String, V> {
    fn schema_type() -> FunctionType {
        FunctionType::Map(Box::new(V::schema_type()))
    }
}
