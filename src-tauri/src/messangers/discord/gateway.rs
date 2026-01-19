use futures::SinkExt;
use serde_json::json;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::AppState;

pub use crate::messangers::discord::event_gateway::EventGatewayClient;
pub use crate::messangers::discord::voice_gateway::VoiceGatewayClient;

pub struct GatewayClient {
    pub event_gateway: Mutex<Option<EventGatewayClient>>,
    pub voice_gateway: Mutex<VoiceGatewayClient>,
    pub last_sequence: Mutex<Option<u64>>,
    pub user_id: Mutex<Option<String>>,
    pub session_id: Mutex<Option<String>>,
}

impl GatewayClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn connect(&self, token: String, app_handle: AppHandle) -> Result<(), String> {
        let mut guard = self.event_gateway.lock().await;
        let event = guard.get_or_insert_with(EventGatewayClient::new);
        let _ = token;
        event.connect(app_handle).await
    }

    /// Drops the event gateway client (by taking it out of the Option).
    /// This triggers `Drop` on `EventGatewayClient`, which will best-effort close the socket.
    pub async fn disconnect_event(&self) {
        let mut event = self.event_gateway.lock().await;
        *event = None;
    }
}

impl Default for GatewayClient {
    fn default() -> Self {
        Self {
            event_gateway: Some(EventGatewayClient::new()).into(),
            voice_gateway: VoiceGatewayClient::new().into(),
            last_sequence: Mutex::new(None).into(),
            user_id: Mutex::new(None).into(),
            session_id: Mutex::new(None).into(),
        }
    }
}

#[tauri::command]
pub async fn voice_init(
    state: State<'_, AppState>,
    guild_id: String,
    channel_id: String,
) -> Result<(), String> {
    let voice_payload = json!({
        "op": 4,  // VOICE_STATE_UPDATE opcode
        "d": {
            "guild_id": guild_id,
            "channel_id": channel_id,  // null to disconnect
            "self_mute": false,
            "self_deaf": false
        }
    });

    let mut gw_guard = state.gateway.event_gateway.lock().await;
    let client = gw_guard
        .as_mut()
        .ok_or_else(|| "event gateway client not initialized".to_string())?;
    let write = client
        .write_stream
        .as_mut()
        .ok_or_else(|| "event gateway not connected".to_string())?;

    write
        .send(Message::Text(voice_payload.to_string().into()))
        .await
        .map_err(|e| format!("Failed to send VOICE_STATE_UPDATE: {}", e))?;
    Ok(())
}
