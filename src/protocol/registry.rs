/// Registry data management for 1.21.x Configuration phase.
/// Embeds per-version binary NBT codec files and tags files from NanoLimbo.
/// Builds pre-encoded RegistryData (0x07) and UpdateTags (0x0D) packets at startup.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::protocol::nbt::{self, NbtValue};
use crate::protocol::packet::RawPacket;
use crate::protocol::varint;
use crate::protocol::versions::*;

// ==================== Embedded codec files (GZIP compressed binary NBT) ====================

static CODEC_1_21: &[u8] = include_bytes!("codecs/codec_1_21.nbt");
static CODEC_1_21_2: &[u8] = include_bytes!("codecs/codec_1_21_2.nbt");
static CODEC_1_21_4: &[u8] = include_bytes!("codecs/codec_1_21_4.nbt");
static CODEC_1_21_5: &[u8] = include_bytes!("codecs/codec_1_21_5.nbt");
static CODEC_1_21_6: &[u8] = include_bytes!("codecs/codec_1_21_6.nbt");
static CODEC_1_21_7: &[u8] = include_bytes!("codecs/codec_1_21_7.nbt");
static CODEC_1_21_9: &[u8] = include_bytes!("codecs/codec_1_21_9.nbt");
static CODEC_1_21_11: &[u8] = include_bytes!("codecs/codec_1_21_11.nbt");

// ==================== Embedded tags files (GZIP compressed binary NBT) ====================

static TAGS_1_21: &[u8] = include_bytes!("codecs/tags_1_21.nbt");
static TAGS_1_21_2: &[u8] = include_bytes!("codecs/tags_1_21_2.nbt");
static TAGS_1_21_4: &[u8] = include_bytes!("codecs/tags_1_21_4.nbt");
static TAGS_1_21_5: &[u8] = include_bytes!("codecs/tags_1_21_5.nbt");
static TAGS_1_21_6: &[u8] = include_bytes!("codecs/tags_1_21_6.nbt");
static TAGS_1_21_7: &[u8] = include_bytes!("codecs/tags_1_21_7.nbt");
static TAGS_1_21_9: &[u8] = include_bytes!("codecs/tags_1_21_9.nbt");
static TAGS_1_21_11: &[u8] = include_bytes!("codecs/tags_1_21_11.nbt");

// ==================== Pre-encoded packets per version ====================

/// Pre-encoded registry data + update tags packets for each protocol version.
struct VersionPackets {
    registry: Vec<RawPacket>,
    update_tags: RawPacket,
}

static REGISTRY_CACHE: OnceLock<BTreeMap<i32, VersionPackets>> = OnceLock::new();

/// Initialize the registry cache. Call once at startup.
pub fn init() {
    REGISTRY_CACHE.get_or_init(|| {
        let mut cache = BTreeMap::new();

        let versions: &[(i32, &[u8], &[u8])] = &[
            (PROTO_1_21,    CODEC_1_21,    TAGS_1_21),
            (PROTO_1_21_2,  CODEC_1_21_2,  TAGS_1_21_2),
            (PROTO_1_21_4,  CODEC_1_21_4,  TAGS_1_21_4),
            (PROTO_1_21_5,  CODEC_1_21_5,  TAGS_1_21_5),
            (PROTO_1_21_6,  CODEC_1_21_6,  TAGS_1_21_6),
            (PROTO_1_21_7,  CODEC_1_21_7,  TAGS_1_21_7),
            (PROTO_1_21_9,  CODEC_1_21_9,  TAGS_1_21_9),
            (PROTO_1_21_11, CODEC_1_21_11, TAGS_1_21_11),
        ];

        for &(proto_ver, codec_data, tags_data) in versions {
            let codec = nbt::read_gzip_nbt(codec_data);
            let registry = build_registry_packets(&codec);

            let tags = nbt::read_gzip_nbt(tags_data);
            let update_tags = build_update_tags_packet(&tags);

            tracing::info!(
                "Loaded {} registry packets + tags for protocol {}",
                registry.len(),
                proto_ver
            );
            cache.insert(proto_ver, VersionPackets { registry, update_tags });
        }

        cache
    });
}

/// Map protocol version to cache key.
fn version_key(protocol_version: i32) -> i32 {
    match protocol_version {
        PROTO_1_21_11.. => PROTO_1_21_11,
        PROTO_1_21_9    => PROTO_1_21_9,
        PROTO_1_21_7    => PROTO_1_21_7,
        PROTO_1_21_6    => PROTO_1_21_6,
        PROTO_1_21_5    => PROTO_1_21_5,
        PROTO_1_21_4    => PROTO_1_21_4,
        PROTO_1_21_2    => PROTO_1_21_2,
        _               => PROTO_1_21,
    }
}

