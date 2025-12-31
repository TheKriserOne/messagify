use std::path::PathBuf;
use tracing::error;

fn get_tokens_file_path() -> PathBuf {
    // Store in project root, outside src-tauri to avoid rebuilds
    PathBuf::from("../tokens.json")
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StoredDiscordToken {
    token: Option<String>,
}

pub async fn load_discord_token_from_file() -> std::io::Result<Option<String>> {
    let path = get_tokens_file_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&path).await?;

    // New format: { "token": "..." } (or null)
    if let Ok(stored) = serde_json::from_str::<StoredDiscordToken>(&content) {
        return Ok(stored.token);
    }

    // Backward-compat: old format was an array of objects like:
    // [{ "token": "...", "messanger_type": "Discord" }]
    // We'll pick the first token if present.
    match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(serde_json::Value::Array(arr)) => {
            let token = arr.into_iter().find_map(|v| {
                v.get("token")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
            });
            Ok(token)
        }
        Ok(_) => Ok(None),
        Err(e) => {
            error!("Failed to parse tokens file: {}", e);
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Parse error: {}", e),
            ))
        }
    }
}

pub async fn save_discord_token_to_file(token: Option<&str>) -> std::io::Result<()> {
    let payload = StoredDiscordToken {
        token: token.map(|s| s.to_string()),
    };

    let json = serde_json::to_string_pretty(&payload).map_err(|e| {
        error!("Failed to serialize token: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, format!("Serialize error: {}", e))
    })?;

    tokio::fs::write(get_tokens_file_path(), json)
        .await
        .map_err(|e| {
            error!("Failed to write tokens to file: {}", e);
            e
        })
}
