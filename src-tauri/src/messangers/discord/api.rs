use tauri::State;
use tracing::error;

use crate::AppState;

pub async fn check_discord_token(token: &str) -> Result<bool, ()> {
    let response = reqwest::Client::new()
        .get("https://discord.com/api/v10/users/@me")
        .header("Authorization", token)
        .send()
        .await
        .map_err(|e| {
            error!("Request failed: {}", e);
        })?;

    if response.status().is_success() {
        return Ok(true);
    } else {
        error!("Token validation failed: {}", response.status());
        return Ok(false);
    }
}

#[tauri::command]
pub async fn fetch_user_guilds(state: State<'_, AppState>) -> Result<String, String> {
    let response = reqwest::Client::new()
        .get("https://discord.com/api/v10/users/@me/guilds")
        .header("Authorization", state.token.lock().await.as_ref().unwrap())
        .send()
        .await
        .map_err(|e| {
            error!("Request failed: {}", e);
            "request_failed".to_string()
        })?;

    if !response.status().is_success() {
        error!("Guild fetch failed: {}", response.status());
        return Err(format!("http_{}", response.status().as_u16()));
    }

    response.text().await.map_err(|e| {
        error!("Failed to parse guild response: {}", e);
        "parse_failed".to_string()
    })
}

#[tauri::command]
pub async fn fetch_guild_channels(
    state: State<'_, AppState>,
    guild_id: String,
) -> Result<String, String> {
    let url = format!("https://discord.com/api/v10/guilds/{}/channels", guild_id);

    let token = state
        .token
        .lock()
        .await
        .clone()
        .ok_or_else(|| "not_authenticated".to_string())?;

    let response = reqwest::Client::new()
        .get(&url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| {
            error!("Request failed: {}", e);
            "request_failed".to_string()
        })?;

    if !response.status().is_success() {
        error!("Guild channels fetch failed: {}", response.status());
        return Err(format!("http_{}", response.status().as_u16()));
    }

    response.text().await.map_err(|e| {
        error!("Failed to parse guild channels response: {}", e);
        "parse_failed".to_string()
    })
}

#[tauri::command]
pub async fn fetch_user_channels(state: State<'_, AppState>) -> Result<String, String> {
    let token = state
        .token
        .lock()
        .await
        .clone()
        .ok_or_else(|| "not_authenticated".to_string())?;

    let response = reqwest::Client::new()
        .get("https://discord.com/api/v10/users/@me/channels")
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| {
            error!("Request failed: {}", e);
            "request_failed".to_string()
        })?;

    if !response.status().is_success() {
        error!("User channels fetch failed: {}", response.status());
        return Err(format!("http_{}", response.status().as_u16()));
    }

    response.text().await.map_err(|e| {
        error!("Failed to parse user channels response: {}", e);
        "parse_failed".to_string()
    })
}

#[tauri::command]
pub async fn fetch_channel_messages(
    state: State<'_, AppState>,
    channel_id: String,
    limit: Option<u32>,
    before: Option<String>,
) -> Result<String, String> {
    let token = state
        .token
        .lock()
        .await
        .clone()
        .ok_or_else(|| "not_authenticated".to_string())?;

    let mut url = format!(
        "https://discord.com/api/v10/channels/{}/messages",
        channel_id
    );
    
    // Build query parameters
    let mut query_params = Vec::new();
    if let Some(limit) = limit {
        query_params.push(format!("limit={}", limit));
    }
    if let Some(before) = before {
        query_params.push(format!("before={}", before));
    }
    
    if !query_params.is_empty() {
        url = format!("{}?{}", url, query_params.join("&"));
    }

    let response = reqwest::Client::new()
        .get(&url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| {
            error!("Request failed: {}", e);
            "request_failed".to_string()
        })?;

    if !response.status().is_success() {
        error!("Channel messages fetch failed: {}", response.status());
        return Err(format!("http_{}", response.status().as_u16()));
    }

    response.text().await.map_err(|e| {
        error!("Failed to parse channel messages response: {}", e);
        "parse_failed".to_string()
    })
}

