use std::fs;

use crate::infra::fs::{read_json, write_json};
use crate::models::{AppConfig, AppError};
use crate::state::AppPaths;

const LOG_LEVELS: [&str; 5] = ["error", "warn", "info", "debug", "trace"];

pub fn load(paths: &AppPaths) -> Result<AppConfig, AppError> {
    if !paths.config_file.exists() {
        let config = AppConfig::default();
        save(paths, &config)?;
        return Ok(config);
    }

    let parsed = read_json::<AppConfig>(&paths.config_file);
    let config = match parsed {
        Ok(config) => config,
        Err(err) => {
            tracing::warn!("configuración ilegible, se regenera: {err}");
            return reset(paths);
        }
    };

    match validate(&config) {
        Ok(()) => Ok(config),
        Err(err) => {
            tracing::warn!("configuración inválida, se regenera: {err}");
            reset(paths)
        }
    }
}

pub fn save(paths: &AppPaths, config: &AppConfig) -> Result<(), AppError> {
    validate(config)?;
    write_json(&paths.config_file, config)
}

pub fn validate(config: &AppConfig) -> Result<(), AppError> {
    if !(200..=5000).contains(&config.autosave_delay_ms) {
        return Err(AppError::InvalidConfig(
            "autosave_delay_ms debe estar entre 200 y 5000".to_string(),
        ));
    }

    if !(1..=200).contains(&config.max_recent_sketches) {
        return Err(AppError::InvalidConfig(
            "max_recent_sketches debe estar entre 1 y 200".to_string(),
        ));
    }

    if !LOG_LEVELS.contains(&config.log_level.as_str()) {
        return Err(AppError::InvalidConfig(format!(
            "log_level inválido: {}",
            config.log_level
        )));
    }

    if let Some(path) = &config.cli_path {
        if path.trim().is_empty() {
            return Err(AppError::InvalidConfig(
                "cli_path no puede estar vacío".to_string(),
            ));
        }
    }

    Ok(())
}

fn reset(paths: &AppPaths) -> Result<AppConfig, AppError> {
    let backup = paths.config_file.with_file_name("config.json.bad");
    let _ = fs::remove_file(&backup);
    let _ = fs::rename(&paths.config_file, &backup);
    let config = AppConfig::default();
    save(paths, &config)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> (tempfile::TempDir, AppPaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path().to_path_buf());
        paths.ensure().unwrap();
        (dir, paths)
    }

    #[test]
    fn crea_configuracion_por_defecto() {
        let (_dir, paths) = paths();
        let config = load(&paths).unwrap();
        assert_eq!(config.autosave_delay_ms, 800);
        assert!(paths.config_file.is_file());
    }

    #[test]
    fn rechaza_delay_fuera_de_rango() {
        let config = AppConfig {
            autosave_delay_ms: 10,
            ..Default::default()
        };
        assert!(validate(&config).is_err());
    }

    #[test]
    fn regenera_configuracion_corrupta() {
        let (_dir, paths) = paths();
        fs::write(&paths.config_file, "{ no es json").unwrap();
        let config = load(&paths).unwrap();
        assert!(config.autosave);
        assert!(paths
            .config_file
            .with_file_name("config.json.bad")
            .is_file());
        assert_eq!(config, load(&paths).unwrap());
    }
}
