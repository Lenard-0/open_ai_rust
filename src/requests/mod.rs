use reqwest::Client;
use serde_json::Value;

use crate::{API_KEY, OPENAI_MSG_ENDPOINT, helpers::{get_key, get_url}, logoi::{input::payload::ChatPayLoad, output::AiMsgResponse}};

pub mod embed;
pub mod audio;

pub async fn open_ai_msg(
    payload: ChatPayLoad
) -> Result<AiMsgResponse, String> {
    let client = Client::new();
    let url = get_url("/v1/chat/completions")?;
    let response = match client.post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", get_key()?))
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