use tauri::ipc::Channel;
use tauri::State;

use crate::models::{AppError, CliInfo, CliStatus, JobEvent};
use crate::services::{arduino, cli_installer};
use crate::state::AppState;

#[tauri::command]
pub async fn get_cli_status(state: State<'_, AppState>) -> Result<CliStatus, AppError> {
    Ok(arduino::get_cli_status(state.inner()).await)
}

#[tauri::command]
pub async fn set_cli_path(
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<CliStatus, AppError> {
    arduino::set_cli_path(state.inner(), path).await
}

#[tauri::command]
pub async fn install_cli(
    state: State<'_, AppState>,
    events: Channel<JobEvent>,
) -> Result<CliInfo, AppError> {
    cli_installer::install(state.inner(), events).await
}
