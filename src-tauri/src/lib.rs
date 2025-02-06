use serde::{Deserialize, Serialize};
use std::fs;


#[derive(Serialize, Deserialize)]
struct Settings {
    endpoint: String,
    model_name: String,
}

#[derive(Serialize, Deserialize)]
struct OllamaResponse {
    response: String,
}


fn load_settings() -> Result<Settings, String> {
    let settings_file = fs::read_to_string("settings.json")
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;
    
    serde_json::from_str(&settings_file)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))
}

#[tauri::command]
async fn query_ollama(prompt: String) -> Result<String, String> {
    let settings = load_settings()?;
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/generate", settings.endpoint))
        .json(&serde_json::json!({
            "model": settings.model_name,
            "prompt": prompt,
            "stream": false
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let response_text = response.text().await.map_err(|e| e.to_string())?;
    Ok(response_text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![query_ollama])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}