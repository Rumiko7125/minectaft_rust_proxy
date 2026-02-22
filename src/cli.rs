use crate::db;
use crate::proxy::Proxy;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Run CLI interactive loop
pub async fn run_cli(proxy: Arc<Proxy>) {
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    println!("Type 'help' for available commands.");

    while let Ok(Some(line)) = lines.next_line().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if matches!(parts[0], "stop" | "exit" | "quit") {
            println!("Shutting down...");
            proxy.shutdown();
            break;
        }

        let output = execute_command(&proxy, trimmed).await;
        if !output.is_empty() {
            println!("{}", output);
        }
    }
}

/// Execute CLI command and return output string (shared by Web API / console)
pub async fn execute_command(proxy: &Arc<Proxy>, command: &str) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }

    match parts[0] {
        "help" => get_help_text().to_string(),
        "stop" | "exit" | "quit" => {
            "Error: 'stop' is not available via web CLI. Use the restart button in Danger Zone.".to_string()
        }
        "list" => cmd_list(proxy, &parts[1..]).await,
        "kick" => cmd_kick(proxy, &parts[1..]).await,
        "ban" => cmd_ban(proxy, &parts[1..]).await,
        "pardon" => cmd_pardon(proxy, &parts[1..]).await,
        "whitelist" => cmd_whitelist(proxy, &parts[1..]).await,
        "backend" => cmd_backend(proxy, &parts[1..]).await,
        "2fa" => cmd_2fa(proxy, &parts[1..]).await,
        "config" => cmd_config(proxy, &parts[1..]).await,
        "modlog" => cmd_modlog(proxy, &parts[1..]).await,
        _ => format!("Unknown command '{}'. Type 'help' for help.", parts[0]),
    }
}

fn get_help_text() -> &'static str {
    r#"Available commands:
  help                               - Show this help
  list players                       - List online players
  list whitelist                     - List whitelist
  list blacklist                     - List blacklist
  kick <name> [reason]               - Kick a player
  ban <name> [reason]                - Ban a player
  pardon <name>                      - Unban a player
  whitelist on|off                   - Enable/disable whitelist
  whitelist add <name>               - Add to whitelist
  whitelist remove <name>            - Remove from whitelist
  backend list                       - List all backend servers
  backend info <id>                  - Show backend details
  backend add <name> <addr> <port>   - Add backend server
  backend remove <id>                - Remove backend server
  backend enable <id>                - Enable backend server
  backend disable <id>               - Disable backend server
  backend default <id>               - Set default backend
  backend maintenance <id> on|off [msg] - Toggle maintenance mode
  2fa list                           - List 2FA-enabled users
  2fa remove <name>                  - Remove user's 2FA
  config list                        - List all config settings
  config set <key> <value>           - Set config value
  config reload                      - Reload config from database
  modlog [player] [n]                - Show moderation logs"#
}

async fn cmd_list(proxy: &Proxy, args: &[&str]) -> String {
    match args.first().copied() {
        Some("players") => {
            let users: Vec<_> = proxy.users.iter().map(|e| {
                let u = e.value().clone();
                (u.username.clone(), u.backend_name.clone(), u.protocol_version)
            }).collect();
            if users.is_empty() {
                "No players online.".to_string()
            } else {
                let mut out = format!("Online players ({}):\n", users.len());
                for (name, backend, protocol) in &users {
                    out.push_str(&format!("  {} -> {} (protocol {})\n", name, backend, protocol));
                }
                out.trim_end().to_string()
            }
        }
        Some("whitelist") => {
            let ac = proxy.access_control.read().await;
            format!(
                "Whitelist ({}): {:?}",
                if ac.whitelist_enabled { "enabled" } else { "disabled" },
                ac.whitelist
            )
        }
        Some("blacklist") => {
            let ac = proxy.access_control.read().await;
            format!("Blacklist: {:?}", ac.blacklist)
        }
        _ => "Usage: list <players|whitelist|blacklist>".to_string(),
    }
}

async fn cmd_kick(proxy: &Proxy, args: &[&str]) -> String {
    if let Some(name) = args.first() {
        let reason = if args.len() > 1 { Some(args[1..].join(" ")) } else { None };
        if proxy.kick_player(name) {
            db::modlog::log_action(&proxy.db_pool, "kick", name, "web-cli", reason.as_deref()).await.ok();
            format!("Kicked {}.", name)
        } else {
            format!("Player {} not found.", name)
        }
    } else {
        "Usage: kick <name> [reason]".to_string()
    }
}

