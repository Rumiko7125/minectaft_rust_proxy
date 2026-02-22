use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 2FA secret persistent storage
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorStorage {
    /// username → base32-encoded secret
    pub secrets: HashMap<String, String>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl TwoFactorStorage {
    /// Load from file or create new storage
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let mut storage: TwoFactorStorage = serde_json::from_str(&content)?;
            storage.file_path = path.to_path_buf();
            Ok(storage)
        } else {
            Ok(Self {
                secrets: HashMap::new(),
                file_path: path.to_path_buf(),
            })
        }
    }

    /// Save to file
    pub fn save(&self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(&self.file_path, json)?;
        Ok(())
    }

    /// Get user's secret (base32 encoded)
    pub fn get_secret(&self, username: &str) -> Option<&str> {
        self.secrets.get(username).map(|s| s.as_str())
    }

    /// Save user's secret (base32 encoded)
    pub fn set_secret(&mut self, username: &str, secret_b32: &str) {
        self.secrets.insert(username.to_string(), secret_b32.to_string());
    }

    /// Check if user has 2FA bound
    pub fn has_secret(&self, username: &str) -> bool {
        self.secrets.contains_key(username)
    }

    /// Delete user's 2FA secret
    pub fn remove_secret(&mut self, username: &str) -> bool {
        self.secrets.remove(username).is_some()
    }
}
