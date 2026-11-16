use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use regex::Regex;
use serde::Deserialize;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::infra::process;
use crate::models::{
    AppError, BoardInfo, BoardListResponse, CliInfo, CliSource, CliStatus, CompileRequest,
    CompileResult, Diagnostic, JobEvent, JobKind, MemoryUsage, Severity, UploadRequest,
    UploadResult,
};
use crate::state::{AppState, JobHandle};

pub const REQUIRED_VERSION: &str = "1.6.0";

const CAPTURE_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn resolve_cli(state: &AppState) -> Result<(PathBuf, CliSource), AppError> {
    let manual = {
        let config = state.config.read().await;
        config.cli_path.clone()
    };

    if let Some(path) = manual {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok((path, CliSource::Manual));
        }
        return Err(AppError::CliNotFound);
    }

    if state.paths.cli_bin.is_file() {
        return Ok((state.paths.cli_bin.clone(), CliSource::Managed));
    }

    if let Some(found) = process::find_executable("arduino-cli") {
        return Ok((found, CliSource::Path));
    }

    Err(AppError::CliNotFound)
}

pub async fn get_cli_status(state: &AppState) -> CliStatus {
    let status = match resolve_cli(state).await {
        Ok((path, source)) => {
            let version = match version_of(&path).await {
                Ok(version) => version,
                Err(err) => {
                    tracing::warn!("no se pudo leer la versión de arduino-cli: {err}");
                    "desconocida".to_string()
                }
            };
            let info = CliInfo {
                path: path.display().to_string(),
                version,
                source,
            };
            *state.cli.write().await = Some(info.clone());
            CliStatus {
                available: true,
                info: Some(info),
                required_version: REQUIRED_VERSION.to_string(),
            }
        }
        Err(_) => {
            *state.cli.write().await = None;
            CliStatus {
                available: false,
                info: None,
                required_version: REQUIRED_VERSION.to_string(),
            }
        }
    };

    tracing::info!("estado de arduino-cli: disponible={}", status.available);
    status
}

pub async fn set_cli_path(state: &AppState, path: Option<String>) -> Result<CliStatus, AppError> {
    if let Some(value) = &path {
        let candidate = PathBuf::from(value);
        if !candidate.is_file() {
            return Err(AppError::CliNotFound);
        }
    }

    {
        let mut config = state.config.write().await;
        config.cli_path = path;
        crate::services::config::save(&state.paths, &config)?;
    }

    *state.cli.write().await = None;
    Ok(get_cli_status(state).await)
}

async fn version_of(cli: &Path) -> Result<String, AppError> {
    let output =
        process::run_capture(cli, &["version".to_string()], Duration::from_secs(10)).await?;

    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let re = Regex::new(r"Version:\s*([0-9]+(?:\.[0-9]+)*)")?;
    if let Some(captures) = re.captures(&text) {
        return Ok(captures[1].to_string());
    }

    if let Some(line) = text.lines().map(str::trim).find(|line| !line.is_empty()) {
        return Ok(line.to_string());
    }

    Err(AppError::Internal(
        "no se pudo leer la versión de arduino-cli".to_string(),
    ))
}

pub async fn detect_boards(state: &AppState) -> Result<Vec<BoardInfo>, AppError> {
    let (cli, _) = resolve_cli(state).await?;
    let args = vec![
        "board".to_string(),
        "list".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];

    let output = process::run_capture(&cli, &args, CAPTURE_TIMEOUT).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AppError::Internal(if stderr.is_empty() {
            "board list falló".to_string()
        } else {
            stderr
        }));
    }

    let parsed: BoardListResponse = serde_json::from_slice(&output.stdout)
        .map_err(|err| AppError::Internal(format!("JSON inválido de board list: {err}")))?;

    Ok(parsed.into_board_infos())
}

