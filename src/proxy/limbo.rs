use crate::auth::qr_map;
use crate::auth::TwoFactorManager;
use crate::error::Result;
use crate::protocol::chat;
use crate::protocol::packet::RawPacket;
use crate::protocol::play;
use crate::protocol::varint;
use crate::protocol::versions::*;
use tokio::net::TcpStream;

/// Limbo session result
pub enum LimboResult {
    /// 2FA verified successfully, can connect to backend
    Verified,
    /// First-time binding successful, need to reconnect
    SetupComplete,
    /// Verification failed/timeout
    Failed(String),
}

use crate::i18n::Messages;

/// Run Limbo session - first-time 2FA binding
pub async fn run_limbo_setup(
    client: &mut TcpStream,
    protocol_version: i32,
    username: &str,
    _uuid_str: &str,
    uuid_bytes: &[u8; 16],
    two_factor: &TwoFactorManager,
    textures_value: Option<&str>,
    textures_signature: Option<&str>,
    i18n: &Messages,
    limbo_message: Option<&str>,
) -> Result<LimboResult> {
    // 1. Send Login Success
    let login_success = play::build_login_success(uuid_bytes, username, protocol_version);
    login_success.write_to(client).await?;

    // 1.20.2+ needs Configuration phase handling
    if protocol_version >= PROTO_1_20_2 {
        handle_configuration_phase(client, protocol_version).await?;
    }

    // 1.7.x needs 100ms delay before sending Play phase packets
    if protocol_version <= PROTO_1_7_MAX {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // 2. Send Play phase packets
    send_play_packets(client, protocol_version, uuid_bytes, username, textures_value, textures_signature).await?;

    // 3. Send backend custom limbo message (if any)
    if let Some(msg) = limbo_message {
        if !msg.is_empty() {
            let custom_msg = chat::build_chat_packet(
                protocol_version,
                &serde_json::json!({"text": msg, "color": "yellow"}).to_string(),
            );
            custom_msg.write_to(client).await?;
        }
    }

    // 6. Generate 2FA secret and QR code
    let (secret, otpauth_uri) = two_factor.setup_2fa(username);

    // 7. Send QR code map (1.7 has completely different format, skip; 1.8+ only)
    // Delayed send: wait for client to complete first frame initialization to avoid inventory reset
    let mut active_map_id: Option<i32> = None;
    if protocol_version >= PROTO_1_8 {
        if let Some(pixels) = qr_map::render_qr_to_map(&otpauth_uri) {
            // Use unique map_id per session to avoid client cache conflicts
            // Limit to 1..=32767 range, compatible with 1.8 i16 damage field
            let map_id = (rand::random::<u16>() % 32767 + 1) as i32;
            active_map_id = Some(map_id);

            // Delay 75ms to wait for client initialization
            tokio::time::sleep(std::time::Duration::from_millis(75)).await;

            // 0. WindowItems: initialize inventory baseline (required for 1.21, otherwise SetSlot is dropped)
            if protocol_version >= PROTO_1_21 {
                let window_items = play::build_window_items_empty(protocol_version);
                window_items.write_to(client).await?;
            }

            // 1. SetSlot: place map item in hotbar 0 (slot 36), with map_id component + custom name
            let set_slot = play::build_set_slot_map(protocol_version, map_id, Some("Scan QR to verify"));
            set_slot.write_to(client).await?;

            // 2. HeldItemChange: switch held slot to 0
            let held_item = play::build_held_item_change(protocol_version);
            held_item.write_to(client).await?;

            // 3. MapData: send map pixel data (send twice, compatible with client cache strategy)
            let map_data = play::build_map_data(protocol_version, map_id, &pixels);
            map_data.write_to(client).await?;
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            map_data.write_to(client).await?;
        }
    }

    // 8. Send chat message: display URI and prompt
    //    For 1.7 versions that don't support maps, provide clickable QR code web link
    let qr_url = format!(
        "https://api.qrserver.com/v1/create-qr-code/?size=256x256&data={}",
        urlencoding::encode(&otpauth_uri)
    );

    let mut extra = vec![
        serde_json::json!({"text": i18n.get("2fa_setup_title"), "color": "gold", "bold": true}),
    ];

    if protocol_version >= PROTO_1_8 {
        // 1.8+: has map QR code, prefer to use
        extra.push(serde_json::json!({"text": i18n.get("2fa_setup_scan"), "color": "yellow"}));
        extra.push(serde_json::json!({"text": i18n.get("2fa_setup_or")}));
    }

    // Clickable QR code web link (available for all versions)
    extra.push(serde_json::json!({
        "text": i18n.get("2fa_setup_click_here"),
        "color": "aqua", "underlined": true,
        "clickEvent": {"action": "open_url", "value": &qr_url}
    }));
    extra.push(serde_json::json!({"text": i18n.get("2fa_setup_copy_uri"), "color": "yellow"}));
    if protocol_version >= PROTO_1_19_2 {
        // 1.19.2+: copy_to_clipboard support
        extra.push(serde_json::json!({
            "text": &otpauth_uri, "color": "aqua", "underlined": true,
            "clickEvent": {"action": "copy_to_clipboard", "value": &otpauth_uri}
        }));
    } else {
        // 1.7-1.8: copy_to_clipboard doesn't exist, use suggest_command so user can see text
        extra.push(serde_json::json!({
            "text": &otpauth_uri, "color": "aqua"
        }));
    }
    extra.push(serde_json::json!({"text": i18n.get("2fa_setup_verify_prompt"), "color": "green"}));

    let setup_msg = chat::build_chat_packet(
        protocol_version,
        &serde_json::json!({"text": "", "extra": extra}).to_string(),
    );
    setup_msg.write_to(client).await?;

    // 9. Wait for verification code input (with Keep Alive and timeout)
    let result = wait_for_verification(
        client,
        protocol_version,
        &secret,
        true,
        active_map_id,
        i18n,
    )
    .await?;

    match result {
        Some(true) => {
            // Save secret
            if let Err(e) = two_factor.confirm_2fa(username, &secret).await {
                tracing::error!("Failed to save 2FA secret for {}: {}", username, e);
            }
            // Record session (tracked by username)
            two_factor.create_session(username).await;

            // Send success message and disconnect
            let success_msg = chat::build_chat_packet(
                protocol_version,
                &serde_json::json!({
                    "text": i18n.get("2fa_setup_complete_chat"),
                    "color": "green",
                    "bold": true
                }).to_string(),
            );
            success_msg.write_to(client).await?;

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let disconnect = play::build_disconnect(
                protocol_version,
                &i18n.get("2fa_setup_complete_disconnect"),
            );
            disconnect.write_to(client).await?;

            Ok(LimboResult::SetupComplete)
        }
        _ => {
            let disconnect = play::build_disconnect(
                protocol_version,
                &i18n.get("2fa_setup_failed"),
            );
            disconnect.write_to(client).await?;
            Ok(LimboResult::Failed("Setup failed or timed out".into()))
        }
    }
}

/// Run Limbo session - verify existing 2FA
pub async fn run_limbo_verify(
    client: &mut TcpStream,
    protocol_version: i32,
    username: &str,
    _uuid_str: &str,
    uuid_bytes: &[u8; 16],
    two_factor: &TwoFactorManager,
    textures_value: Option<&str>,
    textures_signature: Option<&str>,
    i18n: &Messages,
    limbo_message: Option<&str>,
) -> Result<LimboResult> {
    // 1. Send Login Success
    let login_success = play::build_login_success(uuid_bytes, username, protocol_version);
    login_success.write_to(client).await?;

    // 1.20.2+ needs Configuration phase handling
    if protocol_version >= PROTO_1_20_2 {
        handle_configuration_phase(client, protocol_version).await?;
    }

    // 1.7.x needs 100ms delay before sending Play phase packets
    if protocol_version <= PROTO_1_7_MAX {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // 2. Send Play phase packets
    send_play_packets(client, protocol_version, uuid_bytes, username, textures_value, textures_signature).await?;

    // 3. Send backend custom limbo message (if any)
    if let Some(msg) = limbo_message {
        if !msg.is_empty() {
            let custom_msg = chat::build_chat_packet(
                protocol_version,
                &serde_json::json!({"text": msg, "color": "yellow"}).to_string(),
            );
            custom_msg.write_to(client).await?;
        }
    }

    // 6. Send verification prompt
    let verify_msg = chat::build_chat_packet(
        protocol_version,
        &serde_json::json!({
            "text": "",
            "extra": [
                {"text": i18n.get("2fa_verify_title"), "color": "gold", "bold": true},
                {"text": i18n.get("2fa_verify_enter_code"), "color": "yellow"}
            ]
        }).to_string(),
    );
    verify_msg.write_to(client).await?;

    // 7. Wait for verification code (verification mode has no map)
    let result = wait_for_verification_existing(
        client,
        protocol_version,
        username,
        two_factor,
        None,
        i18n,
    )
    .await?;

    match result {
        Some(true) => {
            // Record session (tracked by username)
            two_factor.create_session(username).await;

            let success_msg = chat::build_chat_packet(
                protocol_version,
                &serde_json::json!({
                    "text": i18n.get("2fa_verify_success_chat"),
                    "color": "green",
                    "bold": true
                }).to_string(),
            );
            success_msg.write_to(client).await?;

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let disconnect = play::build_disconnect(
                protocol_version,
                &i18n.get("2fa_verify_success_disconnect"),
            );
            disconnect.write_to(client).await?;

            Ok(LimboResult::Verified)
        }
        _ => {
            let disconnect = play::build_disconnect(
                protocol_version,
                &i18n.get("2fa_verify_failed"),
            );
            disconnect.write_to(client).await?;
            Ok(LimboResult::Failed("Verification failed or timed out".into()))
        }
    }
}

/// Resend SetSlot + HeldItemChange, force restore held map
async fn enforce_map_in_hand(
    client: &mut TcpStream,
    protocol_version: i32,
    map_id: i32,
    custom_name: Option<&str>,
) -> Result<()> {
    let set_slot = play::build_set_slot_map(protocol_version, map_id, custom_name);
    set_slot.write_to(client).await?;
    let held_item = play::build_held_item_change(protocol_version);
    held_item.write_to(client).await?;
    Ok(())
}

/// Check if block_dig packet status is a discard action (status 3=drop stack, 4=drop item)
fn is_drop_action(data: &[u8], protocol_version: i32) -> bool {
    if protocol_version >= PROTO_1_21 {
        // 1.21+: status is VarInt
        if let Ok((status, _)) = varint::read_varint_from_bytes(data) {
            status == 3 || status == 4
        } else {
            false
        }
    } else {
        // 1.7-1.8: status is first byte (Byte)
        data.first().map_or(false, |&s| s == 3 || s == 4)
    }
}

/// Wait for verification code input (for first-time binding - uses temporary secret)
async fn wait_for_verification(
    client: &mut TcpStream,
    protocol_version: i32,
    secret: &[u8],
    _is_setup: bool,
    map_id: Option<i32>,
    i18n: &Messages,
) -> Result<Option<bool>> {
    let timeout = std::time::Duration::from_secs(300); // 5 minutes timeout
    let start = std::time::Instant::now();
    let mut keep_alive_timer = tokio::time::interval(std::time::Duration::from_secs(15));
    let mut keep_alive_id: i64 = 0;

    let chat_id = play::get_client_chat_id(protocol_version);
    let held_item_id = play::get_client_held_item_id(protocol_version);
    let block_dig_id = play::get_client_block_dig_id(protocol_version);
    let window_click_id = play::get_client_window_click_id(protocol_version);

    loop {
        if start.elapsed() > timeout {
            return Ok(None);
        }

        tokio::select! {
            packet = RawPacket::read_from(client) => {
                let packet = packet?;

                if packet.id == chat_id {
                    // Parse chat message
                    let message = extract_chat_message(&packet.data, protocol_version)?;
                    let trimmed = message.trim();

                    if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit()) {
                        if TwoFactorManager::verify_setup_code(secret, trimmed) {
                            return Ok(Some(true));
                        } else {
                            // Send error message
                            let err_msg = chat::build_chat_packet(
                                protocol_version,
                                &serde_json::json!({
                                    "text": i18n.get("2fa_invalid_code"),
                                    "color": "red"
                                }).to_string(),
                            );
                            err_msg.write_to(client).await?;
                        }
                    }
                } else if let Some(mid) = map_id {
                    // Intercept held item switch/discard/inventory click, force restore map
                    if packet.id == held_item_id
                        || packet.id == window_click_id
                        || (packet.id == block_dig_id && is_drop_action(&packet.data, protocol_version))
                    {
                        enforce_map_in_hand(client, protocol_version, mid, Some("Scan QR to verify")).await?;
                    }
                }
                // Ignore other packets (Keep Alive response, etc.)
            }
            _ = keep_alive_timer.tick() => {
                let ka = play::build_keep_alive(protocol_version, keep_alive_id);
                if ka.write_to(client).await.is_err() {
                    return Ok(None);
                }
                keep_alive_id += 1;
            }
        }
    }
}

/// Wait for verification code input (for bound users - uses stored secret)
async fn wait_for_verification_existing(
    client: &mut TcpStream,
    protocol_version: i32,
    username: &str,
    two_factor: &TwoFactorManager,
    map_id: Option<i32>,
    i18n: &Messages,
) -> Result<Option<bool>> {
    let timeout = std::time::Duration::from_secs(300);
    let start = std::time::Instant::now();
    let mut keep_alive_timer = tokio::time::interval(std::time::Duration::from_secs(15));
    let mut keep_alive_id: i64 = 0;
    let mut attempts = 0;
    let max_attempts = 5;

    let chat_id = play::get_client_chat_id(protocol_version);
    let held_item_id = play::get_client_held_item_id(protocol_version);
    let block_dig_id = play::get_client_block_dig_id(protocol_version);
    let window_click_id = play::get_client_window_click_id(protocol_version);

    loop {
        if start.elapsed() > timeout {
            return Ok(None);
        }

        tokio::select! {
            packet = RawPacket::read_from(client) => {
                let packet = packet?;

                if packet.id == chat_id {
                    let message = extract_chat_message(&packet.data, protocol_version)?;
                    let trimmed = message.trim();

                    if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit()) {
                        if two_factor.verify_code(username, trimmed).await {
                            return Ok(Some(true));
                        } else {
                            attempts += 1;
                            if attempts >= max_attempts {
                                return Ok(Some(false));
                            }
                            let err_msg = chat::build_chat_packet(
                                protocol_version,
                                &serde_json::json!({
                                    "text": i18n.get_with_args("2fa_invalid_code_attempts", &[("attempts", &(max_attempts - attempts).to_string())]),
                                    "color": "red"
                                }).to_string(),
                            );
                            err_msg.write_to(client).await?;
                        }
                    }
                } else if let Some(mid) = map_id {
                    if packet.id == held_item_id
                        || packet.id == window_click_id
                        || (packet.id == block_dig_id && is_drop_action(&packet.data, protocol_version))
                    {
                        enforce_map_in_hand(client, protocol_version, mid, Some("Scan QR to verify")).await?;
                    }
                }
            }
            _ = keep_alive_timer.tick() => {
                let ka = play::build_keep_alive(protocol_version, keep_alive_id);
                if ka.write_to(client).await.is_err() {
                    return Ok(None);
                }
                keep_alive_id += 1;
            }
        }
    }
}

