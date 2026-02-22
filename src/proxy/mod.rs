pub mod connection;
pub mod limbo;
pub mod router;

use chrono;
use crate::access_control::AccessControl;
use crate::auth::TwoFactorManager;
use crate::config::Config;
use crate::db;
use crate::db::backend_servers::BackendServer;
use crate::i18n::I18n;
use crate::motd;
use crate::proxy::router::{extract_domain, DomainRoute};
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

/// Backend server info (for in-memory storage)
#[derive(Clone, Debug)]
pub struct BackendServerInfo {
    pub id: i64,
    pub name: String,
    pub remote_address: String,
    pub remote_port: u16,
    pub max_player: i32,
    pub motd_json: Option<String>,
    pub limbo_message: Option<String>,
    pub log_dir: Option<String>,
    pub show_log_level: i32,
    pub save_log_level: i32,
    pub is_default: bool,
    pub enabled: bool,
    pub maintenance: bool,
    pub maintenance_message: Option<String>,
    pub ping_passthrough: bool,
    pub motd_passthrough: bool,
    pub language: String,
}

impl From<BackendServer> for BackendServerInfo {
    fn from(bs: BackendServer) -> Self {
        Self {
            id: bs.id,
            name: bs.name,
            remote_address: bs.remote_address,
            remote_port: bs.remote_port as u16,
            max_player: bs.max_player,
            motd_json: bs.motd_json,
            limbo_message: bs.limbo_message,
            log_dir: bs.log_dir,
            show_log_level: bs.show_log_level,
            save_log_level: bs.save_log_level,
            is_default: bs.is_default,
            enabled: bs.enabled,
            maintenance: bs.maintenance,
            maintenance_message: bs.maintenance_message,
            ping_passthrough: bs.ping_passthrough,
            motd_passthrough: bs.motd_passthrough,
            language: bs.language,
        }
    }
}

/// Target backend information
#[derive(Clone, Debug)]
pub struct TargetInfo {
    pub backend_id: i64,
    pub backend_name: String,
    pub target_addr: String,
    pub target_port: u16,
    pub socket_addr: SocketAddr,
    pub max_player: i32,
    pub maintenance: bool,
    pub maintenance_message: Option<String>,
    pub limbo_message: Option<String>,
    pub log_dir: Option<String>,
    pub ping_passthrough: bool,
    pub motd_passthrough: bool,
    pub language: String,
}

impl TargetInfo {
    fn from_backend(backend: &BackendServerInfo) -> Self {
        let socket_addr = SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            backend.remote_port,
        );
        Self {
            backend_id: backend.id,
            backend_name: backend.name.clone(),
            target_addr: backend.remote_address.clone(),
            target_port: backend.remote_port,
            socket_addr,
            max_player: backend.max_player,
            maintenance: backend.maintenance,
            maintenance_message: backend.maintenance_message.clone(),
            limbo_message: backend.limbo_message.clone(),
            log_dir: backend.log_dir.clone(),
            ping_passthrough: backend.ping_passthrough,
            motd_passthrough: backend.motd_passthrough,
            language: backend.language.clone(),
        }
    }

    fn from_addr(addr: SocketAddr) -> Self {
        Self {
            backend_id: -1,
            backend_name: addr.ip().to_string(),
            target_addr: addr.ip().to_string(),
            target_port: addr.port(),
            socket_addr: addr,
            max_player: -1,
            maintenance: false,
            maintenance_message: None,
            limbo_message: None,
            log_dir: None,
            ping_passthrough: false,
            motd_passthrough: false,
            language: String::new(),
        }
    }
}

/// Online user info
pub struct UserInfo {
    pub username: String,
    pub uuid: String,
    pub protocol_version: i32,
    pub login_time: i64,
    pub upload_bytes: AtomicU64,
    pub download_bytes: AtomicU64,
    pub remote_addr: SocketAddr,
    pub kick_flag: AtomicBool,
    pub kick_reason: Mutex<Option<String>>,
    pub session_id: Option<i64>,
    pub backend_name: String,
}

/// Proxy server core
pub struct Proxy {
    pub config: RwLock<Config>,
    pub db_pool: SqlitePool,
    pub users: Arc<DashMap<String, Arc<UserInfo>>>,
    pub domain_routes: RwLock<Vec<DomainRoute>>,
    pub backends: RwLock<Vec<BackendServerInfo>>,
    pub access_control: RwLock<AccessControl>,
    pub shutdown_tx: tokio::sync::broadcast::Sender<()>,
    pub two_factor: tokio::sync::RwLock<Option<TwoFactorManager>>,
    pub i18n: I18n,
    /// If true, main() will spawn new process and exit after start() returns (graceful restart)
    pub restart_pending: AtomicBool,
    /// Proxy process start Unix timestamp in seconds, resets after restart, used for Dashboard uptime display
    pub start_time: u64,
}

