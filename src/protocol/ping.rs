use crate::error::{ProxyError, Result};
use crate::protocol::varint;
use crate::protocol::versions::PROTO_1_8;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Minecraft Status Ping result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub description: String, // Raw MOTD JSON string
    pub online: i32,
    pub max: i32,
    pub version_name: String,
    pub favicon: Option<String>, // base64 data:image/png;base64,...
    pub latency_ms: u64,
}

/// Send Status Ping to Minecraft server, return result
pub async fn ping_server(addr: &str, port: u16) -> Result<PingResult> {
    let connect_timeout = Duration::from_secs(5);
    let target = format!("{}:{}", addr, port);

    let mut stream = timeout(connect_timeout, TcpStream::connect(&target))
        .await
        .map_err(|_| ProxyError::Other(format!("Connection timeout to {}", target)))?
        .map_err(|e| ProxyError::Other(format!("Failed to connect to {}: {}", target, e)))?;

    stream.set_nodelay(true).ok();

    // 1. Send Handshake packet (protocol_version=47, next_state=1)
    let handshake_data = {
        let mut buf = Vec::new();
        varint::encode_varint(PROTO_1_8, &mut buf); // protocol version
        varint::encode_string(addr, &mut buf);
        buf.extend_from_slice(&port.to_be_bytes());
        varint::encode_varint(1, &mut buf); // next_state = Status
        buf
    };
    write_packet(&mut stream, 0x00, &handshake_data).await?;

    // 2. Send Status Request (0x00, empty data)
    write_packet(&mut stream, 0x00, &[]).await?;

    // 3. Read Status Response
    let (packet_id, status_data) = read_packet(&mut stream).await?;
    if packet_id != 0x00 {
        return Err(ProxyError::Protocol(format!(
            "Expected status response (0x00), got {:#x}",
            packet_id
        )));
    }

    // Parse JSON string
    let (json_str, _) = varint::read_string_from_bytes(&status_data)
        .map_err(|e| ProxyError::Other(format!("Failed to parse status JSON: {}", e)))?;

    // 4. Send Ping and measure time
    let ping_payload: u64 = 0x0123456789ABCDEF;
    let ping_start = Instant::now();
    {
        let mut ping_data = Vec::new();
        ping_data.extend_from_slice(&ping_payload.to_be_bytes());
        write_packet(&mut stream, 0x01, &ping_data).await?;
    }

    // 5. Read Pong (allow timeout)
    let latency_ms = match timeout(Duration::from_secs(3), read_packet(&mut stream)).await {
        Ok(Ok(_)) => ping_start.elapsed().as_millis() as u64,
        _ => ping_start.elapsed().as_millis() as u64,
    };

    // Parse JSON
    let parsed = parse_status_json(&json_str);

    Ok(PingResult {
        description: parsed.0,
        online: parsed.1,
        max: parsed.2,
        version_name: parsed.3,
        favicon: parsed.4,
        latency_ms,
    })
}

/// Parse Status JSON, return (description, online, max, version_name, favicon)
fn parse_status_json(json: &str) -> (String, i32, i32, String, Option<String>) {
    let default = (json.to_string(), 0, 0, "".to_string(), None);
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else {
        return default;
    };

    let description = match v.get("description") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(d) => d.to_string(),
        None => String::new(),
    };

    let online = v
        .get("players")
        .and_then(|p| p.get("online"))
        .and_then(|n| n.as_i64())
        .unwrap_or(0) as i32;

    let max = v
        .get("players")
        .and_then(|p| p.get("max"))
        .and_then(|n| n.as_i64())
        .unwrap_or(0) as i32;

    let version_name = v
        .get("version")
        .and_then(|ver| ver.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    let favicon = v
        .get("favicon")
        .and_then(|f| f.as_str())
        .map(String::from);

    (description, online, max, version_name, favicon)
}

/// Write a packet (varint length prefix + varint packet_id + data)
async fn write_packet(stream: &mut TcpStream, packet_id: u8, data: &[u8]) -> Result<()> {
    let mut id_buf = Vec::new();
    varint::encode_varint(packet_id as i32, &mut id_buf);

    let total_len = id_buf.len() + data.len();
    let mut len_buf = Vec::new();
    varint::encode_varint(total_len as i32, &mut len_buf);

    stream.write_all(&len_buf).await?;
    stream.write_all(&id_buf).await?;
    stream.write_all(data).await?;
    Ok(())
}

/// Read a packet, return (packet_id, data)
async fn read_packet(stream: &mut TcpStream) -> Result<(u8, Vec<u8>)> {
    // Read length varint
    let length = read_varint_async(stream).await? as usize;
    if length == 0 {
        return Ok((0, Vec::new()));
    }

    // Read packet body
    let mut body = vec![0u8; length];
    stream.read_exact(&mut body).await?;

    // Parse packet_id
    let (packet_id, consumed) = varint::read_varint_from_bytes(&body)
        .map_err(|e| ProxyError::Other(format!("Failed to read packet id: {}", e)))?;
    let data = body[consumed..].to_vec();

    Ok((packet_id as u8, data))
}

/// Read varint from async stream
async fn read_varint_async(stream: &mut TcpStream) -> Result<i32> {
    let mut result: i32 = 0;
    let mut shift = 0u32;
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).await?;
        let b = byte[0];
        result |= ((b & 0x7F) as i32) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 35 {
            return Err(ProxyError::Protocol("Varint too long".into()));
        }
    }
    Ok(result)
}
