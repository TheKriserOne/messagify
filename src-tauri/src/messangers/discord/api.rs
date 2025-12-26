use crate::AppState;
use tauri::State;
use tracing::{error, instrument};

#[tauri::command]
#[instrument(skip(state, token))]
pub async fn set_token(state: State<'_, AppState>, token: String) -> Result<String, ()> {
    // info!("Attempting to validate Discord token");

    let response = reqwest::Client::new()
        .get("https://discord.com/api/v10/users/@me")
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| {
            error!("Request failed: {}", e);
        })?;

    if !response.status().is_success() {
        error!("Discord API error: {}", response.status());
        return Err(());
    }

    *state.token.lock().unwrap() = Some(token);
    response.text().await.map_err(|e| {
        error!("Parsing failed: {}", e);
    })
}

#[tauri::command]
#[instrument(skip(state))]
pub fn get_token(state: State<'_, AppState>) -> Option<String> {
    state.token.lock().unwrap().clone()
}
