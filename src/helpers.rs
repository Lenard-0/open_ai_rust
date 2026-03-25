use crate::{API_KEY, EMBEDDINGS_ENDPOINT, OPENAI_MSG_ENDPOINT, RequestType};


pub fn get_url(request_type: RequestType) -> Result<String, String> {
    match request_type {
        RequestType::ChatCompletion => {
            let url = OPENAI_MSG_ENDPOINT
                .lock()
                .map_err(|e| format!("Error locking chat endpoint: {}", e))?
                .clone();

            if !url.is_empty() {
                return Ok(url);
            } else {
                return Ok("https://api.openai.com/v1/chat/completions".to_string())
            }

        }

        RequestType::Embedding => {
            let url = EMBEDDINGS_ENDPOINT
                .lock()
                .map_err(|e| format!("Error locking embeddings endpoint: {}", e))?
                .clone();

            if !url.is_empty() {
                return Ok(url);
            } else {
                Ok("https://api.openai.com/v1/embeddings".to_string())
            }
        }
    }
}

pub fn get_key() -> Result<String, String> {
    match API_KEY.lock() {
        Ok(key) => Ok(key.clone()),
        Err(e) => return Err(format!("Error getting API key from Mutex lock: {}", e))
    }
}