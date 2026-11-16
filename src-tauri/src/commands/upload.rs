use tauri::ipc::Channel;
use tauri::State;

use crate::models::{AppError, JobEvent, UploadRequest, UploadResult};
use crate::services::arduino;
use crate::state::AppState;

#[tauri::command]
pub async fn upload(
    state: State<'_, AppState>,
    request: UploadRequest,
    events: Channel<JobEvent>,
) -> Result<UploadResult, AppError> {
    arduino::upload(state.inner(), request, events).await
}