/// Get registry data packets for a specific protocol version.
pub fn get_registry_packets(protocol_version: i32) -> &'static [RawPacket] {
    let cache = REGISTRY_CACHE
        .get()
        .expect("Registry cache not initialized");
    cache
        .get(&version_key(protocol_version))
        .map(|v| v.registry.as_slice())
        .unwrap_or(&[])
}

/// Get the UpdateTags packet for a specific protocol version.
pub fn get_update_tags_packet(protocol_version: i32) -> Option<&'static RawPacket> {
    let cache = REGISTRY_CACHE
        .get()
        .expect("Registry cache not initialized");
    cache
        .get(&version_key(protocol_version))
        .map(|v| &v.update_tags)
}

/// Build registry data packets from a parsed codec NBT.
fn build_registry_packets(codec: &NbtValue) -> Vec<RawPacket> {
    let keys: Vec<String> = match codec.keys() {
        Some(iter) => iter.cloned().collect(),
        None => return vec![],
    };

    let mut packets = Vec::new();

    for registry_type in &keys {
        let registry = match codec.get(registry_type) {
            Some(r) => r,
            None => continue,
        };

        let values = match registry.get("value").and_then(|v| v.as_list()) {
            Some(v) => v,
            None => continue,
        };

        let mut data = Vec::new();
        varint::encode_string(registry_type, &mut data);
        varint::encode_varint(values.len() as i32, &mut data);

        for entry in values {
            let name = entry
                .get("name")
                .and_then(|v| v.as_string())
                .unwrap_or("minecraft:unknown");
            varint::encode_string(name, &mut data);

            match entry.get("element") {
                Some(element) => {
                    data.push(0x01); // has_data = true
                    let element_nbt = nbt::encode_network_nbt(element);
                    data.extend_from_slice(&element_nbt);
                }
                None => {
                    data.push(0x00); // has_data = false
                }
            }
        }

        packets.push(RawPacket::new(0x07, data));
    }

    packets
}

/// Build UpdateTags packet (0x0D) from parsed tags NBT.
///
/// Tags NBT structure:
/// ```
/// root (Compound)
/// ├── "minecraft:block" (Compound)
/// │   ├── "minecraft:flowers" (List of Int) → [5, 12, ...]
/// │   └── ...
/// ├── "minecraft:item" (Compound)
/// │   └── ...
/// └── ...
/// ```
///
/// Wire format:
/// ```
/// [num_categories: varint]
/// for each category:
///   [category_name: string]
///   [num_tags: varint]
///   for each tag:
///     [tag_name: string]
///     [num_ids: varint]
///     for each id:
///       [id: varint]
/// ```
fn build_update_tags_packet(tags: &NbtValue) -> RawPacket {
    let mut data = Vec::new();

    let categories: Vec<String> = match tags.keys() {
        Some(iter) => iter.cloned().collect(),
        None => {
            varint::encode_varint(0, &mut data);
            return RawPacket::new(0x0D, data);
        }
    };

    varint::encode_varint(categories.len() as i32, &mut data);

    for category_name in &categories {
        varint::encode_string(category_name, &mut data);

        let category = match tags.get(category_name) {
            Some(c) => c,
            None => {
                varint::encode_varint(0, &mut data);
                continue;
            }
        };

        let tag_names: Vec<String> = match category.keys() {
            Some(iter) => iter.cloned().collect(),
            None => {
                varint::encode_varint(0, &mut data);
                continue;
            }
        };

        varint::encode_varint(tag_names.len() as i32, &mut data);

        for tag_name in &tag_names {
            varint::encode_string(tag_name, &mut data);

            let ids = match category.get(tag_name).and_then(|v| v.as_list()) {
                Some(list) => list,
                None => {
                    varint::encode_varint(0, &mut data);
                    continue;
                }
            };

            varint::encode_varint(ids.len() as i32, &mut data);
            for id in ids {
                let id_val = match id {
                    NbtValue::Int(v) => *v,
                    NbtValue::Short(v) => *v as i32,
                    NbtValue::Byte(v) => *v as i32,
                    _ => 0,
                };
                varint::encode_varint(id_val, &mut data);
            }
        }
    }

    RawPacket::new(0x0D, data)
}
