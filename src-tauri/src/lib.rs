use bytes;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{Emitter, Window};

#[derive(Serialize, Deserialize)]
struct Settings {
    endpoint: String,
    model_name: String,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

fn load_settings() -> Result<Settings, String> {
    let settings_file = fs::read_to_string("settings.json")
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;

    serde_json::from_str(&settings_file)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))
}

#[tauri::command]
async fn query_ollama(window: Window, conversation: Vec<Message>) -> Result<(), String> {
    let settings = load_settings()?;
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "model": settings.model_name,
        "messages": conversation,
        "stream": true
    });

    let url = format!("{}/api/chat", settings.endpoint);

    let mut response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut buffer = bytes::BytesMut::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        buffer.extend_from_slice(&chunk);

        while let Some(pos) = buffer.windows(1).position(|w| w == b"\n") {
            let line = buffer.split_to(pos + 1);
            let line = &line[..line.len() - 1];

            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&line) {
                if let Some(response_chunk) = json
                    .get("message")
                    .and_then(|message| message.get("content"))
                    .and_then(|v| v.as_str())
                {
                    window
                        .emit("ollama-chunk", response_chunk)
                        .map_err(|e| format!("Failed to emit event: {}", e))?;
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
fn get_settings() -> Result<Settings, String> {
    let settings = load_settings();
    return settings;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![query_ollama, get_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
