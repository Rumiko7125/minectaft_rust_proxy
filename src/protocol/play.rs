use crate::protocol::packet::RawPacket;
use crate::protocol::varint;
use crate::protocol::versions::*;

/// Supported protocol versions: 1.7.x (4-5), 1.8.x (47), 1.21.x (767+)
pub fn is_supported_version(protocol_version: i32) -> bool {
    matches!(protocol_version, PROTO_1_7_MIN..=PROTO_1_7_MAX | PROTO_1_8 | PROTO_1_21..)
}

/// Login Success packet (Packet ID 0x02)
pub fn build_login_success(
    uuid_bytes: &[u8; 16],
    username: &str,
    protocol_version: i32,
) -> RawPacket {
    let mut data = Vec::new();

    if protocol_version >= PROTO_1_21 {
        // 1.21.x: UUID as 128-bit
        data.extend_from_slice(uuid_bytes);
        varint::encode_string(username, &mut data);
        varint::encode_varint(0, &mut data); // Number Of Properties
        if protocol_version <= PROTO_1_21 {
            // 1.20.5-1.21.1 only: Strict Error Handling (removed in 1.21.2)
            data.push(0x01);
        }
    } else {
        // 1.7-1.8: UUID as string + username string
        let uuid_str = format_uuid(uuid_bytes);
        varint::encode_string(&uuid_str, &mut data);
        varint::encode_string(username, &mut data);
    }

    RawPacket::new(0x02, data)
}

/// Join Game packet - build by protocol version
pub fn build_join_game(protocol_version: i32) -> RawPacket {
    if protocol_version >= PROTO_1_20_2 {
        // 1.21.x (only 767+ reaches here after version check)
        build_join_game_modern(protocol_version)
    } else {
        // 1.7.x, 1.8.x
        build_join_game_legacy(protocol_version)
    }
}

/// 1.21.x Join Game (after Configuration phase)
fn build_join_game_modern(protocol_version: i32) -> RawPacket {
    let mut data = Vec::new();

    data.extend_from_slice(&1i32.to_be_bytes()); // Entity ID
    data.push(0x00); // Is Hardcore
    varint::encode_varint(3, &mut data); // Dimension Count
    varint::encode_string("minecraft:overworld", &mut data);
    varint::encode_string("minecraft:the_nether", &mut data);
    varint::encode_string("minecraft:the_end", &mut data);
    varint::encode_varint(0, &mut data); // Max Players
    varint::encode_varint(2, &mut data); // View Distance
    varint::encode_varint(1, &mut data); // Simulation Distance (1=minimum safe value, 0 causes inventory sync issues)
    data.push(0x00); // Reduced Debug Info
    data.push(0x01); // Enable Respawn Screen
    data.push(0x00); // Do Limited Crafting
    varint::encode_varint(0, &mut data); // Dimension Type
    varint::encode_string("minecraft:overworld", &mut data);
    data.extend_from_slice(&0i64.to_be_bytes()); // Hashed Seed
    data.push(0x02); // Game Mode - Adventure
    data.push(0xFF); // Previous Game Mode
    data.push(0x00); // Is Debug
    data.push(0x01); // Is Flat
    data.push(0x00); // Has Death Location
    varint::encode_varint(0, &mut data); // Portal Cooldown

    if protocol_version >= PROTO_1_21_2 {
        // 1.21.2+: Sea Level (inside SpawnInfo, before enforcesSecureChat)
        varint::encode_varint(63, &mut data);
    }

    data.push(0x00); // Enforces Secure Chat (1.20.5+)

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x30,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x2B,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x2C,  // 1.21.2-1.21.4
        _ => 0x2B,          // 1.21-1.21.1
    };
    RawPacket::new(packet_id, data)
}

