use std::collections::HashMap;
use std::time::Duration;

use regex::Regex;
use serde::Deserialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use crate::infra::process;
use crate::models::{AppError, CoreInfo, JobEvent, JobKind, LibraryInfo};
use crate::services::{arduino, jobs};
use crate::state::AppState;

const CAPTURE_TIMEOUT: Duration = Duration::from_secs(120);

pub async fn list_cores(state: &AppState) -> Result<Vec<CoreInfo>, AppError> {
    let (cli, _) = arduino::resolve_cli(state).await?;
    let args = vec![
        "core".to_string(),
        "list".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];

    let output = process::run_capture(&cli, &args, CAPTURE_TIMEOUT).await?;
    if !output.status.success() {
        return Err(command_error("core list", &output.stderr));
    }

    parse_cores(&output.stdout)
}

pub async fn install_core(
    state: &AppState,
    id: String,
    events: Channel<JobEvent>,
) -> Result<CoreInfo, AppError> {
    validate_core_id(&id)?;
    let (cli, _) = arduino::resolve_cli(state).await?;
    let (job_id, token) = jobs::register(state, JobKind::InstallCore)?;

    let args = vec!["core".to_string(), "install".to_string(), id.clone()];
    let outcome = run_install(&cli, &args, &job_id, JobKind::InstallCore, token, &events).await;

    jobs::remove(state, &job_id);

    match outcome {
        Ok(()) => {
            jobs::send(
                &events,
                JobEvent::Finished {
                    job_id: job_id.clone(),
                    success: true,
                },
            );
            let cores = list_cores(state).await?;
            Ok(cores
                .into_iter()
                .find(|core| core.id == id)
                .unwrap_or(CoreInfo {
                    id,
                    installed: true,
                    version: None,
                    latest_version: None,
                }))
        }
        Err(error) => {
            jobs::send(
                &events,
                JobEvent::Failed {
                    job_id: job_id.clone(),
                    error: error.clone(),
                },
            );
            Err(error)
        }
    }
}

pub async fn uninstall_core(state: &AppState, id: String) -> Result<bool, AppError> {
    validate_core_id(&id)?;
    let (cli, _) = arduino::resolve_cli(state).await?;
    let (job_id, _token) = jobs::register(state, JobKind::InstallCore)?;

    let args = vec!["core".to_string(), "uninstall".to_string(), id.clone()];
    let output = process::run_capture(&cli, &args, CAPTURE_TIMEOUT).await;
    jobs::remove(state, &job_id);

    let output = output?;
    if !output.status.success() {
        return Err(command_error("core uninstall", &output.stderr));
    }

    Ok(true)
}

pub async fn list_libraries(
    state: &AppState,
    query: Option<String>,
) -> Result<Vec<LibraryInfo>, AppError> {
    let (cli, _) = arduino::resolve_cli(state).await?;

    let query = query
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    match query {
        Some(query) => {
            validate_library_query(&query)?;
            let installed = installed_library_index(&cli).await;
            let args = vec![
                "lib".to_string(),
                "search".to_string(),
                query,
                "--format".to_string(),
                "json".to_string(),
            ];
            let output = process::run_capture(&cli, &args, CAPTURE_TIMEOUT).await?;
            if !output.status.success() {
                return Err(command_error("lib search", &output.stderr));
            }
            parse_library_search(&output.stdout, &installed)
        }
        None => {
            let args = vec![
                "lib".to_string(),
                "list".to_string(),
                "--format".to_string(),
                "json".to_string(),
            ];
            let output = process::run_capture(&cli, &args, CAPTURE_TIMEOUT).await?;
            if !output.status.success() {
                return Err(command_error("lib list", &output.stderr));
            }
            parse_installed_libraries(&output.stdout)
        }
    }
}

