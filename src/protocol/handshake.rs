use crate::error::Result;
use crate::protocol::packet::RawPacket;
use crate::protocol::varint;

/// Minecraft Handshake packet (Packet ID 0x00, State 0)
#[derive(Debug, Clone)]
pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String, // may contain FML marker (\0FML\0)
    pub server_port: u16,
    pub next_state: i32, // 1 = Status, 2 = Login
}

impl HandshakePacket {
    /// Parse from RawPacket
    pub fn from_raw(packet: &RawPacket) -> Result<Self> {
        let data = &packet.data;
        let mut offset = 0;

        // Protocol version
        let (protocol_version, consumed) = varint::read_varint_from_bytes(&data[offset..])?;
        offset += consumed;

        // Server address
        let (server_address, consumed) = varint::read_string_from_bytes(&data[offset..])?;
        offset += consumed;

        // Server port (unsigned short, big-endian)
        if data.len() < offset + 2 {
            return Err(crate::error::ProxyError::Protocol(
                "Handshake too short for port".into(),
            ));
        }
        let server_port = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Next state
        let (next_state, _) = varint::read_varint_from_bytes(&data[offset..])?;

        Ok(Self {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }

    /// Encode to RawPacket
    pub fn to_raw(&self) -> RawPacket {
        let mut data = Vec::new();
        varint::encode_varint(self.protocol_version, &mut data);
        varint::encode_string(&self.server_address, &mut data);
        data.extend_from_slice(&self.server_port.to_be_bytes());
        varint::encode_varint(self.next_state, &mut data);

        RawPacket::new(0x00, data)
    }

    /// Create modified Handshake with new address and port (forward to backend)
    pub fn with_target(&self, address: &str, port: u16) -> Self {
        // Keep FML marker: if original server_address contains \0,
        // append the part after \0 to the new address
        let fml_suffix = self
            .server_address
            .find('\0')
            .map(|i| &self.server_address[i..])
            .unwrap_or("");

        Self {
            protocol_version: self.protocol_version,
            server_address: format!("{}{}", address, fml_suffix),
            server_port: port,
            next_state: self.next_state,
        }
    }

}