/// Extract message text from client chat packet
fn extract_chat_message(data: &[u8], protocol_version: i32) -> Result<String> {
    if protocol_version >= PROTO_1_19 {
        // 1.19+: Chat Command / Chat Message packet format is complex
        // Chat Message (0x05): String message, ...
        let (message, _) = varint::read_string_from_bytes(data)?;
        Ok(message)
    } else {
        // Legacy: String message
        let (message, _) = varint::read_string_from_bytes(data)?;
        Ok(message)
    }
}

/// Send Play phase initialization packets
/// Order: JoinGame -> PlayerInfo -> GameEvent+Chunk -> Position -> TeleportConfirm(1.21) -> SpawnPos -> Abilities
async fn send_play_packets(
    client: &mut TcpStream,
    protocol_version: i32,
    uuid_bytes: &[u8; 16],
    username: &str,
    textures_value: Option<&str>,
    textures_signature: Option<&str>,
) -> Result<()> {
    // 1: Join Game
    let join_game = play::build_join_game(protocol_version);
    tracing::info!(
        "Sending Join Game: packet_id=0x{:02X}, data_len={}, proto={}",
        join_game.id,
        join_game.data.len(),
        protocol_version
    );
    join_game.write_to(client).await?;

    // 2: Player Info Add (with textures, only for Tab avatar, doesn't create entity)
    let player_info = play::build_player_info(
        protocol_version,
        uuid_bytes,
        username,
        textures_value,
        textures_signature,
    );
    player_info.write_to(client).await?;

    // 3: Empty Chunk (required for all versions, otherwise map won't render)
    if protocol_version == PROTO_1_8 {
        // 1.8: simple empty chunk (bitmask=0, 256 biome bytes)
        let chunk = play::build_empty_chunk_1_8();
        chunk.write_to(client).await?;
    } else if protocol_version >= PROTO_1_20_3 {
        // 1.21.x: GameEvent(start loading chunk) + empty chunk(0,0)
        let game_event = play::build_game_event_start_waiting_chunks(protocol_version);
        game_event.write_to(client).await?;

        let chunk = play::build_empty_chunk_modern(protocol_version, 0, 0);
        chunk.write_to(client).await?;
    }

    // 4: Player Position And Look
    let position = play::build_player_position_and_look(protocol_version);
    position.write_to(client).await?;

    // 5: 1.21+ must wait for Teleport Confirm
    if protocol_version >= PROTO_1_21 {
        wait_for_teleport_confirm(client).await?;
    }

    // 6: Spawn Position (1.21.x)
    if protocol_version >= PROTO_1_19_3 {
        let spawn_pos = play::build_spawn_position(protocol_version);
        spawn_pos.write_to(client).await?;
    }

    // 7: Player Abilities (flags=0x02, invulnerable only, no flying to avoid jitter)
    let abilities = play::build_player_abilities(protocol_version);
    abilities.write_to(client).await?;

    Ok(())
}

