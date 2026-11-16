use tauri::State;

use crate::models::{AppError, Sketch, SketchSummary};
use crate::services::sketches;
use crate::state::AppState;

#[tauri::command]
pub fn list_sketches(state: State<'_, AppState>) -> Result<Vec<SketchSummary>, AppError> {
    sketches::list(&state.paths)
}

#[tauri::command]
pub fn read_sketch(state: State<'_, AppState>, id: String) -> Result<Sketch, AppError> {
    sketches::read(&state.paths, &id)
}

#[tauri::command]
pub fn save_sketch(state: State<'_, AppState>, sketch: Sketch) -> Result<SketchSummary, AppError> {
    sketches::save(&state.paths, &sketch)
}

#[tauri::command]
pub fn create_sketch(state: State<'_, AppState>, name: String) -> Result<Sketch, AppError> {
    sketches::create(&state.paths, &name)
}

#[tauri::command]
pub fn rename_sketch(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<SketchSummary, AppError> {
    sketches::rename(&state.paths, &id, &name)
}

#[tauri::command]
pub fn delete_sketch(state: State<'_, AppState>, id: String) -> Result<bool, AppError> {
    sketches::delete(&state.paths, &id)
}

#[tauri::command]
pub fn export_sketch(
    state: State<'_, AppState>,
    id: String,
    target_path: String,
) -> Result<String, AppError> {
    sketches::export(&state.paths, &id, &target_path)
}
