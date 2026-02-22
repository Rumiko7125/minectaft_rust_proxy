use crate::error::{ProxyError, Result};
use std::path::Path;

/// Load custom MOTD JSON from file
pub fn load_motd(path: &Path) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ProxyError::Config(format!("Failed to read MOTD file: {}", e)))?;
    let motd: serde_json::Value = serde_json::from_str(&content)?;
    Ok(motd)
}

/// Generate default MOTD JSON
pub fn default_motd(
    version_name: &str,
    protocol: i32,
    max_players: i32,
    online: i32,
    description: &str,
) -> serde_json::Value {
    serde_json::json!({
        "version": {
            "name": version_name,
            "protocol": protocol
        },
        "players": {
            "max": max_players,
            "online": online
        },
        "description": {
            "text": description
        }
    })
}

/// Update online player count in MOTD
pub fn update_motd_players(motd: &mut serde_json::Value, online: i32, max_players: i32) {
    if let Some(players) = motd.get_mut("players") {
        players["online"] = serde_json::json!(online);
        if max_players >= 0 {
            players["max"] = serde_json::json!(max_players);
        }
    }
}