/// 1.7-1.8 Join Game
fn build_join_game_legacy(protocol_version: i32) -> RawPacket {
    let mut data = Vec::new();

    data.extend_from_slice(&1i32.to_be_bytes()); // Entity ID
    data.push(0x02); // Game Mode - Adventure
    data.push(0x00); // Dimension (Byte) - both 1.7 and 1.8 are Byte

    data.push(0x00); // Difficulty
    data.push(0x01); // Max Players
    varint::encode_string("flat", &mut data); // Level Type

    if protocol_version >= PROTO_1_8 {
        data.push(0x00); // Reduced Debug Info (1.8+)
    }

    RawPacket::new(0x01, data)
}

/// Player Position And Look (Server → Client)
pub fn build_player_position_and_look(protocol_version: i32) -> RawPacket {
    let mut data = Vec::new();

    if protocol_version >= PROTO_1_21_2 {
        // 1.21.2+: teleportId at the front, new dx/dy/dz, flags is u32
        varint::encode_varint(1, &mut data); // Teleport ID
        data.extend_from_slice(&0.0f64.to_be_bytes()); // X
        data.extend_from_slice(&400.0f64.to_be_bytes()); // Y (high to avoid triggering terrain loading)
        data.extend_from_slice(&0.0f64.to_be_bytes()); // Z
        data.extend_from_slice(&0.0f64.to_be_bytes()); // dX (velocity)
        data.extend_from_slice(&0.0f64.to_be_bytes()); // dY (velocity)
        data.extend_from_slice(&0.0f64.to_be_bytes()); // dZ (velocity)
        data.extend_from_slice(&0.0f32.to_be_bytes()); // Yaw
        data.extend_from_slice(&0.0f32.to_be_bytes()); // Pitch
        data.extend_from_slice(&0u32.to_be_bytes()); // Flags (u32) - all absolute
    } else if protocol_version >= PROTO_1_21 {
        // 1.21-1.21.1: old format, flags is u8, teleportId at end
        data.extend_from_slice(&0.0f64.to_be_bytes()); // X
        data.extend_from_slice(&400.0f64.to_be_bytes()); // Y (high to avoid triggering terrain loading)
        data.extend_from_slice(&0.0f64.to_be_bytes()); // Z
        data.extend_from_slice(&0.0f32.to_be_bytes()); // Yaw
        data.extend_from_slice(&0.0f32.to_be_bytes()); // Pitch
        data.push(0x08); // Flags (u8) - 0x08 = yaw relative
        varint::encode_varint(1, &mut data); // Teleport ID
    } else if protocol_version >= PROTO_1_8 {
        // 1.8.x: X, Y, Z, Yaw, Pitch, Flags (u8)
        data.extend_from_slice(&0.0f64.to_be_bytes()); // X
        data.extend_from_slice(&64.0f64.to_be_bytes()); // Y (ground)
        data.extend_from_slice(&0.0f64.to_be_bytes()); // Z
        data.extend_from_slice(&0.0f32.to_be_bytes()); // Yaw
        data.extend_from_slice(&0.0f32.to_be_bytes()); // Pitch
        data.push(0x08); // Flags (u8) - 0x08 = yaw relative
    } else {
        // 1.7.x: X, Y (feet + eye height), Z, Yaw, Pitch, On Ground
        data.extend_from_slice(&0.0f64.to_be_bytes()); // X
        data.extend_from_slice(&65.62f64.to_be_bytes()); // Y (64 + 1.62 eye height)
        data.extend_from_slice(&0.0f64.to_be_bytes()); // Z
        data.extend_from_slice(&0.0f32.to_be_bytes()); // Yaw
        data.extend_from_slice(&0.0f32.to_be_bytes()); // Pitch
        data.push(0x01); // On Ground = true
    }

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x46,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x41,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x42,  // 1.21.2-1.21.4
        PROTO_1_21 => 0x40,        // 1.21-1.21.1
        PROTO_1_8 => 0x08,         // 1.8.x
        _ => 0x08,          // 1.7.x
    };
    RawPacket::new(packet_id, data)
}

