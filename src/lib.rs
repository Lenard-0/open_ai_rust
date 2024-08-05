use std::sync::Mutex;

pub mod logoi;
pub mod requests;

static API_KEY: Mutex<String> = Mutex::new(String::new());

pub fn set_key(value: String) {
    *API_KEY.lock().unwrap() = value;
}