pub async fn compile(
    state: &AppState,
    request: CompileRequest,
    events: Channel<JobEvent>,
) -> Result<CompileResult, AppError> {
    validate_compile_request(state, &request)?;
    let (cli, _) = resolve_cli(state).await?;
    let (job_id, token) = register_job(state, JobKind::Compile)?;

    let outcome = run_compile(&cli, &request, token, &job_id, &events).await;
    remove_job(state, &job_id);

    match &outcome {
        Ok(_) => send_event(
            &events,
            JobEvent::Finished {
                job_id: job_id.clone(),
                success: true,
            },
        ),
        Err(err) => send_event(
            &events,
            JobEvent::Failed {
                job_id: job_id.clone(),
                error: err.clone(),
            },
        ),
    }

    outcome
}

async fn run_compile(
    cli: &Path,
    request: &CompileRequest,
    token: CancellationToken,
    job_id: &str,
    events: &Channel<JobEvent>,
) -> Result<CompileResult, AppError> {
    send_event(
        events,
        JobEvent::Started {
            job_id: job_id.to_string(),
            kind: JobKind::Compile,
        },
    );
    send_event(
        events,
        JobEvent::Progress {
            job_id: job_id.to_string(),
            stage: "compilando".to_string(),
            percent: Some(5),
        },
    );

    let args = build_compile_args(request);
    let streamed = process::run_streaming(cli, &args, token, job_id, events.clone()).await?;

    let json = parse_compile_json(&streamed.stdout);
    tracing::debug!("salida cruda de compilación: {}", streamed.stdout);

    let (stdout, stderr) = match &json {
        Some(value) if !value.compiler_out.is_empty() || !value.compiler_err.is_empty() => {
            (value.compiler_out.clone(), value.compiler_err.clone())
        }
        _ => (streamed.stdout.clone(), streamed.stderr.clone()),
    };

    let combined = format!("{stdout}\n{stderr}");
    let diagnostics = parse_diagnostics(&combined, json.as_ref());
    let memory = parse_memory(&combined, json.as_ref());
    let build_dir = json
        .as_ref()
        .and_then(|value| value.builder_result.as_ref())
        .and_then(|builder| builder.build_path.clone());
    let success = streamed.status.success()
        && json
            .as_ref()
            .and_then(|value| value.success)
            .unwrap_or(true);

    if success {
        send_event(
            events,
            JobEvent::Progress {
                job_id: job_id.to_string(),
                stage: "completado".to_string(),
                percent: Some(100),
            },
        );
        Ok(CompileResult {
            success: true,
            stdout,
            stderr,
            diagnostics,
            memory,
            build_dir,
        })
    } else {
        Err(AppError::CompileFailed {
            stdout,
            stderr,
            diagnostics,
        })
    }
}

pub async fn upload(
    state: &AppState,
    request: UploadRequest,
    events: Channel<JobEvent>,
) -> Result<UploadResult, AppError> {
    validate_upload_request(state, &request)?;
    let (cli, _) = resolve_cli(state).await?;
    let (job_id, token) = register_job(state, JobKind::Upload)?;

    let outcome = run_upload(&cli, &request, token, &job_id, &events).await;
    remove_job(state, &job_id);

    match &outcome {
        Ok(_) => send_event(
            &events,
            JobEvent::Finished {
                job_id: job_id.clone(),
                success: true,
            },
        ),
        Err(err) => send_event(
            &events,
            JobEvent::Failed {
                job_id: job_id.clone(),
                error: err.clone(),
            },
        ),
    }

    outcome
}