/// Keep Alive (Server → Client)
pub fn build_keep_alive(protocol_version: i32, id: i64) -> RawPacket {
    let mut data = Vec::new();

    if protocol_version >= PROTO_1_21 {
        // 1.21.x: Long
        data.extend_from_slice(&id.to_be_bytes());
    } else if protocol_version >= PROTO_1_8 {
        // 1.8.x: VarInt
        varint::encode_varint(id as i32, &mut data);
    } else {
        // 1.7.x: Int (4 bytes, NOT VarInt)
        data.extend_from_slice(&(id as i32).to_be_bytes());
    }

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x2B,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x26,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x27,  // 1.21.2-1.21.4
        PROTO_1_21 => 0x26,        // 1.21-1.21.1
        PROTO_1_8 => 0x00,         // 1.8.x
        _ => 0x00,          // 1.7.x
    };
    RawPacket::new(packet_id, data)
}

/// Get Held Item Change packet ID sent by client (C→S)
pub fn get_client_held_item_id(protocol_version: i32) -> i32 {
    match protocol_version {
        PROTO_1_21_9.. => 0x34,      // 1.21.9+
        PROTO_1_21_6..=PROTO_1_21_7 => 0x34,  // 1.21.6-1.21.8
        PROTO_1_21_4..=PROTO_1_21_5 => 0x33,  // 1.21.4-1.21.5
        PROTO_1_21_2 => 0x31,        // 1.21.2-1.21.3
        PROTO_1_21 => 0x2f,        // 1.21-1.21.1
        _ => 0x09,          // 1.7-1.8
    }
}

/// Get Block Dig (Player Digging) packet ID sent by client (C→S)
pub fn get_client_block_dig_id(protocol_version: i32) -> i32 {
    match protocol_version {
        PROTO_1_21_9.. => 0x28,      // 1.21.9+
        PROTO_1_21_6..=PROTO_1_21_7 => 0x28,  // 1.21.6-1.21.8
        PROTO_1_21_4..=PROTO_1_21_5 => 0x27,  // 1.21.4-1.21.5
        PROTO_1_21_2 => 0x26,        // 1.21.2-1.21.3
        PROTO_1_21 => 0x24,        // 1.21-1.21.1
        _ => 0x07,          // 1.7-1.8
    }
}

/// Get Window Click (Click Container) packet ID sent by client (C→S)
pub fn get_client_window_click_id(protocol_version: i32) -> i32 {
    match protocol_version {
        PROTO_1_21_9.. => 0x11,      // 1.21.9+
        PROTO_1_21_6..=PROTO_1_21_7 => 0x11,  // 1.21.6-1.21.8
        PROTO_1_21_4..=PROTO_1_21_5 => 0x10,  // 1.21.4-1.21.5
        PROTO_1_21_2 => 0x10,        // 1.21.2-1.21.3
        PROTO_1_21 => 0x0e,        // 1.21-1.21.1
        _ => 0x0e,          // 1.7-1.8
    }
}

/// Get Chat Message packet ID sent by client (for receiving verification code)
pub fn get_client_chat_id(protocol_version: i32) -> i32 {
    match protocol_version {
        PROTO_1_21_6.. => 0x08,      // 1.21.6+
        PROTO_1_21_2..=PROTO_1_21_5 => 0x07,  // 1.21.2-1.21.5
        PROTO_1_21 => 0x06,        // 1.21-1.21.1
        PROTO_1_8 => 0x01,         // 1.8.x
        _ => 0x01,          // 1.7.x
    }
}

