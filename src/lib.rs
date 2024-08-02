use std::sync::Mutex;

static API_KEY: Mutex<String> = Mutex::new(String::new());

pub mod logoi;
pub mod requests;

pub fn set_key(value: String) {
    *API_KEY.lock().unwrap() = value;
}