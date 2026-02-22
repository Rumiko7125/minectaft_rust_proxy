/// Binary NBT reader and writer for Minecraft protocol.
/// Supports reading standard (named root) and network (nameless root) NBT formats.
/// Supports GZIP-compressed input.

use std::collections::BTreeMap;
use std::io::Read;

#[derive(Debug, Clone)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NbtValue>),
    Compound(BTreeMap<String, NbtValue>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

// ==================== NBT Tag Type IDs ====================

const TAG_END: u8 = 0;
const TAG_BYTE: u8 = 1;
const TAG_SHORT: u8 = 2;
const TAG_INT: u8 = 3;
const TAG_LONG: u8 = 4;
const TAG_FLOAT: u8 = 5;
const TAG_DOUBLE: u8 = 6;
const TAG_BYTE_ARRAY: u8 = 7;
const TAG_STRING: u8 = 8;
const TAG_LIST: u8 = 9;
const TAG_COMPOUND: u8 = 10;
const TAG_INT_ARRAY: u8 = 11;
const TAG_LONG_ARRAY: u8 = 12;

// ==================== Binary NBT Reader ====================

struct NbtReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> NbtReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_u8(&mut self) -> u8 {
        let v = self.data[self.pos];
        self.pos += 1;
        v
    }

    fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }

    fn read_i16(&mut self) -> i16 {
        let bytes = &self.data[self.pos..self.pos + 2];
        self.pos += 2;
        i16::from_be_bytes([bytes[0], bytes[1]])
    }

    fn read_i32(&mut self) -> i32 {
        let bytes = &self.data[self.pos..self.pos + 4];
        self.pos += 4;
        i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn read_i64(&mut self) -> i64 {
        let bytes = &self.data[self.pos..self.pos + 8];
        self.pos += 8;
        i64::from_be_bytes(bytes.try_into().unwrap())
    }

    fn read_f32(&mut self) -> f32 {
        f32::from_bits(self.read_i32() as u32)
    }

    fn read_f64(&mut self) -> f64 {
        f64::from_bits(self.read_i64() as u64)
    }

    fn read_string(&mut self) -> String {
        let len = self.read_i16() as usize;
        let s = std::str::from_utf8(&self.data[self.pos..self.pos + len])
            .unwrap_or("")
            .to_string();
        self.pos += len;
        s
    }

    fn read_payload(&mut self, tag_type: u8) -> NbtValue {
        match tag_type {
            TAG_BYTE => NbtValue::Byte(self.read_i8()),
            TAG_SHORT => NbtValue::Short(self.read_i16()),
            TAG_INT => NbtValue::Int(self.read_i32()),
            TAG_LONG => NbtValue::Long(self.read_i64()),
            TAG_FLOAT => NbtValue::Float(self.read_f32()),
            TAG_DOUBLE => NbtValue::Double(self.read_f64()),
            TAG_BYTE_ARRAY => {
                let len = self.read_i32() as usize;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(self.read_i8());
                }
                NbtValue::ByteArray(arr)
            }
            TAG_STRING => NbtValue::String(self.read_string()),
            TAG_LIST => {
                let elem_type = self.read_u8();
                let len = self.read_i32() as usize;
                let mut items = Vec::with_capacity(len);
                for _ in 0..len {
                    items.push(self.read_payload(elem_type));
                }
                NbtValue::List(items)
            }
            TAG_COMPOUND => {
                let mut map = BTreeMap::new();
                loop {
                    let child_type = self.read_u8();
                    if child_type == TAG_END {
                        break;
                    }
                    let name = self.read_string();
                    let value = self.read_payload(child_type);
                    map.insert(name, value);
                }
                NbtValue::Compound(map)
            }
            TAG_INT_ARRAY => {
                let len = self.read_i32() as usize;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(self.read_i32());
                }
                NbtValue::IntArray(arr)
            }
            TAG_LONG_ARRAY => {
                let len = self.read_i32() as usize;
                let mut arr = Vec::with_capacity(len);
                for _ in 0..len {
                    arr.push(self.read_i64());
                }
                NbtValue::LongArray(arr)
            }
            _ => panic!("Unknown NBT tag type: {}", tag_type),
        }
    }
}

/// Read a named root compound from standard NBT binary data.
/// Format: TAG_Compound(0x0A) + root_name(string) + compound_payload
pub fn read_nbt(data: &[u8]) -> NbtValue {
    let mut reader = NbtReader::new(data);
    let tag_type = reader.read_u8();
    assert_eq!(tag_type, TAG_COMPOUND, "Root tag must be TAG_Compound");
    let _root_name = reader.read_string(); // consume root name (usually empty)
    reader.read_payload(TAG_COMPOUND)
}

/// Read a GZIP-compressed NBT file into an NbtValue.
pub fn read_gzip_nbt(compressed: &[u8]) -> NbtValue {
    let mut decoder = flate2::read::GzDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .expect("Failed to decompress GZIP NBT data");
    read_nbt(&decompressed)
}

// ==================== Binary NBT Encoder (Network format) ====================

/// Encode an NbtValue as Network NBT (nameless root compound).
/// Format: TAG_Compound(0x0A) + compound_payload (no root name)
pub fn encode_network_nbt(value: &NbtValue) -> Vec<u8> {
    let mut buf = Vec::new();
    match value {
        NbtValue::Compound(_) => {
            buf.push(TAG_COMPOUND);
            encode_compound_payload(value, &mut buf);
        }
        _ => panic!("Root NBT value must be a Compound"),
    }
    buf
}

