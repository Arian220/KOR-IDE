use std::io::ErrorKind;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use tauri::ipc::Channel;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::models::{AppError, JobEvent, OutputStream};

const MAX_JOB_TIME: Duration = Duration::from_secs(10 * 60);

pub struct StreamResult {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub async fn run_capture(
    cli: &Path,
    args: &[String],
    timeout: Duration,
) -> Result<std::process::Output, AppError> {
    let mut command = Command::new(cli);
    command.args(args).stdin(Stdio::null()).kill_on_drop(true);

    match tokio::time::timeout(timeout, command.output()).await {
        Err(_) => Err(AppError::Internal(
            "tiempo de espera agotado al ejecutar arduino-cli".to_string(),
        )),
        Ok(Err(err)) if err.kind() == ErrorKind::NotFound => Err(AppError::CliNotFound),
        Ok(Err(err)) => Err(err.into()),
        Ok(Ok(output)) => Ok(output),
    }
}

pub async fn run_streaming(
    cli: &Path,
    args: &[String],
    token: CancellationToken,
    job_id: &str,
    events: Channel<JobEvent>,
) -> Result<StreamResult, AppError> {
    let mut command = Command::new(cli);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = command.spawn().map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            AppError::CliNotFound
        } else {
            AppError::Io(err.to_string())
        }
    })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Internal("sin stdout del proceso".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Internal("sin stderr del proceso".to_string()))?;

    let stdout_task = tokio::spawn(read_stream(
        stdout,
        OutputStream::Stdout,
        job_id.to_string(),
        events.clone(),
    ));
    let stderr_task = tokio::spawn(read_stream(
        stderr,
        OutputStream::Stderr,
        job_id.to_string(),
        events,
    ));

    let mut wait_task = tokio::spawn(async move { child.wait().await });
    let abort_handle = wait_task.abort_handle();

    let status = tokio::select! {
        _ = token.cancelled() => {
            abort_handle.abort();
            return Err(AppError::Cancelled);
        }
        result = tokio::time::timeout(MAX_JOB_TIME, &mut wait_task) => {
            match result {
                Err(_) => {
                    abort_handle.abort();
                    return Err(AppError::Internal("tiempo de espera agotado".to_string()));
                }
                Ok(Err(join_err)) => {
                    return Err(AppError::Internal(format!(
                        "el proceso terminó de forma inesperada: {join_err}"
                    )));
                }
                Ok(Ok(Err(io_err))) => return Err(io_err.into()),
                Ok(Ok(Ok(status))) => status,
            }
        }
    };

    let stdout = stdout_task
        .await
        .map_err(|err| AppError::Internal(err.to_string()))??;
    let stderr = stderr_task
        .await
        .map_err(|err| AppError::Internal(err.to_string()))??;

    Ok(StreamResult {
        status,
        stdout,
        stderr,
    })
}

async fn read_stream<R>(
    reader: R,
    stream: OutputStream,
    job_id: String,
    events: Channel<JobEvent>,
) -> Result<String, AppError>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut reader = BufReader::new(reader);
    let mut accumulated = String::new();
    let mut line = String::new();

    loop {
        line.clear();
        let read = reader.read_line(&mut line).await?;
        if read == 0 {
            break;
        }

        accumulated.push_str(&line);
        let _ = events.send(JobEvent::Output {
            job_id: job_id.clone(),
            stream,
            chunk: line.clone(),
        });
    }

    Ok(accumulated)
}

pub fn find_executable(name: &str) -> Option<std::path::PathBuf> {
    if let Ok(found) = which::which(name) {
        return Some(found);
    }

    let file_name = if cfg!(target_os = "windows") && !name.ends_with(".exe") {
        format!("{name}.exe")
    } else {
        name.to_string()
    };

    let mut dirs: Vec<std::path::PathBuf> = std::env::var_os("PATH")
        .map(|value| std::env::split_paths(&value).collect())
        .unwrap_or_default();
    dirs.extend(fallback_search_dirs());

    dirs.into_iter()
        .map(|dir| dir.join(&file_name))
        .find(|candidate| is_executable(candidate))
}

fn home_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var_os("USERPROFILE").filter(|value| !value.is_empty()))
        .map(std::path::PathBuf::from)
}

fn fallback_search_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = home_dir() {
        dirs.push(home.join(".local").join("bin"));
        dirs.push(home.join(".cargo").join("bin"));
        dirs.push(home.join(".arduino15").join("bin"));
        dirs.push(home.join("bin"));
    }

    if cfg!(target_os = "windows") {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let local = std::path::PathBuf::from(local);
            dirs.push(local.join("Programs").join("arduino-cli"));
            dirs.push(local.join("Programs").join("Arduino"));
            dirs.push(local.join("Microsoft").join("WinGet").join("Links"));
        }
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            dirs.push(
                std::path::PathBuf::from(profile)
                    .join("scoop")
                    .join("shims"),
            );
        }
        if let Some(program_data) = std::env::var_os("ProgramData") {
            dirs.push(
                std::path::PathBuf::from(program_data)
                    .join("chocolatey")
                    .join("bin"),
            );
        }
        dirs.push(std::path::PathBuf::from(r"C:\Program Files\Arduino"));
    } else if cfg!(target_os = "macos") {
        dirs.push(std::path::PathBuf::from("/opt/homebrew/bin"));
        dirs.push(std::path::PathBuf::from("/usr/local/bin"));
        dirs.push(std::path::PathBuf::from("/opt/local/bin"));
        dirs.push(std::path::PathBuf::from("/usr/bin"));
        dirs.push(std::path::PathBuf::from("/bin"));
    } else {
        dirs.push(std::path::PathBuf::from("/usr/local/bin"));
        dirs.push(std::path::PathBuf::from("/usr/bin"));
        dirs.push(std::path::PathBuf::from("/bin"));
        dirs.push(std::path::PathBuf::from("/snap/bin"));
    }

    dirs
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valida_ejecutables() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("arduino-cli");
        std::fs::write(&file, b"").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
            assert!(!is_executable(&file));

            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
            assert!(is_executable(&file));
        }

        #[cfg(not(unix))]
        assert!(is_executable(&file));

        assert!(!is_executable(&dir.path().join("no-existe")));
    }

    #[test]
    fn incluye_rutas_de_usuario() {
        let dirs = fallback_search_dirs();
        if let Some(home) = home_dir() {
            assert!(dirs.contains(&home.join(".local").join("bin")));
            assert!(dirs.contains(&home.join(".cargo").join("bin")));
        }
    }
}
