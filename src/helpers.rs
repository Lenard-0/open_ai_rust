use crate::{API_KEY, OPENAI_MSG_ENDPOINT};



pub fn get_url(end_point: &str) -> Result<String, String> {
    let mut url = {
        let url = OPENAI_MSG_ENDPOINT.lock().map_err(|e| format!("Error getting Embeddings endpoint from Mutex lock: {}", e))?;
        if url.is_empty() {
            "https://api.openai.com".to_string()
        } else {
            return Ok(url.to_string())
        }
    };

    url.push_str(end_point);
    println!("Using URL: {}", url);
    Ok(url)
}

pub fn get_key() -> Result<String, String> {
    match API_KEY.lock() {
        Ok(key) => Ok(key.clone()),
        Err(e) => return Err(format!("Error getting API key from Mutex lock: {}", e))
    }
}