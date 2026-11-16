use serde::Serialize;

use super::compile::Diagnostic;

#[allow(dead_code)]
#[derive(Clone, Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum AppError {
    #[error("arduino-cli no está disponible")]
    CliNotFound,
    #[error("fallo al descargar arduino-cli: {0}")]
    CliDownload(String),
    #[error("checksum inválido para arduino-cli")]
    CliChecksumMismatch,
    #[error("versión de arduino-cli no soportada: {0}")]
    CliUnsupported(String),
    #[error("no se detectaron placas")]
    BoardNotFound,
    #[error("puerto inválido: {0}")]
    InvalidPort(String),
    #[error("FQBN inválido: {0}")]
    InvalidFqbn(String),
    #[error("compilación fallida")]
    CompileFailed {
        stdout: String,
        stderr: String,
        diagnostics: Vec<Diagnostic>,
    },
    #[error("carga fallida")]
    UploadFailed { stdout: String, stderr: String },
    #[error("operación cancelada")]
    Cancelled,
    #[error("job no encontrado: {0}")]
    JobNotFound(String),
    #[error("sketch no encontrado: {0}")]
    SketchNotFound(String),
    #[error("ruta fuera del área permitida")]
    PathNotAllowed,
    #[error("error de entrada/salida: {0}")]
    Io(String),
    #[error("error de red: {0}")]
    Network(String),
    #[error("configuración inválida: {0}")]
    InvalidConfig(String),
    #[error("error interno: {0}")]
    Internal(String),
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Internal(format!("JSON inválido: {value}"))
    }
}

impl From<regex::Error> for AppError {
    fn from(value: regex::Error) -> Self {
        AppError::Internal(format!("regex inválida: {value}"))
    }
}

impl From<which::Error> for AppError {
    fn from(_: which::Error) -> Self {
        AppError::CliNotFound
    }
}

impl From<tauri::Error> for AppError {
    fn from(value: tauri::Error) -> Self {
        AppError::Internal(value.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        AppError::Internal("tiempo de espera agotado".to_string())
    }
}