fn encode_named_tag(name: &str, value: &NbtValue, buf: &mut Vec<u8>) {
    buf.push(value_tag_type(value));
    encode_nbt_string(name, buf);
    encode_payload(value, buf);
}

fn encode_payload(value: &NbtValue, buf: &mut Vec<u8>) {
    match value {
        NbtValue::Byte(v) => buf.push(*v as u8),
        NbtValue::Short(v) => buf.extend_from_slice(&v.to_be_bytes()),
        NbtValue::Int(v) => buf.extend_from_slice(&v.to_be_bytes()),
        NbtValue::Long(v) => buf.extend_from_slice(&v.to_be_bytes()),
        NbtValue::Float(v) => buf.extend_from_slice(&v.to_be_bytes()),
        NbtValue::Double(v) => buf.extend_from_slice(&v.to_be_bytes()),
        NbtValue::ByteArray(arr) => {
            buf.extend_from_slice(&(arr.len() as i32).to_be_bytes());
            for v in arr {
                buf.push(*v as u8);
            }
        }
        NbtValue::String(s) => encode_nbt_string(s, buf),
        NbtValue::List(items) => {
            if items.is_empty() {
                buf.push(TAG_END);
                buf.extend_from_slice(&0i32.to_be_bytes());
            } else {
                buf.push(value_tag_type(&items[0]));
                buf.extend_from_slice(&(items.len() as i32).to_be_bytes());
                for item in items {
                    encode_payload(item, buf);
                }
            }
        }
        NbtValue::Compound(_) => encode_compound_payload(value, buf),
        NbtValue::IntArray(arr) => {
            buf.extend_from_slice(&(arr.len() as i32).to_be_bytes());
            for v in arr {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
        NbtValue::LongArray(arr) => {
            buf.extend_from_slice(&(arr.len() as i32).to_be_bytes());
            for v in arr {
                buf.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
}

fn encode_compound_payload(value: &NbtValue, buf: &mut Vec<u8>) {
    if let NbtValue::Compound(map) = value {
        for (name, val) in map {
            encode_named_tag(name, val, buf);
        }
        buf.push(TAG_END);
    }
}

fn encode_nbt_string(s: &str, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(s.len() as u16).to_be_bytes());
    buf.extend_from_slice(s.as_bytes());
}

fn value_tag_type(value: &NbtValue) -> u8 {
    match value {
        NbtValue::Byte(_) => TAG_BYTE,
        NbtValue::Short(_) => TAG_SHORT,
        NbtValue::Int(_) => TAG_INT,
        NbtValue::Long(_) => TAG_LONG,
        NbtValue::Float(_) => TAG_FLOAT,
        NbtValue::Double(_) => TAG_DOUBLE,
        NbtValue::ByteArray(_) => TAG_BYTE_ARRAY,
        NbtValue::String(_) => TAG_STRING,
        NbtValue::List(_) => TAG_LIST,
        NbtValue::Compound(_) => TAG_COMPOUND,
        NbtValue::IntArray(_) => TAG_INT_ARRAY,
        NbtValue::LongArray(_) => TAG_LONG_ARRAY,
    }
}

/// Encode any NbtValue as Network NBT (tag_type byte + payload, no name).
/// Unlike encode_network_nbt, this allows non-Compound root types (e.g. TAG_String for simple chat).
pub fn encode_network_nbt_value(value: &NbtValue) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(value_tag_type(value));
    encode_payload(value, &mut buf);
    buf
}

/// Convert a serde_json::Value to NbtValue (for chat Text Component encoding).
pub fn json_to_nbt(json: &serde_json::Value) -> NbtValue {
    match json {
        serde_json::Value::String(s) => NbtValue::String(s.clone()),
        serde_json::Value::Bool(b) => NbtValue::Byte(if *b { 1 } else { 0 }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                NbtValue::Int(i as i32)
            } else {
                NbtValue::Double(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::Array(arr) => {
            NbtValue::List(arr.iter().map(json_to_nbt).collect())
        }
        serde_json::Value::Object(map) => {
            let mut nbt_map = BTreeMap::new();
            for (key, val) in map {
                nbt_map.insert(key.clone(), json_to_nbt(val));
            }
            NbtValue::Compound(nbt_map)
        }
        serde_json::Value::Null => NbtValue::Byte(0),
    }
}

// ==================== Helper accessors ====================

impl NbtValue {
    /// Get a compound child by key
    pub fn get(&self, key: &str) -> Option<&NbtValue> {
        match self {
            NbtValue::Compound(map) => map.get(key),
            _ => None,
        }
    }

    /// Get string value
    pub fn as_string(&self) -> Option<&str> {
        match self {
            NbtValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get list value
    pub fn as_list(&self) -> Option<&Vec<NbtValue>> {
        match self {
            NbtValue::List(items) => Some(items),
            _ => None,
        }
    }

    /// Get compound keys
    pub fn keys(&self) -> Option<impl Iterator<Item = &String>> {
        match self {
            NbtValue::Compound(map) => Some(map.keys()),
            _ => None,
        }
    }
}
