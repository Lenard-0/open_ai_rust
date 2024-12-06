
use serde::Deserialize;
use serde_json::{json, Value};
use crate::{logoi::output::Usage, API_KEY};

#[derive(Debug, Deserialize)]
pub struct EmbedResponse {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
    pub usage: Usage
}

#[derive(Debug, Deserialize)]
pub struct Embedding {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u32
}

pub async fn embed(
    text: String,
    model: Option<String> // if None, use default model
) -> Result<Vec<f32>, String> {
    let url = "https://api.openai.com/v1/embeddings";

    let client = reqwest::Client::new();
    let api_key = {
        match API_KEY.lock() {
            Ok(key) => key.clone(),
            Err(e) => return Err(format!("Error getting API key from Mutex lock: {}", e))
        }
    };

    let model = model.unwrap_or("text-embedding-ada-002".to_string());

    let body = json!({
        "input": text,
        "model": model,
        "encoding_format": "float"
      });

    let response = match client.post(url)
        .header("Content-Type", "application/json")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await {
            Ok(data) => data,
            Err(e) => return Err(format!("Error sending request to Open Ai: {}", e))
        };

    if response.status().is_success() {
        let json: Value = response.json().await.map_err(|e| format!("Error reading response JSON: {}", e))?;
        let json: EmbedResponse = serde_json::from_value(json).map_err(|e| format!("Error parsing OpenAI response: {}", e))?;
        return match json.data.get(0) {
            Some(embedding) => Ok(embedding.embedding.clone()),
            None => Err("No embeddings found in response".to_string())
        }
    } else {
        let status = response.status();
        return Err(format!("Open Ai Error! Status: {status}       Err Msgs: {}", match response.text().await {
            Ok(data) => data,
            Err(e) => format!("Error parsing Open Ai response: {}", e)
        }))
    }
}