/// Map Data packet (for displaying QR code)
pub fn build_map_data(protocol_version: i32, map_id: i32, pixels: &[u8; 128 * 128]) -> RawPacket {
    let mut data = Vec::new();

    varint::encode_varint(map_id, &mut data); // Map ID
    data.push(0x00); // Scale

    if protocol_version >= PROTO_1_21 {
        // 1.21.x: Locked (Boolean)
        data.push(0x01);
        // Has Icons (Boolean) = false → don't write icon_count
        data.push(0x00);
    } else {
        // 1.7-1.8: Icon Count = 0 (always present, no conditional)
        varint::encode_varint(0, &mut data);
    }

    data.push(128); // Columns
    data.push(128); // Rows
    data.push(0);   // X Offset
    data.push(0);   // Z Offset
    varint::encode_varint(128 * 128, &mut data); // Length
    data.extend_from_slice(pixels);

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x31,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x2C,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x2D,  // 1.21.2-1.21.4
        PROTO_1_21 => 0x2C,        // 1.21-1.21.1
        PROTO_1_8 => 0x34,         // 1.8.x
        _ => 0x34,          // 1.7.x
    };
    RawPacket::new(packet_id, data)
}

/// Get filled_map item ID (differs per version, from minecraft-data)
fn get_filled_map_item_id(protocol_version: i32) -> i32 {
    match protocol_version {
        PROTO_1_21_11.. => 1104,      // 1.21.11+
        PROTO_1_21_9 => 1104,        // 1.21.9-1.21.10
        PROTO_1_21_6..=PROTO_1_21_7 => 1059,  // 1.21.6-1.21.8
        PROTO_1_21_5 => 1042,        // 1.21.5
        PROTO_1_21_4 => 1031,        // 1.21.3-1.21.4
        PROTO_1_21_2 => 1022,        // 1.21.2
        PROTO_1_21 => 982,         // 1.21-1.21.1
        _ => 358,           // 1.7-1.8
    }
}

/// Get minecraft:map_id component type ID (differs per version)
fn get_map_id_component_type(protocol_version: i32) -> i32 {
    match protocol_version {
        PROTO_1_21_11.. => 44,        // 1.21.11+
        PROTO_1_21_5..=PROTO_1_21_9 => 37,    // 1.21.5-1.21.9
        PROTO_1_21_2..=PROTO_1_21_4 => 36,    // 1.21.2-1.21.4
        _ => 26,            // 1.21-1.21.1
    }
}

/// Get minecraft:custom_name component type ID
fn get_custom_name_component_type(protocol_version: i32) -> i32 {
    match protocol_version {
        PROTO_1_21_11.. => 6,         // 1.21.11+
        _ => 5,             // 1.21-1.21.9
    }
}