async fn run_upload(
    cli: &Path,
    request: &UploadRequest,
    token: CancellationToken,
    job_id: &str,
    events: &Channel<JobEvent>,
) -> Result<UploadResult, AppError> {
    send_event(
        events,
        JobEvent::Started {
            job_id: job_id.to_string(),
            kind: JobKind::Upload,
        },
    );
    send_event(
        events,
        JobEvent::Progress {
            job_id: job_id.to_string(),
            stage: if request.compile_first {
                "compilando y cargando".to_string()
            } else {
                "cargando".to_string()
            },
            percent: Some(5),
        },
    );

    let args = build_upload_args(request);
    let streamed = process::run_streaming(cli, &args, token, job_id, events.clone()).await?;

    if streamed.status.success() {
        send_event(
            events,
            JobEvent::Progress {
                job_id: job_id.to_string(),
                stage: "completado".to_string(),
                percent: Some(100),
            },
        );
        Ok(UploadResult {
            success: true,
            stdout: streamed.stdout,
            stderr: streamed.stderr,
            port: request.port.clone(),
            fqbn: request.fqbn.clone(),
        })
    } else {
        Err(AppError::UploadFailed {
            stdout: streamed.stdout,
            stderr: streamed.stderr,
        })
    }
}

pub fn cancel_job(state: &AppState, job_id: &str) -> Result<bool, AppError> {
    let jobs = state
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("estado de jobs bloqueado".to_string()))?;

    if let Some(job) = jobs.get(job_id) {
        job.cancel.cancel();
        Ok(true)
    } else {
        Ok(false)
    }
}

fn register_job(state: &AppState, kind: JobKind) -> Result<(String, CancellationToken), AppError> {
    let mut jobs = state
        .jobs
        .lock()
        .map_err(|_| AppError::Internal("estado de jobs bloqueado".to_string()))?;

    if jobs
        .values()
        .any(|job| matches!(job.kind, JobKind::Compile | JobKind::Upload))
    {
        return Err(AppError::Internal(
            "ya hay una compilación o carga en curso".to_string(),
        ));
    }

    let job_id = Uuid::new_v4().to_string();
    let token = CancellationToken::new();
    jobs.insert(
        job_id.clone(),
        JobHandle {
            kind,
            cancel: token.clone(),
            started_at: Instant::now(),
        },
    );

    Ok((job_id, token))
}

fn remove_job(state: &AppState, job_id: &str) {
    if let Ok(mut jobs) = state.jobs.lock() {
        jobs.remove(job_id);
    }
}

fn send_event(events: &Channel<JobEvent>, event: JobEvent) {
    let _ = events.send(event);
}

fn validate_compile_request(state: &AppState, request: &CompileRequest) -> Result<(), AppError> {
    validate_fqbn(&request.fqbn)?;
    validate_sketch_path(state, &request.sketch_path)?;
    validate_warnings(&request.warnings)?;
    validate_build_properties(&request.build_properties)?;
    Ok(())
}

fn validate_upload_request(state: &AppState, request: &UploadRequest) -> Result<(), AppError> {
    validate_fqbn(&request.fqbn)?;
    validate_port(&request.port)?;
    validate_sketch_path(state, &request.sketch_path)?;
    validate_build_properties(&request.build_properties)?;
    validate_programmer(&request.programmer)?;
    validate_baudrate(request.baudrate)?;
    Ok(())
}

pub fn validate_fqbn(value: &str) -> Result<(), AppError> {
    let re =
        Regex::new(r"^[A-Za-z0-9_-]+:[A-Za-z0-9_-]+:[A-Za-z0-9_.-]+(?::[A-Za-z0-9_.,=:-]+)?$")?;

    if re.is_match(value) {
        Ok(())
    } else {
        Err(AppError::InvalidFqbn(value.to_string()))
    }
}

pub fn validate_port(value: &str) -> Result<(), AppError> {
    let pattern = if cfg!(target_os = "windows") {
        r"^COM\d+$"
    } else {
        r"^/dev/(tty|cu)[A-Za-z0-9._-]+$"
    };

    if Regex::new(pattern)?.is_match(value) {
        Ok(())
    } else {
        Err(AppError::InvalidPort(value.to_string()))
    }
}

