use crate::db;
use crate::web_api::handlers::AppState;
use axum::body::Bytes;
use axum::extract::{Multipart, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use serde_json::{json, Value};
use sqlx::Row;
use std::io::{Cursor, Read, Write};
use std::sync::Arc;

type ApiResult = std::result::Result<Json<Value>, (StatusCode, Json<Value>)>;

fn error_response(code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"code": code, "message": message})),
    )
}

fn ok_response(data: Value) -> ApiResult {
    Ok(Json(data))
}

/// GET /api/v1/migration/export - Export as ZIP archive
pub async fn export_migration(State(state): State<Arc<AppState>>) -> Response {
    match export_migration_inner(&state).await {
        Ok((filename, bytes)) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("application/zip"));
            headers.insert(
                "Content-Disposition",
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
                    .unwrap_or_else(|_| HeaderValue::from_static("attachment; filename=\"export.zip\"")),
            );
            (headers, Bytes::from(bytes)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code": "EXPORT_FAILED", "message": e})),
        )
            .into_response(),
    }
}

async fn export_migration_inner(state: &AppState) -> Result<(String, Vec<u8>), String> {
    let pool = &state.proxy.db_pool;
    let now = chrono::Utc::now();
    let ts = now.format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("proxy_migration_{}.zip", ts);

    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // manifest.json
    let manifest = serde_json::to_string_pretty(&json!({
        "version": 3,
        "exported_at": now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    }))
    .unwrap();
    zip.start_file("manifest.json", options).map_err(|e| e.to_string())?;
    zip.write_all(manifest.as_bytes()).map_err(|e| e.to_string())?;

    // settings.json
    let settings_rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let mut settings = json!({});
    for row in &settings_rows {
        let key: String = row.get(0);
        let value: String = row.get(1);
        settings[key] = Value::String(value);
    }
    let settings_json = serde_json::to_string_pretty(&settings).unwrap();
    zip.start_file("settings.json", options).map_err(|e| e.to_string())?;
    zip.write_all(settings_json.as_bytes()).map_err(|e| e.to_string())?;

    // backends.json
    let backends = db::backend_servers::get_all_backend_servers(pool)
        .await
        .map_err(|e| e.to_string())?;
    let backends_json_val: Vec<Value> = backends
        .iter()
        .map(|b| {
            json!({
                "name": b.name,
                "remote_address": b.remote_address,
                "remote_port": b.remote_port,
                "max_player": b.max_player,
                "motd_json": b.motd_json,
                "limbo_message": b.limbo_message,
                "log_dir": b.log_dir,
                "show_log_level": b.show_log_level,
                "save_log_level": b.save_log_level,
                "is_default": b.is_default,
                "enabled": b.enabled,
                "maintenance": b.maintenance,
                "maintenance_message": b.maintenance_message,
                "ping_passthrough": b.ping_passthrough,
                "motd_passthrough": b.motd_passthrough,
            })
        })
        .collect();
    let backends_json = serde_json::to_string_pretty(&backends_json_val).unwrap();
    zip.start_file("backends.json", options).map_err(|e| e.to_string())?;
    zip.write_all(backends_json.as_bytes()).map_err(|e| e.to_string())?;

    // domain_routes.json
    let routes = db::domain_routes::get_all_domain_routes(pool)
        .await
        .map_err(|e| e.to_string())?;
    let routes_json_val: Vec<Value> = routes
        .iter()
        .map(|r| {
            let backend_name = r
                .backend_id
                .and_then(|bid| backends.iter().find(|b| b.id == bid).map(|b| b.name.clone()));
            json!({
                "pattern": r.pattern,
                "target_addr": r.target_addr,
                "target_port": r.target_port,
                "priority": r.priority,
                "backend_name": backend_name,
            })
        })
        .collect();
    let routes_json = serde_json::to_string_pretty(&routes_json_val).unwrap();
    zip.start_file("domain_routes.json", options).map_err(|e| e.to_string())?;
    zip.write_all(routes_json.as_bytes()).map_err(|e| e.to_string())?;

    // whitelist.json
    let wl_rows = sqlx::query("SELECT username, added_at FROM whitelist ORDER BY added_at ASC LIMIT 100000")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let whitelist_val: Vec<Value> = wl_rows
        .iter()
        .map(|r| json!({"username": r.get::<String, _>(0), "added_at": r.get::<String, _>(1)}))
        .collect();
    let whitelist_json = serde_json::to_string_pretty(&whitelist_val).unwrap();
    zip.start_file("whitelist.json", options).map_err(|e| e.to_string())?;
    zip.write_all(whitelist_json.as_bytes()).map_err(|e| e.to_string())?;

    // blacklist.json
    let bl_rows = sqlx::query("SELECT username, reason, added_at FROM blacklist ORDER BY added_at ASC LIMIT 100000")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let blacklist_val: Vec<Value> = bl_rows
        .iter()
        .map(|r| {
            json!({"username": r.get::<String, _>(0), "reason": r.get::<Option<String>, _>(1), "added_at": r.get::<String, _>(2)})
        })
        .collect();
    let blacklist_json = serde_json::to_string_pretty(&blacklist_val).unwrap();
    zip.start_file("blacklist.json", options).map_err(|e| e.to_string())?;
    zip.write_all(blacklist_json.as_bytes()).map_err(|e| e.to_string())?;

    // two_factor_secrets.json
    let tf_rows = sqlx::query("SELECT username, secret, created_at FROM two_factor_secrets ORDER BY created_at ASC")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let tf_val: Vec<Value> = tf_rows
        .iter()
        .map(|r| json!({"username": r.get::<String, _>(0), "secret": r.get::<String, _>(1), "created_at": r.get::<String, _>(2)}))
        .collect();
    let tf_json = serde_json::to_string_pretty(&tf_val).unwrap();
    zip.start_file("two_factor_secrets.json", options).map_err(|e| e.to_string())?;
    zip.write_all(tf_json.as_bytes()).map_err(|e| e.to_string())?;

    // moderation_logs.csv
    let modlog_rows = sqlx::query(
        "SELECT action, target, operator, reason, created_at FROM moderation_logs ORDER BY created_at ASC LIMIT 1000000",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut modlog_csv = String::from("action,target,operator,reason,created_at\n");
    for row in &modlog_rows {
        let action: String = row.get(0);
        let target: String = row.get(1);
        let operator: String = row.get(2);
        let reason: Option<String> = row.get(3);
        let created_at: String = row.get(4);
        modlog_csv.push_str(&csv_row(&[
            &action,
            &target,
            &operator,
            reason.as_deref().unwrap_or(""),
            &created_at,
        ]));
    }
    zip.start_file("moderation_logs.csv", options).map_err(|e| e.to_string())?;
    zip.write_all(modlog_csv.as_bytes()).map_err(|e| e.to_string())?;

    // player_sessions.csv
    let session_rows = sqlx::query(
        r#"SELECT username, uuid, backend_addr, backend_port, login_at, logout_at,
                  upload_bytes, download_bytes, protocol_version, kick_reason
           FROM player_sessions ORDER BY login_at ASC LIMIT 1000000"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut sessions_csv = String::from("username,uuid,backend_addr,backend_port,login_at,logout_at,upload_bytes,download_bytes,protocol_version,kick_reason\n");
    for row in &session_rows {
        let username: String = row.get(0);
        let uuid: String = row.get(1);
        let backend_addr: String = row.get(2);
        let backend_port: i32 = row.get(3);
        let login_at: String = row.get(4);
        let logout_at: Option<String> = row.get(5);
        let upload_bytes: i64 = row.get(6);
        let download_bytes: i64 = row.get(7);
        let protocol_version: i32 = row.get(8);
        let kick_reason: Option<String> = row.get(9);
        sessions_csv.push_str(&csv_row(&[
            &username,
            &uuid,
            &backend_addr,
            &backend_port.to_string(),
            &login_at,
            logout_at.as_deref().unwrap_or(""),
            &upload_bytes.to_string(),
            &download_bytes.to_string(),
            &protocol_version.to_string(),
            kick_reason.as_deref().unwrap_or(""),
        ]));
    }
    zip.start_file("player_sessions.csv", options).map_err(|e| e.to_string())?;
    zip.write_all(sessions_csv.as_bytes()).map_err(|e| e.to_string())?;

    let result = zip.finish().map_err(|e| e.to_string())?;
    Ok((filename, result.into_inner()))
}

/// Format a list of fields into a single CSV row
fn csv_row(fields: &[&str]) -> String {
    let escaped: Vec<String> = fields
        .iter()
        .map(|f| {
            if f.contains(',') || f.contains('"') || f.contains('\n') {
                format!("\"{}\"", f.replace('"', "\"\""))
            } else {
                f.to_string()
            }
        })
        .collect();
    format!("{}\n", escaped.join(","))
}

/// POST /api/v1/migration/import - Import ZIP or JSON (backward compatible)
pub async fn import_migration(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ApiResult {
    // Read uploaded file
    let mut file_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| error_response("READ_ERROR", &e.to_string()))?;
            file_bytes = Some(data.to_vec());
            break;
        }
    }

    let bytes = file_bytes.ok_or_else(|| error_response("NO_FILE", "No file uploaded"))?;

    // Detect ZIP or JSON
    if bytes.starts_with(b"PK") {
        import_from_zip(&state, bytes).await
    } else {
        // Try to process as JSON (backward compatible with old format)
        let body: Value = serde_json::from_slice(&bytes)
            .map_err(|e| error_response("PARSE_ERROR", &e.to_string()))?;
        import_from_json(&state, &body).await
    }
}

