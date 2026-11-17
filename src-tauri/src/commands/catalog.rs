use tauri::ipc::Channel;
use tauri::State;

use crate::models::{AppError, CoreInfo, JobEvent, LibraryInfo};
use crate::services::catalog;
use crate::state::AppState;

#[tauri::command]
pub async fn list_cores(state: State<'_, AppState>) -> Result<Vec<CoreInfo>, AppError> {
    catalog::list_cores(state.inner()).await
}

#[tauri::command]
pub async fn install_core(
    state: State<'_, AppState>,
    id: String,
    events: Channel<JobEvent>,
) -> Result<CoreInfo, AppError> {
    catalog::install_core(state.inner(), id, events).await
}

#[tauri::command]
pub async fn uninstall_core(state: State<'_, AppState>, id: String) -> Result<bool, AppError> {
    catalog::uninstall_core(state.inner(), id).await
}

#[tauri::command]
pub async fn list_libraries(
    state: State<'_, AppState>,
    query: Option<String>,
) -> Result<Vec<LibraryInfo>, AppError> {
    catalog::list_libraries(state.inner(), query).await
}

#[tauri::command]
pub async fn install_library(
    state: State<'_, AppState>,
    name: String,
    version: Option<String>,
    events: Channel<JobEvent>,
) -> Result<LibraryInfo, AppError> {
    catalog::install_library(state.inner(), name, version, events).await
}

#[tauri::command]
pub async fn uninstall_library(state: State<'_, AppState>, name: String) -> Result<bool, AppError> {
    catalog::uninstall_library(state.inner(), name).await
}
