use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::models::{AppConfig, AppError, CliInfo, JobKind};

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub root: PathBuf,
    pub sketches: PathBuf,
    pub logs: PathBuf,
    pub tmp: PathBuf,
    pub config_file: PathBuf,
    pub cli_dir: PathBuf,
    pub cli_bin: PathBuf,
}

impl AppPaths {
    pub fn new(root: PathBuf) -> Self {
        let cli_dir = root.join("cli");
        let cli_bin = if cfg!(target_os = "windows") {
            cli_dir.join("arduino-cli.exe")
        } else {
            cli_dir.join("arduino-cli")
        };
        Self {
            sketches: root.join("sketches"),
            logs: root.join("logs"),
            tmp: root.join("tmp"),
            config_file: root.join("config.json"),
            cli_dir,
            cli_bin,
            root,
        }
    }

    pub fn ensure(&self) -> Result<(), AppError> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(&self.sketches)?;
        std::fs::create_dir_all(&self.logs)?;
        std::fs::create_dir_all(&self.tmp)?;
        std::fs::create_dir_all(&self.cli_dir)?;
        Ok(())
    }

    pub fn sketch_dir(&self, name: &str) -> PathBuf {
        self.sketches.join(name)
    }

    pub fn sketch_ino(&self, name: &str) -> PathBuf {
        self.sketch_dir(name).join(format!("{name}.ino"))
    }

    pub fn ensure_inside_sketches(&self, path: &Path) -> Result<PathBuf, AppError> {
        crate::infra::fs::ensure_inside(&self.sketches, path)
    }
}

pub struct JobHandle {
    pub kind: JobKind,
    pub cancel: CancellationToken,
    #[allow(dead_code)]
    pub started_at: Instant,
}

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub cli: RwLock<Option<CliInfo>>,
    pub jobs: Mutex<HashMap<String, JobHandle>>,
    pub paths: AppPaths,
}

impl AppState {
    pub fn new(paths: AppPaths, config: AppConfig) -> Self {
        Self {
            config: RwLock::new(config),
            cli: RwLock::new(None),
            jobs: Mutex::new(HashMap::new()),
            paths,
        }
    }
}