async fn import_from_zip(state: &AppState, bytes: Vec<u8>) -> ApiResult {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| error_response("ZIP_ERROR", &e.to_string()))?;

    // Read each file
    let mut settings_val: Option<Value> = None;
    let mut backends_val: Option<Value> = None;
    let mut routes_val: Option<Value> = None;
    let mut whitelist_val: Option<Value> = None;
    let mut blacklist_val: Option<Value> = None;
    let mut tf_val: Option<Value> = None;
    let mut modlog_csv: Option<String> = None;
    let mut sessions_csv: Option<String> = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| error_response("ZIP_ERROR", &e.to_string()))?;
        let name = file.name().to_string();
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| error_response("ZIP_READ_ERROR", &e.to_string()))?;

        match name.as_str() {
            "settings.json" => settings_val = serde_json::from_str(&content).ok(),
            "backends.json" => backends_val = serde_json::from_str(&content).ok(),
            "domain_routes.json" => routes_val = serde_json::from_str(&content).ok(),
            "whitelist.json" => whitelist_val = serde_json::from_str(&content).ok(),
            "blacklist.json" => blacklist_val = serde_json::from_str(&content).ok(),
            "two_factor_secrets.json" => tf_val = serde_json::from_str(&content).ok(),
            "moderation_logs.csv" => modlog_csv = Some(content),
            "player_sessions.csv" => sessions_csv = Some(content),
            _ => {}
        }
    }

    let body = json!({
        "settings": settings_val.unwrap_or(json!({})),
        "backends": backends_val.unwrap_or(json!([])),
        "domain_routes": routes_val.unwrap_or(json!([])),
        "whitelist": whitelist_val.unwrap_or(json!([])),
        "blacklist": blacklist_val.unwrap_or(json!([])),
        "two_factor_secrets": tf_val.unwrap_or(json!([])),
        "moderation_logs_csv": modlog_csv.unwrap_or_default(),
        "player_sessions_csv": sessions_csv.unwrap_or_default(),
    });

    import_from_json_ext(state, &body).await
}

