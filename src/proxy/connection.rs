use crate::db;
use crate::error::{ProxyError, Result};
use crate::motd;
use crate::protocol::handshake::HandshakePacket;
use crate::protocol::login::{LoginFailurePacket, LoginStartPacket};
use crate::protocol::packet::RawPacket;
use crate::protocol::status::{PingPacket, StatusResponse};
use crate::proxy::limbo;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::Proxy;

/// Check if client connection domain matches trusted domain rules
fn check_domain(domain: &str, trusted_domain: &str) -> bool {
    if trusted_domain.is_empty() {
        return true;
    }
    // Remove port number (e.g., :25565)
    let clean = domain.split(':').next().unwrap_or(domain);
    // FML marker already removed in extract_domain, clean is pure domain
    clean == trusted_domain || clean.ends_with(&format!(".{}", trusted_domain))
}

/// Check if string looks like an IP address (IPv4 / IPv6)
fn looks_like_ip(s: &str) -> bool {
    let host = s.split(':').next().unwrap_or(s);
    host.parse::<std::net::IpAddr>().is_ok()
}


/// Handle complete lifecycle of single client connection
pub async fn handle_connection(proxy: Arc<Proxy>, mut client: TcpStream, client_addr: SocketAddr) {
    if let Err(e) = handle_connection_inner(proxy, &mut client, client_addr).await {
        match &e {
            ProxyError::Io(io_err) if io_err.kind() == std::io::ErrorKind::UnexpectedEof => {}
            ProxyError::Io(io_err) if io_err.kind() == std::io::ErrorKind::ConnectionReset => {}
            _ => {
                tracing::debug!("Connection {} error: {}", client_addr, e);
            }
        }
    }
}

async fn handle_connection_inner(
    proxy: Arc<Proxy>,
    client: &mut TcpStream,
    client_addr: SocketAddr,
) -> Result<()> {
    client.set_nodelay(true).ok();

    // 1. Read Handshake packet
    let handshake_raw = RawPacket::read_from(client).await?;
    if handshake_raw.id != 0x00 {
        return Err(ProxyError::InvalidPacketId(handshake_raw.id));
    }
    let handshake = HandshakePacket::from_raw(&handshake_raw)?;

    tracing::debug!(
        "Handshake from {}: protocol={}, addr={}, state={}",
        client_addr,
        handshake.protocol_version,
        handshake.server_address,
        handshake.next_state
    );

    match handshake.next_state {
        1 => handle_status(proxy, client, &handshake).await,
        2 => handle_login(proxy, client, client_addr, handshake).await,
        _ => Err(ProxyError::Protocol(format!(
            "Invalid next_state: {}",
            handshake.next_state
        ))),
    }
}

/// Handle Status request (server list query)
async fn handle_status(
    proxy: Arc<Proxy>,
    client: &mut TcpStream,
    handshake: &HandshakePacket,
) -> Result<()> {
    let domain = super::router::extract_domain(&handshake.server_address);
    let trusted_domain = proxy.config.read().await.trusted_domain.clone();

    // Domain validation: untrusted domain -> special MOTD; IP direct allowed (route to default backend)
    let domain_ok = if !trusted_domain.is_empty() {
        looks_like_ip(domain) || check_domain(domain, &trusted_domain)
    } else {
        true
    };

    // Passthrough: handle separately after finding backend
    if domain_ok {
        // Domain isolation: if route rules exist but current domain doesn't match any, skip passthrough
        let domain_has_route = {
            let routes = proxy.domain_routes.read().await;
            routes.is_empty() || routes.iter().any(|r| r.matches(domain))
        };
        if domain_has_route {
            if let Some(backend) = proxy.find_backend_for_domain(domain) {
                if backend.motd_passthrough {
                    // Copy MOTD: fully relay backend MOTD + latency
                    return handle_status_motd_passthrough(client, handshake, &backend).await;
                }
                if backend.ping_passthrough {
                    // Ping passthrough: local MOTD + relay latency
                    return handle_status_ping_relay(proxy.clone(), client, handshake, &backend).await;
                }
            }
        }
    }

    loop {
        let packet = RawPacket::read_from(client).await?;
        match packet.id {
            0x00 => {
                let motd = if !domain_ok {
                    // Untrusted domain -> prompt to use correct domain
                    let msg = proxy.i18n.messages.get_with_args("use_domain_to_connect", &[("domain", &trusted_domain)]);
                    motd::default_motd("Minecraft Proxy", handshake.protocol_version, 0, 0, &msg)
                        .to_string()
                } else {
                    proxy.get_motd(handshake.protocol_version, &handshake.server_address)
                };
                let response = StatusResponse { json: motd };
                response.to_raw().write_to(client).await?;
            }
            0x01 => {
                let ping = PingPacket::from_raw(&packet)?;
                ping.to_pong().write_to(client).await?;
                return Ok(());
            }
            _ => return Err(ProxyError::InvalidPacketId(packet.id)),
        }
    }
}