fn validate_warnings(value: &Option<String>) -> Result<(), AppError> {
    if let Some(level) = value {
        if !["none", "default", "all"].contains(&level.as_str()) {
            return Err(AppError::InvalidConfig(format!(
                "nivel de warnings inválido: {level}"
            )));
        }
    }
    Ok(())
}

fn validate_build_properties(props: &[String]) -> Result<(), AppError> {
    for prop in props {
        if prop.len() > 2048 || !prop.contains('=') || prop.contains('\0') {
            return Err(AppError::InvalidConfig(format!(
                "build_property inválida: {prop}"
            )));
        }
    }
    Ok(())
}

fn validate_programmer(value: &Option<String>) -> Result<(), AppError> {
    if let Some(programmer) = value {
        if !programmer.is_empty() && !Regex::new(r"^[A-Za-z0-9_.:-]+$")?.is_match(programmer) {
            return Err(AppError::InvalidConfig(format!(
                "programador inválido: {programmer}"
            )));
        }
    }
    Ok(())
}

fn validate_baudrate(value: Option<u32>) -> Result<(), AppError> {
    if let Some(baudrate) = value {
        if !(300..=2_000_000).contains(&baudrate) {
            return Err(AppError::InvalidConfig(format!(
                "baudrate inválido: {baudrate}"
            )));
        }
    }
    Ok(())
}

fn validate_sketch_path(state: &AppState, value: &str) -> Result<PathBuf, AppError> {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        let path = state.paths.ensure_inside_sketches(&path)?;
        if !path.exists() {
            return Err(AppError::SketchNotFound(value.to_string()));
        }
        return Ok(path);
    }

    let id_re = Regex::new(r"^[a-f0-9-]{36}$")?;
    if id_re.is_match(value) {
        let dir = state.paths.sketch_dir(value);
        if dir.is_dir() {
            return state.paths.ensure_inside_sketches(&dir);
        }
        return Err(AppError::SketchNotFound(value.to_string()));
    }

    Err(AppError::PathNotAllowed)
}

pub fn build_compile_args(request: &CompileRequest) -> Vec<String> {
    let mut args = vec![
        "compile".to_string(),
        "--fqbn".to_string(),
        request.fqbn.clone(),
        "--format".to_string(),
        "json".to_string(),
    ];

    if request.clean {
        args.push("--clean".to_string());
    }

    let warnings = request.warnings.as_deref().unwrap_or("all");
    args.push("--warnings".to_string());
    args.push(warnings.to_string());

    for prop in &request.build_properties {
        args.push("--build-property".to_string());
        args.push(prop.clone());
    }

    args.push(request.sketch_path.clone());
    args
}

pub fn build_upload_args(request: &UploadRequest) -> Vec<String> {
    let mut args = if request.compile_first {
        vec![
            "compile".to_string(),
            "--fqbn".to_string(),
            request.fqbn.clone(),
            "--port".to_string(),
            request.port.clone(),
            "--upload".to_string(),
        ]
    } else {
        vec![
            "upload".to_string(),
            "--fqbn".to_string(),
            request.fqbn.clone(),
            "--port".to_string(),
            request.port.clone(),
        ]
    };

    if request.verify {
        args.push("--verify".to_string());
    }

    if let Some(programmer) = &request.programmer {
        if !programmer.trim().is_empty() {
            args.push("--programmer".to_string());
            args.push(programmer.clone());
        }
    }

    if let Some(baudrate) = request.baudrate {
        args.push("--baudrate".to_string());
        args.push(baudrate.to_string());
    }

    if request.compile_first {
        for prop in &request.build_properties {
            args.push("--build-property".to_string());
            args.push(prop.clone());
        }
    }

    args.push(request.sketch_path.clone());
    args
}