/// Set Slot packet - put map item in specified slot (slot 36 = hotbar 0)
/// 1.21: contains minecraft:map_id + custom_name component
/// 1.8: contains display.Name NBT
/// custom_name: item display name (1.8 uses § color codes, 1.21 uses JSON Component)
pub fn build_set_slot_map(protocol_version: i32, map_id: i32, custom_name: Option<&str>) -> RawPacket {
    use crate::protocol::nbt;

    let mut data = Vec::new();

    data.push(0x00); // Window ID (player inventory)

    if protocol_version >= PROTO_1_21 {
        // 1.21.x: State ID (VarInt) - must increment after WindowItems(state_id=0)
        varint::encode_varint(1, &mut data);
    }

    data.extend_from_slice(&36i16.to_be_bytes()); // Slot = hotbar 0

    if protocol_version >= PROTO_1_20_5 {
        // 1.20.5+ / 1.21.x Slot format (wiki.vg):
        // count(VarInt) → item_id(VarInt) → add_count(VarInt) → remove_count(VarInt)
        // → [add components: type(VarInt) + data]* → [remove components: type(VarInt)]*
        let add_count = if custom_name.is_some() { 2 } else { 1 };
        varint::encode_varint(1, &mut data); // count = 1 (also means present)
        varint::encode_varint(get_filled_map_item_id(protocol_version), &mut data);
        varint::encode_varint(add_count, &mut data); // components to add
        varint::encode_varint(0, &mut data); // components to remove: 0

        // Component 1: minecraft:map_id
        varint::encode_varint(get_map_id_component_type(protocol_version), &mut data);
        varint::encode_varint(map_id, &mut data);

        // Component 2: minecraft:custom_name (Text Component as Network NBT)
        if let Some(name) = custom_name {
            varint::encode_varint(get_custom_name_component_type(protocol_version), &mut data);
            // custom_name payload is Text Component, encoded as Network NBT
            let name_json = serde_json::json!({
                "text": name,
                "color": "gold",
                "italic": false
            });
            let nbt_value = nbt::json_to_nbt(&name_json);
            let nbt_bytes = nbt::encode_network_nbt_value(&nbt_value);
            data.extend_from_slice(&nbt_bytes);
        }
    } else {
        // 1.7-1.8: item_id(short) + count(byte) + damage(short) + nbt
        data.extend_from_slice(&358i16.to_be_bytes()); // filled_map
        data.push(0x01); // count
        data.extend_from_slice(&(map_id as i16).to_be_bytes()); // damage = map id

        if let Some(name) = custom_name {
            // 1.8 NBT: standard format with named root
            // TAG_Compound(0x0A) + root_name("") + display compound + TAG_End
            data.push(0x0A); // TAG_Compound
            data.extend_from_slice(&0u16.to_be_bytes()); // root name length = 0
            // display compound
            data.push(0x0A); // TAG_Compound
            let display_name = b"display";
            data.extend_from_slice(&(display_name.len() as u16).to_be_bytes());
            data.extend_from_slice(display_name);
            // Name string
            data.push(0x08); // TAG_String
            let name_key = b"Name";
            data.extend_from_slice(&(name_key.len() as u16).to_be_bytes());
            data.extend_from_slice(name_key);
            // 1.8 uses § color code format
            let legacy_name = format!("§6{}", name);
            data.extend_from_slice(&(legacy_name.len() as u16).to_be_bytes());
            data.extend_from_slice(legacy_name.as_bytes());
            data.push(0x00); // TAG_End (display compound)
            data.push(0x00); // TAG_End (root compound)
        } else {
            data.push(0x00); // no NBT (TAG_End)
        }
    }

    let packet_id = match protocol_version {
        PROTO_1_21_5.. => 0x14,      // 1.21.5+
        PROTO_1_21_2..=PROTO_1_21_4 => 0x15,  // 1.21.2-1.21.4
        PROTO_1_21 => 0x15,        // 1.21-1.21.1
        PROTO_1_8 => 0x2F,         // 1.8.x
        _ => 0x2F,          // 1.7.x
    };
    RawPacket::new(packet_id, data)
}

/// WindowItems (ContainerSetContent) - initialize player inventory (46 empty slots)
/// 1.21 must be sent before SetSlot, otherwise client state_id desync will drop SetSlot
pub fn build_window_items_empty(protocol_version: i32) -> RawPacket {
    let mut data = Vec::new();

    data.push(0x00); // Window ID (player inventory)
    varint::encode_varint(0, &mut data); // State ID = 0

    // 46 slots (player inventory: 0-8 hotbar, 9-35 main, 36-39 armor, 40 offhand, 41-45 crafting)
    varint::encode_varint(46, &mut data);
    for _ in 0..46 {
        // 1.20.5+ Slot: count=0 means empty slot
        varint::encode_varint(0, &mut data);
    }

    // Carried Item (item on mouse) = empty
    varint::encode_varint(0, &mut data);

    let packet_id = match protocol_version {
        PROTO_1_21_5.. => 0x12,      // 1.21.5+
        _ => 0x13,          // 1.21-1.21.4
    };
    RawPacket::new(packet_id, data)
}

