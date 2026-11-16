use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliSource {
    Managed,
    Manual,
    Path,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CliInfo {
    pub path: String,
    pub version: String,
    pub source: CliSource,
}

#[derive(Clone, Debug, Serialize)]
pub struct CliStatus {
    pub available: bool,
    pub info: Option<CliInfo>,
    pub required_version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub cli_path: Option<String>,
    pub cli_version: Option<String>,
    pub theme: Theme,
    pub last_fqbn: Option<String>,
    pub last_port: Option<String>,
    pub autosave: bool,
    pub autosave_delay_ms: u32,
    pub log_level: String,
    pub max_recent_sketches: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cli_path: None,
            cli_version: None,
            theme: Theme::System,
            last_fqbn: None,
            last_port: None,
            autosave: true,
            autosave_delay_ms: 800,
            log_level: "info".to_string(),
            max_recent_sketches: 20,
        }
    }
}