async fn cmd_ban(proxy: &Proxy, args: &[&str]) -> String {
    if let Some(name) = args.first() {
        let reason = if args.len() > 1 { Some(args[1..].join(" ")) } else { None };
        let mut ac = proxy.access_control.write().await;
        ac.add_blacklist(name, reason.as_deref()).await;
        proxy.kick_player(name);
        drop(ac);
        db::modlog::log_action(&proxy.db_pool, "ban", name, "web-cli", reason.as_deref()).await.ok();
        format!("Banned {}.", name)
    } else {
        "Usage: ban <name> [reason]".to_string()
    }
}

async fn cmd_pardon(proxy: &Proxy, args: &[&str]) -> String {
    if let Some(name) = args.first() {
        let mut ac = proxy.access_control.write().await;
        ac.remove_blacklist(name).await;
        drop(ac);
        db::modlog::log_action(&proxy.db_pool, "pardon", name, "web-cli", None).await.ok();
        format!("Pardoned {}.", name)
    } else {
        "Usage: pardon <name>".to_string()
    }
}

async fn cmd_whitelist(proxy: &Proxy, args: &[&str]) -> String {
    match args.first().copied() {
        Some("on") => {
            let mut ac = proxy.access_control.write().await;
            ac.whitelist_enabled = true;
            "Whitelist enabled.".to_string()
        }
        Some("off") => {
            let mut ac = proxy.access_control.write().await;
            ac.whitelist_enabled = false;
            "Whitelist disabled.".to_string()
        }
        Some("add") => {
            if let Some(name) = args.get(1) {
                let mut ac = proxy.access_control.write().await;
                ac.add_whitelist(name).await;
                format!("Added {} to whitelist.", name)
            } else {
                "Usage: whitelist add <name>".to_string()
            }
        }
        Some("remove") => {
            if let Some(name) = args.get(1) {
                let mut ac = proxy.access_control.write().await;
                ac.remove_whitelist(name).await;
                format!("Removed {} from whitelist.", name)
            } else {
                "Usage: whitelist remove <name>".to_string()
            }
        }
        _ => "Usage: whitelist <on|off|add|remove> [name]".to_string(),
    }
}

