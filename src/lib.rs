use std::sync::Mutex;


pub mod logoi;
pub mod requests;
pub mod helpers;

static API_KEY: Mutex<String> = Mutex::new(String::new());
static OPENAI_MSG_ENDPOINT: Mutex<String> = Mutex::new(String::new());
static EMBEDDINGS_ENDPOINT: Mutex<String> = Mutex::new(String::new());

pub fn set_key(value: String) {
    *API_KEY.lock().unwrap() = value;
}

pub fn set_ai_msg_endpoint(value: String) {
    *OPENAI_MSG_ENDPOINT.lock().unwrap() = value;
}

pub fn set_ai_msg_endpoint_default() {
    *OPENAI_MSG_ENDPOINT.lock().unwrap() = "https://api.openai.com/v1/chat/completions".to_string();
}

pub fn set_embeddings_endpoint(value: String) {
    *EMBEDDINGS_ENDPOINT.lock().unwrap() = value;
}

pub fn set_embeddings_endpoint_default() {
    *EMBEDDINGS_ENDPOINT.lock().unwrap() = "https://api.openai.com/v1/embeddings".to_string();
}

#[derive(Clone, Copy)]
pub enum RequestType {
    ChatCompletion,
    Embedding,
}