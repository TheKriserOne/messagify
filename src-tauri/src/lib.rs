use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use tracing::{error, info, instrument};
use tracing_subscriber::EnvFilter;

use crate::messangers::{
    discord::{
        api::{
            check_discord_token, fetch_channel_messages, fetch_guild_channels, fetch_user_channels,
            fetch_user_guilds, send_message,
        },
        gateway::GatewayClient,
    },
    token_storage,
};

mod messangers;

pub struct AppState {
    token: Mutex<Option<String>>,
    gateway: Mutex<GatewayClient>,
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
async fn set_token(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    token: String,
) -> Result<bool, ()> {
    if !check_discord_token(&token).await.unwrap_or(false) {
        return Ok(false);
    }

    if let Err(e) = token_storage::save_discord_token_to_file(Some(&token)).await {
        error!("Failed to save token to file: {}", e);
        return Ok(false);
    }

    // Store token in state
    *state.token.lock().await = Some(token.clone());

    // Start Gateway connection
    if let Err(e) = state.gateway.lock().await.connect(token, app_handle).await {
        error!("Failed to start Gateway: {}", e);
    }

    info!("Token saved successfully");
    Ok(true)
}

#[tauri::command]
async fn start_gateway(state: State<'_, AppState>, app_handle: AppHandle) -> Result<bool, String> {
    let token = state
        .token
        .lock()
        .await
        .clone()
        .ok_or_else(|| "No token available".to_string())?;

    state
        .gateway
        .lock()
        .await
        .connect(token, app_handle)
        .await?;

    Ok(true)
}

#[tauri::command]
async fn stop_gateway(state: State<'_, AppState>) -> Result<(), ()> {
    state.gateway.lock().await.disconnect().await;
    Ok(())
}

#[tauri::command]
async fn is_gateway_connected(state: State<'_, AppState>) -> Result<bool, ()> {
    Ok(state.gateway.lock().await.is_connected().await)
}

#[tauri::command]
#[instrument(skip(state))]
async fn get_token(state: State<'_, AppState>) -> Result<String, ()> {
    if let Some(token) = state.token.lock().await.clone() {
        Ok(token)
    } else {
        Err(())
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

            // Initialize Gateway client
            let gateway = GatewayClient::new();

            // Auto-start Gateway if token exists
            if let Some(ref tok) = token {
                let tok = tok.clone();
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let mut gw = GatewayClient::new();
                    if let Err(e) = gw.connect(tok, app_handle).await {
                        error!("Failed to auto-start Gateway: {}", e);
                    }
                });
            }

            // Store state
            app.manage(AppState {
                token: Mutex::new(token),
                gateway: Mutex::new(gateway),
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_token,
            set_token,
            start_gateway,
            stop_gateway,
            is_gateway_connected,
            fetch_user_guilds,
            fetch_guild_channels,
            fetch_user_channels,
            fetch_channel_messages,
            send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