async fn import_from_json(state: &AppState, body: &Value) -> ApiResult {
    import_from_json_ext(state, body).await
}

async fn import_from_json_ext(state: &AppState, body: &Value) -> ApiResult {
    let pool = &state.proxy.db_pool;

    // ── settings (upsert) ─────────────────────────────────────────────────────
    if let Some(settings) = body.get("settings").and_then(|s| s.as_object()) {
        for (key, value) in settings {
            let val_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => continue,
            };
            let _ = db::settings::set_setting(pool, key, &val_str).await;
        }
    }

    // ── backend_servers (dedupe by name, skip if exists) ─────────────────────
    if let Some(backends) = body.get("backends").and_then(|b| b.as_array()) {
        for backend in backends {
            let name = match backend.get("name").and_then(|n| n.as_str()) {
                Some(n) => n,
                None => continue,
            };
            let remote_address = backend.get("remote_address").and_then(|v| v.as_str()).unwrap_or("");
            let remote_port = backend.get("remote_port").and_then(|v| v.as_i64()).unwrap_or(25565) as i32;
            let max_player = backend.get("max_player").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            let motd_json = backend.get("motd_json").and_then(|v| v.as_str()).map(String::from);
            let limbo_message = backend.get("limbo_message").and_then(|v| v.as_str()).map(String::from);
            let log_dir = backend.get("log_dir").and_then(|v| v.as_str()).map(String::from);
            let show_log_level = backend.get("show_log_level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let save_log_level = backend.get("save_log_level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let enabled = backend.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true) as i32;
            let maintenance = backend.get("maintenance").and_then(|v| v.as_bool()).unwrap_or(false) as i32;
            let is_default = backend.get("is_default").and_then(|v| v.as_bool()).unwrap_or(false) as i32;
            let maintenance_message = backend.get("maintenance_message").and_then(|v| v.as_str()).map(String::from);
            let ping_passthrough = backend.get("ping_passthrough").and_then(|v| v.as_bool()).unwrap_or(false) as i32;
            let motd_passthrough = backend.get("motd_passthrough").and_then(|v| v.as_bool()).unwrap_or(false) as i32;

            let _ = sqlx::query(
                r#"INSERT OR IGNORE INTO backend_servers
                    (name, remote_address, remote_port, max_player, motd_json, limbo_message,
                     log_dir, show_log_level, save_log_level, enabled, maintenance,
                     maintenance_message, is_default, ping_passthrough, motd_passthrough)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(name)
            .bind(remote_address)
            .bind(remote_port)
            .bind(max_player)
            .bind(motd_json)
            .bind(limbo_message)
            .bind(log_dir)
            .bind(show_log_level)
            .bind(save_log_level)
            .bind(enabled)
            .bind(maintenance)
            .bind(maintenance_message)
            .bind(is_default)
            .bind(ping_passthrough)
            .bind(motd_passthrough)
            .execute(pool)
            .await;
        }
    }

    // Re-query backends to get new IDs (for domain route association)
    let current_backends = db::backend_servers::get_all_backend_servers(pool)
        .await
        .unwrap_or_default();

    // ── domain_routes (dedupe by pattern, skip if exists) ──────────────────
    if let Some(routes) = body.get("domain_routes").and_then(|r| r.as_array()) {
        for route in routes {
            let pattern = match route.get("pattern").and_then(|p| p.as_str()) {
                Some(p) => p,
                None => continue,
            };
            let target_addr = route.get("target_addr").and_then(|v| v.as_str()).unwrap_or("");
            let target_port = route.get("target_port").and_then(|v| v.as_i64()).unwrap_or(25565) as i32;
            let priority = route.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            let backend_id: Option<i64> = route
                .get("backend_name")
                .and_then(|n| n.as_str())
                .and_then(|name| current_backends.iter().find(|b| b.name == name).map(|b| b.id));

            let _ = sqlx::query(
                "INSERT OR IGNORE INTO domain_routes (pattern, target_addr, target_port, priority, backend_id) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(pattern)
            .bind(target_addr)
            .bind(target_port)
            .bind(priority)
            .bind(backend_id)
            .execute(pool)
            .await;
        }
    }

    // ── whitelist (dedupe by username) ──────────────────────────────────────
    if let Some(whitelist) = body.get("whitelist").and_then(|w| w.as_array()) {
        for entry in whitelist {
            let (username, added_at) = if let Some(obj) = entry.as_object() {
                let u = obj.get("username").and_then(|v| v.as_str()).unwrap_or("");
                let t = obj.get("added_at").and_then(|v| v.as_str());
                (u.to_string(), t.map(String::from))
            } else if let Some(u) = entry.as_str() {
                (u.to_string(), None)
            } else {
                continue;
            };
            let at = added_at.unwrap_or_else(|| "datetime('now')".to_string());
            let _ = sqlx::query("INSERT OR IGNORE INTO whitelist (username, added_at) VALUES (?, ?)")
                .bind(&username)
                .bind(&at)
                .execute(pool)
                .await;
        }
    }

    // ── blacklist (dedupe by username) ─────────────────────────────────────
    if let Some(blacklist) = body.get("blacklist").and_then(|b| b.as_array()) {
        for entry in blacklist {
            let username = match entry.get("username").and_then(|u| u.as_str()) {
                Some(u) => u,
                None => continue,
            };
            let reason = entry.get("reason").and_then(|r| r.as_str());
            let added_at = entry.get("added_at").and_then(|v| v.as_str()).unwrap_or("datetime('now')");
            let _ = sqlx::query("INSERT OR IGNORE INTO blacklist (username, reason, added_at) VALUES (?, ?, ?)")
                .bind(username)
                .bind(reason)
                .bind(added_at)
                .execute(pool)
                .await;
        }
    }

    // ── two_factor_secrets (dedupe by username) ────────────────────────────
    if let Some(secrets) = body.get("two_factor_secrets").and_then(|s| s.as_array()) {
        for entry in secrets {
            let username = match entry.get("username").and_then(|v| v.as_str()) {
                Some(u) => u,
                None => continue,
            };
            let secret = match entry.get("secret").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let created_at = entry.get("created_at").and_then(|v| v.as_str()).unwrap_or("datetime('now')");
            let _ = sqlx::query("INSERT OR IGNORE INTO two_factor_secrets (username, secret, created_at) VALUES (?, ?, ?)")
                .bind(username)
                .bind(secret)
                .bind(created_at)
                .execute(pool)
                .await;
        }
    }

    // ── moderation_logs (CSV format, no dedupe for logs) ───────────────────
    if let Some(csv) = body.get("moderation_logs_csv").and_then(|v| v.as_str()) {
        for (i, line) in csv.lines().enumerate() {
            if i == 0 { continue; } // Skip header
            let fields = parse_csv_line(line);
            if fields.len() < 5 { continue; }
            let _ = sqlx::query(
                "INSERT INTO moderation_logs (action, target, operator, reason, created_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&fields[0])
            .bind(&fields[1])
            .bind(&fields[2])
            .bind(if fields[3].is_empty() { None } else { Some(fields[3].as_str()) })
            .bind(&fields[4])
            .execute(pool)
            .await;
        }
    } else if let Some(logs) = body.get("moderation_logs").and_then(|l| l.as_array()) {
        // Backward compatible with old JSON format
        for log in logs {
            let action = match log.get("action").and_then(|v| v.as_str()) { Some(a) => a, None => continue };
            let target = log.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let operator = log.get("operator").and_then(|v| v.as_str()).unwrap_or("");
            let reason = log.get("reason").and_then(|v| v.as_str());
            let created_at = log.get("created_at").and_then(|v| v.as_str()).unwrap_or("datetime('now')");
            let _ = sqlx::query(
                "INSERT INTO moderation_logs (action, target, operator, reason, created_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(action).bind(target).bind(operator).bind(reason).bind(created_at)
            .execute(pool).await;
        }
    }

    // ── player_sessions (CSV format, no dedupe for historical data) ────────
    if let Some(csv) = body.get("player_sessions_csv").and_then(|v| v.as_str()) {
        for (i, line) in csv.lines().enumerate() {
            if i == 0 { continue; } // Skip header
            let fields = parse_csv_line(line);
            if fields.len() < 10 { continue; }
            let backend_port: i32 = fields[3].parse().unwrap_or(25565);
            let upload_bytes: i64 = fields[6].parse().unwrap_or(0);
            let download_bytes: i64 = fields[7].parse().unwrap_or(0);
            let protocol_version: i32 = fields[8].parse().unwrap_or(0);
            let _ = sqlx::query(
                r#"INSERT INTO player_sessions
                    (username, uuid, backend_addr, backend_port, login_at, logout_at,
                     upload_bytes, download_bytes, protocol_version, kick_reason)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&fields[0])
            .bind(&fields[1])
            .bind(&fields[2])
            .bind(backend_port)
            .bind(&fields[4])
            .bind(if fields[5].is_empty() { None } else { Some(fields[5].as_str()) })
            .bind(upload_bytes)
            .bind(download_bytes)
            .bind(protocol_version)
            .bind(if fields[9].is_empty() { None } else { Some(fields[9].as_str()) })
            .execute(pool)
            .await;
        }
    } else if let Some(sessions) = body.get("player_sessions").and_then(|s| s.as_array()) {
        // Backward compatible with old JSON format
        for session in sessions {
            let username = match session.get("username").and_then(|v| v.as_str()) { Some(u) => u, None => continue };
            let uuid = session.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
            let backend_addr = session.get("backend_addr").and_then(|v| v.as_str()).unwrap_or("");
            let backend_port = session.get("backend_port").and_then(|v| v.as_i64()).unwrap_or(25565) as i32;
            let login_at = session.get("login_at").and_then(|v| v.as_str()).unwrap_or("datetime('now')");
            let logout_at = session.get("logout_at").and_then(|v| v.as_str());
            let upload_bytes = session.get("upload_bytes").and_then(|v| v.as_i64()).unwrap_or(0);
            let download_bytes = session.get("download_bytes").and_then(|v| v.as_i64()).unwrap_or(0);
            let protocol_version = session.get("protocol_version").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let kick_reason = session.get("kick_reason").and_then(|v| v.as_str());
            let _ = sqlx::query(
                r#"INSERT INTO player_sessions
                    (username, uuid, backend_addr, backend_port, login_at, logout_at,
                     upload_bytes, download_bytes, protocol_version, kick_reason)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(username).bind(uuid).bind(backend_addr).bind(backend_port)
            .bind(login_at).bind(logout_at).bind(upload_bytes).bind(download_bytes)
            .bind(protocol_version).bind(kick_reason)
            .execute(pool).await;
        }
    }

    // Reload proxy in-memory data
    state.proxy.reload_backends().await;
    state.proxy.reload_domain_routes().await;

    ok_response(json!({
        "status": "ok",
        "message": "Migration import complete"
    }))
}

/// Parse one line of CSV, handle quote escaping
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if in_quotes {
            if c == '"' {
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    current.push('"');
                    i += 2;
                    continue;
                }
                in_quotes = false;
            } else {
                current.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == ',' {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
        i += 1;
    }
    fields.push(current);
    fields
}
