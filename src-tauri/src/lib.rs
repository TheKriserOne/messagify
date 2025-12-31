use tauri::{Manager, State};
use tokio::sync::Mutex;
use tracing::{error, info, instrument};
use tracing_subscriber::EnvFilter;

use crate::messangers::{
    discord::api::{
        check_discord_token, fetch_guild_channels, fetch_user_channels, fetch_user_guilds,
    },
    token_storage,
};

mod messangers;

pub struct AppState {
    token: Mutex<Option<String>>,
}

#[instrument]
async fn load_validate_and_persist_token() -> Option<String> {
    let loaded_token = match token_storage::load_discord_token_from_file().await {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to load token from file: {}", e);
            None
        }
    };

    let Some(token) = loaded_token else {
        info!("No token found in file");
        return None;
    };

    info!("Token found in file, validating...");

    let is_valid = check_discord_token(&token).await.unwrap_or(false);
    if !is_valid {
        info!("Stored token is invalid, clearing");
        if let Err(e) = token_storage::save_discord_token_to_file(None).await {
            error!("Failed to clear token file: {}", e);
        }
        return None;
    }

    // Persist normalized new-format file even if it was in the old array format.
    if let Err(e) = token_storage::save_discord_token_to_file(Some(&token)).await {
        error!("Failed to save token to file: {}", e);
    }

    Some(token)
}

#[tauri::command]
async fn set_token(state: State<'_, AppState>, token: String) -> Result<bool, ()> {
    if !check_discord_token(&token).await.unwrap_or(false) {
        return Ok(false);
    }

    {
        let mut guard = state.token.lock().await;
        *guard = Some(token.clone());
    };

    if let Err(e) = token_storage::save_discord_token_to_file(Some(&token)).await {
        error!("Failed to save token to file: {}", e);
        return Ok(false);
    }

    info!("Token saved successfully");
    Ok(true)
}

#[tauri::command]
#[instrument(skip(state))]
async fn get_token(state: State<'_, AppState>) -> Result<String, ()> {
    if let Some(token) = state.token.lock().await.clone() {
        return Ok(token);
    } else {
        return Err(());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("messagify=debug,info")),
        )
        .with_file(true)
        .with_line_number(true)
        .without_time()
        .init();

    info!("Starting Messagify application");

    tauri::Builder::default()
        .setup(|app| {
            let token = tauri::async_runtime::block_on(load_validate_and_persist_token());

            // Store tokens in application state
            app.manage(AppState {
                token: Mutex::new(token),
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_token,
            set_token,
            fetch_user_guilds,
            fetch_guild_channels,
            fetch_user_channels
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
