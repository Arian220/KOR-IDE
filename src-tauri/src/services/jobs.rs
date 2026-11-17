use std::time::Instant;

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::models::{AppError, JobEvent, JobKind};
use crate::state::{AppState, JobHandle};

pub fn is_exclusive(kind: JobKind) -> bool {
    matches!(
        kind,
        JobKind::Compile
            | JobKind::Upload
            | JobKind::InstallCore
            | JobKind::InstallLibrary
            | JobKind::DownloadCli
    )
}

pub fn register(state: &AppState, kind: JobKind) -> Result<(String, CancellationToken), AppError> {
    let mut jobs = state
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("estado de jobs bloqueado".to_string()))?;

    if is_exclusive(kind) && jobs.values().any(|job| is_exclusive(job.kind)) {
        return Err(AppError::Internal(
            "ya hay una operación de arduino-cli en curso".to_string(),
        ));
    }

    let job_id = Uuid::new_v4().to_string();
    let token = CancellationToken::new();
    jobs.insert(
        job_id.clone(),
        JobHandle {
            kind,
            cancel: token.clone(),
            started_at: Instant::now(),
        },
    );

    Ok((job_id, token))
}

pub fn remove(state: &AppState, job_id: &str) {
    if let Ok(mut jobs) = state.jobs.lock() {
        jobs.remove(job_id);
    }
}

pub fn cancel(state: &AppState, job_id: &str) -> Result<bool, AppError> {
    let jobs = state
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("estado de jobs bloqueado".to_string()))?;

    if let Some(job) = jobs.get(job_id) {
        job.cancel.cancel();
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn send(events: &Channel<JobEvent>, event: JobEvent) {
    let _ = events.send(event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AppConfig;
    use crate::state::AppPaths;

    fn state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path().to_path_buf());
        paths.ensure().unwrap();
        (dir, AppState::new(paths, AppConfig::default()))
    }

    #[test]
    fn bloquea_dos_operaciones_exclusivas() {
        let (_dir, state) = state();
        let (job_id, _token) = register(&state, JobKind::Compile).unwrap();
        assert!(register(&state, JobKind::DownloadCli).is_err());
        remove(&state, &job_id);
        assert!(register(&state, JobKind::DownloadCli).is_ok());
    }

    #[test]
    fn cancela_jobs_existentes() {
        let (_dir, state) = state();
        let (job_id, token) = register(&state, JobKind::Compile).unwrap();
        assert!(cancel(&state, &job_id).unwrap());
        assert!(token.is_cancelled());
        assert!(!cancel(&state, "no-existe").unwrap());
    }
}
