// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::State;

struct AppState {
    token: Mutex<Option<String>>,
}

#[tauri::command]
async fn set_token(state: State<'_, AppState>, token: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    
    // Fetch current user from Discord API to validate token
    let response = client
        .get("https://discord.com/api/v10/users/@me")
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Discord API error: {}", response.status()));
    }

    // Store the token
    *state.token.lock().unwrap() = Some(token);
    Ok(())
}

#[tauri::command]
fn get_token(state: State<AppState>) -> Option<String> {
    state.token.lock().unwrap().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            token: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![set_token, get_token])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
