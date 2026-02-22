use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// i18n message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Messages {
    #[serde(flatten)]
    entries: HashMap<String, String>,
}

impl Messages {
    /// Load language file from file
    pub fn load(lang: &str) -> Self {
        let path = format!("i18n/{}.json", lang);
        match Self::load_from_path(&path) {
            Ok(msgs) => msgs,
            Err(e) => {
                tracing::warn!("Failed to load language {}: {}, falling back to English", lang, e);
                match Self::load_from_path("i18n/en.json") {
                    Ok(msgs) => msgs,
                    Err(e) => {
                        tracing::error!("Failed to load fallback English language: {}", e);
                        Self::default()
                    }
                }
            }
        }
    }

    fn load_from_path(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let entries: HashMap<String, String> = serde_json::from_str(&content)?;
        Ok(Messages { entries })
    }

    /// Get message
    pub fn get(&self, key: &str) -> String {
        self.entries.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    /// Get message (with argument replacement)
    pub fn get_with_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let msg = self.get(key);
        let mut result = msg;
        for (k, v) in args {
            result = result.replace(&format!("{{{}}}", k), v);
        }
        result
    }
}

impl Default for Messages {
    fn default() -> Self {
        let mut entries = HashMap::new();
        entries.insert("unsupported_version".to_string(), "Unsupported version!".to_string());
        entries.insert("banned".to_string(), "You are banned from this server.".to_string());
        entries.insert("not_whitelisted".to_string(), "You are not whitelisted on this server.".to_string());
        entries.insert("server_full".to_string(), "Server is full.".to_string());
        entries.insert("already_connected".to_string(), "You are already connected to this proxy.".to_string());
        entries.insert("2fa_setup_title".to_string(), "=== 2FA Setup ===".to_string());
        entries.insert("2fa_setup_scan".to_string(), "Scan the QR code on your map with an authenticator app,\n".to_string());
        entries.insert("2fa_setup_or".to_string(), "or ".to_string());
        entries.insert("2fa_setup_click_here".to_string(), "[Click here to view QR code]\n".to_string());
        entries.insert("2fa_setup_copy_uri".to_string(), "or copy this URI: ".to_string());
        entries.insert("2fa_setup_verify_prompt".to_string(), "\n\nType your 6-digit code in chat to verify.".to_string());
        entries.insert("2fa_setup_complete_chat".to_string(), "2FA setup complete! Reconnecting...".to_string());
        entries.insert("2fa_verify_title".to_string(), "=== 2FA Verification ===\n".to_string());
        entries.insert("2fa_verify_enter_code".to_string(), "Please enter your 6-digit verification code in chat.".to_string());
        entries.insert("2fa_verify_success_chat".to_string(), "Verification successful! Reconnecting...".to_string());
        entries.insert("2fa_invalid_code".to_string(), "Invalid code. Please try again.".to_string());
        entries.insert("2fa_invalid_code_attempts".to_string(), "Invalid code. {attempts} attempts remaining.".to_string());
        entries.insert("2fa_setup_complete_disconnect".to_string(), "2FA setup complete. Please reconnect.".to_string());
        entries.insert("2fa_setup_failed".to_string(), "2FA setup failed or timed out.".to_string());
        entries.insert("2fa_verify_success_disconnect".to_string(), "Verification successful. Please reconnect.".to_string());
        entries.insert("2fa_verify_failed".to_string(), "Verification failed or timed out.".to_string());
        entries.insert("use_domain_to_connect".to_string(), "Please use {domain} to connect.".to_string());
        Messages { entries }
    }
}

/// Global i18n state
pub struct I18n {
    pub messages: Messages,
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        I18n {
            messages: Messages::load(lang),
        }
    }
}
