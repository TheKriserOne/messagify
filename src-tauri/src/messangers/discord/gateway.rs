use std::sync::Arc;

use flate2::read::ZlibDecoder;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use tokio::{
    sync::{Mutex, mpsc},
    time::{Duration, interval},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";

// Gateway opcodes
const OP_DISPATCH: u8 = 0;
const OP_HEARTBEAT: u8 = 1;
const OP_IDENTIFY: u8 = 2;
#[allow(dead_code)]
const OP_RESUME: u8 = 6;
const OP_RECONNECT: u8 = 7;
const OP_INVALID_SESSION: u8 = 9;
const OP_HELLO: u8 = 10;
const OP_HEARTBEAT_ACK: u8 = 11;

#[derive(Debug, Deserialize)]
struct GatewayPayload {
    op: u8,
    d: Option<Value>,
    s: Option<u64>,
    t: Option<String>,
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

pub struct GatewayClient {
    shutdown_tx: Option<mpsc::Sender<()>>,
    is_connected: Arc<Mutex<bool>>,
}

impl GatewayClient {
    pub fn new() -> Self {
        Self {
            shutdown_tx: None,
            is_connected: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn is_connected(&self) -> bool {
        *self.is_connected.lock().await
    }

    pub async fn connect(&mut self, token: String, app_handle: AppHandle) -> Result<(), String> {
        if self.is_connected().await {
            info!("Gateway already connected, disconnecting first");
            self.disconnect().await;
        }

        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let is_connected = self.is_connected.clone();

        // Spawn the Gateway connection task
        tokio::spawn(async move {
            if let Err(e) = run_gateway(token, app_handle.clone(), shutdown_rx, is_connected).await
            {
                error!("Gateway error: {}", e);
                let _ = app_handle.emit("discord-gateway", GatewayEvent::GatewayError(e));
            }
        });

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        *self.is_connected.lock().await = false;
    }
}

impl Default for GatewayClient {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_gateway(
    token: String,
    app_handle: AppHandle,
    mut shutdown_rx: mpsc::Receiver<()>,
    is_connected: Arc<Mutex<bool>>,
) -> Result<(), String> {
    info!("Connecting to Discord Gateway...");

    let (ws_stream, _) = connect_async(GATEWAY_URL)
        .await
        .map_err(|e| format!("WebSocket connection failed: {}", e))?;

    let (mut write, mut read) = ws_stream.split();

    *is_connected.lock().await = true;
    let _ = app_handle.emit("discord-gateway", GatewayEvent::Connected);
    info!("Connected to Discord Gateway");

    let mut heartbeat_interval: Option<u64> = None;
    let mut last_sequence: Option<u64> = None;
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
            "intents": 1 << 12 | 1 << 9 | 1 << 15 // GUILDS | DIRECT_MESSAGES | MESSAGE_CONTENT
        }
    });

    write
        .send(Message::Text(identify.to_string().into()))
        .await
        .map_err(|e| format!("Failed to send IDENTIFY: {}", e))?;

    info!("Sent IDENTIFY payload");

    // Start heartbeat task
    let heartbeat_interval_ms = heartbeat_interval.unwrap_or(41250);
    let heartbeat_ack = heartbeat_ack_received.clone();
    let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel::<Value>(16);

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(heartbeat_interval_ms));
        loop {
            ticker.tick().await;

            // Check if we received ACK for the last heartbeat
            {
                let mut ack = heartbeat_ack.lock().await;
                if !*ack {
                    warn!("No heartbeat ACK received, connection may be zombied");
                }
                *ack = false;
            }

            let heartbeat = json!({
                "op": OP_HEARTBEAT,
                "d": null // TODO: send last_sequence
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
                let _ = write.send(Message::Close(None)).await;
                break;
            }

            // Send heartbeats
            Some(heartbeat) = heartbeat_rx.recv() => {
                if let Err(e) = write.send(Message::Text(heartbeat.to_string().into())).await {
                    error!("Failed to send heartbeat: {}", e);
                    break;
                }
                debug!("Sent heartbeat");
            }

            // Read messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_message(&text, &app_handle, &mut last_sequence, &heartbeat_ack_received).await {
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
                            if let Err(e) = handle_message(&text, &app_handle, &mut last_sequence, &heartbeat_ack_received).await {
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

    *is_connected.lock().await = false;
    let _ = app_handle.emit("discord-gateway", GatewayEvent::Disconnected);
    info!("Gateway disconnected");

    Ok(())
}

async fn handle_message(
    text: &str,
    app_handle: &AppHandle,
    last_sequence: &mut Option<u64>,
    heartbeat_ack: &Arc<Mutex<bool>>,
) -> Result<(), String> {
    let payload: GatewayPayload =
        serde_json::from_str(text).map_err(|e| format!("Failed to parse payload: {}", e))?;
    // Update sequence number
    if let Some(s) = payload.s {
        *last_sequence = Some(s);
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
    let event = match event_type {
        "READY" => {
            info!("Gateway READY");
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