/// Wait for client to send Teleport Confirm (Packet ID 0x00 in Play state)
async fn wait_for_teleport_confirm(client: &mut TcpStream) -> Result<()> {
    let timeout = std::time::Duration::from_secs(5);
    match tokio::time::timeout(timeout, async {
        loop {
            let packet = RawPacket::read_from(client).await?;
            if packet.id == 0x00 {
                // Confirm teleportation
                return Ok::<(), crate::error::ProxyError>(());
            }
            // Ignore other packets (like Client Information, etc.)
            tracing::debug!("Waiting for teleport confirm, ignoring packet 0x{:02X}", packet.id);
        }
    })
    .await
    {
        Ok(result) => result,
        Err(_) => {
            tracing::warn!("Teleport confirm timeout, continuing anyway");
            Ok(())
        }
    }
}

/// Handle 1.20.2+ Configuration phase
/// Strategy: skip Known Packs exchange, directly send full registry data (NanoLimbo style).
/// Send an independent RegistryData packet (0x07) for each registry type, has_data=true.
async fn handle_configuration_phase(
    client: &mut TcpStream,
    protocol_version: i32,
) -> Result<()> {
    use crate::protocol::registry;

    // 1. Wait for Login Acknowledged (0x03)
    let ack = RawPacket::read_from(client).await?;
    if ack.id != 0x03 {
        tracing::warn!("Expected Login Acknowledged, got 0x{:02X}", ack.id);
    }

    // 2. Send all Registry Data packets (from pre-encoded cache)
    let registry_packets = registry::get_registry_packets(protocol_version);
    tracing::info!(
        "Sending {} registry packets for protocol {}",
        registry_packets.len(),
        protocol_version
    );
    for packet in registry_packets {
        packet.write_to(client).await?;
    }

    // 3. Send UpdateTags packet (enchantment etc. registry references tag definitions)
    if let Some(tags_packet) = registry::get_update_tags_packet(protocol_version) {
        tags_packet.write_to(client).await?;
    }

    // 4. Finish Configuration
    play::build_finish_configuration().write_to(client).await?;

    // 5. Wait for client to confirm Finish Configuration (0x03)
    // Ignore other packets the client might send (e.g., Known Packs response 0x07, Plugin Message, etc.)
    loop {
        let packet = RawPacket::read_from(client).await?;
        if packet.id == 0x03 {
            tracing::info!("Configuration completed");
            break;
        }
        tracing::debug!(
            "Configuration phase: ignoring client packet 0x{:02X}",
            packet.id
        );
    }

    Ok(())
}