/// Held Item Change (Server → Client) - set selected hotbar slot
pub fn build_held_item_change(protocol_version: i32) -> RawPacket {
    let mut data = Vec::new();
    data.push(0x00); // Slot = 0

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x67,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x62,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x63,  // 1.21.2-1.21.4
        PROTO_1_21 => 0x53,        // 1.21-1.21.1
        PROTO_1_8 => 0x09,         // 1.8.x
        _ => 0x09,          // 1.7.x
    };
    RawPacket::new(packet_id, data)
}

/// Player Abilities (Server → Client) - set flying + invulnerable
pub fn build_player_abilities(protocol_version: i32) -> RawPacket {
    let mut data = Vec::new();
    // Flags: only use Invulnerable(0x01), don't enable flying to avoid position jitter
    data.push(0x02);
    // Flying Speed (Float) = 0.0
    data.extend_from_slice(&0.0f32.to_be_bytes());
    // Field of View Modifier (Float)
    data.extend_from_slice(&0.1f32.to_be_bytes());

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x3E,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x39,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x3A,  // 1.21.2-1.21.4
        PROTO_1_21 => 0x38,        // 1.21-1.21.1
        PROTO_1_8 => 0x39,         // 1.8.x
        _ => 0x39,          // 1.7.x
    };
    RawPacket::new(packet_id, data)
}

/// Build simple empty chunk packet (1.8.x only)
/// bitmask=0 (no sections), only contains 256 biome bytes
/// Prevent 1.8 client from getting stuck on "Downloading terrain"
pub fn build_empty_chunk_1_8() -> RawPacket {
    let mut data = Vec::new();
    data.extend_from_slice(&0i32.to_be_bytes()); // Chunk X
    data.extend_from_slice(&0i32.to_be_bytes()); // Chunk Z
    data.push(0x01); // Ground-Up Continuous: true
    data.extend_from_slice(&0u16.to_be_bytes()); // Primary Bit Mask: 0 (no sections)
    varint::encode_varint(256, &mut data); // Data length: 256 bytes (biome only)
    data.extend_from_slice(&[0u8; 256]); // Biome data: 256 bytes (Ocean)

    RawPacket::new(0x21, data)
}

/// Disconnect (Play) packet - kick player
pub fn build_disconnect(protocol_version: i32, reason: &str) -> RawPacket {
    use crate::protocol::nbt;

    let mut data = Vec::new();
    let json = serde_json::json!({"text": reason});

    if protocol_version >= PROTO_1_21 {
        // 1.21+: Text Component uses Network NBT encoding
        let nbt_value = nbt::json_to_nbt(&json);
        let nbt_bytes = nbt::encode_network_nbt_value(&nbt_value);
        data.extend_from_slice(&nbt_bytes);
    } else {
        // 1.7-1.8: JSON string
        varint::encode_string(&json.to_string(), &mut data);
    }

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x20,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x1C,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x1D,  // 1.21.2-1.21.4
        PROTO_1_21 => 0x1D,        // 1.21-1.21.1
        PROTO_1_8 => 0x40,         // 1.8.x
        _ => 0x40,          // 1.7.x
    };
    RawPacket::new(packet_id, data)
}

