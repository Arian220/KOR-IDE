use tauri::State;

use crate::models::{AppError, BoardInfo};
use crate::services::arduino;
use crate::state::AppState;

#[tauri::command]
pub async fn detect_boards(state: State<'_, AppState>) -> Result<Vec<BoardInfo>, AppError> {
    arduino::detect_boards(state.inner()).await
}
