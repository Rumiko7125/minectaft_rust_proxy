use crate::db;
use crate::error::{ProxyError, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_local_address")]
    pub local_address: String,

    #[serde(default = "default_port")]
    pub local_port: u16,

    #[serde(default)]
    pub enable_whitelist: bool,

    #[serde(default = "default_true")]
    pub allow_input: bool,

    #[serde(default = "default_log_dir")]
    pub log_dir: String,

    #[serde(default)]
    pub show_log_level: u8,

    #[serde(default)]
    pub save_log_level: u8,

    #[serde(default = "default_true")]
    pub web_api_enable: bool,

    #[serde(default = "default_web_address")]
    pub web_api_address: String,

    #[serde(default = "default_web_port")]
    pub web_api_port: u16,

    #[serde(default)]
    pub enable_2fa: bool,

    #[serde(default = "default_2fa_hours")]
    pub two_factor_session_hours: u64,

    #[serde(default = "default_2fa_issuer")]
    pub two_factor_issuer: String,

    #[serde(default = "default_language")]
    pub language: String,

    #[serde(default)]
    pub trusted_domain: String,
}

fn default_local_address() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    25565
}
fn default_true() -> bool {
    true
}
fn default_log_dir() -> String {
    "./logs".to_string()
}
fn default_web_address() -> String {
    "127.0.0.1".to_string()
}
fn default_web_port() -> u16 {
    20220
}
fn default_2fa_hours() -> u64 {
    12
}
fn default_2fa_issuer() -> String {
    "MinecraftProxy".to_string()
}
fn default_language() -> String {
    "en".to_string()
}

impl Config {
    /// Load config from database
    pub async fn load(pool: &SqlitePool) -> Result<Self> {
        let settings = db::settings::get_all_settings(pool)
            .await
            .map_err(|e| ProxyError::Config(format!("Failed to load settings: {}", e)))?;

        let mut config = Config::default();

        if let Some(v) = settings.get("local_address") {
            config.local_address = v.clone();
        }
        if let Some(v) = settings.get("local_port") {
            config.local_port = v.parse().unwrap_or(25565);
        }
        if let Some(v) = settings.get("whitelist_enabled") {
            config.enable_whitelist = v.parse().unwrap_or(false);
        }
        if let Some(v) = settings.get("allow_input") {
            config.allow_input = v.parse().unwrap_or(true);
        }
        if let Some(v) = settings.get("log_dir") {
            config.log_dir = v.clone();
        }
        if let Some(v) = settings.get("show_log_level") {
            config.show_log_level = v.parse().unwrap_or(0);
        }
        if let Some(v) = settings.get("save_log_level") {
            config.save_log_level = v.parse().unwrap_or(0);
        }
        if let Some(v) = settings.get("web_api_enable") {
            config.web_api_enable = v.parse().unwrap_or(true);
        }
        if let Some(v) = settings.get("web_api_address") {
            config.web_api_address = v.clone();
        }
        if let Some(v) = settings.get("web_api_port") {
            config.web_api_port = v.parse().unwrap_or(20220);
        }
        if let Some(v) = settings.get("enable_2fa") {
            config.enable_2fa = v.parse().unwrap_or(false);
        }
        if let Some(v) = settings.get("two_factor_session_hours") {
            config.two_factor_session_hours = v.parse().unwrap_or(12);
        }
        if let Some(v) = settings.get("two_factor_issuer") {
            config.two_factor_issuer = v.clone();
        }
        if let Some(v) = settings.get("language") {
            config.language = v.clone();
        }
        if let Some(v) = settings.get("trusted_domain") {
            config.trusted_domain = v.clone();
        }

        Ok(config)
    }

    /// Save config to database
    pub async fn save(&self, pool: &SqlitePool) -> Result<()> {
        let fields = [
            ("local_address", self.local_address.clone()),
            ("local_port", self.local_port.to_string()),
            ("whitelist_enabled", self.enable_whitelist.to_string()),
            ("allow_input", self.allow_input.to_string()),
            ("log_dir", self.log_dir.clone()),
            ("show_log_level", self.show_log_level.to_string()),
            ("save_log_level", self.save_log_level.to_string()),
            ("web_api_enable", self.web_api_enable.to_string()),
            ("web_api_address", self.web_api_address.clone()),
            ("web_api_port", self.web_api_port.to_string()),
            ("enable_2fa", self.enable_2fa.to_string()),
            ("two_factor_session_hours", self.two_factor_session_hours.to_string()),
            ("two_factor_issuer", self.two_factor_issuer.clone()),
            ("language", self.language.clone()),
            ("trusted_domain", self.trusted_domain.clone()),
        ];

        for (key, value) in fields {
            db::settings::set_setting(pool, key, &value).await
                .map_err(|e| ProxyError::Config(format!("Failed to save setting {}: {}", key, e)))?;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_address: default_local_address(),
            local_port: default_port(),
            enable_whitelist: false,
            allow_input: true,
            log_dir: default_log_dir(),
            show_log_level: 0,
            save_log_level: 0,
            web_api_enable: true,
            web_api_address: default_web_address(),
            web_api_port: default_web_port(),
            enable_2fa: false,
            two_factor_session_hours: default_2fa_hours(),
            two_factor_issuer: default_2fa_issuer(),
            language: default_language(),
            trusted_domain: String::new(),
        }
    }
}
