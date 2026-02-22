pub mod access;
pub mod admin;
pub mod auth;
pub mod backend;
pub mod cli_handler;
pub mod config;
pub mod logs;
pub mod migration;
pub mod players;
pub mod routes;
pub mod server;
pub mod server_ping;
pub mod two_factor;

use crate::proxy::Proxy;
use axum::{
    response::Html,
};
use std::sync::Arc;

/// App State
pub struct AppState {
    pub proxy: Arc<Proxy>,
}

/// Serve frontend index.html
pub async fn serve_index() -> Html<String> {
    // Try reading index.html from different paths
    let paths = [
        "web/dist/index.html",
        "../web/dist/index.html",
        "../../web/dist/index.html",
    ];

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            return Html(content);
        }
    }

    // If not found, return a simple HTML
    Html(r#"<!DOCTYPE html>
<html>
<head><title>Minecraft Proxy Admin</title></head>
<body>
    <h1>Minecraft Proxy Admin</h1>
    <p>Please build the frontend: cd web && npm run build</p>
</body>
</html>"#.to_string())
}

// Re-export handler functions for easy access
pub use access::{
    get_whitelist, add_whitelist, remove_whitelist,
    get_blacklist, add_blacklist, remove_blacklist, kick_player,
    toggle_whitelist,
};
pub use players::{
    get_players as get_online_users,
    get_players_history as player_history,
    get_players_stats as player_sessions,
    get_dashboard_recent,
};
pub use routes::{
    get_domain_routes,
    add_domain_route,
    update_domain_route,
    delete_domain_route,
};
pub use config::{get_config as get_settings, update_config as update_settings, reload_config};
pub use logs::{get_moderation_logs, get_session_logs, export_moderation, export_sessions};
pub use two_factor::{list_player_2fa as two_fa_list, unbind_player_2fa as two_fa_remove, get_player_2fa_qr};
pub use server::restart_server;
pub use admin::{list_admins, create_admin, delete_admin, reset_admin_2fa, change_password, update_locale};
pub use backend::{get_backends, get_backend, create_backend, update_backend, delete_backend, enable_backend, disable_backend, set_default_backend, get_default_backend, toggle_maintenance, unset_default_backend_handler as unset_default_backend};
pub use migration::{export_migration, import_migration};
pub use server_ping::ping_server_handler;
pub use cli_handler::execute_cli;
