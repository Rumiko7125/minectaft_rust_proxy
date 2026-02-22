use crate::db;
use crate::db::access::{BlacklistEntry, WhitelistEntry};
use sqlx::SqlitePool;
use std::collections::HashSet;

/// Access control (loaded from database)
pub struct AccessControl {
    pub whitelist_enabled: bool,
    pub whitelist: HashSet<String>,
    pub blacklist: HashSet<String>,
    pub pool: SqlitePool,
}

impl AccessControl {
    /// Load from database
    pub async fn new(pool: SqlitePool, whitelist_enabled: bool) -> Self {
        let whitelist: HashSet<String> = db::access::get_whitelist(&pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.username)
            .collect();
        let blacklist: HashSet<String> = db::access::get_blacklist(&pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.username)
            .collect();

        Self {
            whitelist_enabled,
            whitelist,
            blacklist,
            pool,
        }
    }

    /// Get whitelist details
    pub async fn get_whitelist_details(&self) -> Vec<WhitelistEntry> {
        db::access::get_whitelist(&self.pool).await.unwrap_or_default()
    }

    /// Get blacklist details
    pub async fn get_blacklist_details(&self) -> Vec<BlacklistEntry> {
        db::access::get_blacklist(&self.pool).await.unwrap_or_default()
    }

    /// Check if user is allowed to connect
    pub async fn is_allowed(&self, username: &str) -> bool {
        // Check blacklist
        if db::access::is_blacklisted(&self.pool, username).await.unwrap_or(false) {
            return false;
        }
        // Check whitelist
        if self.whitelist_enabled {
            if !db::access::is_whitelisted(&self.pool, username).await.unwrap_or(false) {
                return false;
            }
        }
        true
    }

    /// Add to whitelist
    pub async fn add_whitelist(&mut self, username: &str) {
        if db::access::add_whitelist(&self.pool, username).await.is_ok() {
            self.whitelist.insert(username.to_string());
        }
    }

    /// Remove from whitelist
    pub async fn remove_whitelist(&mut self, username: &str) {
        if db::access::remove_whitelist(&self.pool, username).await.is_ok() {
            self.whitelist.remove(username);
        }
    }

    /// Add to blacklist
    pub async fn add_blacklist(&mut self, username: &str, reason: Option<&str>) {
        if db::access::add_blacklist(&self.pool, username, reason).await.is_ok() {
            self.blacklist.insert(username.to_string());
        }
    }

    /// Remove from blacklist
    pub async fn remove_blacklist(&mut self, username: &str) {
        if db::access::remove_blacklist(&self.pool, username).await.is_ok() {
            self.blacklist.remove(username);
        }
    }
}
