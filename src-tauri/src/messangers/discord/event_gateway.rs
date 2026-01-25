use std::sync::Arc;

use flate2::read::ZlibDecoder;
use futures::{SinkExt, StreamExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager};
use tokio::{
    sync::{Mutex, mpsc},
    time::{Duration, interval},
};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, frame::coding::CloseCode};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, error, info, warn};

use crate::AppState;

pub const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";

// Gateway opcodes
pub const OP_DISPATCH: u8 = 0;
pub const OP_HEARTBEAT: u8 = 1;
pub const OP_IDENTIFY: u8 = 2;
#[allow(dead_code)]
pub const OP_RESUME: u8 = 6;
pub const OP_RECONNECT: u8 = 7;
pub const OP_INVALID_SESSION: u8 = 9;
pub const OP_HELLO: u8 = 10;
pub const OP_HEARTBEAT_ACK: u8 = 11;

#[derive(Debug, Deserialize)]
pub struct GatewayPayload {
    pub op: u8,
    pub d: Option<Value>,
    pub s: Option<u64>,
    pub t: Option<String>,
}

// Identify payload structs - kept for documentation, using json! macro instead
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct IdentifyPayload {
    token: String,
    properties: IdentifyProperties,
    intents: u32,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct IdentifyProperties {
    os: String,
    browser: String,
    device: String,
}

/// Events emitted to the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum GatewayEvent {
    MessageCreate(Value),
    MessageUpdate(Value),
    MessageDelete(Value),
    Ready(Value),
    GatewayError(String),
    Connected,
    Disconnected,
}

pub struct EventGatewayClient {
    pub shutdown_tx: Option<mpsc::Sender<()>>,
    pub write_stream:
        Option<SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>,
    pub is_connected: bool,
}