/// Copy MOTD: fully relay client Status request to backend, return real MOTD + real latency
async fn handle_status_motd_passthrough(
    client: &mut TcpStream,
    handshake: &HandshakePacket,
    backend: &super::BackendServerInfo,
) -> Result<()> {
    use tokio::time::timeout;
    use std::time::Duration;

    // Connect to backend
    let target = format!("{}:{}", backend.remote_address, backend.remote_port);
    let Ok(Ok(mut server)) = timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(&target)).await else {
        // Backend unreachable, fallback to local MOTD
        return handle_status_local(client, handshake).await;
    };
    server.set_nodelay(true).ok();

    // Send Handshake to backend (state=1)
    let backend_handshake = handshake.with_target(&backend.remote_address, backend.remote_port);
    backend_handshake.to_raw().write_to(&mut server).await?;
    // Send Status Request
    RawPacket::new(0x00, vec![]).write_to(&mut server).await?;

    // Read Status Response
    let status_raw = match timeout(Duration::from_secs(5), RawPacket::read_from(&mut server)).await {
        Ok(Ok(p)) => p,
        _ => return handle_status_local(client, handshake).await,
    };

    // Relay to client
    loop {
        let packet = RawPacket::read_from(client).await?;
        match packet.id {
            0x00 => {
                status_raw.write_to(client).await?;
            }
            0x01 => {
                // Relay Ping to backend
                packet.write_to(&mut server).await?;
                // Read Pong
                if let Ok(Ok(pong)) = timeout(Duration::from_secs(3), RawPacket::read_from(&mut server)).await {
                    pong.write_to(client).await?;
                } else {
                    let ping = PingPacket::from_raw(&packet)?;
                    ping.to_pong().write_to(client).await?;
                }
                return Ok(());
            }
            _ => return Err(ProxyError::InvalidPacketId(packet.id)),
        }
    }
}

/// Fallback: build local MOTD response
async fn handle_status_local(
    client: &mut TcpStream,
    handshake: &HandshakePacket,
) -> Result<()> {
    loop {
        let packet = RawPacket::read_from(client).await?;
        match packet.id {
            0x00 => {
                let motd = motd::default_motd("Minecraft Proxy", handshake.protocol_version, 0, 0, "").to_string();
                let response = StatusResponse { json: motd };
                response.to_raw().write_to(client).await?;
            }
            0x01 => {
                let ping = PingPacket::from_raw(&packet)?;
                ping.to_pong().write_to(client).await?;
                return Ok(());
            }
            _ => return Err(ProxyError::InvalidPacketId(packet.id)),
        }
    }
}

/// Ping passthrough: use local MOTD, but relay Ping/Pong to backend to reflect real latency
async fn handle_status_ping_relay(
    proxy: Arc<Proxy>,
    client: &mut TcpStream,
    handshake: &HandshakePacket,
    backend: &super::BackendServerInfo,
) -> Result<()> {
    use tokio::time::timeout;
    use std::time::Duration;

    // Pre-connect to backend, send Handshake(state=1) + Status Request, discard Status Response
    let target = format!("{}:{}", backend.remote_address, backend.remote_port);
    let mut server_opt: Option<TcpStream> = match timeout(Duration::from_secs(5), TcpStream::connect(&target)).await {
        Ok(Ok(mut s)) => {
            s.set_nodelay(true).ok();
            let backend_handshake = handshake.with_target(&backend.remote_address, backend.remote_port);
            let ok = backend_handshake.to_raw().write_to(&mut s).await.is_ok()
                && RawPacket::new(0x00, vec![]).write_to(&mut s).await.is_ok();
            if ok {
                // Read and discard backend Status Response
                let _ = timeout(Duration::from_secs(3), RawPacket::read_from(&mut s)).await;
                Some(s)
            } else {
                None
            }
        }
        _ => None,
    };

    // Local MOTD (built by proxy)
    let local_motd = proxy.get_motd(handshake.protocol_version, &handshake.server_address);

    loop {
        let packet = RawPacket::read_from(client).await?;
        match packet.id {
            0x00 => {
                // Return local MOTD
                let response = StatusResponse { json: local_motd.clone() };
                response.to_raw().write_to(client).await?;
            }
            0x01 => {
                // Relay Ping to backend to get real latency
                if let Some(ref mut server) = server_opt {
                    if packet.write_to(server).await.is_ok() {
                        if let Ok(Ok(pong)) = timeout(Duration::from_secs(3), RawPacket::read_from(server)).await {
                            pong.write_to(client).await?;
                            return Ok(());
                        }
                    }
                }
                // Local response when backend is unreachable
                let ping = PingPacket::from_raw(&packet)?;
                ping.to_pong().write_to(client).await?;
                return Ok(());
            }
            _ => return Err(ProxyError::InvalidPacketId(packet.id)),
        }
    }
}

