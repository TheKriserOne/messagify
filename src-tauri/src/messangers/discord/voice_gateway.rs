use std::net::SocketAddr;
use std::sync::Arc;

use futures::{SinkExt, StreamExt, stream::SplitSink};
use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, http::Request};
use tokio::{
    net::UdpSocket,
    sync::mpsc,
    time::{Duration, interval},
};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, frame::coding::CloseCode};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, error, info, warn};

// Voice Gateway opcodes
pub const VOICE_OP_IDENTIFY: u8 = 0;
pub const VOICE_OP_SELECT_PROTOCOL: u8 = 1;
pub const VOICE_OP_READY: u8 = 2;
pub const VOICE_OP_HEARTBEAT: u8 = 3;
pub const VOICE_OP_SESSION_DESCRIPTION: u8 = 4;
pub const VOICE_OP_SPEAKING: u8 = 5;
pub const VOICE_OP_HEARTBEAT_ACK: u8 = 6;
#[allow(dead_code)]
pub const VOICE_OP_RESUME: u8 = 7;
pub const VOICE_OP_HELLO: u8 = 8;
pub const VOICE_OP_RESUMED: u8 = 9;
pub const VOICE_OP_CLIENT_CONNECT: u8 = 12;
pub const VOICE_OP_CLIENT_DISCONNECT: u8 = 13;

// Voice Gateway payload structures
#[derive(Debug, Deserialize)]
pub struct VoiceGatewayPayload {
    pub op: u8,
    pub d: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct VoiceReadyData {
    pub ssrc: u32,
    pub ip: String,
    pub port: u16,
    pub modes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct VoiceSessionDescription {
    pub mode: String,
    #[serde(rename = "secret_key")]
    pub secret_key: Vec<u8>,
}

pub struct VoiceGatewayClient {
    pub shutdown_tx: Option<mpsc::Sender<()>>,
    pub write_stream:
        Option<SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>,
    pub is_connected: bool,
}

impl VoiceGatewayClient {
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

    pub async fn disconnect(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        self.is_connected = false;
        self.write_stream = None;
    }

    pub async fn connect(
        &mut self,
        app_handle: &AppHandle,
        server_id: String,
        token: String,
        endpoint: String,
    ) -> Result<(), String> {
        if self.is_connected().await {
            info!("Voice gateway already connected, disconnecting first");
            self.disconnect().await;
        }

        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let app_handle = app_handle.clone();
        // Spawn the voice gateway connection task
        tokio::spawn(async move {
            if let Err(e) = run_voice_gateway(app_handle, token, server_id, endpoint, shutdown_rx).await {
                error!("Voice gateway error: {}", e);
            }
        });

        Ok(())
    }
}

impl Drop for VoiceGatewayClient {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

impl Default for VoiceGatewayClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the Discord Voice Gateway connection
/// Follows Discord's Voice Gateway protocol (v4)
/// Documentation: https://discord.com/developers/docs/topics/voice-connections
async fn run_voice_gateway(
    app_handle: AppHandle,
    token: String,
    guild_id: String,
    endpoint: String,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), String> {
    info!("Connecting to Discord Voice Gateway...");

    // Pull user_id/session_id from the already-connected event gateway state
    let state = app_handle.state::<crate::AppState>();
    let user_id = state
        .gateway
        .user_id
        .lock()
        .await
        .clone()
        .ok_or_else(|| "voice gateway: missing user_id (not READY yet?)".to_string())?;
    let session_id = state
        .gateway
        .session_id
        .lock()
        .await
        .clone()
        .ok_or_else(|| "voice gateway: missing session_id (not READY yet?)".to_string())?;

    let voice_url = format!("wss://{}/?v=8", endpoint);
    let (ws_stream, _) = connect_async(&voice_url)
        .await
        .map_err(|e| format!("Voice WebSocket connection failed: {}", e))?;

    let (mut write, mut read) = ws_stream.split();

    info!("Connected to Discord Voice Gateway");

    let mut heartbeat_interval: Option<u64> = None;
    let mut ssrc: Option<u32> = None;
    let mut ip: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut secret_key: Option<Vec<u8>> = None;
    let mut udp_socket: Option<Arc<UdpSocket>> = None;

    // Step 1: Read HELLO message (opcode 8)
    // The server sends this immediately after connection with heartbeat_interval
    if let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(payload) = serde_json::from_str::<VoiceGatewayPayload>(&text) {
                    info!("{:?}", serde_json::from_str::<serde_json::Value>(&text));
                    if payload.op == VOICE_OP_HELLO {
                        if let Some(d) = payload.d {
                            heartbeat_interval = d["heartbeat_interval"].as_u64();
                            info!(
                                "Received VOICE HELLO, heartbeat_interval: {:?} ms",
                                heartbeat_interval
                            );
                        }
                    } else {
                        return Err(format!(
                            "Expected HELLO (opcode 8), got opcode {}",
                            payload.op
                        ));
                    }
                }
            }
            Ok(Message::Close(_)) => {
                return Err("Voice connection closed immediately".to_string());
            }
            Err(e) => {
                return Err(format!("Error reading VOICE HELLO: {}", e));
            }
            _ => {
                return Err("Unexpected message type for HELLO".to_string());
            }
        }
    } else {
        return Err("No HELLO message received".to_string());
    }

