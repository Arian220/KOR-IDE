mod commands;
mod infra;
mod models;
mod services;
mod state;

use tauri::Manager;

use crate::infra::logging;
use crate::state::{AppPaths, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .map_err(|err| format!("no se pudo obtener app_data: {err}"))?;

            let paths = AppPaths::new(app_data);
            paths.ensure()?;

            let config = services::config::load(&paths)?;
            let log_guard = logging::init(&paths, &config)?;

            app.manage(log_guard);
            app.manage(AppState::new(paths, config));

            tracing::info!("KOR iniciado");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::boards::detect_boards,
            commands::compile::compile,
            commands::compile::cancel_job,
            commands::upload::upload,
            commands::config::get_config,
            commands::config::save_config,
            commands::cli::get_cli_status,
            commands::cli::set_cli_path,
            commands::cli::install_cli,
            commands::catalog::list_cores,
            commands::catalog::install_core,
            commands::catalog::uninstall_core,
            commands::catalog::list_libraries,
            commands::catalog::install_library,
            commands::catalog::uninstall_library,
            commands::sketches::list_sketches,
            commands::sketches::read_sketch,
            commands::sketches::save_sketch,
            commands::sketches::create_sketch,
            commands::sketches::rename_sketch,
            commands::sketches::delete_sketch,
            commands::sketches::export_sketch,
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar KOR");
}
