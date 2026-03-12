// use reqwest::multipart;
// use serde::Deserialize;
// use crate::helpers::{get_key, get_url};


// #[derive(Deserialize)]
// struct WhisperResponse {
//     text: String,
// }

// pub async fn open_ai_speech_to_text(
//     mp3_bytes: Vec<u8>,
// ) -> Result<String, String> {
//     let client = reqwest::Client::new();

//     let part = multipart::Part::bytes(mp3_bytes)
//         .file_name("audio.mp3")
//         .mime_str("audio/mpeg")
//         .map_err(|e| format!("Error creating multipart part: {}", e))?;

//     let form = multipart::Form::new()
//         .text("model", "gpt-4o-mini-transcribe")
//         .part("file", part);

//     let url = get_url("/v1/audio/transcriptions")?;

//     let res = client
//         .post(url)
//         .bearer_auth(get_key()?)
//         .multipart(form)
//         .send()
//         .await
//         .map_err(|e| format!("Error sending request to OpenAI: {}", e))?;

//     if !res.status().is_success() {
//         let err_text = res.text().await.map_err(|e| format!("Error reading error response: {}", e))?;
//         return Err(format!("OpenAI API error: {}", err_text));
//     }

//     let transcript: WhisperResponse = res.json().await.map_err(|e| format!("Error parsing response: {}", e))?;

//     Ok(transcript.text)
// }