/// Handle Login and establish proxy forwarding
async fn handle_login(
    proxy: Arc<Proxy>,
    client: &mut TcpStream,
    client_addr: SocketAddr,
    handshake: HandshakePacket,
) -> Result<()> {
    // 2. Read Login Start packet
    let login_raw = RawPacket::read_from(client).await?;
    if login_raw.id != 0x00 {
        return Err(ProxyError::InvalidPacketId(login_raw.id));
    }
    let login = LoginStartPacket::from_raw(&login_raw)?;
    let username = login.username.clone();

    // 2.3 Trusted domain check: only reject wrong domain, IP direct connect allowed (route to default backend)
    {
        let trusted_domain = proxy.config.read().await.trusted_domain.clone();
        if !trusted_domain.is_empty() {
            let server_domain = super::router::extract_domain(&handshake.server_address);
            if !looks_like_ip(server_domain) && !check_domain(server_domain, &trusted_domain) {
                let msg = proxy.i18n.messages.get_with_args("use_domain_to_connect", &[("domain", &trusted_domain)]);
                let failure = LoginFailurePacket::new(&msg);
                failure.to_raw().write_to(client).await?;
                return Ok(());
            }
        }
    }

    // 2.5 Version check
    if !crate::protocol::play::is_supported_version(handshake.protocol_version) {
        tracing::info!(
            "Player {} rejected: unsupported protocol version {}",
            username,
            handshake.protocol_version
        );
        let failure = LoginFailurePacket::new(&proxy.i18n.messages.get("unsupported_version"));
        failure.to_raw().write_to(client).await?;
        return Ok(());
    }

    // Extract pure domain (remove FML marker)
    let server_domain = super::router::extract_domain(&handshake.server_address);
    tracing::info!(
        "Player {} connecting (server: {}, from: {})",
        username,
        server_domain,
        client_addr
    );

    // 3. Resolve target backend early (for maintenance mode and max player check)
    let target = proxy.resolve_target(&username, &handshake.server_address).await;

    // 3.5 Maintenance mode check
    if target.maintenance {
        let default_maintenance_msg = proxy.i18n.messages.get("server_under_maintenance");
        let msg = target.maintenance_message.as_deref()
            .unwrap_or(&default_maintenance_msg);
        tracing::info!("Player {} rejected: backend {} is in maintenance", username, target.backend_name);
        let failure = LoginFailurePacket::new(msg);
        failure.to_raw().write_to(client).await?;
        return Ok(());
    }

    // 4. Get UUID and skin
    let mojang_profile = crate::protocol::login::lookup_mojang_profile(&username).await;
    let uuid_bytes = if let Some(uuid) = login.uuid {
        uuid
    } else if let Some(ref profile) = mojang_profile {
        profile.uuid
    } else {
        login.uuid_bytes()
    };
    let uuid_str = crate::protocol::login::format_uuid_bytes(&uuid_bytes);
    let textures_value = mojang_profile.as_ref().and_then(|p| p.textures_value.clone());
    let textures_signature = mojang_profile.as_ref().and_then(|p| p.textures_signature.clone());

    // 5. Access control check
    let (whitelist_enabled, in_blacklist, in_whitelist_mem) = {
        let ac = proxy.access_control.read().await;
        (
            ac.whitelist_enabled,
            ac.blacklist.contains(&username.to_lowercase()),
            ac.whitelist.contains(&username.to_lowercase()),
        )
    };

    if in_blacklist {
        tracing::info!("Player {} rejected: in blacklist", username);
        let ban_reason = db::access::get_blacklist_reason(&proxy.db_pool, &username)
            .await
            .ok()
            .flatten();
        let message = ban_reason.unwrap_or_else(|| proxy.i18n.messages.get("banned").clone());
        let failure = LoginFailurePacket::new(&message);
        failure.to_raw().write_to(client).await?;
        return Ok(());
    }

    let is_allowed = if whitelist_enabled && !in_whitelist_mem {
        db::access::is_whitelisted(&proxy.db_pool, &username)
            .await
            .unwrap_or(false)
    } else {
        !whitelist_enabled || in_whitelist_mem
    };

    if !is_allowed {
        tracing::info!("Player {} rejected: not whitelisted", username);
        let failure = LoginFailurePacket::new(&proxy.i18n.messages.get("not_whitelisted"));
        failure.to_raw().write_to(client).await?;
        return Ok(());
    }

    // 6. Per-backend max player check
    if target.max_player >= 0 {
        // Count players connected to this backend
        let backend_online = proxy.users.iter()
            .filter(|e| e.value().backend_name == target.backend_name)
            .count() as i32;
        if backend_online >= target.max_player {
            tracing::info!("Player {} rejected: backend {} is full ({}/{})", username, target.backend_name, backend_online, target.max_player);
            let failure = LoginFailurePacket::new(&proxy.i18n.messages.get("server_full"));
            failure.to_raw().write_to(client).await?;
            return Ok(());
        }
    }

    // 7. 2FA check
    let tf_guard = proxy.two_factor.read().await;
    if let Some(ref two_factor) = *tf_guard {
        if !two_factor.has_valid_session(&username).await {
            // Load backend language i18n messages
            let backend_msgs = if !target.language.is_empty() {
                crate::i18n::Messages::load(&target.language)
            } else {
                proxy.i18n.messages.clone()
            };

            if two_factor.has_2fa(&username).await {
                tracing::info!("Player {} entering 2FA verification limbo", username);
                let result = limbo::run_limbo_verify(
                    client,
                    handshake.protocol_version,
                    &username,
                    &uuid_str,
                    &uuid_bytes,
                    two_factor,
                    textures_value.as_deref(),
                    textures_signature.as_deref(),
                    &backend_msgs,
                    target.limbo_message.as_deref(),
                )
                .await;
                match result {
                    Ok(limbo::LimboResult::Verified) => {
                        tracing::info!("Player {} 2FA verified, please reconnect", username);
                    }
                    Ok(limbo::LimboResult::Failed(reason)) => {
                        tracing::info!("Player {} 2FA failed: {}", username, reason);
                    }
                    Err(e) => {
                        tracing::debug!("Player {} limbo error: {}", username, e);
                    }
                    _ => {}
                }
                return Ok(());
            } else {
                tracing::info!("Player {} entering 2FA setup limbo", username);
                let result = limbo::run_limbo_setup(
                    client,
                    handshake.protocol_version,
                    &username,
                    &uuid_str,
                    &uuid_bytes,
                    two_factor,
                    textures_value.as_deref(),
                    textures_signature.as_deref(),
                    &backend_msgs,
                    target.limbo_message.as_deref(),
                )
                .await;
                match result {
                    Ok(limbo::LimboResult::SetupComplete) => {
                        tracing::info!("Player {} 2FA setup complete", username);
                    }
                    Ok(limbo::LimboResult::Failed(reason)) => {
                        tracing::info!("Player {} 2FA setup failed: {}", username, reason);
                    }
                    Err(e) => {
                        tracing::debug!("Player {} limbo error: {}", username, e);
                    }
                    _ => {}
                }
                return Ok(());
            }
        }
        tracing::debug!("Player {} has valid 2FA session", username);
    }
    drop(tf_guard); // Release lock to avoid holding across await

    // 8. Duplicate connection check
    if proxy.users.contains_key(&username) {
        tracing::info!("Player {} already connected, rejecting duplicate", username);
        let failure = LoginFailurePacket::new(&proxy.i18n.messages.get("already_connected"));
        failure.to_raw().write_to(client).await?;
        return Ok(());
    }

    tracing::info!(
        "Connecting {} to backend {} ({}:{})",
        username,
        target.backend_name,
        target.target_addr,
        target.target_port
    );

    // 9. Connect to backend server
    let target_host = format!("{}:{}", target.target_addr, target.target_port);
    let server_addr = tokio::net::lookup_host(&target_host)
        .await?
        .next()
        .ok_or_else(|| {
            ProxyError::Other(format!("Failed to resolve backend address: {}", target_host))
        })?;
    let mut server = TcpStream::connect(server_addr).await.map_err(|e| {
        ProxyError::Other(format!(
            "Failed to connect to backend {} ({}:{}): {}",
            target.backend_name, target.target_addr, target.target_port, e
        ))
    })?;
    server.set_nodelay(true).ok();

    // 10. Forward Handshake to backend
    let target_handshake = handshake.with_target(&target.target_addr, target.target_port);
    target_handshake.to_raw().write_to(&mut server).await?;

    // 11. Forward raw Login packet to backend
    login_raw.write_to(&mut server).await?;

    // 12. Register online user
    let login_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let session_id = match db::sessions::create_session(
        &proxy.db_pool,
        &username,
        &uuid_str,
        &target.target_addr,
        target.target_port,
        handshake.protocol_version,
    )
    .await
    {
        Ok(id) => Some(id),
        Err(e) => {
            tracing::warn!("Failed to create session for {}: {}", username, e);
            None
        }
    };

    let user_info = Arc::new(super::UserInfo {
        username: username.clone(),
        uuid: uuid_str.clone(),
        protocol_version: handshake.protocol_version,
        login_time: login_timestamp,
        upload_bytes: std::sync::atomic::AtomicU64::new(0),
        download_bytes: std::sync::atomic::AtomicU64::new(0),
        remote_addr: target.socket_addr,
        kick_flag: std::sync::atomic::AtomicBool::new(false),
        kick_reason: std::sync::Mutex::new(None),
        session_id,
        backend_name: target.backend_name.clone(),
    });
    proxy.users.insert(username.clone(), user_info.clone());

    tracing::info!("Player {} connected to {}", username, target.backend_name);
    // Write backend-specific log
    if let Some(ref log_dir) = target.log_dir {
        if !log_dir.is_empty() {
            super::Proxy::write_backend_log(
                log_dir,
                &format!("CONNECT {} ({}) → {}", username, client_addr, target.backend_name),
            );
        }
    }

    // 13. Raw TCP bidirectional relay
    let result = forward_raw_tcp(client, &mut server, &user_info).await;

    // 14. Cleanup
    proxy.users.remove(&username);

    if let Some(sid) = session_id {
        let upload = user_info.upload_bytes.load(Ordering::Relaxed);
        let download = user_info.download_bytes.load(Ordering::Relaxed);
        let kick_reason = user_info.kick_reason.lock().unwrap().take();
        if let Err(e) = db::sessions::update_session(
            &proxy.db_pool,
            sid,
            upload,
            download,
            kick_reason.as_deref(),
        )
        .await
        {
            tracing::warn!("Failed to update session for {}: {}", username, e);
        }
    }

    let up = user_info.upload_bytes.load(Ordering::Relaxed);
    let down = user_info.download_bytes.load(Ordering::Relaxed);
    tracing::info!(
        "Player {} disconnected from {} (up: {} bytes, down: {} bytes)",
        username, target.backend_name, up, down
    );
    // Write backend-specific log
    if let Some(ref log_dir) = target.log_dir {
        if !log_dir.is_empty() {
            super::Proxy::write_backend_log(
                log_dir,
                &format!("DISCONNECT {} ({}) ← {} (up: {} bytes, down: {} bytes)", username, client_addr, target.backend_name, up, down),
            );
        }
    }

    result
}