#[derive(Clone, Debug, Default, Deserialize)]
struct CompileJson {
    #[serde(default, alias = "compilerOut")]
    compiler_out: String,
    #[serde(default, alias = "compilerErr")]
    compiler_err: String,
    #[serde(default, alias = "builderResult")]
    builder_result: Option<BuilderResult>,
    #[serde(default)]
    success: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct BuilderResult {
    #[serde(default, alias = "buildPath")]
    build_path: Option<String>,
    #[serde(default, alias = "executableSectionsSize")]
    executable_sections_size: Vec<SectionSize>,
}

#[derive(Clone, Debug, Deserialize)]
struct SectionSize {
    name: String,
    size: u64,
    #[serde(default, alias = "maxSize")]
    max_size: u64,
}

fn parse_compile_json(stdout: &str) -> Option<CompileJson> {
    let text = stdout.trim();
    if text.is_empty() {
        return None;
    }

    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }

    serde_json::from_str(&text[start..=end]).ok()
}

fn diagnostic_regexes() -> &'static (Regex, Regex, Regex) {
    static RE: OnceLock<(Regex, Regex, Regex)> = OnceLock::new();
    RE.get_or_init(|| {
        (
            Regex::new(
                r"^(?P<file>.*?):(?P<line>\d+):(?P<col>\d+):\s*(?P<sev>fatal error|error|warning|note):\s*(?P<msg>.+?)\s*$",
            )
            .expect("regex de diagnóstico con ubicación"),
            Regex::new(
                r"^(?P<file>.*?):(?P<line>\d+):\s*(?P<sev>fatal error|error|warning|note):\s*(?P<msg>.+?)\s*$",
            )
            .expect("regex de diagnóstico sin columna"),
            Regex::new(r"^(?P<sev>fatal error|error|warning|note):\s*(?P<msg>.+?)\s*$")
                .expect("regex de diagnóstico sin archivo"),
        )
    })
}

fn parse_diagnostics(combined: &str, json: Option<&CompileJson>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = HashSet::new();

    if let Some(json) = json {
        collect_diagnostics(&json.compiler_err, &mut diagnostics, &mut seen);
        collect_diagnostics(&json.compiler_out, &mut diagnostics, &mut seen);
    }
    collect_diagnostics(combined, &mut diagnostics, &mut seen);

    diagnostics.truncate(200);
    diagnostics
}

type DiagnosticKey = (Option<String>, Option<u32>, Option<u32>, Severity, String);

fn collect_diagnostics(text: &str, out: &mut Vec<Diagnostic>, seen: &mut HashSet<DiagnosticKey>) {
    let (with_location, without_column, bare) = diagnostic_regexes();

    for raw_line in text.lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }

        let diagnostic = if let Some(captures) = with_location.captures(line) {
            Some(Diagnostic {
                severity: severity_from(&captures["sev"]),
                message: captures["msg"].trim().to_string(),
                file: Some(captures["file"].trim().to_string()),
                line: captures["line"].parse().ok(),
                column: captures["col"].parse().ok(),
            })
        } else if let Some(captures) = without_column.captures(line) {
            Some(Diagnostic {
                severity: severity_from(&captures["sev"]),
                message: captures["msg"].trim().to_string(),
                file: Some(captures["file"].trim().to_string()),
                line: captures["line"].parse().ok(),
                column: None,
            })
        } else {
            bare.captures(line).map(|captures| Diagnostic {
                severity: severity_from(&captures["sev"]),
                message: captures["msg"].trim().to_string(),
                file: None,
                line: None,
                column: None,
            })
        };

        if let Some(diagnostic) = diagnostic {
            let key = (
                diagnostic.file.clone(),
                diagnostic.line,
                diagnostic.column,
                diagnostic.severity,
                diagnostic.message.clone(),
            );
            if seen.insert(key) {
                out.push(diagnostic);
            }
        }
    }
}

fn severity_from(value: &str) -> Severity {
    match value {
        "warning" => Severity::Warning,
        "note" => Severity::Note,
        _ => Severity::Error,
    }
}