/// Player Info packet - add player (with skin textures)
pub fn build_player_info(
    protocol_version: i32,
    uuid_bytes: &[u8; 16],
    username: &str,
    textures_value: Option<&str>,
    textures_signature: Option<&str>,
) -> RawPacket {
    let mut data = Vec::new();

    if protocol_version >= PROTO_1_19_3 {
        // 1.21.x: Player Info Update
        data.push(0x01); // Actions: 0x01 = Add Player
        varint::encode_varint(1, &mut data); // Number Of Players
        data.extend_from_slice(uuid_bytes); // UUID

        // Name
        varint::encode_string(username, &mut data);

        // Properties
        if let Some(value) = textures_value {
            varint::encode_varint(1, &mut data);
            varint::encode_string("textures", &mut data);
            varint::encode_string(value, &mut data);
            if let Some(sig) = textures_signature.filter(|s| !s.is_empty()) {
                data.push(0x01); // Has Signature
                varint::encode_string(sig, &mut data);
            } else {
                data.push(0x00); // No Signature
            }
        } else {
            varint::encode_varint(0, &mut data);
        }

        let packet_id = match protocol_version {
            PROTO_1_21_9.. => 0x44,      // 1.21.9+
            PROTO_1_21_5..=PROTO_1_21_7 => 0x3F,  // 1.21.5-1.21.8
            PROTO_1_21_2..=PROTO_1_21_4 => 0x40,  // 1.21.2-1.21.4
            _ => 0x3E,          // 1.21-1.21.1
        };
        RawPacket::new(packet_id, data)
    } else if protocol_version >= PROTO_1_8 {
        // 1.8.x: Player Info (Action = 0, Add Player)
        varint::encode_varint(0, &mut data); // Action: Add Player
        varint::encode_varint(1, &mut data); // Number Of Players
        data.extend_from_slice(uuid_bytes); // UUID: 128-bit raw bytes

        varint::encode_string(username, &mut data); // Name

        // Properties
        if let Some(value) = textures_value {
            varint::encode_varint(1, &mut data);
            varint::encode_string("textures", &mut data);
            varint::encode_string(value, &mut data);
            if let Some(sig) = textures_signature.filter(|s| !s.is_empty()) {
                data.push(0x01);
                varint::encode_string(sig, &mut data);
            } else {
                data.push(0x00); // No Signature
            }
        } else {
            varint::encode_varint(0, &mut data);
        }

        varint::encode_varint(2, &mut data); // Gamemode: adventure
        varint::encode_varint(0, &mut data); // Ping
        data.push(0x00); // Has Display Name: false

        RawPacket::new(0x38, data)
    } else {
        // 1.7.x: Player List Item
        varint::encode_string(username, &mut data);
        data.push(0x01); // online = true (Boolean)
        data.extend_from_slice(&0i16.to_be_bytes()); // ping (Short)

        RawPacket::new(0x38, data)
    }
}

/// Game Event packet (1.20.3+, for notifying client to start loading chunks)
/// type=13 means LEVEL_CHUNKS_LOAD_START
pub fn build_game_event_start_waiting_chunks(protocol_version: i32) -> RawPacket {
    let mut data = Vec::new();
    data.push(13); // Event type: Start waiting chunks
    data.extend_from_slice(&0.0f32.to_be_bytes()); // Value

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x26,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x22,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x23,  // 1.21.2-1.21.4
        _ => 0x22,          // 1.21-1.21.1
    };
    RawPacket::new(packet_id, data)
}

