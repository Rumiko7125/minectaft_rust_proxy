mod access_control;
mod auth;
mod cli;
mod config;
mod db;
mod error;
mod i18n;
mod motd;
mod protocol;
mod proxy;
mod web_api;

use clap::Parser;
use config::Config;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// Current console log level (0=off, 1=ERROR, 2=WARN, 3=INFO, 4+=DEBUG+TRACE), can be modified dynamically
pub static CONSOLE_LOG_LEVEL: AtomicU8 = AtomicU8::new(3);
/// Current file log level (0=no file, 1=ERROR, 2=WARN, 3=INFO, 4+=DEBUG+TRACE), can be modified dynamically
pub static FILE_LOG_LEVEL: AtomicU8 = AtomicU8::new(0);

/// Dynamic log filter: reads global AtomicU8 to determine current log level
/// Level meaning: 0=off, 1=ERROR, 2=WARN, 3=INFO, 4+=DEBUG+TRACE
struct AtomicLevelFilter(pub &'static AtomicU8);

impl<S: tracing::Subscriber> tracing_subscriber::layer::Filter<S> for AtomicLevelFilter {
    fn enabled(&self, meta: &tracing::Metadata<'_>, _cx: &tracing_subscriber::layer::Context<'_, S>) -> bool {
        let level_setting = self.0.load(Ordering::Relaxed);
        let max = match level_setting {
            0 => return false,
            1 => tracing_subscriber::filter::LevelFilter::ERROR,
            2 => tracing_subscriber::filter::LevelFilter::WARN,
            3 => tracing_subscriber::filter::LevelFilter::INFO,
            _ => tracing_subscriber::filter::LevelFilter::TRACE,
        };
        max >= *meta.level()
    }
}

/// Wait for specified PID process to exit (max 30 seconds)
/// Windows uses WaitForSingleObject to detect process exit; Linux polls /proc; other Unix sleeps 3s
#[cfg(target_os = "windows")]
fn wait_for_pid_exit(pid: u32) {
    extern "system" {
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
        fn WaitForSingleObject(hHandle: isize, dwMilliseconds: u32) -> u32;
        fn CloseHandle(hObject: isize) -> i32;
    }
    const SYNCHRONIZE: u32 = 0x00100000;
    unsafe {
        let handle = OpenProcess(SYNCHRONIZE, 0, pid);
        if handle == 0 {
            return; // Process exited or no permission to open (treat as exited)
        }
        WaitForSingleObject(handle, 30_000);
        CloseHandle(handle);
    }
}

#[cfg(not(target_os = "windows"))]
fn wait_for_pid_exit(pid: u32) {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(30);
    let proc_path = format!("/proc/{}", pid);
    if std::path::Path::new(&proc_path).exists() {
        // Linux: poll /proc/{pid}
        while Instant::now() < deadline {
            if !std::path::Path::new(&proc_path).exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    } else {
        // macOS and others: fallback to fixed wait
        std::thread::sleep(Duration::from_secs(3));
    }
}

/// Resolve relative path to absolute path relative to executable directory
pub fn resolve_log_dir(log_dir: &str) -> String {
    if log_dir.is_empty() {
        return String::new();
    }
    let p = std::path::Path::new(log_dir);
    if p.is_absolute() {
        return log_dir.to_string();
    }
    // Relative path -> based on exe directory
    let base = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join(p).to_string_lossy().to_string()
}

/// Initialize logging system (console + optional file output)
/// show_log_level / save_log_level: 0=off, 1=ERROR, 2=WARN, 3=INFO, 4+=DEBUG+TRACE
fn init_logging(config: &Config) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

    // Write config values to global Atomic, subsequent dynamic modifications also use these two Atomics
    CONSOLE_LOG_LEVEL.store(config.show_log_level, Ordering::Relaxed);
    FILE_LOG_LEVEL.store(config.save_log_level, Ordering::Relaxed);

    if config.save_log_level > 0 && !config.log_dir.is_empty() {
        let abs_log_dir = resolve_log_dir(&config.log_dir);
        let _ = std::fs::create_dir_all(&abs_log_dir);
        let log_path = format!("{}/proxy.log", abs_log_dir);
        eprintln!("File logging enabled: {}", log_path);
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(false)
                        .with_filter(AtomicLevelFilter(&CONSOLE_LOG_LEVEL)),
                )
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_ansi(false)
                        .with_target(false)
                        .with_writer(std::sync::Mutex::new(file))
                        .with_filter(AtomicLevelFilter(&FILE_LOG_LEVEL)),
                )
                .init();
            return;
        } else {
            eprintln!("Warning: could not open log file {}, falling back to console only", log_path);
        }
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_filter(AtomicLevelFilter(&CONSOLE_LOG_LEVEL)),
        )
        .init();
}

