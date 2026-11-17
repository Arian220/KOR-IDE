pub mod board;
pub mod catalog;
pub mod compile;
pub mod config;
pub mod error;
pub mod job;
pub mod sketch;
pub mod upload;

pub use board::{BoardInfo, BoardListResponse};
pub use catalog::{CoreInfo, LibraryInfo};
pub use compile::{CompileRequest, CompileResult, Diagnostic, MemoryUsage, Severity};
pub use config::{AppConfig, CliInfo, CliSource, CliStatus};
pub use error::AppError;
pub use job::{JobEvent, JobKind, OutputStream};
pub use sketch::{Sketch, SketchSummary};
pub use upload::{UploadRequest, UploadResult};
