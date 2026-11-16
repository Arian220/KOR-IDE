use std::fs;
use std::path::Path;
use std::sync::Mutex;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

use crate::models::{AppConfig, AppError};
use crate::state::AppPaths;

const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

#[allow(dead_code)]
pub struct LogGuard(Mutex<WorkerGuard>);

impl LogGuard {
    pub fn new(guard: WorkerGuard) -> Self {
        Self(Mutex::new(guard))
    }
}

pub fn init(paths: &AppPaths, config: &AppConfig) -> Result<LogGuard, AppError> {
    fs::create_dir_all(&paths.logs)?;
    let log_file = paths.logs.join("kor.log");
    rotate(&log_file)?;

    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;
    let (writer, guard) = tracing_appender::non_blocking(file);

    let level = if config.log_level.trim().is_empty() {
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "info"
        }
    } else {
        config.log_level.as_str()
    };

    let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .with_target(false)
        .try_init()
        .map_err(|err| AppError::Internal(format!("no se pudo iniciar el logging: {err}")))?;

    Ok(LogGuard::new(guard))
}

fn rotate(path: &Path) -> Result<(), AppError> {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.len() > MAX_LOG_BYTES {
            let backup = path.with_file_name("kor.log.1");
            let _ = fs::remove_file(&backup);
            fs::rename(path, backup)?;
        }
    }
    Ok(())
}