async fn cmd_backend(proxy: &Proxy, args: &[&str]) -> String {
    match args.first().copied() {
        Some("list") => {
            match db::backend_servers::get_all_backend_servers(&proxy.db_pool).await {
                Ok(backends) => {
                    if backends.is_empty() {
                        "No backend servers configured.".to_string()
                    } else {
                        let mut out = "Backend Servers:\n".to_string();
                        for b in &backends {
                            let status = match (b.enabled, b.maintenance) {
                                (false, _) => "[DISABLED]",
                                (true, true) => "[MAINTENANCE]",
                                (true, false) => "[ENABLED]",
                            };
                            let default = if b.is_default { " (DEFAULT)" } else { "" };
                            out.push_str(&format!(
                                "  [{}] {} {} {}{}\n       -> {}:{}\n",
                                b.id, b.name, status, default,
                                if b.max_player < 0 { String::new() } else { format!("(max: {})", b.max_player) },
                                b.remote_address, b.remote_port
                            ));
                        }
                        out.trim_end().to_string()
                    }
                }
                Err(e) => format!("Database error: {}", e),
            }
        }
        Some("info") => {
            if let Some(id_str) = args.get(1) {
                if let Ok(id) = id_str.parse::<i64>() {
                    match db::backend_servers::get_backend_server(&proxy.db_pool, id).await {
                        Ok(Some(b)) => format!(
                            "Backend [{}] {}:\n  Address:     {}:{}\n  Max Players: {}\n  Status:      {}\n  Default:     {}\n  Maintenance: {}{}",
                            b.id, b.name,
                            b.remote_address, b.remote_port,
                            if b.max_player < 0 { "unlimited".to_string() } else { b.max_player.to_string() },
                            if b.enabled { "enabled" } else { "disabled" },
                            b.is_default,
                            b.maintenance,
                            b.maintenance_message.map(|m| format!("\n  Maint. Msg:  {}", m)).unwrap_or_default()
                        ),
                        Ok(None) => format!("Backend id={} not found.", id),
                        Err(e) => format!("Database error: {}", e),
                    }
                } else {
                    "Invalid id.".to_string()
                }
            } else {
                "Usage: backend info <id>".to_string()
            }
        }
        Some("add") => {
            if args.len() >= 4 {
                let name = args[1];
                let addr = args[2];
                let port: u16 = args[3].parse().unwrap_or(25565);
                let req = db::backend_servers::CreateBackendRequest {
                    name: name.to_string(),
                    remote_address: addr.to_string(),
                    remote_port: Some(port as i32),
                    max_player: Some(-1),
                    motd_json: None,
                    limbo_message: None,
                    log_dir: None,
                    show_log_level: Some(0),
                    save_log_level: Some(0),
                    enabled: Some(true),
                    maintenance: Some(false),
                    maintenance_message: None,
                    ping_passthrough: Some(false),
                    motd_passthrough: Some(false),
                    language: None,
                };
                match db::backend_servers::create_backend_server(&proxy.db_pool, &req).await {
                    Ok(id) => {
                        proxy.reload_backends().await;
                        format!("Added backend '{}' -> {}:{} (id={})", name, addr, port, id)
                    }
                    Err(e) => format!("Database error: {}", e),
                }
            } else {
                "Usage: backend add <name> <addr> <port>".to_string()
            }
        }
        Some("remove") => {
            if let Some(id_str) = args.get(1) {
                if let Ok(id) = id_str.parse::<i64>() {
                    match db::backend_servers::is_backend_in_use(&proxy.db_pool, id).await {
                        Ok(true) => "Cannot remove: backend is used by domain routes.".to_string(),
                        Ok(false) => {
                            match db::backend_servers::delete_backend_server(&proxy.db_pool, id).await {
                                Ok(_) => { proxy.reload_backends().await; format!("Removed backend id={}.", id) }
                                Err(e) => format!("Database error: {}", e),
                            }
                        }
                        Err(e) => format!("Database error: {}", e),
                    }
                } else { "Invalid id.".to_string() }
            } else { "Usage: backend remove <id>".to_string() }
        }
        Some("enable") => {
            if let Some(id_str) = args.get(1) {
                if let Ok(id) = id_str.parse::<i64>() {
                    match db::backend_servers::enable_backend_server(&proxy.db_pool, id).await {
                        Ok(_) => { proxy.reload_backends().await; format!("Enabled backend id={}.", id) }
                        Err(e) => format!("Error: {}", e),
                    }
                } else { "Invalid id.".to_string() }
            } else { "Usage: backend enable <id>".to_string() }
        }
        Some("disable") => {
            if let Some(id_str) = args.get(1) {
                if let Ok(id) = id_str.parse::<i64>() {
                    match db::backend_servers::disable_backend_server(&proxy.db_pool, id).await {
                        Ok(_) => { proxy.reload_backends().await; format!("Disabled backend id={}.", id) }
                        Err(e) => format!("Error: {}", e),
                    }
                } else { "Invalid id.".to_string() }
            } else { "Usage: backend disable <id>".to_string() }
        }
        Some("default") => {
            if let Some(id_str) = args.get(1) {
                if let Ok(id) = id_str.parse::<i64>() {
                    match db::backend_servers::set_default_backend(&proxy.db_pool, id).await {
                        Ok(_) => { proxy.reload_backends().await; format!("Set default backend id={}.", id) }
                        Err(e) => format!("Error: {}", e),
                    }
                } else { "Invalid id.".to_string() }
            } else {
                match db::backend_servers::get_default_backend(&proxy.db_pool).await {
                    Ok(Some(b)) => format!("Default backend: {} (id={})", b.name, b.id),
                    Ok(None) => "No default backend set.".to_string(),
                    Err(e) => format!("Error: {}", e),
                }
            }
        }
        Some("maintenance") => {
            if args.len() >= 3 {
                if let Ok(id) = args[1].parse::<i64>() {
                    let on = args[2] == "on";
                    let message = if args.len() > 3 { Some(args[3..].join(" ")) } else { None };
                    match db::backend_servers::toggle_maintenance(&proxy.db_pool, id, on, message).await {
                        Ok(_) => {
                            proxy.reload_backends().await;
                            format!("Backend id={} maintenance mode: {}", id, if on { "ON" } else { "OFF" })
                        }
                        Err(e) => format!("Error: {}", e),
                    }
                } else { "Invalid id.".to_string() }
            } else { "Usage: backend maintenance <id> on|off [message]".to_string() }
        }
        _ => "Usage: backend <list|info|add|remove|enable|disable|default|maintenance> ...".to_string(),
    }
}

