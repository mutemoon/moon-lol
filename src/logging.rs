use bevy::prelude::*;
use chrono::Utc;
use std::fs::File;
use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Resource to store the log file path
#[derive(Resource)]
pub struct LogFilePath(pub PathBuf);

/// Plugin for setting up file-based logging
pub struct PluginLogging;

impl Plugin for PluginLogging {
    fn build(&self, app: &mut App) {
        // Create log file path
        let temp_dir = std::env::temp_dir();
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let log_filename = format!("moon_lol_debug_{}.log", timestamp);
        let log_path = temp_dir.join(log_filename);

        // Set up file logging
        setup_file_logging(&log_path);

        println!("🔍 Debug logs are being saved to: {}", log_path.display());

        app.insert_resource(LogFilePath(log_path));
        app.add_systems(Startup, log_startup_info);
    }
}

/// Set up file-based logging
fn setup_file_logging(log_path: &PathBuf) {
    // Create the log file
    let log_file = File::create(log_path).expect("Failed to create log file");

    // Set up tracing subscriber with file output only
    let file_layer = fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    // Set up environment filter - default to INFO level, but allow override
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,moon_lol=debug"));

    // Try to set up tracing subscriber with only file output, but don't panic if it's already initialized
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .try_init();
}

/// System to log startup information
fn log_startup_info(log_path: Res<LogFilePath>) {
    info!("🚀 Moon LoL game started");
    info!("📁 Log file location: {}", log_path.0.display());
    info!("💡 Set RUST_LOG environment variable to control log levels");
    debug!("🔧 Debug logging is enabled for all systems");
}

/// Macro to create consistent debug logs for systems
#[macro_export]
macro_rules! system_debug {
    ($system_name:expr, $($arg:tt)*) => {
        tracing::debug!(
            target: "moon_lol::systems",
            system = $system_name,
            $($arg)*
        );
    };
}

/// Macro to create consistent info logs for systems
#[macro_export]
macro_rules! system_info {
    ($system_name:expr, $($arg:tt)*) => {
        tracing::info!(
            target: "moon_lol::systems",
            system = $system_name,
            $($arg)*
        );
    };
}

/// Macro to create consistent warn logs for systems
#[macro_export]
macro_rules! system_warn {
    ($system_name:expr, $($arg:tt)*) => {
        tracing::warn!(
            target: "moon_lol::systems",
            system = $system_name,
            $($arg)*
        );
    };
}

/// Macro to create consistent error logs for systems
#[macro_export]
macro_rules! system_error {
    ($system_name:expr, $($arg:tt)*) => {
        tracing::error!(
            target: "moon_lol::systems",
            system = $system_name,
            $($arg)*
        );
    };
}
