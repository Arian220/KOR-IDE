use tauri::State;

use crate::models::{AppConfig, AppError};
use crate::state::AppState;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, AppError> {
    Ok(state.config.read().await.clone())
}

#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<AppConfig, AppError> {
    crate::services::config::save(&state.paths, &config)?;
    *state.config.write().await = config.clone();
    Ok(config)
}