impl Proxy {
    pub async fn new(config: Config, pool: SqlitePool) -> anyhow::Result<Arc<Self>> {
        let access_control = AccessControl::new(pool.clone(), config.enable_whitelist).await;

        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        let two_factor = if config.enable_2fa {
            match TwoFactorManager::new(
                &pool,
                config.two_factor_session_hours,
                &config.two_factor_issuer,
            ).await {
                Ok(tf) => {
                    tracing::info!("2FA enabled");
                    Some(tf)
                }
                Err(e) => {
                    tracing::error!("Failed to initialize 2FA: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let i18n = I18n::new(&config.language);

        let db_routes = db::domain_routes::get_all_domain_routes(&pool)
            .await
            .unwrap_or_default();
        let domain_routes_vec: Vec<DomainRoute> = db_routes
            .into_iter()
            .filter_map(|r| DomainRoute::from_db(&r).ok())
            .collect();

        let db_backends = db::backend_servers::get_all_backend_servers(&pool)
            .await
            .unwrap_or_default();
        let backends_vec: Vec<BackendServerInfo> = db_backends
            .into_iter()
            .map(BackendServerInfo::from)
            .collect();

        let proxy = Arc::new(Self {
            config: RwLock::new(config),
            db_pool: pool,
            users: Arc::new(DashMap::new()),
            domain_routes: RwLock::new(domain_routes_vec),
            backends: RwLock::new(backends_vec),
            access_control: RwLock::new(access_control),
            shutdown_tx,
            two_factor: tokio::sync::RwLock::new(two_factor),
            i18n,
            restart_pending: AtomicBool::new(false),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });

        Ok(proxy)
    }

    pub async fn start(self: &Arc<Self>) -> anyhow::Result<()> {
        let bind_addr = {
            let config = self.config.read().await;
            format!("{}:{}", config.local_address, config.local_port)
        };

        let listener = {
            let mut last_err: Option<std::io::Error> = None;
            let mut bound = None;
            for attempt in 0..10u32 {
                match TcpListener::bind(&bind_addr).await {
                    Ok(l) => {
                        bound = Some(l);
                        break;
                    }
                    Err(e) => {
                        if attempt > 0 {
                            tracing::warn!(
                                "Proxy port {} still in use (attempt {}/10), retrying...",
                                bind_addr,
                                attempt + 1
                            );
                        }
                        last_err = Some(e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
            match bound {
                Some(l) => {
                    tracing::info!("Proxy listening on {}", bind_addr);
                    l
                }
                None => {
                    let e = last_err.unwrap();
                    tracing::error!("Failed to bind proxy on {}: {}", bind_addr, e);
                    return Err(e.into());
                }
            }
        };
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, addr)) => {
                            let proxy = Arc::clone(self);
                            tokio::spawn(async move {
                                connection::handle_connection(proxy, stream, addr).await;
                            });
                        }
                        Err(e) => {
                            tracing::error!("Accept error: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("Proxy shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Resolve connection target: domain routing > default backend
    /// If any domain routing rules exist but none match, reject connection (no fallback to default backend)
    pub async fn resolve_target(&self, username: &str, server_address: &str) -> TargetInfo {
        let _ = username;

        let domain = extract_domain(server_address);
        tracing::info!("resolve_target: raw='{}' → domain='{}'", server_address, domain);

        // 1. Domain routing matching (using .read().await to ensure lock is always available, no fallback to default backend)
        let routes = self.domain_routes.read().await;
        let backends = self.backends.read().await;

        let has_routes = !routes.is_empty();
        tracing::info!("resolve_target: {} route(s) loaded, checking domain='{}'", routes.len(), domain);

        for route in routes.iter() {
            tracing::debug!("  checking route pattern='{}' (regex={})", route.pattern_str, route.pattern.as_str());
            if route.matches(domain) {
                if let Some(backend_id) = route.backend_id {
                    if let Some(backend) = backends.iter().find(|b| b.id == backend_id) {
                        if backend.enabled {
                            tracing::info!("Domain '{}' matched route '{}' → backend '{}'", domain, route.pattern_str, backend.name);
                            return TargetInfo::from_backend(backend);
                        } else {
                            // Route points to disabled backend
                            tracing::info!("Domain '{}' matched route '{}' but backend '{}' is disabled", domain, route.pattern_str, backend.name);
                            return TargetInfo {
                                backend_id: backend.id,
                                backend_name: backend.name.clone(),
                                target_addr: "".to_string(),
                                target_port: 0,
                                socket_addr: "0.0.0.0:0".parse().unwrap(),
                                max_player: 0,
                                maintenance: true,
                                maintenance_message: Some(self.i18n.messages.get("server_offline")),
                                limbo_message: None,
                                log_dir: None,
                                ping_passthrough: false,
                                motd_passthrough: false,
                                language: String::new(),
                            };
                        }
                    }
                }
                // Address directly specified in route
                if !route.target_addr.is_empty() {
                    let socket_addr = format!("{}:{}", route.target_addr, route.target_port)
                        .parse()
                        .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                    tracing::info!("Domain '{}' matched route '{}' → direct {}:{}", domain, route.pattern_str, route.target_addr, route.target_port);
                    return TargetInfo {
                        backend_id: -1,
                        backend_name: format!("{}:{}", route.target_addr, route.target_port),
                        target_addr: route.target_addr.clone(),
                        target_port: route.target_port,
                        socket_addr,
                        max_player: -1,
                        maintenance: false,
                        maintenance_message: None,
                        limbo_message: None,
                        log_dir: None,
                        ping_passthrough: false,
                        motd_passthrough: false,
                        language: String::new(),
                    };
                }
            }
        }

        // Has route rules but none match -> reject, no fallback to default backend
        if has_routes {
            tracing::info!("Domain '{}' did not match any of {} route(s), rejecting", domain, routes.len());
            return TargetInfo {
                backend_id: -1,
                backend_name: "rejected".to_string(),
                target_addr: "".to_string(),
                target_port: 0,
                socket_addr: "0.0.0.0:0".parse().unwrap(),
                max_player: 0,
                maintenance: true,
                maintenance_message: Some(self.i18n.messages.get("no_route_for_domain")),
                limbo_message: None,
                log_dir: None,
                ping_passthrough: false,
                motd_passthrough: false,
                language: String::new(),
            };
        }

        drop(routes);
        drop(backends);

        // 2. No routing rules -> use default backend
        let backends = self.backends.read().await;
        if let Some(backend) = backends.iter().find(|b| b.is_default && b.enabled) {
            tracing::info!("No routes configured, using default backend '{}'", backend.name);
            return TargetInfo::from_backend(backend);
        }
        if let Some(backend) = backends.iter().find(|b| b.enabled) {
            tracing::info!("No routes configured, using first available backend '{}'", backend.name);
            return TargetInfo::from_backend(backend);
        }

        // No backend available
        TargetInfo {
            backend_id: -1,
            backend_name: "none".to_string(),
            target_addr: "".to_string(),
            target_port: 0,
            socket_addr: "0.0.0.0:0".parse().unwrap(),
            max_player: 0,
            maintenance: true,
            maintenance_message: Some(self.i18n.messages.get("proxy_not_configured")),
            limbo_message: None,
            log_dir: None,
            ping_passthrough: false,
            motd_passthrough: false,
            language: String::new(),
        }
    }

    /// Write backend-specific log file (log_dir field), relative path based on exe directory
    pub fn write_backend_log(log_dir: &str, message: &str) {
        let abs_dir = crate::resolve_log_dir(log_dir);
        if abs_dir.is_empty() { return; }
        if let Err(e) = std::fs::create_dir_all(&abs_dir) {
            tracing::warn!("Failed to create backend log dir '{}': {}", abs_dir, e);
            return;
        }
        let log_path = format!("{}/proxy.log", abs_dir);
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let line = format!("[{}] {}\n", now, message);
        use std::io::Write;
        if let Err(e) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut f| f.write_all(line.as_bytes()))
        {
            tracing::warn!("Failed to write backend log '{}': {}", log_path, e);
        }
    }

    /// Resolve MOTD based on server_address (for Status phase)
    pub fn get_motd(&self, protocol_version: i32, server_address: &str) -> String {
        let domain = extract_domain(server_address);

        // Domain isolation: if routing rules exist but current domain doesn't match, show "no route" MOTD
        if let Ok(routes) = self.domain_routes.try_read() {
            if !routes.is_empty() && !routes.iter().any(|r| r.matches(domain)) {
                let msg = self.i18n.messages.get("no_route_for_domain");
                return motd::default_motd("Minecraft Proxy", protocol_version, 0, 0, &msg)
                    .to_string();
            }
        }

        // First find backend corresponding to domain routing
        let backend = self.find_backend_for_domain(domain);

        if let Some(backend) = backend {
            return self.build_backend_motd(&backend, protocol_version);
        }

        // Show message when no backend available
        motd::default_motd(
            "Minecraft Rust Proxy",
            protocol_version,
            0,
            0,
            &self.i18n.messages.get("no_backend_configured"),
        )
        .to_string()
    }

    /// Find backend server corresponding to domain
    pub fn find_backend_for_domain(&self, domain: &str) -> Option<BackendServerInfo> {
        // First check domain routing
        if let Ok(routes) = self.domain_routes.try_read() {
            for route in routes.iter() {
                if route.matches(domain) {
                    if let Some(backend_id) = route.backend_id {
                        if let Ok(backends) = self.backends.try_read() {
                            if let Some(backend) = backends.iter().find(|b| b.id == backend_id) {
                                return Some(backend.clone());
                            }
                        }
                    }
                }
            }
        }

        // Then check default backend
        if let Ok(backends) = self.backends.try_read() {
            if let Some(b) = backends.iter().find(|b| b.is_default && b.enabled) {
                return Some(b.clone());
            }
            if let Some(b) = backends.iter().find(|b| b.enabled) {
                return Some(b.clone());
            }
        }

        None
    }

    /// Build MOTD JSON string for backend
    fn build_backend_motd(&self, backend: &BackendServerInfo, protocol_version: i32) -> String {
        // Disabled status MOTD
        if !backend.enabled {
            let msg = self.i18n.messages.get("server_offline");
            return motd::default_motd(&backend.name, protocol_version, 0, 0, &msg).to_string();
        }

        // Maintenance mode MOTD
        if backend.maintenance {
            let default_maintenance_msg = self.i18n.messages.get("server_under_maintenance");
            let msg = backend.maintenance_message.as_deref()
                .unwrap_or(&default_maintenance_msg);
            return motd::default_motd(
                &backend.name,
                protocol_version,
                0,
                0,
                msg,
            ).to_string();
        }

        // Count online users for this backend
        let online_count = self.users.iter()
            .filter(|e| e.value().backend_name == backend.name)
            .count() as i32;

        // Custom MOTD JSON
        if let Some(motd_json) = &backend.motd_json {
            if let Ok(mut motd) = serde_json::from_str::<serde_json::Value>(motd_json) {
                // Always override with client protocol number to avoid version text showing as "Old"
                if motd.get("version").is_none() {
                    motd["version"] = serde_json::json!({"name": backend.name.as_str(), "protocol": protocol_version});
                } else if let Some(version) = motd.get_mut("version") {
                    version["protocol"] = serde_json::json!(protocol_version);
                }
                if let Some(players) = motd.get_mut("players") {
                    players["online"] = serde_json::json!(online_count);
                    if backend.max_player >= 0 {
                        players["max"] = serde_json::json!(backend.max_player);
                    }
                }
                return motd.to_string();
            }
        }

        // Default MOTD
        let max = if backend.max_player < 0 { 100 } else { backend.max_player };
        motd::default_motd(
            &backend.name,
            protocol_version,
            max,
            online_count,
            "A Minecraft Proxy powered by Rust",
        )
        .to_string()
    }

    /// Kick player
    pub fn kick_player(&self, username: &str) -> bool {
        if let Some(user_info) = self.users.get(username) {
            user_info.kick_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Reload backend server list
    pub async fn reload_backends(&self) {
        match db::backend_servers::get_all_backend_servers(&self.db_pool).await {
            Ok(db_backends) => {
                let backends_vec: Vec<BackendServerInfo> = db_backends
                    .into_iter()
                    .map(BackendServerInfo::from)
                    .collect();
                let len = backends_vec.len();
                *self.backends.write().await = backends_vec;
                tracing::info!("Reloaded {} backends", len);
            }
            Err(e) => {
                tracing::error!("Failed to reload backends: {}", e);
            }
        }
    }

    /// Reload domain routing
    pub async fn reload_domain_routes(&self) {
        match db::domain_routes::get_all_domain_routes(&self.db_pool).await {
            Ok(db_routes) => {
                let routes: Vec<DomainRoute> = db_routes
                    .into_iter()
                    .filter_map(|r| DomainRoute::from_db(&r).ok())
                    .collect();
                let len = routes.len();
                *self.domain_routes.write().await = routes;
                tracing::info!("Reloaded {} domain routes", len);
            }
            Err(e) => {
                tracing::error!("Failed to reload domain routes: {}", e);
            }
        }
    }

    /// Get online user list
    pub fn get_online_users(&self) -> Vec<(String, String, String, i64, u64, u64, i32)> {
        self.users
            .iter()
            .map(|entry| {
                let info = entry.value();
                (
                    info.username.clone(),
                    info.uuid.clone(),
                    info.backend_name.clone(),
                    info.login_time,
                    info.upload_bytes.load(std::sync::atomic::Ordering::Relaxed),
                    info.download_bytes.load(std::sync::atomic::Ordering::Relaxed),
                    info.protocol_version,
                )
            })
            .collect()
    }
}
