use hmac::{Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

/// Generate random TOTP secret (20 bytes, base32 encoded)
pub fn generate_secret() -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let secret: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
    secret
}

/// Encode secret to base32 string
pub fn encode_secret(secret: &[u8]) -> String {
    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, secret)
}

/// Decode secret from base32 string
pub fn decode_secret(encoded: &str) -> Option<Vec<u8>> {
    base32::decode(base32::Alphabet::Rfc4648 { padding: false }, encoded)
}

/// Generate otpauth:// URI
pub fn generate_otpauth_uri(secret: &[u8], username: &str, issuer: &str) -> String {
    let encoded_secret = encode_secret(secret);
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        issuer, username, encoded_secret, issuer
    )
}

/// Generate TOTP code (given time step)
fn generate_totp_at(secret: &[u8], time_step: u64) -> u32 {
    let time_bytes = time_step.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();

    // Dynamic truncation
    let offset = (result[19] & 0x0F) as usize;
    let code = ((result[offset] as u32 & 0x7F) << 24)
        | ((result[offset + 1] as u32) << 16)
        | ((result[offset + 2] as u32) << 8)
        | (result[offset + 3] as u32);

    code % 1_000_000
}

/// Verify TOTP code (allows +/-1 time window, 90 second tolerance)
pub fn verify_totp(secret: &[u8], code: &str) -> bool {
    let code_num: u32 = match code.parse() {
        Ok(n) => n,
        Err(_) => return false,
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let current_step = now / 30;

    // Check current step + one before and after
    for offset in [0i64, -1, 1] {
        let step = (current_step as i64 + offset) as u64;
        if generate_totp_at(secret, step) == code_num {
            return true;
        }
    }

    false
}

/// Generate current TOTP code (for testing)
#[allow(dead_code)]
pub fn generate_current_totp(secret: &[u8]) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let step = now / 30;
    format!("{:06}", generate_totp_at(secret, step))
}