    // Step 2: Send IDENTIFY (opcode 0)
    // This identifies the client to the voice server
    let identify = json!({
        "op": VOICE_OP_IDENTIFY,
        "d": {
            "server_id": guild_id,
            "user_id": user_id,
            "session_id": format!("{}", session_id),
            "token": token,
            "max_dave_protocol_version": 1
        }
    });

    write
        .send(Message::Text(identify.to_string().into()))
        .await
        .map_err(|e| format!("Failed to send VOICE IDENTIFY: {}", e))?;

    info!("Sent VOICE IDENTIFY payload");

    // Store write stream + mark connected in the shared state, like event_gateway.rs does.
    if let Some(vg) = state.gateway.voice_gateway.lock().await.as_mut() {
        vg.is_connected = true;
        vg.write_stream = Some(write);
    } else {
        return Err("voice gateway client not initialized".to_string());
    }

    // Step 3: Start heartbeat task
    // Heartbeats keep the connection alive (opcode 3)
    let heartbeat_interval_ms = heartbeat_interval.unwrap_or(41250);
    let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel::<Value>(16);

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(heartbeat_interval_ms));
        loop {
            ticker.tick().await;
            // Discord voice heartbeats accept any monotonically changing nonce.
            // Using epoch millis avoids cross-gateway coupling and prevents panics.
            let nonce: u64 = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let heartbeat = json!({
                "op": VOICE_OP_HEARTBEAT,
                "d": {
                    "t": nonce,
                    "seq_ack": null
                }
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
                info!("Voice gateway shutdown requested");
                let mut guard = state.gateway.voice_gateway.lock().await;
                if let Some(vg) = guard.as_mut() {
                    if let Some(ws) = vg.write_stream.as_mut() {
                        let _ = ws
                            .send(Message::Close(Some(CloseFrame {
                                code: CloseCode::Normal,
                                reason: std::borrow::Cow::Borrowed(""),
                            })))
                            .await;
                    }
                }
                break;
            }

            // Send heartbeats (opcode 3)
            Some(heartbeat) = heartbeat_rx.recv() => {
                let mut guard = state.gateway.voice_gateway.lock().await;
                if let Some(vg) = guard.as_mut() {
                    if let Some(ws) = vg.write_stream.as_mut() {
                        if let Err(e) = ws.send(Message::Text(heartbeat.to_string().into())).await {
                            error!("Failed to send voice heartbeat: {}", e);
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
                debug!("Sent voice heartbeat");
            }


            // Read messages from the voice gateway
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_voice_message(
                            &text,
                            &app_handle,
                            &mut ssrc,
                            &mut ip,
                            &mut port,
                            &mut secret_key,
                            &mut udp_socket,
                        ).await {
                            error!("Error handling voice message: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Close(frame))) => {
                        warn!("Voice gateway closed: {:?}", frame);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("Voice WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("Voice WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Mark disconnected / clear write_stream in shared state (like event_gateway.rs does).
    if let Some(vg) = state.gateway.voice_gateway.lock().await.as_mut() {
        vg.is_connected = false;
        vg.write_stream = None;
    }
    info!("Voice gateway disconnected");

    Ok(())
}

async fn handle_voice_message(
    text: &str,
    app_handle: &AppHandle,
    ssrc: &mut Option<u32>,
    ip: &mut Option<String>,
    port: &mut Option<u16>,
    secret_key: &mut Option<Vec<u8>>,
    udp_socket: &mut Option<Arc<UdpSocket>>,
) -> Result<(), String> {
    let payload: VoiceGatewayPayload =
        serde_json::from_str(text).map_err(|e| format!("Failed to parse voice payload: {}", e))?;
        info!("{}", serde_json::from_str::<serde_json::Value>(&text).unwrap());
    match payload.op {
        VOICE_OP_READY => {
            // Step 4: Receive READY (opcode 2) with SSRC, IP, port, and encryption modes
            if let Some(d) = payload.d {
                let ready_data: VoiceReadyData = serde_json::from_value(d.clone())
                    .map_err(|e| format!("Failed to parse VOICE READY: {}", e))?;

                *ssrc = Some(ready_data.ssrc);
                *ip = Some(ready_data.ip.clone());
                *port = Some(ready_data.port);

                info!(
                    "VOICE READY: ssrc={}, ip={}, port={}, modes={:?}",
                    ready_data.ssrc, ready_data.ip, ready_data.port, ready_data.modes
                );

                // Find a supported encryption mode (prefer xsalsa20_poly1305)
                let mode = ready_data
                    .modes
                    .iter()
                    .find(|m| *m == "xsalsa20_poly1305")
                    .or_else(|| ready_data.modes.first())
                    .ok_or_else(|| "No supported encryption mode".to_string())?;

                // Step 5: Send SELECT_PROTOCOL (opcode 1)
                // This selects UDP transport and encryption mode
                let select_protocol = json!({
                    "op": VOICE_OP_SELECT_PROTOCOL,
                    "d": {
                        "protocol": "udp",
                        "data": {
                            "address": ready_data.ip,
                            "port": ready_data.port,
                            "mode": mode
                        }
                    }
                });

                let state = app_handle.state::<crate::AppState>();
                let mut guard = state.gateway.voice_gateway.lock().await;
                let vg = guard
                    .as_mut()
                    .ok_or_else(|| "voice gateway client not initialized".to_string())?;
                let ws = vg
                    .write_stream
                    .as_mut()
                    .ok_or_else(|| "voice gateway write_stream not available".to_string())?;
                ws.send(Message::Text(select_protocol.to_string().into()))
                    .await
                    .map_err(|e| format!("Failed to send SELECT_PROTOCOL: {}", e))?;
                info!("Sent SELECT_PROTOCOL with mode: {}", mode);
            }
        }

        VOICE_OP_SESSION_DESCRIPTION => {
            // Step 6: Receive SESSION_DESCRIPTION (opcode 4) with encryption secret key
            if let Some(d) = payload.d {
                let session: VoiceSessionDescription = serde_json::from_value(d.clone())
                    .map_err(|e| format!("Failed to parse SESSION_DESCRIPTION: {}", e))?;

                *secret_key = Some(session.secret_key.clone());

                info!(
                    "Received SESSION_DESCRIPTION, mode: {}, key length: {}",
                    session.mode,
                    session.secret_key.len()
                );

                // Now we can establish UDP connection for audio transport
                if let (Some(ssrc_val), Some(ip_val), Some(port_val), Some(key)) = (
                    ssrc.as_ref(),
                    ip.as_ref(),
                    port.as_ref(),
                    secret_key.as_ref(),
                ) {
                    if let Err(e) =
                        establish_udp_connection(*ssrc_val, ip_val, *port_val, key, udp_socket)
                            .await
                    {
                        error!("Failed to establish UDP connection: {}", e);
                    } else {
                        info!("Voice connection fully established and ready for audio");
                    }
                }
            }
        }

        VOICE_OP_HEARTBEAT_ACK => {
            debug!("Received voice heartbeat ACK");
            debug!("{:?}", payload);
        }

        VOICE_OP_SPEAKING => {
            debug!("Received SPEAKING event");
        }

        VOICE_OP_RESUMED => {
            info!("Voice gateway resumed");
        }

        VOICE_OP_CLIENT_CONNECT => {
            debug!("Client connected to voice channel");
        }

        VOICE_OP_CLIENT_DISCONNECT => {
            debug!("Client disconnected from voice channel");
        }

        _ => {
            debug!("Received voice opcode: {:?}", payload);
        }
    }

    Ok(())
}

async fn establish_udp_connection(
    ssrc: u32,
    ip: &str,
    port: u16,
    secret_key: &[u8],
    udp_socket: &mut Option<Arc<UdpSocket>>,
) -> Result<(), String> {
    info!("Establishing UDP connection to {}:{}", ip, port);

    // Create UDP socket bound to any available port
    let local_addr = "0.0.0.0:0";
    let socket = UdpSocket::bind(local_addr)
        .await
        .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;

    let remote_addr: SocketAddr = format!("{}:{}", ip, port)
        .parse()
        .map_err(|e| format!("Invalid remote address: {}", e))?;

    // Send IP discovery packet (70 bytes)
    // Format: [type: 2 bytes][length: 2 bytes][SSRC: 4 bytes][padding: 62 bytes]
    let mut discovery_packet = vec![0u8; 70];
    discovery_packet[0] = 0x01; // Type: request
    discovery_packet[1] = 0x00; // Length (high byte)
    discovery_packet[2] = 0x46; // Length (low byte) = 70
    discovery_packet[3] = 0x00;
    // SSRC in bytes 4-7 (big endian)
    discovery_packet[4] = ((ssrc >> 24) & 0xFF) as u8;
    discovery_packet[5] = ((ssrc >> 16) & 0xFF) as u8;
    discovery_packet[6] = ((ssrc >> 8) & 0xFF) as u8;
    discovery_packet[7] = (ssrc & 0xFF) as u8;

    socket
        .send_to(&discovery_packet, &remote_addr)
        .await
        .map_err(|e| format!("Failed to send discovery packet: {}", e))?;

    info!("Sent UDP IP discovery packet");

    // Receive IP discovery response
    let mut buffer = [0u8; 70];
    let (size, _) = socket
        .recv_from(&mut buffer)
        .await
        .map_err(|e| format!("Failed to receive IP discovery: {}", e))?;

    if size < 70 {
        return Err("Invalid IP discovery response".to_string());
    }

    // Extract our external IP from the response (bytes 4-7 are our IP)
    let our_ip = format!("{}.{}.{}.{}", buffer[4], buffer[5], buffer[6], buffer[7]);
    // Extract our port from the response (bytes 58-59)
    let our_port = u16::from_be_bytes([buffer[58], buffer[59]]);

    info!(
        "UDP connection established. External IP: {}, Port: {}",
        our_ip, our_port
    );
    info!("Secret key length: {} bytes", secret_key.len());

    *udp_socket = Some(Arc::new(socket));

    Ok(())
}
