pub mod qr_map;
pub mod storage;
pub mod totp;

use crate::db::two_factor;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::RwLock;

/// 2FA session record (uses Unix timestamp, can be persisted to database)
struct Session {
    /// Session expiration time (Unix seconds)
    expires_at_unix: i64,
}

/// 2FA Manager
pub struct TwoFactorManager {
    /// Database connection pool
    pool: SqlitePool,
    /// Verified sessions (username -> Session)
    sessions: RwLock<HashMap<String, Session>>,
    /// Session validity duration (seconds)
    session_secs: i64,
    /// TOTP issuer name
    pub issuer: String,
}

impl TwoFactorManager {
    /// Initialize and load existing sessions from database
    pub async fn new(
        pool: &SqlitePool,
        session_hours: u64,
        issuer: &str,
    ) -> anyhow::Result<Self> {
        let session_secs = (session_hours * 3600) as i64;
        let now = chrono::Utc::now().timestamp();

        // Load unexpired sessions from database
        let rows = sqlx::query("SELECT username, expires_at FROM two_factor_sessions WHERE expires_at > ?")
            .bind(now)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        let mut sessions = HashMap::new();
        for row in rows {
            use sqlx::Row;
            let username: String = row.try_get("username").unwrap_or_default();
            let expires_at: i64 = row.try_get("expires_at").unwrap_or(0);
            if !username.is_empty() {
                sessions.insert(username, Session { expires_at_unix: expires_at });
            }
        }

        tracing::info!("2FA: loaded {} active sessions from database", sessions.len());

        Ok(Self {
            pool: pool.clone(),
            sessions: RwLock::new(sessions),
            session_secs,
            issuer: issuer.to_string(),
        })
    }

    /// Check if user has 2FA bound
    pub async fn has_2fa(&self, username: &str) -> bool {
        two_factor::has_2fa(&self.pool, username)
            .await
            .unwrap_or(false)
    }

    /// Check if user has valid session (also verify database to prevent bypass after manual table clear)
    pub async fn has_valid_session(&self, username: &str) -> bool {
        let now = chrono::Utc::now().timestamp();
        // Check memory cache first
        let in_memory = {
            if let Ok(sessions) = self.sessions.read() {
                sessions.get(username).map(|s| now < s.expires_at_unix).unwrap_or(false)
            } else {
                false
            }
        };
        if !in_memory {
            return false;
        }
        // Also confirm DB has valid record (prevent admin manual table clear)
        let db_valid = sqlx::query(
            "SELECT 1 FROM two_factor_sessions WHERE username = ? AND expires_at > ?"
        )
        .bind(username)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map(|r| r.is_some())
        .unwrap_or(false);

        if !db_valid {
            // Deleted from DB, clean memory cache
            if let Ok(mut sessions) = self.sessions.write() {
                sessions.remove(username);
            }
        }
        db_valid
    }

    /// Invalidate session (used when 2FA is unbound), also delete from database
    pub async fn invalidate_session(&self, username: &str) {
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.remove(username);
        }
        let _ = sqlx::query("DELETE FROM two_factor_sessions WHERE username = ?")
            .bind(username)
            .execute(&self.pool)
            .await;
    }

    /// Record successful verification session, also persist to database
    pub async fn create_session(&self, username: &str) {
        let expires_at = chrono::Utc::now().timestamp() + self.session_secs;
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.insert(username.to_string(), Session { expires_at_unix: expires_at });
        }
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO two_factor_sessions (username, expires_at) VALUES (?, ?)"
        )
        .bind(username)
        .bind(expires_at)
        .execute(&self.pool)
        .await;
    }

    /// Generate new 2FA secret for user, return (raw_secret, otpauth_uri)
    pub fn setup_2fa(&self, username: &str) -> (Vec<u8>, String) {
        let secret = totp::generate_secret();
        let uri = totp::generate_otpauth_uri(&secret, username, &self.issuer);
        (secret, uri)
    }

    /// Confirm and save user's 2FA secret (called after verification code is correct)
    pub async fn confirm_2fa(&self, username: &str, secret: &[u8]) -> anyhow::Result<()> {
        let encoded = totp::encode_secret(secret);
        two_factor::save_secret(&self.pool, username, &encoded).await?;
        Ok(())
    }

    /// Verify user's TOTP code
    pub async fn verify_code(&self, username: &str, code: &str) -> bool {
        if let Ok(Some(secret_b32)) = two_factor::get_secret(&self.pool, username).await {
            if let Some(secret) = totp::decode_secret(&secret_b32) {
                return totp::verify_totp(&secret, code);
            }
        }
        false
    }

    /// Verify temporary secret code (for initial binding)
    pub fn verify_setup_code(secret: &[u8], code: &str) -> bool {
        totp::verify_totp(secret, code)
    }

    /// Delete user's 2FA, also clear session
    pub async fn remove_2fa(&self, username: &str) -> anyhow::Result<bool> {
        two_factor::delete_secret(&self.pool, username).await?;
        self.invalidate_session(username).await;
        Ok(true)
    }

    /// Get list of all users with 2FA bound
    pub async fn list_2fa_users(&self) -> Vec<String> {
        two_factor::get_all_2fa_users(&self.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|u| u.username)
            .collect()
    }

    /// Clean up expired sessions in memory and database
    pub async fn cleanup_sessions(&self) {
        let now = chrono::Utc::now().timestamp();
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.retain(|_, s| now < s.expires_at_unix);
        }
        let _ = sqlx::query("DELETE FROM two_factor_sessions WHERE expires_at <= ?")
            .bind(now)
            .execute(&self.pool)
            .await;
    }
}
