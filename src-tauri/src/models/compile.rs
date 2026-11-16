use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct CompileRequest {
    pub sketch_path: String,
    pub fqbn: String,
    #[serde(default)]
    pub build_properties: Vec<String>,
    pub warnings: Option<String>,
    #[serde(default)]
    pub clean: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub diagnostics: Vec<Diagnostic>,
    pub memory: Option<MemoryUsage>,
    pub build_dir: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub flash_used: u64,
    pub flash_max: u64,
    pub flash_percent: f32,
    pub sram_used: u64,
    pub sram_max: u64,
    pub sram_percent: f32,
}
