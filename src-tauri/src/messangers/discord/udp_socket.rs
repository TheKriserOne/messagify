use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;
use tracing::info;

pub struct UdpConnection {
    pub socket: Arc<UdpSocket>,
    pub remote_addr: SocketAddr,
    pub external_ip: String,
    pub external_port: u16,
}

pub async fn establish_udp_connection(
    ssrc: u32,
    ip: &str,
    port: u16,
    secret_key: &[u8],
) -> Result<UdpConnection, String> {
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

    let socket = Arc::new(socket);
    Ok(UdpConnection {
        socket,
        remote_addr,
        external_ip: our_ip,
        external_port: our_port,
    })
}