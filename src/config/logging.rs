use std::{env, fs, path::PathBuf};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub struct LoggingConfig {
    pub level: String,
    pub directory: String,
    pub filename: String,
    pub format: String,
    pub console_enabled: bool,
    pub file_enabled: bool,
}

impl LoggingConfig {
    pub fn from_env() -> Self {
        Self {
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            directory: env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string()),
            filename: env::var("LOG_FILE").unwrap_or_else(|_| "app.log".to_string()),
            format: env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string()),
            console_enabled: env_bool("LOG_TO_CONSOLE", true),
            file_enabled: env_bool("LOG_TO_FILE", true),
        }
    }
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

// Return guard agar worker penulisan file tetap hidup
pub fn init_logging() -> Option<WorkerGuard> {
    let config = LoggingConfig::from_env();

    let filter = EnvFilter::try_new(&config.level).expect("Invalid LOG_LEVEL configuration");

    let mut guard = None;

    // Console output
    let console_layer = config.console_enabled.then(|| {
        fmt::layer()
            .pretty()
            .with_target(true)
            .with_ansi(true)
            .with_writer(std::io::stdout)
    });

    // File output
    let file_layer = if config.file_enabled {
        let directory = PathBuf::from(&config.directory);

        fs::create_dir_all(&directory).expect("Failed to create log directory");

        // Never rolling = nama file tetap app.log
        let appender = tracing_appender::rolling::never(directory, &config.filename);

        let (writer, worker_guard) = tracing_appender::non_blocking(appender);

        guard = Some(worker_guard);

        Some(
            fmt::layer()
                .json()
                .with_ansi(false)
                .with_current_span(true)
                .with_span_list(true)
                .with_writer(writer),
        )
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!(
        level = %config.level,
        console = config.console_enabled,
        file =  config.file_enabled,
        format = %config.format,
        "Logging initialized"
    );

    guard
}
