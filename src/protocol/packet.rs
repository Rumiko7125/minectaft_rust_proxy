use crate::error::{ProxyError, Result};
use crate::protocol::varint;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Raw Minecraft packet
#[derive(Debug, Clone)]
pub struct RawPacket {
    pub id: i32,
    pub data: Vec<u8>,
}

impl RawPacket {
    pub fn new(id: i32, data: Vec<u8>) -> Self {
        Self { id, data }
    }

    /// Read a complete packet from async stream
    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let total_len = varint::read_varint(reader).await? as usize;
        if total_len == 0 {
            return Err(ProxyError::Protocol("Empty packet".into()));
        }

        let mut payload = vec![0u8; total_len];
        reader.read_exact(&mut payload).await?;

        let (packet_id, id_len) = varint::read_varint_from_bytes(&payload)?;
        let data = payload[id_len..].to_vec();

        Ok(Self {
            id: packet_id,
            data,
        })
    }

    /// Write packet to async stream
    pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let id_len = varint::varint_len(self.id);
        let total_len = id_len + self.data.len();

        let mut buf = Vec::with_capacity(varint::varint_len(total_len as i32) + total_len);
        varint::encode_varint(total_len as i32, &mut buf);
        varint::encode_varint(self.id, &mut buf);
        buf.extend_from_slice(&self.data);

        writer.write_all(&buf).await?;
        Ok(())
    }

    /// Serialize packet to bytes (without compression frame)
    pub fn to_bytes(&self) -> Vec<u8> {
        let id_len = varint::varint_len(self.id);
        let total_len = id_len + self.data.len();

        let mut buf = Vec::with_capacity(varint::varint_len(total_len as i32) + total_len);
        varint::encode_varint(total_len as i32, &mut buf);
        varint::encode_varint(self.id, &mut buf);
        buf.extend_from_slice(&self.data);
        buf
    }

}
