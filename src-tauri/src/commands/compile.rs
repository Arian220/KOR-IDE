use tauri::ipc::Channel;
use tauri::State;

use crate::models::{AppError, CompileRequest, CompileResult, JobEvent};
use crate::services::{arduino, jobs};
use crate::state::AppState;

#[tauri::command]
pub async fn compile(
    state: State<'_, AppState>,
    request: CompileRequest,
    events: Channel<JobEvent>,
) -> Result<CompileResult, AppError> {
    arduino::compile(state.inner(), request, events).await
}

#[tauri::command]
pub fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<bool, AppError> {
    jobs::cancel(state.inner(), &job_id)
}
