use std::collections::HashMap;

use super::{FunctionCall, FunctionType};


pub mod fn_macro;

pub trait FunctionCallable {
    fn to_fn_call(&self) -> FunctionCall;
    fn to_fn_type(&self) -> FunctionType;
}

impl FunctionCallable for String {
    fn to_fn_call(&self) -> FunctionCall { FunctionCall::new() }
    fn to_fn_type(&self) -> FunctionType { FunctionType::String }
}

impl FunctionCallable for bool {
    fn to_fn_call(&self) -> FunctionCall { FunctionCall::new() }
    fn to_fn_type(&self) -> FunctionType { FunctionType::Boolean }
}

macro_rules! implement_int {
    ($($int:ty),+) => {
        $(
            impl FunctionCallable for $int {
                fn to_fn_call(&self) -> FunctionCall { FunctionCall::new() }
                fn to_fn_type(&self) -> FunctionType { FunctionType::Number }
            }
        )+
    };
}

implement_int!(u8, u16, u32, u64, usize, isize, i8, i16, i32, i64, f32, f64);

impl<T: FunctionCallable> FunctionCallable for Vec<T> {
    fn to_fn_call(&self) -> FunctionCall { FunctionCall::new() }
    fn to_fn_type(&self) -> FunctionType {
        if self.is_empty() {
            panic!("Empty vec when trying to fn call parse. Need val for type")
        }
        let first_val = &self[0];
        return FunctionType::Array(Box::new(first_val.to_fn_type()))
    }
}

impl<T: FunctionCallable> FunctionCallable for Option<T> {
    fn to_fn_call(&self) -> FunctionCall { FunctionCall::new() }
    fn to_fn_type(&self) -> FunctionType { match self {
        Some(v) => FunctionType::Option(Box::new(v.to_fn_type())),
        None => panic!("Empty option when trying to fn call parse. Need val for type")
    }}
}

// impl<K: FunctionCallable, V: FunctionCallable> FunctionCallable for HashMap<K, V> {
//     fn to_fn_call(&self) -> String {
//         todo!()
//     }
// }