/// Raw TCP bidirectional relay
async fn forward_raw_tcp(
    client: &mut TcpStream,
    server: &mut TcpStream,
    user: &super::UserInfo,
) -> Result<()> {
    let mut client_buf = vec![0u8; 32768];
    let mut server_buf = vec![0u8; 32768];

    loop {
        tokio::select! {
            result = client.read(&mut client_buf) => {
                let n = result?;
                if n == 0 {
                    tracing::debug!("Player {} client closed connection", user.username);
                    break;
                }
                if user.kick_flag.load(Ordering::SeqCst) {
                    tracing::info!("Player {} kicked, closing connection", user.username);
                    break;
                }
                user.upload_bytes.fetch_add(n as u64, Ordering::Relaxed);
                server.write_all(&client_buf[..n]).await?;
            }
            result = server.read(&mut server_buf) => {
                let n = result?;
                if n == 0 {
                    let down = user.download_bytes.load(Ordering::Relaxed);
                    if down == 0 {
                        tracing::warn!(
                            "Player {} backend closed connection immediately (0 bytes received) - check backend server configuration",
                            user.username
                        );
                    } else {
                        tracing::debug!("Player {} backend closed connection", user.username);
                    }
                    break;
                }
                if user.kick_flag.load(Ordering::SeqCst) {
                    tracing::info!("Player {} kicked, closing connection", user.username);
                    break;
                }
                user.download_bytes.fetch_add(n as u64, Ordering::Relaxed);
                client.write_all(&server_buf[..n]).await?;
            }
        }
    }

    Ok(())
}