fn parse_memory(combined: &str, json: Option<&CompileJson>) -> Option<MemoryUsage> {
    if let Some(usage) = json.and_then(memory_from_json) {
        return Some(usage);
    }
    memory_from_text(combined)
}

fn memory_from_json(json: &CompileJson) -> Option<MemoryUsage> {
    let builder = json.builder_result.as_ref()?;

    let flash = builder
        .executable_sections_size
        .iter()
        .find(|section| matches_name(&section.name, &["flash", "text"]));
    let sram = builder
        .executable_sections_size
        .iter()
        .find(|section| matches_name(&section.name, &["data", "sram", "bss"]));

    match (flash, sram) {
        (Some(flash), Some(sram)) => Some(build_usage(
            flash.size,
            flash.max_size,
            sram.size,
            sram.max_size,
        )),
        _ => None,
    }
}

fn matches_name(name: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

fn memory_from_text(text: &str) -> Option<MemoryUsage> {
    let flash_re = Regex::new(
        r"(?i)Sketch uses\s+(\d+)\s+bytes\s+\(\d+%\)\s+of program storage space\.\s+Maximum is\s+(\d+)\s+bytes",
    )
    .ok()?;
    let sram_re = Regex::new(
        r"(?is)Global variables use\s+(\d+)\s+bytes\s+\(\d+%\)\s+of dynamic memory.*?Maximum is\s+(\d+)\s+bytes",
    )
    .ok()?;

    let flash = flash_re.captures(text)?;
    let sram = sram_re.captures(text)?;

    Some(build_usage(
        parse_capture(&flash, 1),
        parse_capture(&flash, 2),
        parse_capture(&sram, 1),
        parse_capture(&sram, 2),
    ))
}

fn parse_capture(captures: &regex::Captures<'_>, index: usize) -> u64 {
    captures
        .get(index)
        .and_then(|value| value.as_str().parse().ok())
        .unwrap_or(0)
}

fn build_usage(flash_used: u64, flash_max: u64, sram_used: u64, sram_max: u64) -> MemoryUsage {
    MemoryUsage {
        flash_used,
        flash_max,
        flash_percent: percent(flash_used, flash_max),
        sram_used,
        sram_max,
        sram_percent: percent(sram_used, sram_max),
    }
}

fn percent(used: u64, max: u64) -> f32 {
    if max == 0 {
        0.0
    } else {
        ((used as f64 / max as f64) * 1000.0).round() as f32 / 10.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_request() -> CompileRequest {
        CompileRequest {
            sketch_path: "/tmp/Blink".to_string(),
            fqbn: "arduino:avr:uno".to_string(),
            build_properties: vec!["build.extra_flags=-DKOR".to_string()],
            warnings: Some("all".to_string()),
            clean: true,
        }
    }

    #[test]
    fn construye_argumentos_de_compilacion() {
        let args = build_compile_args(&compile_request());
        assert_eq!(args[0], "compile");
        assert!(args.contains(&"--format".to_string()));
        assert!(args.contains(&"json".to_string()));
        assert!(args.contains(&"--clean".to_string()));
        assert!(args.contains(&"--fqbn".to_string()));
        assert_eq!(args.last().unwrap(), "/tmp/Blink");
    }

    #[test]
    fn rechaza_fqbn_con_caracteres_peligrosos() {
        assert!(validate_fqbn("arduino:avr:uno").is_ok());
        assert!(validate_fqbn("arduino:avr:uno; rm -rf /").is_err());
        assert!(validate_fqbn("arduino:avr").is_err());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn valida_puertos_unix() {
        assert!(validate_port("/dev/ttyACM0").is_ok());
        assert!(validate_port("/dev/cu.usbmodem14201").is_ok());
        assert!(validate_port("/dev/ttyACM0; rm -rf /").is_err());
        assert!(validate_port("COM3").is_err());
    }

    #[test]
    fn parsea_memoria_desde_texto() {
        let text = "Sketch uses 924 bytes (2%) of program storage space. Maximum is 32256 bytes.\nGlobal variables use 9 bytes (0%) of dynamic memory, leaving 2039 bytes for local variables. Maximum is 2048 bytes.";
        let usage = memory_from_text(text).unwrap();
        assert_eq!(usage.flash_used, 924);
        assert_eq!(usage.flash_max, 32256);
        assert_eq!(usage.sram_used, 9);
        assert_eq!(usage.sram_max, 2048);
    }

    #[test]
    fn parsea_memoria_desde_json() {
        let json = r#"{
            "success": true,
            "builder_result": {
                "build_path": "/tmp/build",
                "executable_sections_size": [
                    { "name": "flash", "size": 924, "max_size": 32256 },
                    { "name": "data", "size": 9, "max_size": 2048 }
                ]
            }
        }"#;
        let parsed = parse_compile_json(json).unwrap();
        let usage = memory_from_json(&parsed).unwrap();
        assert_eq!(usage.flash_used, 924);
        assert_eq!(usage.sram_max, 2048);
        assert_eq!(
            parsed.builder_result.unwrap().build_path.as_deref(),
            Some("/tmp/build")
        );
    }

    #[test]
    fn parsea_diagnosticos_del_compilador() {
        let text = "/tmp/Blink/Blink.ino:10:5: error: 'x' was not declared in this scope\n/tmp/Blink/Blink.ino:12:1: warning: unused variable 'y'";
        let diagnostics = parse_diagnostics(text, None);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].severity, Severity::Error);
        assert_eq!(diagnostics[0].line, Some(10));
        assert_eq!(diagnostics[0].column, Some(5));
        assert_eq!(diagnostics[1].severity, Severity::Warning);
    }

    #[test]
    fn construye_argumentos_de_carga_sin_compilar() {
        let request = UploadRequest {
            sketch_path: "/tmp/Blink".to_string(),
            fqbn: "arduino:avr:uno".to_string(),
            port: "/dev/ttyACM0".to_string(),
            compile_first: false,
            verify: true,
            programmer: Some("usbasp".to_string()),
            baudrate: Some(115200),
            build_properties: vec![],
        };
        let args = build_upload_args(&request);
        assert_eq!(args[0], "upload");
        assert!(args.contains(&"--verify".to_string()));
        assert!(args.contains(&"--programmer".to_string()));
        assert!(args.contains(&"--baudrate".to_string()));
        assert_eq!(args.last().unwrap(), "/tmp/Blink");
    }

    #[test]
    fn acepta_id_de_sketch_como_ruta() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::state::AppPaths::new(dir.path().to_path_buf());
        paths.ensure().unwrap();

        let id = uuid::Uuid::new_v4().to_string();
        let sketch_dir = paths.sketch_dir(&id);
        std::fs::create_dir_all(&sketch_dir).unwrap();
        std::fs::write(paths.sketch_ino(&id), "void setup() {}").unwrap();

        let state = AppState::new(paths, crate::models::AppConfig::default());
        assert!(validate_sketch_path(&state, &id).is_ok());
        assert!(validate_sketch_path(&state, "no-es-un-uuid").is_err());
    }

    #[test]
    fn construye_argumentos_de_carga_compilando() {
        let request = UploadRequest {
            sketch_path: "/tmp/Blink".to_string(),
            fqbn: "arduino:avr:uno".to_string(),
            port: "/dev/ttyACM0".to_string(),
            compile_first: true,
            verify: false,
            programmer: None,
            baudrate: None,
            build_properties: vec!["build.extra_flags=-DKOR".to_string()],
        };
        let args = build_upload_args(&request);
        assert_eq!(args[0], "compile");
        assert!(args.contains(&"--upload".to_string()));
        assert!(args.contains(&"--build-property".to_string()));
    }
}
