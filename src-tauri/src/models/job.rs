use serde::{Deserialize, Serialize};

use super::error::AppError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Compile,
    Upload,
    InstallCore,
    InstallLibrary,
    DownloadCli,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum JobEvent {
    Started {
        job_id: String,
        kind: JobKind,
    },
    Progress {
        job_id: String,
        stage: String,
        percent: Option<u8>,
    },
    Output {
        job_id: String,
        stream: OutputStream,
        chunk: String,
    },
    Finished {
        job_id: String,
        success: bool,
    },
    Failed {
        job_id: String,
        error: AppError,
    },
}
