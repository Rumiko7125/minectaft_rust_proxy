use crate::error::Result;
use crate::protocol::packet::RawPacket;
use crate::protocol::varint;

/// Status Request (Packet ID 0x00, no data)
pub fn status_request_packet() -> RawPacket {
    RawPacket::new(0x00, Vec::new())
}

/// Status Response (Packet ID 0x00, JSON string)
pub struct StatusResponse {
    pub json: String,
}

impl StatusResponse {
    pub fn from_raw(packet: &RawPacket) -> Result<Self> {
        let (json, _) = varint::read_string_from_bytes(&packet.data)?;
        Ok(Self { json })
    }

    pub fn to_raw(&self) -> RawPacket {
        let mut data = Vec::new();
        varint::encode_string(&self.json, &mut data);
        RawPacket::new(0x00, data)
    }
}

/// Ping packet (Packet ID 0x01, 8 byte payload)
pub struct PingPacket {
    pub payload: i64,
}

impl PingPacket {
    pub fn from_raw(packet: &RawPacket) -> Result<Self> {
        if packet.data.len() < 8 {
            return Err(crate::error::ProxyError::Protocol(
                "Ping packet too short".into(),
            ));
        }
        let payload = i64::from_be_bytes(packet.data[..8].try_into().unwrap());
        Ok(Self { payload })
    }

    pub fn to_pong(&self) -> RawPacket {
        RawPacket::new(0x01, self.payload.to_be_bytes().to_vec())
    }
}
