pub mod auth;
pub mod handlers;

use crate::proxy::Proxy;
use auth::auth_middleware;
use axum::{
    routing::{get, post, delete, patch},
    Router,
    middleware::Next,
    extract::Request,
    response::Response,
};
use handlers::AppState;
use std::sync::Arc;
use tower_http::services::ServeDir;

/// API request log middleware: records method + path + response status code
async fn log_api_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    tracing::info!("API {} {} → {}", method, path, response.status().as_u16());
    response
}

/// Create Web API router
pub fn create_router(proxy: Arc<Proxy>) -> Router {
    let state = Arc::new(AppState { proxy });

    // Public routes (no auth required) - use /api/v1 prefix
    let public_routes = Router::new()
        .route("/api/v1/auth/status", get(handlers::auth::get_auth_status))
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/auth/setup", post(handlers::auth::setup_admin))
        .route("/api/v1/auth/totp/setup", post(handlers::auth::totp_setup))
        .route("/api/v1/auth/totp/confirm", post(handlers::auth::totp_confirm))
        .route("/api/v1/auth/totp/verify", post(handlers::auth::totp_verify))
        .route("/api/v1/auth/logout", post(handlers::auth::logout))
        // Frontend static pages (all frontend routes return index.html)
        .route("/", get(handlers::serve_index))
        .route("/login", get(handlers::serve_index))
        .route("/setup", get(handlers::serve_index))
        .route("/dashboard", get(handlers::serve_index))
        .route("/players", get(handlers::serve_index))
        .route("/access", get(handlers::serve_index))
        .route("/routes", get(handlers::serve_index))
        .route("/backend", get(handlers::serve_index))
        .route("/twofactor", get(handlers::serve_index))
        .route("/logs", get(handlers::serve_index))
        .route("/config", get(handlers::serve_index))
        .route("/account", get(handlers::serve_index))
        // Static assets (CSS, JS, favicon, etc.)
        .nest_service("/assets", ServeDir::new("web/dist/assets"))
        .fallback_service(ServeDir::new("web/dist"));

    // Protected routes (auth required) - use /api/v1 prefix
    let protected_routes = Router::new()
        // Dashboard
        .route("/api/v1/dashboard/recent", get(handlers::get_dashboard_recent))
        // Players
        .route("/api/v1/players", get(handlers::get_online_users))
        .route("/api/v1/players/history", get(handlers::player_history))
        .route("/api/v1/players/stats", get(handlers::player_sessions))
        // Access
        .route("/api/v1/whitelist", get(handlers::get_whitelist))
        .route("/api/v1/whitelist", post(handlers::add_whitelist))
        .route("/api/v1/whitelist/{username}", delete(handlers::remove_whitelist))
        .route("/api/v1/whitelist", patch(handlers::toggle_whitelist))
        .route("/api/v1/blacklist", get(handlers::get_blacklist))
        .route("/api/v1/blacklist", post(handlers::add_blacklist))
        .route("/api/v1/blacklist/{username}", delete(handlers::remove_blacklist))
        .route("/api/v1/kick", post(handlers::kick_player))
        // Routes - Domain
        .route("/api/v1/routes/domain", get(handlers::get_domain_routes))
        .route("/api/v1/routes/domain", post(handlers::add_domain_route))
        .route("/api/v1/routes/domain/{id}", delete(handlers::delete_domain_route))
        .route("/api/v1/routes/domain/{id}", patch(handlers::update_domain_route))
        // Backend
        .route("/api/v1/backend", get(handlers::get_backends))
        .route("/api/v1/backend", post(handlers::create_backend))
        .route("/api/v1/backend/{id}", get(handlers::get_backend))
        .route("/api/v1/backend/{id}", patch(handlers::update_backend))
        .route("/api/v1/backend/{id}", delete(handlers::delete_backend))
        .route("/api/v1/backend/{id}/enable", post(handlers::enable_backend))
        .route("/api/v1/backend/{id}/disable", post(handlers::disable_backend))
        .route("/api/v1/backend/{id}/set-default", post(handlers::set_default_backend))
        .route("/api/v1/backend/{id}/maintenance", post(handlers::toggle_maintenance))
        .route("/api/v1/backend/default", get(handlers::get_default_backend))
        .route("/api/v1/backend/unset-default", post(handlers::unset_default_backend))
        // 2FA
        .route("/api/v1/2fa/players", get(handlers::two_fa_list))
        .route("/api/v1/2fa/players/{username}", delete(handlers::two_fa_remove))
        .route("/api/v1/2fa/players/{username}/qr", get(handlers::get_player_2fa_qr))
        // Config
        .route("/api/v1/config", get(handlers::get_settings))
        .route("/api/v1/config", patch(handlers::update_settings))
        .route("/api/v1/config/reload", post(handlers::reload_config))
        // Server
        .route("/api/v1/server/restart", post(handlers::restart_server))
        // Logs
        .route("/api/v1/logs/moderation", get(handlers::get_moderation_logs))
        .route("/api/v1/logs/moderation/export", get(handlers::export_moderation))
        .route("/api/v1/logs/sessions", get(handlers::get_session_logs))
        .route("/api/v1/logs/sessions/export", get(handlers::export_sessions))
        // Migration
        .route("/api/v1/migration/export", get(handlers::export_migration))
        .route("/api/v1/migration/import", post(handlers::import_migration))
        // Server Ping
        .route("/api/v1/ping", get(handlers::ping_server_handler))
        // CLI
        .route("/api/v1/cli", post(handlers::execute_cli))
        // Admin
        .route("/api/v1/admin/accounts", get(handlers::list_admins))
        .route("/api/v1/admin/accounts", post(handlers::create_admin))
        .route("/api/v1/admin/accounts/{id}", delete(handlers::delete_admin))
        .route("/api/v1/admin/accounts/{id}/reset-2fa", post(handlers::reset_admin_2fa))
        .route("/api/v1/admin/me/password", post(handlers::change_password))
        .route("/api/v1/admin/me/locale", patch(handlers::update_locale))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(axum::middleware::from_fn(log_api_request));

    public_routes
        .merge(protected_routes)
        .with_state(state)
}

/// Start Web API server
pub async fn start_web_api(proxy: Arc<Proxy>) -> anyhow::Result<()> {
    let config = proxy.config.read().await;
    let addr = format!("{}:{}", config.web_api_address, config.web_api_port);
    drop(config);

    // Subscribe to shutdown signal before proxy is moved into router
    let mut shutdown_rx = proxy.shutdown_tx.subscribe();
    let router = create_router(proxy);

    // Try to bind port, max 10 retries (1 second interval)
    let listener = {
        let mut last_err = None;
        let mut bound = None;
        for attempt in 0..10u32 {
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => {
                    bound = Some(l);
                    break;
                }
                Err(e) => {
                    if attempt > 0 {
                        tracing::warn!(
                            "Web API port {} still in use (attempt {}/10), retrying...",
                            addr,
                            attempt + 1
                        );
                    }
                    last_err = Some(e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
        match bound {
            Some(l) => l,
            None => return Err(anyhow::anyhow!("Failed to bind Web API on {}: {}", addr, last_err.unwrap())),
        }
    };

    tracing::info!("Web API listening on {}", addr);

    // with_graceful_shutdown: after receiving shutdown signal, stop accepting new connections,
    // wait for in-flight requests (including restart HTTP response) to complete before dropping listener,
    // so port is released before main() spawns new process.
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await?;

    Ok(())
}