impl EventGatewayClient {
    pub fn new() -> Self {
        Self {
            shutdown_tx: None,
            write_stream: None,
            is_connected: false,
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub async fn connect(&mut self, app_handle: AppHandle) -> Result<(), String> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Spawn the Gateway connection task
        tokio::spawn(async move {
            if let Err(e) = run_gateway(app_handle.clone(), shutdown_rx).await {
                error!("Gateway error: {}", e);
                let _ = app_handle.emit("discord-gateway", GatewayEvent::GatewayError(e));
            }
        });

        Ok(())
    }
}

impl Drop for EventGatewayClient {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

impl Default for EventGatewayClient {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_gateway(
    app_handle: AppHandle,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), String> {
    info!("Connecting to Discord Gateway...");

    let state = app_handle.state::<AppState>();
    let (ws_stream, _) = connect_async(GATEWAY_URL)
        .await
        .map_err(|e| format!("WebSocket connection failed: {}", e))?;

    let (mut write, mut read) = ws_stream.split();
    let _ = app_handle.emit("discord-gateway", GatewayEvent::Connected);
    info!("Connected to Discord Gateway");

    let mut heartbeat_interval: Option<u64> = None;
    let heartbeat_ack_received = Arc::new(Mutex::new(true));

    // Read the first message (should be HELLO)
    if let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(payload) = serde_json::from_str::<GatewayPayload>(&text) {
                    if payload.op == OP_HELLO {
                        if let Some(d) = payload.d {
                            heartbeat_interval = d["heartbeat_interval"].as_u64();
                            info!(
                                "Received HELLO, heartbeat_interval: {:?} ms",
                                heartbeat_interval
                            );
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                return Err("Connection closed immediately".to_string());
            }
            Err(e) => {
                return Err(format!("Error reading HELLO: {}", e));
            }
            _ => {}
        }
    }

    *state.gateway.last_sequence.lock().await = None;

    let Some(token) = state.token.lock().await.clone() else {
        return Err("no token found".to_string());
    };
    // Send IDENTIFY
    let identify = json!({
        "op": OP_IDENTIFY,
        "d": {
            "token": token,
            "properties": {
                "os": "windows",
                "browser": "messagify",
                "device": "messagify"
            },
            "intents": 1 << 12 | 1 << 9 | 1 << 15 // GUILDS | GUILD_MESSAGES | DIRECT_MESSAGES | MESSAGE_CONTENT
        }
    });

    
    if let Some(state) = state.gateway.event_gateway.lock().await.as_mut() {
        write
            .send(Message::Text(identify.to_string().into()))
            .await
            .map_err(|e| format!("Failed to send IDENTIFY: {}", e))?;
        state.is_connected = true;
        state.write_stream = Some(write);
    }

    info!("Sent IDENTIFY payload");
    // Start heartbeat task
    let heartbeat_interval_ms = heartbeat_interval.unwrap_or(41250);
    let heartbeat_ack = heartbeat_ack_received.clone();
    let app_handle_clone = app_handle.clone();
    let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel::<Value>(16);

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(heartbeat_interval_ms));
        loop {
            ticker.tick().await;
            let ref_handle = app_handle_clone.state::<AppState>();
            // Check if we received ACK for the last heartbeat
            {
                let mut ack = heartbeat_ack.lock().await;
                if !*ack {
                    warn!("No heartbeat ACK received, connection may be zombied");
                }
                *ack = false;
            }

            // Get the current last_sequence value
            let seq_guard = ref_handle.gateway.last_sequence.lock().await.clone();
            let heartbeat = json!({
                "op": OP_HEARTBEAT,
                "d": seq_guard
            });

            if heartbeat_tx.send(heartbeat).await.is_err() {
                break;
            }
        }
    });
    // Main event loop
    loop {
        tokio::select! {
            // Check for shutdown signal

            Some(_) = shutdown_rx.recv() => {
                info!("Gateway shutdown requested");
                let mut guard = state.gateway.event_gateway.lock().await;
                let guard = guard.as_mut().unwrap();
                let guard = guard.write_stream.as_mut().unwrap();
                let _ = guard
                    .send(Message::Close(Some(CloseFrame {
                        code: CloseCode::Normal,
                        reason: std::borrow::Cow::Borrowed(""),
                    })))
                    .await;
            }

            // Send heartbeats
            Some(heartbeat) = heartbeat_rx.recv() => {
                let mut guard = state.gateway.event_gateway.lock().await;
                let guard = guard.as_mut().unwrap();
                let guard = guard.write_stream.as_mut().unwrap();
                guard.send(Message::Text(heartbeat.to_string().into())).await;
                debug!("Sent heartbeat");
            }

            // Read messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_message(&text, &app_handle, &heartbeat_ack_received).await {
                            error!("Error handling message: {}", e);
                        }
                    }
                    Some(Ok(Message::Close(frame))) => {
                        warn!("Gateway closed: {:?}", frame);
                        break;
                    }
                    Some(Ok(Message::Binary(data))) => {
                        // Handle zlib-compressed messages if needed
                        if let Ok(text) = decompress_zlib(&data) {
                            if let Err(e) = handle_message(&text, &app_handle, &heartbeat_ack_received).await {
                                error!("Error handling compressed message: {}", e);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(state) = state.gateway.event_gateway.lock().await.as_mut() {
        state.is_connected = false;
    }
    let _ = app_handle.emit("discord-gateway", GatewayEvent::Disconnected);
    info!("Gateway disconnected");

    Ok(())
}

async fn handle_message(
    text: &str,
    app_handle: &AppHandle,
    heartbeat_ack: &Arc<Mutex<bool>>,
) -> Result<(), String> {
    let payload: GatewayPayload =
        serde_json::from_str(text).map_err(|e| format!("Failed to parse payload: {}", e))?;
    // Update sequence number
    if let Some(s) = payload.s {
        let state = app_handle.state::<AppState>();
        *state.gateway.last_sequence.lock().await = Some(s);
    }
    match payload.op {
        OP_DISPATCH => {
            if let (Some(event_type), Some(data)) = (payload.t.as_deref(), payload.d) {
                handle_dispatch_event(event_type, data, app_handle).await?;
            }
        }
        OP_HEARTBEAT => {
            debug!("Server requested heartbeat");
            // The heartbeat will be sent by the heartbeat task
        }
        OP_HEARTBEAT_ACK => {
            debug!("Received heartbeat ACK");
            *heartbeat_ack.lock().await = true;
        }
        OP_RECONNECT => {
            warn!("Server requested reconnect");
            return Err("Reconnect requested".to_string());
        }
        OP_INVALID_SESSION => {
            error!("Invalid session");
            return Err("Invalid session".to_string());
        }
        _ => {
            debug!("Received opcode: {}", payload.op);
        }
    }

    Ok(())
}

async fn handle_dispatch_event(
    event_type: &str,
    data: Value,
    app_handle: &AppHandle,
) -> Result<(), String> {
    let state = app_handle.try_state::<AppState>();
    let event = match event_type {
        "READY" => {
            info!("Gateway READY");
            // Extract user_id and session_id from READY event
            if let (Some(user_id), Some(session_id)) =
                (data["user"]["id"].as_str(), data["session_id"].as_str())
            {
                if let Some(state) = state {
                    *state.gateway.user_id.lock().await = Some(user_id.to_string());
                    *state.gateway.session_id.lock().await = Some(session_id.to_string());
                    info!("Stored user_id: {}, session_id: {}", user_id, session_id);
                }
            }

            Some(GatewayEvent::Ready(data))
        }
        "MESSAGE_CREATE" => {
            debug!("MESSAGE_CREATE: channel_id={}", data["channel_id"]);
            Some(GatewayEvent::MessageCreate(data))
        }
        "MESSAGE_UPDATE" => {
            debug!("MESSAGE_UPDATE: message_id={}", data["id"]);
            Some(GatewayEvent::MessageUpdate(data))
        }
        "MESSAGE_DELETE" => {
            debug!("MESSAGE_DELETE: message_id={}", data["id"]);
            Some(GatewayEvent::MessageDelete(data))
        }
        "VOICE_SERVER_UPDATE" => {
            debug!("TRYING TO CONNECT");
            if let Some(state) = state {
                let server_id = data["guild_id"].as_str().unwrap().to_owned();
                let token = data["token"].as_str().unwrap().to_owned();
                let endpoint = data["endpoint"].as_str().unwrap().to_owned();
                // Connect voice gateway
                let mut voice_guard = state.gateway.voice_gateway.lock().await;
                let voice = voice_guard.get_or_insert_with(crate::messangers::discord::voice_gateway::VoiceGatewayClient::new);
                voice
                    .connect(app_handle, server_id, token, endpoint)
                    .await?;
            }
            Some(GatewayEvent::Connected)
        }
        _ => {
            debug!("Unhandled event: {}", event_type);
            None
        }
    };

    if let Some(evt) = event {
        app_handle
            .emit("discord-gateway", evt)
            .map_err(|e| format!("Failed to emit event: {}", e))?;
    }

    Ok(())
}

fn decompress_zlib(data: &[u8]) -> Result<String, String> {
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(data);
    let mut output = String::new();
    decoder
        .read_to_string(&mut output)
        .map_err(|e| format!("Decompression failed: {}", e))?;
    Ok(output)
}