#[derive(Parser)]
#[command(name = "minecraft-rust-proxy")]
#[command(about = "A high-performance Minecraft proxy written in Rust")]
#[command(version)]
struct Cli {
    /// Database file path
    #[arg(short = 'd', long = "db")]
    db_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Restart mode: wait for old process to fully exit before binding port
    // PROXY_PARENT_PID: old process PID, use OS API to precisely detect its exit
    if let Ok(pid_str) = std::env::var("PROXY_PARENT_PID") {
        std::env::remove_var("PROXY_PARENT_PID");
        if let Ok(pid) = pid_str.parse::<u32>() {
            eprintln!("Restart mode: waiting for parent process (pid={}) to exit...", pid);
            wait_for_pid_exit(pid);
            eprintln!("Parent process (pid={}) has exited.", pid);
        }
    }
    // PROXY_RESTART_DELAY: optional extra buffer time (ms), wait for port to be released by OS
    if let Ok(delay_str) = std::env::var("PROXY_RESTART_DELAY") {
        std::env::remove_var("PROXY_RESTART_DELAY");
        if let Ok(delay_ms) = delay_str.parse::<u64>() {
            if delay_ms > 0 {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            }
        }
    }

    let cli = Cli::parse();

    // Determine database path
    let db_path = if let Some(p) = cli.db_path {
        p
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("proxy.db")
    };
    let db_path_str = db_path.to_string_lossy().to_string();

    // Initialize database first (before tracing), so we can read log config
    let pool = db::init_db(&db_path_str).await?;

    // Initialize default config (first time only)
    if let Err(e) = db::init_default_settings(&pool).await {
        eprintln!("Init settings warning: {}", e);
    }

    // Initialize default backend server (if none exists)
    if let Err(e) = db::backend_servers::init_default_backend(&pool).await {
        eprintln!("Init backend warning: {}", e);
    }

    // Load config from database
    let config = Config::load(&pool).await?;

    // Initialize logging based on config (supports console + file)
    init_logging(&config);

    // Initialize 1.21.x registry data cache
    protocol::registry::init();

    let allow_input = config.allow_input;
    let web_api_enable = config.web_api_enable;

    // Create proxy
    let proxy = proxy::Proxy::new(config, pool).await?;

    // Start Web API (keep handle to wait for graceful restart)
    let web_api_handle = if web_api_enable {
        let proxy_clone = Arc::clone(&proxy);
        Some(tokio::spawn(async move {
            if let Err(e) = web_api::start_web_api(proxy_clone).await {
                tracing::error!("Web API error: {}", e);
            }
        }))
    } else {
        None
    };

    // Start CLI
    if allow_input {
        let proxy_clone = Arc::clone(&proxy);
        tokio::spawn(async move {
            cli::run_cli(proxy_clone).await;
        });
    }

    // Start proxy (blocks until shutdown)
    if let Err(e) = proxy.start().await {
        tracing::error!("Proxy startup failed: {}", e);
        return Err(e);
    }

    // Check if graceful restart requested
    let do_restart = proxy.restart_pending.load(std::sync::atomic::Ordering::Relaxed);

    // Wait for Web API graceful shutdown (max 5 seconds), ensure listener is dropped and port is released
    if let Some(handle) = web_api_handle {
        let _ = tokio::time::timeout(tokio::time::Duration::from_secs(5), handle).await;
    }

    if do_restart {
        // At this point, both proxy listener and web API listener have been dropped, ports are released
        // New process can bind immediately without any delay
        let current_exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args().skip(1).collect();

        tracing::info!("Graceful restart: spawning new process...");

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            std::process::Command::new(&current_exe)
                .args(&args)
                .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::process::Command::new(&current_exe)
                .args(&args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;
        }

        tracing::info!("Graceful restart: new process spawned, exiting current process");
        std::process::exit(0);
    }

    Ok(())
}
