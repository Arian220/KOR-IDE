use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct UploadRequest {
    pub sketch_path: String,
    pub fqbn: String,
    pub port: String,
    #[serde(default)]
    pub compile_first: bool,
    #[serde(default)]
    pub verify: bool,
    pub programmer: Option<String>,
    pub baudrate: Option<u32>,
    #[serde(default)]
    pub build_properties: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UploadResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub port: String,
    pub fqbn: String,
}
