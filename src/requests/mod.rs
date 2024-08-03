use reqwest::Client;
use serde_json::Value;

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

    let payload_as_json = match serde_json::to_value(&payload) {
        Ok(data) => data,
        Err(e) => return Err(format!("Error serializing payload to JSON: {}", e))
    };

    println!("Payload: {:#?}", payload_as_json);

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
            println!("Response JSON: {:#?}", json);
            let response_data: AiMsgResponse = serde_json::from_value(json).map_err(|e| format!("Error parsing OpenAI response: {}", e))?;
            println!("Parsed Response: {:#?}", response_data);
            Ok(response_data)
        } else {
        let status = response.status();
        return Err(format!("Open Ai Error! Status: {status}       Err Msgs: {}", match response.text().await {
            Ok(data) => data,
            Err(e) => format!("Error parsing Open Ai response: {}", e)
        }))
    }
}