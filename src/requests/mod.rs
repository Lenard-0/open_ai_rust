use reqwest::Client;

use crate::{logoi::{input::payload::ChatPayLoad, output::AiMsgResponse}, API_KEY};


pub async fn open_ai_msg(
    payload: ChatPayLoad
) -> Result<AiMsgResponse, String> {
    let url = "https://api.openai.com/v1/chat/completions";

    let client = Client::new();

    let api_key = {
        match API_KEY.lock() {
            Ok(key) => key.clone(),
            Err(e) => return Err(format!("Error getting API key from Mutex lock: {}", e))
        }
    };

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
        let response_data: AiMsgResponse = match &response.json::<AiMsgResponse>().await {
            Ok(data) => data.clone(),
            Err(e) => return Err(format!("Error parsing Open Ai response: {}", e))
        };
        return Ok(response_data)
    } else {
        let status = response.status();
        return Err(format!("Open Ai Error! Status: {status}       Err Msgs: {}", match response.text().await {
            Ok(data) => data,
            Err(e) => format!("Error parsing Open Ai response: {}", e)
        }))
    }
}