pub async fn install_library(
    state: &AppState,
    name: String,
    version: Option<String>,
    events: Channel<JobEvent>,
) -> Result<LibraryInfo, AppError> {
    validate_library_name(&name)?;
    if let Some(version) = &version {
        validate_library_version(version)?;
    }

    let (cli, _) = arduino::resolve_cli(state).await?;
    let (job_id, token) = jobs::register(state, JobKind::InstallLibrary)?;

    let target = match &version {
        Some(version) => format!("{name}@{version}"),
        None => name.clone(),
    };
    let args = vec!["lib".to_string(), "install".to_string(), target];
    let outcome = run_install(
        &cli,
        &args,
        &job_id,
        JobKind::InstallLibrary,
        token,
        &events,
    )
    .await;

    jobs::remove(state, &job_id);

    match outcome {
        Ok(()) => {
            jobs::send(
                &events,
                JobEvent::Finished {
                    job_id: job_id.clone(),
                    success: true,
                },
            );
            let installed = list_libraries(state, None).await.unwrap_or_default();
            Ok(installed
                .into_iter()
                .find(|library| library.name.eq_ignore_ascii_case(&name))
                .unwrap_or(LibraryInfo {
                    name,
                    installed_version: version,
                    latest_version: None,
                    author: None,
                    sentence: None,
                }))
        }
        Err(error) => {
            jobs::send(
                &events,
                JobEvent::Failed {
                    job_id: job_id.clone(),
                    error: error.clone(),
                },
            );
            Err(error)
        }
    }
}

pub async fn uninstall_library(state: &AppState, name: String) -> Result<bool, AppError> {
    validate_library_name(&name)?;
    let (cli, _) = arduino::resolve_cli(state).await?;
    let (job_id, _token) = jobs::register(state, JobKind::InstallLibrary)?;

    let args = vec!["lib".to_string(), "uninstall".to_string(), name];
    let output = process::run_capture(&cli, &args, CAPTURE_TIMEOUT).await;
    jobs::remove(state, &job_id);

    let output = output?;
    if !output.status.success() {
        return Err(command_error("lib uninstall", &output.stderr));
    }

    Ok(true)
}

async fn run_install(
    cli: &std::path::Path,
    args: &[String],
    job_id: &str,
    kind: JobKind,
    token: CancellationToken,
    events: &Channel<JobEvent>,
) -> Result<(), AppError> {
    jobs::send(
        events,
        JobEvent::Started {
            job_id: job_id.to_string(),
            kind,
        },
    );
    jobs::send(
        events,
        JobEvent::Progress {
            job_id: job_id.to_string(),
            stage: "instalando".to_string(),
            percent: Some(5),
        },
    );

    let streamed = process::run_streaming(cli, args, token, job_id, events.clone()).await?;
    if !streamed.status.success() {
        let stderr = streamed.stderr.trim().to_string();
        let stdout = streamed.stdout.trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        return Err(AppError::Internal(format!(
            "la instalación falló: {detail}"
        )));
    }

    jobs::send(
        events,
        JobEvent::Progress {
            job_id: job_id.to_string(),
            stage: "completado".to_string(),
            percent: Some(100),
        },
    );

    Ok(())
}

async fn installed_library_index(cli: &std::path::Path) -> HashMap<String, LibraryInfo> {
    let args = vec![
        "lib".to_string(),
        "list".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];

    let Ok(output) = process::run_capture(cli, &args, CAPTURE_TIMEOUT).await else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }

    parse_installed_libraries(&output.stdout)
        .unwrap_or_default()
        .into_iter()
        .map(|library| (library.name.clone(), library))
        .collect()
}

pub fn validate_core_id(value: &str) -> Result<(), AppError> {
    let re = Regex::new(r"^[A-Za-z0-9_-]{1,64}:[A-Za-z0-9_-]{1,64}$")?;
    if re.is_match(value) {
        Ok(())
    } else {
        Err(AppError::InvalidConfig(format!("core inválido: {value}")))
    }
}

pub fn validate_library_name(value: &str) -> Result<(), AppError> {
    let re = Regex::new(r"^[A-Za-z0-9][A-Za-z0-9 ._+@-]{0,127}$")?;
    if re.is_match(value) && !value.starts_with('-') {
        Ok(())
    } else {
        Err(AppError::InvalidConfig(format!(
            "librería inválida: {value}"
        )))
    }
}

fn validate_library_version(value: &str) -> Result<(), AppError> {
    let re = Regex::new(r"^[A-Za-z0-9][A-Za-z0-9.+_-]{0,63}$")?;
    if re.is_match(value) {
        Ok(())
    } else {
        Err(AppError::InvalidConfig(format!(
            "versión de librería inválida: {value}"
        )))
    }
}

