use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::models::AppError;

const MAIN_WINDOW_LABEL: &str = "main";
const SPLASH_WINDOW_LABEL: &str = "splash";
const MAIN_WIDTH: f64 = 1200.0;
const MAIN_HEIGHT: f64 = 760.0;
const MAIN_MIN_WIDTH: f64 = 900.0;
const MAIN_MIN_HEIGHT: f64 = 600.0;

/// Crea la ventana principal (Hub/IDE) y cierra la ventana de carga.
///
/// En Wayland el compositor ignora `set_position`/`center`, así que redimensionar
/// la ventana del splash la deja anclada arriba a la izquierda. Crear una ventana
/// nueva permite que el compositor la coloque centrada con su tamaño definitivo.
#[tauri::command]
pub async fn open_main_window(app: AppHandle) -> Result<(), AppError> {
    if let Some(existing) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        existing
            .show()
            .map_err(|err| AppError::Internal(err.to_string()))?;
        existing
            .set_focus()
            .map_err(|err| AppError::Internal(err.to_string()))?;
    } else {
        WebviewWindowBuilder::new(
            &app,
            MAIN_WINDOW_LABEL,
            WebviewUrl::App("index.html#/hub".into()),
        )
        .title("KOR")
        .inner_size(MAIN_WIDTH, MAIN_HEIGHT)
        .min_inner_size(MAIN_MIN_WIDTH, MAIN_MIN_HEIGHT)
        .decorations(false)
        .center()
        .build()
        .map_err(|err| AppError::Internal(err.to_string()))?;
    }

    if let Some(splash) = app.get_webview_window(SPLASH_WINDOW_LABEL) {
        let _ = splash.close();
    }

    Ok(())
}