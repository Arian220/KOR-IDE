use tauri::State;

use crate::models::{AppError, CliStatus};
use crate::services::arduino;
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
