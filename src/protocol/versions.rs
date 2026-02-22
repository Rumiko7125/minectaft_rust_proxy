/// Minecraft protocol version constants.
///
/// Only the version boundaries relevant to this proxy are defined here.
/// Supported versions: 1.7.x (4-5), 1.8.x (47), 1.21.x (767+).

// ── 1.7.x ──────────────────────────────────────────────────────────────────
/// 1.7.2–1.7.9
pub const PROTO_1_7_MIN: i32 = 4;
/// 1.7.10
pub const PROTO_1_7_MAX: i32 = 5;

// ── 1.8.x ──────────────────────────────────────────────────────────────────
/// 1.8–1.8.9
pub const PROTO_1_8: i32 = 47;

// ── 1.19.x ─────────────────────────────────────────────────────────────────
/// 1.19 / 1.19.1
pub const PROTO_1_19: i32 = 759;
/// 1.19.2
pub const PROTO_1_19_2: i32 = 760;
/// 1.19.3
pub const PROTO_1_19_3: i32 = 761;

// ── 1.20.x ─────────────────────────────────────────────────────────────────
/// 1.20.2 / 1.20.3 (Configuration phase introduced)
pub const PROTO_1_20_2: i32 = 764;
/// 1.20.3 / 1.20.4 (GameEvent chunk loading)
pub const PROTO_1_20_3: i32 = 765;
/// 1.20.5 / 1.20.6 (New Slot format)
pub const PROTO_1_20_5: i32 = 766;

// ── 1.21.x ─────────────────────────────────────────────────────────────────
/// 1.21 / 1.21.1
pub const PROTO_1_21: i32 = 767;
/// 1.21.2 / 1.21.3
pub const PROTO_1_21_2: i32 = 768;
/// 1.21.4
pub const PROTO_1_21_4: i32 = 769;
/// 1.21.5
pub const PROTO_1_21_5: i32 = 770;
/// 1.21.6
pub const PROTO_1_21_6: i32 = 771;
/// 1.21.7 / 1.21.8
pub const PROTO_1_21_7: i32 = 772;
/// 1.21.9 / 1.21.10
pub const PROTO_1_21_9: i32 = 773;
/// 1.21.11+
pub const PROTO_1_21_11: i32 = 774;