fn validate_library_query(value: &str) -> Result<(), AppError> {
    if value.len() > 128 || value.starts_with('-') {
        return Err(AppError::InvalidConfig(
            "búsqueda de librería inválida".to_string(),
        ));
    }
    Ok(())
}

fn command_error(command: &str, stderr: &[u8]) -> AppError {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if stderr.is_empty() {
        AppError::Internal(format!("{command} falló"))
    } else {
        AppError::Internal(format!("{command} falló: {stderr}"))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct CoreListResponse {
    #[serde(default)]
    platforms: Vec<CoreEntry>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct CoreEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    installed_version: Option<String>,
    #[serde(default)]
    latest_version: Option<String>,
}

fn parse_cores(bytes: &[u8]) -> Result<Vec<CoreInfo>, AppError> {
    let parsed: CoreListResponse = serde_json::from_slice(bytes)
        .map_err(|err| AppError::Internal(format!("JSON inválido de core list: {err}")))?;

    Ok(parsed
        .platforms
        .into_iter()
        .filter(|core| !core.id.is_empty())
        .map(|core| CoreInfo {
            id: core.id,
            installed: core.installed_version.is_some(),
            version: core.installed_version,
            latest_version: core.latest_version,
        })
        .collect())
}

#[derive(Clone, Debug, Default, Deserialize)]
struct LibrarySearchResponse {
    #[serde(default)]
    libraries: Vec<LibrarySearchEntry>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct LibrarySearchEntry {
    #[serde(default)]
    name: String,
    #[serde(default)]
    latest: Option<LibraryRelease>,
    #[serde(default)]
    releases: HashMap<String, LibraryRelease>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct LibraryRelease {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    sentence: Option<String>,
}

fn parse_library_search(
    bytes: &[u8],
    installed: &HashMap<String, LibraryInfo>,
) -> Result<Vec<LibraryInfo>, AppError> {
    let parsed: LibrarySearchResponse = serde_json::from_slice(bytes)
        .map_err(|err| AppError::Internal(format!("JSON inválido de lib search: {err}")))?;

    Ok(parsed
        .libraries
        .into_iter()
        .filter(|library| !library.name.is_empty())
        .map(|library| {
            let latest = library.latest.unwrap_or_else(|| {
                library
                    .releases
                    .values()
                    .next()
                    .cloned()
                    .unwrap_or_default()
            });
            let latest_version = latest.version.or_else(|| {
                library
                    .releases
                    .iter()
                    .max_by(|left, right| left.0.cmp(right.0))
                    .map(|(version, _)| version.clone())
            });
            let installed_version = installed
                .get(&library.name)
                .and_then(|entry| entry.installed_version.clone());

            LibraryInfo {
                name: library.name,
                installed_version,
                latest_version,
                author: latest.author,
                sentence: latest.sentence,
            }
        })
        .collect())
}

pub fn parse_installed_libraries(bytes: &[u8]) -> Result<Vec<LibraryInfo>, AppError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|err| AppError::Internal(format!("JSON inválido de lib list: {err}")))?;

    let entries = value
        .get("installed_libraries")
        .and_then(|entries| entries.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let library = entry.get("library").unwrap_or(&entry);
            let name = field_str(library, "name")?;
            Some(LibraryInfo {
                name,
                installed_version: field_str(library, "version"),
                latest_version: None,
                author: field_str(library, "author"),
                sentence: field_str(library, "sentence"),
            })
        })
        .collect())
}

