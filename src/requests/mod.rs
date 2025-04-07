use reqwest::Client;
use serde_json::Value;

use crate::{logoi::{input::payload::ChatPayLoad, output::AiMsgResponse}, API_KEY, OPENAI_MSG_ENDPOINT};

pub mod embed;

pub async fn open_ai_msg(
    payload: ChatPayLoad
) -> Result<AiMsgResponse, String> {
    let url = {
        let url = OPENAI_MSG_ENDPOINT.lock().map_err(|e| format!("Error getting Embeddings endpoint from Mutex lock: {}", e))?;
        if url.is_empty() {
            "https://api.openai.com/v1/chat/completions".to_string()
        } else {
            url.to_string()
        }
    };

    let client = Client::new();

    let api_key = {
        match API_KEY.lock() {
            Ok(key) => key.clone(),
            Err(e) => return Err(format!("Error getting API key from Mutex lock: {}", e))
        }
    };

    // let payload_as_json = match serde_json::to_value(&payload) {
    //     Ok(data) => data,
    //     Err(e) => return Err(format!("Error serializing payload to JSON: {}", e))
    // };


    let response = match client.post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await {
            Ok(data) => data,
            Err(e) => return Err(format!("Error sending request to Open Ai: {}", e))
        };

        if response.status().is_success() {
            let json: Value = response.json().await.map_err(|e| format!("Error reading response JSON: {}", e))?;
            let response_data: AiMsgResponse = serde_json::from_value(json).map_err(|e| format!("Error parsing OpenAI response: {}", e))?;
            Ok(response_data)
        } else {
        let status = response.status();
        return Err(format!("Open Ai Error! Status: {status}       Err Msgs: {}", match response.text().await {
            Ok(data) => data,
            Err(e) => format!("Error parsing Open Ai response: {}", e)
        }))
    }
}