/// Build empty chunk packet (1.20.3+ / 1.21.x)
/// Map rendering requires at least one loaded chunk
pub fn build_empty_chunk_modern(protocol_version: i32, chunk_x: i32, chunk_z: i32) -> RawPacket {
    let mut data = Vec::new();

    data.extend_from_slice(&chunk_x.to_be_bytes());
    data.extend_from_slice(&chunk_z.to_be_bytes());

    // Heightmaps: format changed in 1.21.5 (protocol 770)
    if protocol_version >= PROTO_1_21_5 {
        // 1.21.5+: direct array format (no NBT)
        // count(VarInt) + [type(VarInt) + array_len(VarInt) + longs]*
        varint::encode_varint(1, &mut data); // 1 heightmap
        varint::encode_varint(4, &mut data); // MOTION_BLOCKING ordinal = 4
        varint::encode_varint(37, &mut data); // array length = 37
        data.extend_from_slice(&[0u8; 37 * 8]); // 37 longs, all zero
    } else {
        // Pre-1.21.5: NBT format (nameless compound with MOTION_BLOCKING)
        data.push(0x0A); // TAG_Compound (root)
        let name = "MOTION_BLOCKING";
        data.push(0x0C); // TAG_Long_Array
        data.extend_from_slice(&(name.len() as u16).to_be_bytes());
        data.extend_from_slice(name.as_bytes());
        data.extend_from_slice(&37i32.to_be_bytes()); // array length
        data.extend_from_slice(&[0u8; 37 * 8]); // 37 longs, all zero
        data.push(0x00); // TAG_End
    }

    // Chunk section data: 24 sections (1.18+)
    // Each section: block_count(i16) + block_states(palette) + biomes(palette)
    let mut section_buf = Vec::new();
    for _ in 0..24 {
        section_buf.extend_from_slice(&0i16.to_be_bytes()); // block count = 0
        // Block states: single-value palette
        section_buf.push(0); // bits per block = 0 (single value)
        varint::encode_varint(0, &mut section_buf); // palette entry = air (0)
        varint::encode_varint(0, &mut section_buf); // data array length = 0
        // Biomes: single-value palette
        section_buf.push(0); // bits per biome = 0 (single value)
        varint::encode_varint(0, &mut section_buf); // palette entry = plains (0)
        varint::encode_varint(0, &mut section_buf); // data array length = 0
    }
    varint::encode_varint(section_buf.len() as i32, &mut data);
    data.extend_from_slice(&section_buf);

    // Block entity count = 0
    varint::encode_varint(0, &mut data);

    // Light data: all empty
    varint::encode_varint(0, &mut data); // sky light mask (empty BitSet)
    varint::encode_varint(0, &mut data); // block light mask (empty BitSet)
    varint::encode_varint(0, &mut data); // empty sky light mask (empty BitSet)
    varint::encode_varint(0, &mut data); // empty block light mask (empty BitSet)
    varint::encode_varint(0, &mut data); // sky light arrays count = 0
    varint::encode_varint(0, &mut data); // block light arrays count = 0

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x2C,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x27,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x28,  // 1.21.2-1.21.4
        _ => 0x27,          // 1.21-1.21.1
    };
    RawPacket::new(packet_id, data)
}

/// Spawn Position packet (1.19.3+)
/// 1.21.9+ adds dimension key and pitch fields
pub fn build_spawn_position(protocol_version: i32) -> RawPacket {
    let mut data = Vec::new();

    // 1.21.9+: Dimension Key (NamespacedKey / Identifier string)
    if protocol_version >= PROTO_1_21_9 {
        varint::encode_string("minecraft:overworld", &mut data);
    }

    // Position: packed X/Y/Z as u64 (X=0, Y=400, Z=0)
    // Format: X(26 bits) << 38 | Z(26 bits) << 12 | Y(12 bits)
    let y: i64 = 400;
    let position: u64 = (y & 0xFFF) as u64; // X=0, Z=0, Y=400
    data.extend_from_slice(&position.to_be_bytes());
    // Yaw (Float) = 0.0
    data.extend_from_slice(&0.0f32.to_be_bytes());

    // 1.21.9+: Pitch (Float)
    if protocol_version >= PROTO_1_21_9 {
        data.extend_from_slice(&0.0f32.to_be_bytes());
    }

    let packet_id = match protocol_version {
        PROTO_1_21_9.. => 0x5F,      // 1.21.9+
        PROTO_1_21_5..=PROTO_1_21_7 => 0x5A,  // 1.21.5-1.21.8
        PROTO_1_21_2..=PROTO_1_21_4 => 0x5B,  // 1.21.2-1.21.4
        _ => 0x56,          // 1.21-1.21.1
    };
    RawPacket::new(packet_id, data)
}

// ==================== Configuration phase packets ====================

/// Build Finish Configuration packet (Configuration state, Packet ID 0x03)
pub fn build_finish_configuration() -> RawPacket {
    RawPacket::new(0x03, Vec::new())
}

/// Format 16-byte UUID to standard string
fn format_uuid(bytes: &[u8; 16]) -> String {
    let hex = hex::encode(bytes);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}
