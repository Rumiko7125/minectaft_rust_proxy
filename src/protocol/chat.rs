use crate::protocol::nbt;
use crate::protocol::packet::RawPacket;
use crate::protocol::varint;
use crate::protocol::versions::*;

/// Get chat packet ID by protocol version
fn get_chat_packet_id(protocol_version: i32) -> (i32, ChatKind) {
    match protocol_version {
        // 1.21.9+
        PROTO_1_21_9.. => (0x77, ChatKind::System),

        // 1.21.5 – 1.21.8
        PROTO_1_21_5..=PROTO_1_21_7 => (0x72, ChatKind::System),

        // 1.21.2 – 1.21.4
        PROTO_1_21_2..=PROTO_1_21_4 => (0x73, ChatKind::System),

        // 1.21 – 1.21.1
        PROTO_1_21 => (0x6C, ChatKind::System),

        // 1.19 – 1.20.4
        PROTO_1_19..=PROTO_1_20_5 => (0x0F, ChatKind::System),

        // 1.8.x
        PROTO_1_8 => (0x02, ChatKind::LegacyChat),

        // 1.7.x
        _ => (0x02, ChatKind::LegacyChatNoPos),
    }
}

#[derive(Copy, Clone)]
enum ChatKind {
    /// 1.19+ System Chat (NBT)
    System,

    /// 1.8 Chat (String + position)
    LegacyChat,

    /// 1.7 Chat (String only)
    LegacyChatNoPos,
}

/// Build cross-version chat packet
pub fn build_chat_packet(protocol_version: i32, json_text: &str) -> RawPacket {
    let (packet_id, kind) = get_chat_packet_id(protocol_version);
    let mut data = Vec::new();

    match kind {
        // ==============================
        // 1.19+ System Chat (NBT)
        // ==============================
        ChatKind::System => {
            let mut json_value: serde_json::Value =
                serde_json::from_str(json_text)
                    .unwrap_or(serde_json::Value::String(json_text.to_string()));

            // 1.21.5+ needs snake_case
            if protocol_version >= PROTO_1_21_5 {
                transform_chat_component_keys(&mut json_value);
            }

            let nbt_value = nbt::json_to_nbt(&json_value);
            let nbt_bytes = nbt::encode_network_nbt_value(&nbt_value);

            data.extend_from_slice(&nbt_bytes);

            // overlay = false → chat bar
            data.push(0x00);
        }

        // ==============================
        // 1.8.x Chat (String + position)
        // Always use position = 0 to ensure display
        // ==============================
        ChatKind::LegacyChat => {
            varint::encode_string(json_text, &mut data);
            data.push(0x00); // chat bar (don't use 1)
        }

        // ==============================
        // 1.7.x Chat (String only)
        // ==============================
        ChatKind::LegacyChatNoPos => {
            varint::encode_string(json_text, &mut data);
        }
    }

    RawPacket::new(packet_id, data)
}


/// Recursively transform Text Component JSON keys: camelCase → snake_case (1.21.5+)
/// Also fix clickEvent action's value field name
fn transform_chat_component_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(obj) => {
            // Recursively process extra array
            if let Some(extra) = obj.get_mut("extra") {
                if let Some(arr) = extra.as_array_mut() {
                    for item in arr {
                        transform_chat_component_keys(item);
                    }
                }
            }

            // clickEvent → click_event
            if let Some(click_event) = obj.remove("clickEvent") {
                if let serde_json::Value::Object(mut ce) = click_event {
                    let action = ce.get("action").and_then(|a| a.as_str()).unwrap_or("").to_string();
                    if let Some(val) = ce.remove("value") {
                        let new_val = match action.as_str() {
                            "open_url" => { ce.insert("url".to_string(), val.clone()); val }
                            "copy_to_clipboard" => { ce.insert("contents".to_string(), val.clone()); val }
                            "run_command" | "suggest_command" => { ce.insert("command".to_string(), val.clone()); val }
                            "change_page" => {
                                if let Some(s) = val.as_str() {
                                    if let Ok(n) = s.parse::<i64>() {
                                        ce.insert("page".to_string(), serde_json::Value::Number(n.into()));
                                    } else {
                                        ce.insert("page".to_string(), val.clone());
                                    }
                                } else {
                                    ce.insert("page".to_string(), val.clone());
                                }
                                val
                            }
                            _ => val
                        };
                        // Keep original value field, compatible with NBT encoder
                        ce.insert("value".to_string(), new_val);
                    }
                    obj.insert("click_event".to_string(), serde_json::Value::Object(ce));
                }
            }

            // hoverEvent → hover_event
            if let Some(hover_event) = obj.remove("hoverEvent") {
                if let serde_json::Value::Object(mut he) = hover_event {
                    if let Some(contents) = he.get_mut("contents") {
                        transform_chat_component_keys(contents);
                    }
                    if let Some(val) = he.get_mut("value") {
                        transform_chat_component_keys(val);
                    }
                    obj.insert("hover_event".to_string(), serde_json::Value::Object(he));
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                transform_chat_component_keys(item);
            }
        }
        _ => {}
    }
}

