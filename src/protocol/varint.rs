use crate::error::{ProxyError, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

/// Read a Varint from async stream (max 5 bytes, 32-bit)
pub async fn read_varint<R: AsyncRead + Unpin>(reader: &mut R) -> Result<i32> {
    let mut value: i32 = 0;
    let mut position: u32 = 0;

    loop {
        let byte = reader.read_u8().await?;
        value |= ((byte & SEGMENT_BITS) as i32) << position;
        if byte & CONTINUE_BIT == 0 {
            break;
        }
        position += 7;
        if position >= 32 {
            return Err(ProxyError::VarintTooLarge);
        }
    }

    Ok(value)
}

/// Encode i32 as Varint and write to buffer
pub fn encode_varint(value: i32, buf: &mut Vec<u8>) {
    let mut uval = value as u32;
    loop {
        if uval & !(SEGMENT_BITS as u32) == 0 {
            buf.push(uval as u8);
            return;
        }
        buf.push((uval as u8 & SEGMENT_BITS) | CONTINUE_BIT);
        uval >>= 7;
    }
}

/// Write Varint to async stream
pub async fn write_varint<W: AsyncWrite + Unpin>(writer: &mut W, value: i32) -> Result<()> {
    let mut buf = Vec::with_capacity(5);
    encode_varint(value, &mut buf);
    writer.write_all(&buf).await?;
    Ok(())
}

/// Calculate byte length after Varint encoding
pub fn varint_len(value: i32) -> usize {
    let mut uval = value as u32;
    let mut len = 0;
    loop {
        len += 1;
        if uval & !(SEGMENT_BITS as u32) == 0 {
            return len;
        }
        uval >>= 7;
    }
}

/// Read Minecraft protocol string from async stream (varint length + UTF-8)
pub async fn read_string<R: AsyncRead + Unpin>(reader: &mut R) -> Result<String> {
    let len = read_varint(reader).await? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    String::from_utf8(buf).map_err(|e| ProxyError::Protocol(format!("Invalid UTF-8: {}", e)))
}

/// Encode string to Minecraft protocol format and append to buffer
pub fn encode_string(s: &str, buf: &mut Vec<u8>) {
    encode_varint(s.len() as i32, buf);
    buf.extend_from_slice(s.as_bytes());
}

/// Read Varint from byte slice, return (value, bytes consumed)
pub fn read_varint_from_bytes(data: &[u8]) -> Result<(i32, usize)> {
    let mut value: i32 = 0;
    let mut position: u32 = 0;
    let mut i = 0;

    loop {
        if i >= data.len() {
            return Err(ProxyError::Protocol("Unexpected end of varint".into()));
        }
        let byte = data[i];
        value |= ((byte & SEGMENT_BITS) as i32) << position;
        i += 1;
        if byte & CONTINUE_BIT == 0 {
            break;
        }
        position += 7;
        if position >= 32 {
            return Err(ProxyError::VarintTooLarge);
        }
    }

    Ok((value, i))
}

/// Read string from byte slice, return (string, bytes consumed)
pub fn read_string_from_bytes(data: &[u8]) -> Result<(String, usize)> {
    let (len, consumed) = read_varint_from_bytes(data)?;
    let len = len as usize;
    if data.len() < consumed + len {
        return Err(ProxyError::Protocol("String data truncated".into()));
    }
    let s = String::from_utf8(data[consumed..consumed + len].to_vec())
        .map_err(|e| ProxyError::Protocol(format!("Invalid UTF-8: {}", e)))?;
    Ok((s, consumed + len))
}