async fn cmd_2fa(proxy: &Proxy, args: &[&str]) -> String {
    match args.first().copied() {
        Some("list") => {
            match db::two_factor::get_all_2fa_users(&proxy.db_pool).await {
                Ok(users) => {
                    if users.is_empty() {
                        "No 2FA-enabled users.".to_string()
                    } else {
                        let mut out = "2FA users:\n".to_string();
                        for u in users {
                            out.push_str(&format!("  {} (bound at: {})\n", u.username, u.created_at));
                        }
                        out.trim_end().to_string()
                    }
                }
                Err(e) => format!("Database error: {}", e),
            }
        }
        Some("remove") => {
            if let Some(name) = args.get(1) {
                match db::two_factor::delete_secret(&proxy.db_pool, name).await {
                    Ok(_) => format!("Removed 2FA for {}.", name),
                    Err(e) => format!("Database error: {}", e),
                }
            } else {
                "Usage: 2fa remove <name>".to_string()
            }
        }
        _ => "Usage: 2fa <list|remove> ...".to_string(),
    }
}

async fn cmd_config(proxy: &Proxy, args: &[&str]) -> String {
    match args.first().copied() {
        Some("list") => {
            let config = proxy.config.read().await;
            let ac = proxy.access_control.read().await;
            format!(
                "Current config:\n  local_address:           {}\n  local_port:              {}\n  whitelist_enabled:       {}\n  enable_2fa:              {}\n  two_factor_session_hours:{}\n  two_factor_issuer:       {}\n  web_api_enable:          {}\n  web_api_address:         {}\n  web_api_port:            {}\n  language:                {}\n  log_dir:                 {}\n  show_log_level:          {}\n  save_log_level:          {}",
                config.local_address,
                config.local_port,
                ac.whitelist_enabled,
                config.enable_2fa,
                config.two_factor_session_hours,
                config.two_factor_issuer,
                config.web_api_enable,
                config.web_api_address,
                config.web_api_port,
                config.language,
                config.log_dir,
                config.show_log_level,
                config.save_log_level
            )
        }
        Some("set") => {
            if args.len() >= 3 {
                let key = args[1];
                let value = args[2];
                match db::settings::set_setting(&proxy.db_pool, key, value).await {
                    Ok(_) => {
                        if let Ok(new_config) = crate::config::Config::load(&proxy.db_pool).await {
                            *proxy.config.write().await = new_config;
                            format!("Set {} = {} (config reloaded)", key, value)
                        } else {
                            format!("Set {} = {} (reload failed)", key, value)
                        }
                    }
                    Err(e) => format!("Failed to set config: {}", e),
                }
            } else {
                "Usage: config set <key> <value>".to_string()
            }
        }
        Some("reload") => {
            match crate::config::Config::load(&proxy.db_pool).await {
                Ok(new_config) => {
                    *proxy.config.write().await = new_config;
                    "Config reloaded from database.".to_string()
                }
                Err(e) => format!("Failed to reload config: {}", e),
            }
        }
        _ => "Usage: config <list|set|reload>".to_string(),
    }
}

async fn cmd_modlog(proxy: &Proxy, args: &[&str]) -> String {
    let (target, limit) = match args.len() {
        0 => (None, 20),
        1 => {
            if let Ok(n) = args[0].parse::<usize>() {
                (None, n)
            } else {
                (Some(args[0]), 20)
            }
        }
        _ => (Some(args[0]), args[1].parse().unwrap_or(20)),
    };

    match db::modlog::get_logs(&proxy.db_pool, target, limit).await {
        Ok(logs) => {
            if logs.is_empty() {
                "No moderation logs found.".to_string()
            } else {
                let mut out = String::new();
                for log in logs {
                    let reason = log.reason.map(|r| format!(" - {}", r)).unwrap_or_default();
                    out.push_str(&format!(
                        "[{}] {:6} {}  (by {}){}\n",
                        log.created_at, log.action, log.target, log.operator, reason
                    ));
                }
                out.trim_end().to_string()
            }
        }
        Err(e) => format!("Failed to get logs: {}", e),
    }
}
