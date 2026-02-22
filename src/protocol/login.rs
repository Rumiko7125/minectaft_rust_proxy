use crate::error::Result;
use crate::protocol::packet::RawPacket;
use crate::protocol::varint;

/// Login Start packet (Packet ID 0x00)
#[derive(Debug, Clone)]
pub struct LoginStartPacket {
    pub username: String,
    pub uuid: Option<[u8; 16]>,
}

impl LoginStartPacket {
    /// Parse from RawPacket
    pub fn from_raw(packet: &RawPacket) -> Result<Self> {
        let data = &packet.data;
        let (username, consumed) = varint::read_string_from_bytes(data)?;

        // UUID exists in newer protocol versions (at least 16 bytes remaining)
        let uuid = if data.len() >= consumed + 16 {
            let mut uuid_bytes = [0u8; 16];
            uuid_bytes.copy_from_slice(&data[consumed..consumed + 16]);
            Some(uuid_bytes)
        } else {
            None
        };

        Ok(Self { username, uuid })
    }

    /// Encode to RawPacket
    pub fn to_raw(&self) -> RawPacket {
        let mut data = Vec::new();
        varint::encode_string(&self.username, &mut data);
        if let Some(uuid) = &self.uuid {
            data.extend_from_slice(uuid);
        }
        RawPacket::new(0x00, data)
    }

    /// Get UUID bytes, legacy clients generate offline UUID automatically
    pub fn uuid_bytes(&self) -> [u8; 16] {
        match &self.uuid {
            Some(bytes) => *bytes,
            None => generate_offline_uuid(&self.username),
        }
    }

    /// Get formatted UUID string
    pub fn uuid_string(&self) -> String {
        format_uuid(&self.uuid_bytes())
    }
}

/// Login Failure packet (Packet ID 0x00)
pub struct LoginFailurePacket {
    pub reason: serde_json::Value,
}

impl LoginFailurePacket {
    pub fn new(reason: &str) -> Self {
        Self {
            reason: serde_json::json!({"text": reason}),
        }
    }

    pub fn to_raw(&self) -> RawPacket {
        let mut data = Vec::new();
        let json_str = self.reason.to_string();
        varint::encode_string(&json_str, &mut data);
        RawPacket::new(0x00, data)
    }
}

/// Generate Minecraft offline mode UUID (matches Java `UUID.nameUUIDFromBytes`)
fn generate_offline_uuid(username: &str) -> [u8; 16] {
    let input = format!("OfflinePlayer:{}", username);
    // MD5 hash
    let digest = md5_hash(input.as_bytes());
    let mut uuid = digest;
    // Set version = 3 (name-based MD5)
    uuid[6] = (uuid[6] & 0x0F) | 0x30;
    // Set variant = 2 (RFC 4122)
    uuid[8] = (uuid[8] & 0x3F) | 0x80;
    uuid
}

/// Simple MD5 implementation (only for offline UUID generation)
fn md5_hash(data: &[u8]) -> [u8; 16] {
    // MD5 constants
    const S: [u32; 64] = [
        7,12,17,22, 7,12,17,22, 7,12,17,22, 7,12,17,22,
        5, 9,14,20, 5, 9,14,20, 5, 9,14,20, 5, 9,14,20,
        4,11,16,23, 4,11,16,23, 4,11,16,23, 4,11,16,23,
        6,10,15,21, 6,10,15,21, 6,10,15,21, 6,10,15,21,
    ];
    const K: [u32; 64] = [
        0xd76aa478,0xe8c7b756,0x242070db,0xc1bdceee,0xf57c0faf,0x4787c62a,0xa8304613,0xfd469501,
        0x698098d8,0x8b44f7af,0xffff5bb1,0x895cd7be,0x6b901122,0xfd987193,0xa679438e,0x49b40821,
        0xf61e2562,0xc040b340,0x265e5a51,0xe9b6c7aa,0xd62f105d,0x02441453,0xd8a1e681,0xe7d3fbc8,
        0x21e1cde6,0xc33707d6,0xf4d50d87,0x455a14ed,0xa9e3e905,0xfcefa3f8,0x676f02d9,0x8d2a4c8a,
        0xfffa3942,0x8771f681,0x6d9d6122,0xfde5380c,0xa4beea44,0x4bdecfa9,0xf6bb4b60,0xbebfbc70,
        0x289b7ec6,0xeaa127fa,0xd4ef3085,0x04881d05,0xd9d4d039,0xe6db99e5,0x1fa27cf8,0xc4ac5665,
        0xf4292244,0x432aff97,0xab9423a7,0xfc93a039,0x655b59c3,0x8f0ccc92,0xffeff47d,0x85845dd1,
        0x6fa87e4f,0xfe2ce6e0,0xa3014314,0x4e0811a1,0xf7537e82,0xbd3af235,0x2ad7d2bb,0xeb86d391,
    ];

    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    for chunk in msg.chunks(64) {
        let mut m = [0u32; 16];
        for (i, c) in chunk.chunks(4).enumerate() {
            m[i] = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        }

        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);

        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | (!d)), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(K[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(S[i]));
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut result = [0u8; 16];
    result[0..4].copy_from_slice(&a0.to_le_bytes());
    result[4..8].copy_from_slice(&b0.to_le_bytes());
    result[8..12].copy_from_slice(&c0.to_le_bytes());
    result[12..16].copy_from_slice(&d0.to_le_bytes());
    result
}

/// Mojang player profile (UUID + skin textures)
#[derive(Debug, Clone)]
pub struct MojangProfile {
    pub uuid: [u8; 16],
    /// Base64 encoded texture data
    pub textures_value: Option<String>,
    /// Base64 encoded signature
    pub textures_signature: Option<String>,
}

/// Query full profile (UUID + skin) of premium player from Mojang API
pub async fn lookup_mojang_profile(username: &str) -> Option<MojangProfile> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;

    // 1. Get UUID
    let url = format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        username
    );
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    let id_str = json.get("id")?.as_str()?;
    let bytes = hex::decode(id_str).ok()?;
    if bytes.len() != 16 {
        return None;
    }
    let mut uuid = [0u8; 16];
    uuid.copy_from_slice(&bytes);

    // 2. Get skin textures (from session server)
    let session_url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=false",
        id_str
    );
    let (textures_value, textures_signature) =
        match client.get(&session_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    extract_textures(&json)
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        };

    Some(MojangProfile {
        uuid,
        textures_value,
        textures_signature,
    })
}

/// Extract texture data from session server response
fn extract_textures(json: &serde_json::Value) -> (Option<String>, Option<String>) {
    if let Some(properties) = json.get("properties").and_then(|p| p.as_array()) {
        for prop in properties {
            if prop.get("name").and_then(|n| n.as_str()) == Some("textures") {
                let value = prop.get("value").and_then(|v| v.as_str()).map(String::from);
                let signature = prop
                    .get("signature")
                    .and_then(|s| s.as_str())
                    .map(String::from);
                return (value, signature);
            }
        }
    }
    (None, None)
}

/// Public UUID formatting function
pub fn format_uuid_bytes(bytes: &[u8; 16]) -> String {
    format_uuid(bytes)
}

/// Format 16-byte UUID to standard string
fn format_uuid(bytes: &[u8; 16]) -> String {
    let hex = hex::encode(bytes);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}