fn field_str(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|field| field.as_str())
        .map(|field| field.to_string())
        .filter(|field| !field.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_core_list() {
        let json = r#"{
            "platforms": [
                {
                    "id": "arduino:avr",
                    "installed_version": "1.8.8",
                    "latest_version": "1.8.8"
                },
                {
                    "id": "esp32:esp32",
                    "latest_version": "3.0.7"
                }
            ]
        }"#;

        let cores = parse_cores(json.as_bytes()).unwrap();
        assert_eq!(cores.len(), 2);
        assert_eq!(cores[0].id, "arduino:avr");
        assert!(cores[0].installed);
        assert_eq!(cores[0].version.as_deref(), Some("1.8.8"));
        assert_eq!(cores[0].latest_version.as_deref(), Some("1.8.8"));
        assert!(!cores[1].installed);
        assert_eq!(cores[1].version, None);
    }

    #[test]
    fn parsea_lib_search_y_mezcla_instaladas() {
        let json = r#"{
            "libraries": [
                {
                    "name": "Servo",
                    "latest": {
                        "version": "1.3.0",
                        "author": "Michael Margolis, Arduino",
                        "sentence": "Controla servomotores."
                    },
                    "releases": {
                        "1.0.0": {"version": "1.0.0"},
                        "1.3.0": {"version": "1.3.0"}
                    }
                },
                {
                    "name": "Otra",
                    "releases": {
                        "2.1.0": {"version": "2.1.0", "author": "Autor"}
                    }
                }
            ]
        }"#;

        let mut installed = HashMap::new();
        installed.insert(
            "Servo".to_string(),
            LibraryInfo {
                name: "Servo".to_string(),
                installed_version: Some("1.1.8".to_string()),
                latest_version: None,
                author: None,
                sentence: None,
            },
        );

        let libraries = parse_library_search(json.as_bytes(), &installed).unwrap();
        assert_eq!(libraries.len(), 2);
        assert_eq!(libraries[0].name, "Servo");
        assert_eq!(libraries[0].installed_version.as_deref(), Some("1.1.8"));
        assert_eq!(libraries[0].latest_version.as_deref(), Some("1.3.0"));
        assert_eq!(
            libraries[0].author.as_deref(),
            Some("Michael Margolis, Arduino")
        );
        assert_eq!(libraries[1].name, "Otra");
        assert_eq!(libraries[1].installed_version, None);
        assert_eq!(libraries[1].latest_version.as_deref(), Some("2.1.0"));
    }

    #[test]
    fn parsea_lib_list_con_y_sin_envoltorio() {
        let json = r#"{
            "installed_libraries": [
                {
                    "library": {
                        "name": "Servo",
                        "version": "1.1.8",
                        "author": "Arduino",
                        "sentence": "Servos."
                    }
                },
                {
                    "name": "SD",
                    "version": "1.2.4"
                }
            ]
        }"#;

        let libraries = parse_installed_libraries(json.as_bytes()).unwrap();
        assert_eq!(libraries.len(), 2);
        assert_eq!(libraries[0].name, "Servo");
        assert_eq!(libraries[0].installed_version.as_deref(), Some("1.1.8"));
        assert_eq!(libraries[1].name, "SD");
        assert_eq!(libraries[1].author, None);
    }

    #[test]
    fn valida_identificadores() {
        assert!(validate_core_id("arduino:avr").is_ok());
        assert!(validate_core_id("esp32:esp32").is_ok());
        assert!(validate_core_id("arduino").is_err());
        assert!(validate_core_id("arduino:avr; rm -rf /").is_err());

        assert!(validate_library_name("Servo").is_ok());
        assert!(validate_library_name("Adafruit GFX Library").is_ok());
        assert!(validate_library_name("-peligro").is_err());
        assert!(validate_library_name("nombre;rm").is_err());

        assert!(validate_library_version("1.2.3").is_ok());
        assert!(validate_library_version("1.2.3-beta.1").is_ok());
        assert!(validate_library_version("1.2.3;rm").is_err());
    }

    async fn real_state() -> (tempfile::TempDir, AppState) {
        use crate::models::AppConfig;
        use crate::state::AppPaths;

        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path().to_path_buf());
        paths.ensure().unwrap();
        (dir, AppState::new(paths, AppConfig::default()))
    }

    #[tokio::test]
    #[ignore = "requiere arduino-cli instalado en PATH"]
    async fn lista_cores_real() {
        let (_dir, state) = real_state().await;
        let cores = list_cores(&state).await.unwrap();
        assert!(cores.iter().any(|core| core.id == "arduino:avr"));
    }

    #[tokio::test]
    #[ignore = "requiere arduino-cli instalado en PATH"]
    async fn lista_librerias_real() {
        let (_dir, state) = real_state().await;
        let installed = list_libraries(&state, None).await.unwrap();
        assert!(installed.is_empty() || !installed[0].name.is_empty());

        let results = list_libraries(&state, Some("Servo".to_string()))
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|library| library.name == "Servo"));
    }
}
