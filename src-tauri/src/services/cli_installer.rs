use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use flate2::read::GzDecoder;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tauri::ipc::Channel;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use crate::models::{AppError, CliInfo, CliSource, JobEvent, JobKind};
use crate::services::{arduino, config, jobs};
use crate::state::AppState;

const DOWNLOAD_BASE: &str = "https://downloads.arduino.cc/arduino-cli/";
const CHECKSUMS_URL: &str =
    "https://github.com/arduino/arduino-cli/releases/download/v1.6.0/1.6.0-checksums.txt";
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const MAX_CHECKSUMS_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveKind {
    TarGz,
    Zip,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub kind: ArchiveKind,
}

pub fn asset_candidates(os: &str, arch: &str) -> Vec<Asset> {
    let os = os.to_ascii_lowercase();
    let arch = arch.to_ascii_lowercase();

    let os_name = match os.as_str() {
        "linux" => "Linux",
        "macos" | "darwin" => "macOS",
        "windows" => "Windows",
        _ => return Vec::new(),
    };

    let arch_name = match arch.as_str() {
        "x86_64" | "amd64" | "x64" => "64bit",
        "aarch64" | "arm64" => "ARM64",
        "x86" | "i686" | "i386" => "32bit",
        "arm" | "armv7" => "ARMv7",
        _ => return Vec::new(),
    };

    let (extension, kind) = if os_name == "Windows" {
        ("zip", ArchiveKind::Zip)
    } else {
        ("tar.gz", ArchiveKind::TarGz)
    };

    let version = arduino::REQUIRED_VERSION;
    let mut candidates = vec![Asset {
        name: format!("arduino-cli_{version}_{os_name}_{arch_name}.{extension}"),
        kind,
    }];

    if os_name == "macOS" && arch_name == "ARM64" {
        candidates.push(Asset {
            name: format!("arduino-cli_{version}_macOS_64bit.tar.gz"),
            kind: ArchiveKind::TarGz,
        });
    }

    candidates
}

pub async fn install(state: &AppState, events: Channel<JobEvent>) -> Result<CliInfo, AppError> {
    let (job_id, token) = jobs::register(state, JobKind::DownloadCli)?;
    let outcome = install_with_job(state, &job_id, &token, &events).await;
    jobs::remove(state, &job_id);

    match &outcome {
        Ok(_) => jobs::send(
            &events,
            JobEvent::Finished {
                job_id: job_id.clone(),
                success: true,
            },
        ),
        Err(error) => jobs::send(
            &events,
            JobEvent::Failed {
                job_id: job_id.clone(),
                error: error.clone(),
            },
        ),
    }

    outcome
}

async fn install_with_job(
    state: &AppState,
    job_id: &str,
    token: &CancellationToken,
    events: &Channel<JobEvent>,
) -> Result<CliInfo, AppError> {
    send_started(events, job_id);
    progress(events, job_id, "descargando lista de checksums", None);

    let checksums = download_checksums(token).await?;

    let candidates = asset_candidates(std::env::consts::OS, std::env::consts::ARCH);
    if candidates.is_empty() {
        return Err(AppError::CliDownload(format!(
            "plataforma no soportada: {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )));
    }

    let mut last_error = None;
    for asset in &candidates {
        let url = format!("{DOWNLOAD_BASE}{}", asset.name);
        let temp_archive = state.paths.tmp.join(format!(
            "arduino-cli-{}.{}",
            uuid::Uuid::new_v4(),
            extension(asset.kind)
        ));

        let result = download_asset(&url, &temp_archive, asset, token, job_id, events).await;
        match result {
            Ok(()) => {
                let outcome =
                    verify_and_install(state, &temp_archive, asset, &checksums, job_id, events)
                        .await;
                let _ = std::fs::remove_file(&temp_archive);
                return outcome;
            }
            Err(error) => {
                let _ = std::fs::remove_file(&temp_archive);
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::CliDownload("no hay un binario disponible para esta plataforma".to_string())
    }))
}

async fn download_asset(
    url: &str,
    destination: &Path,
    asset: &Asset,
    token: &CancellationToken,
    job_id: &str,
    events: &Channel<JobEvent>,
) -> Result<(), AppError> {
    progress(
        events,
        job_id,
        &format!("descargando {}", asset.name),
        Some(0),
    );
    download_to(url, destination, token, job_id, events).await
}

async fn download_checksums(
    token: &CancellationToken,
) -> Result<HashMap<String, String>, AppError> {
    let client = http_client()?;

    let response = tokio::select! {
        _ = token.cancelled() => return Err(AppError::Cancelled),
        result = client.get(CHECKSUMS_URL).send() => {
            result.map_err(|err| AppError::Network(err.to_string()))?
        }
    };

    if !response.status().is_success() {
        return Err(AppError::CliDownload(format!(
            "no se pudo descargar la lista de checksums: HTTP {}",
            response.status()
        )));
    }

    if let Some(length) = response.content_length() {
        if length as usize > MAX_CHECKSUMS_BYTES {
            return Err(AppError::CliDownload(
                "la lista de checksums es demasiado grande".to_string(),
            ));
        }
    }

    let body = response
        .text()
        .await
        .map_err(|err| AppError::Network(err.to_string()))?;

    if body.len() > MAX_CHECKSUMS_BYTES {
        return Err(AppError::CliDownload(
            "la lista de checksums es demasiado grande".to_string(),
        ));
    }

    let checksums = parse_checksums(&body);
    if checksums.is_empty() {
        return Err(AppError::CliDownload(
            "la lista de checksums está vacía o es inválida".to_string(),
        ));
    }

    Ok(checksums)
}

async fn download_to(
    url: &str,
    destination: &Path,
    token: &CancellationToken,
    job_id: &str,
    events: &Channel<JobEvent>,
) -> Result<(), AppError> {
    let client = http_client()?;

    let inner = async {
        let response = tokio::select! {
            _ = token.cancelled() => return Err(AppError::Cancelled),
            result = client.get(url).send() => {
                result.map_err(|err| AppError::Network(err.to_string()))?
            }
        };

        if !response.status().is_success() {
            return Err(AppError::CliDownload(format!(
                "HTTP {} al descargar {url}",
                response.status()
            )));
        }

        let total = response.content_length();
        let mut stream = response.bytes_stream();
        let mut file = tokio::fs::File::create(destination).await?;
        let mut downloaded = 0u64;
        let mut last_percent = u8::MAX;

        loop {
            let next = tokio::select! {
                _ = token.cancelled() => return Err(AppError::Cancelled),
                chunk = stream.next() => chunk,
            };

            let Some(chunk) = next else {
                break;
            };

            let chunk = chunk.map_err(|err| AppError::Network(err.to_string()))?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if let Some(total) = total {
                if let Some(percent) = downloaded.saturating_mul(100).checked_div(total) {
                    let percent = percent.min(100) as u8;
                    if percent != last_percent {
                        last_percent = percent;
                        progress(events, job_id, "descargando arduino-cli", Some(percent));
                    }
                }
            }
        }

        file.flush().await?;
        Ok(())
    };

    match tokio::time::timeout(DOWNLOAD_TIMEOUT, inner).await {
        Ok(result) => result,
        Err(_) => Err(AppError::Internal(
            "tiempo de espera agotado al descargar arduino-cli".to_string(),
        )),
    }
}

async fn verify_and_install(
    state: &AppState,
    archive: &Path,
    asset: &Asset,
    checksums: &HashMap<String, String>,
    job_id: &str,
    events: &Channel<JobEvent>,
) -> Result<CliInfo, AppError> {
    let expected = checksums.get(&asset.name).ok_or_else(|| {
        AppError::CliDownload(format!("no hay checksum publicado para {}", asset.name))
    })?;

    progress(events, job_id, "verificando checksum", None);
    let actual = sha256_file_async(archive.to_path_buf()).await?;
    if !actual.eq_ignore_ascii_case(expected) {
        tracing::error!(
            "checksum inválido para {}: esperado {expected}, obtenido {actual}",
            asset.name
        );
        return Err(AppError::CliChecksumMismatch);
    }
    tracing::info!("checksum verificado para {}: {actual}", asset.name);

    progress(events, job_id, "extrayendo arduino-cli", None);
    let extracted = state
        .paths
        .tmp
        .join(format!("arduino-cli-extracted-{}", uuid::Uuid::new_v4()));

    let archive_path = archive.to_path_buf();
    let extracted_path = extracted.clone();
    let kind = asset.kind;
    tokio::task::spawn_blocking(move || extract_binary(&archive_path, &extracted_path, kind))
        .await
        .map_err(|err| AppError::Internal(err.to_string()))??;

    progress(events, job_id, "instalando arduino-cli", None);
    let installed = state.paths.cli_bin.clone();
    let extracted_for_install = extracted.clone();
    tokio::task::spawn_blocking(move || install_binary(&extracted_for_install, &installed))
        .await
        .map_err(|err| AppError::Internal(err.to_string()))??;
    let _ = std::fs::remove_file(&extracted);

    let version = match arduino::version_of(&state.paths.cli_bin).await {
        Ok(version) => version,
        Err(err) => {
            let _ = std::fs::remove_file(&state.paths.cli_bin);
            return Err(err);
        }
    };

    if !arduino::version_at_least(&version, arduino::REQUIRED_VERSION) {
        let _ = std::fs::remove_file(&state.paths.cli_bin);
        return Err(AppError::CliUnsupported(version));
    }

    {
        let mut config = state.config.write().await;
        config.cli_path = None;
        config.cli_version = Some(version.clone());
        config::save(&state.paths, &config)?;
    }

    let info = CliInfo {
        path: state.paths.cli_bin.display().to_string(),
        version,
        source: CliSource::Managed,
    };
    *state.cli.write().await = Some(info.clone());

    progress(events, job_id, "completado", Some(100));
    Ok(info)
}

fn http_client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .user_agent(format!("KOR/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|err| AppError::Network(err.to_string()))
}

fn extension(kind: ArchiveKind) -> &'static str {
    match kind {
        ArchiveKind::TarGz => "tar.gz",
        ArchiveKind::Zip => "zip",
    }
}

fn is_cli_binary_name(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("arduino-cli") | Some("arduino-cli.exe")
    )
}

fn extract_binary(archive: &Path, destination: &Path, kind: ArchiveKind) -> Result<(), AppError> {
    match kind {
        ArchiveKind::TarGz => {
            let file = std::fs::File::open(archive)?;
            let decoder = GzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);

            for entry in archive.entries()? {
                let mut entry = entry?;
                if !entry.header().entry_type().is_file() {
                    continue;
                }
                if is_cli_binary_name(&entry.path()?) {
                    copy_entry(&mut entry, destination)?;
                    return Ok(());
                }
            }
        }
        ArchiveKind::Zip => {
            let file = std::fs::File::open(archive)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|err| AppError::CliDownload(format!("zip inválido: {err}")))?;

            for index in 0..archive.len() {
                let mut entry = archive
                    .by_index(index)
                    .map_err(|err| AppError::CliDownload(format!("zip inválido: {err}")))?;
                if entry.is_dir() {
                    continue;
                }
                if is_cli_binary_name(Path::new(entry.name())) {
                    copy_entry(&mut entry, destination)?;
                    return Ok(());
                }
            }
        }
    }

    Err(AppError::CliDownload(
        "el archivo descargado no contiene arduino-cli".to_string(),
    ))
}

fn copy_entry<R: Read>(entry: &mut R, destination: &Path) -> Result<(), AppError> {
    let mut output = std::fs::File::create(destination)?;
    std::io::copy(entry, &mut output)?;
    output.sync_all()?;
    Ok(())
}

fn install_binary(source: &Path, destination: &Path) -> Result<(), AppError> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(source, std::fs::Permissions::from_mode(0o755))?;
    }

    #[cfg(target_os = "windows")]
    let _ = std::fs::remove_file(destination);

    std::fs::rename(source, destination)?;
    Ok(())
}

async fn sha256_file_async(path: PathBuf) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || sha256_file(&path))
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?
}

fn sha256_file(path: &Path) -> Result<String, AppError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn parse_checksums(text: &str) -> HashMap<String, String> {
    let mut checksums = HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((hash, rest)) = line.split_once(char::is_whitespace) else {
            continue;
        };

        let name = rest.trim().trim_start_matches('*');
        if hash.len() != 64 || !hash.chars().all(|character| character.is_ascii_hexdigit()) {
            continue;
        }

        checksums.insert(name.to_string(), hash.to_ascii_lowercase());
    }

    checksums
}

fn send_started(events: &Channel<JobEvent>, job_id: &str) {
    jobs::send(
        events,
        JobEvent::Started {
            job_id: job_id.to_string(),
            kind: JobKind::DownloadCli,
        },
    );
}

fn progress(events: &Channel<JobEvent>, job_id: &str, stage: &str, percent: Option<u8>) {
    jobs::send(
        events,
        JobEvent::Progress {
            job_id: job_id.to_string(),
            stage: stage.to_string(),
            percent,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn selecciona_asset_por_so_y_arquitectura() {
        let linux = asset_candidates("linux", "x86_64");
        assert_eq!(linux[0].name, "arduino-cli_1.6.0_Linux_64bit.tar.gz");
        assert_eq!(linux[0].kind, ArchiveKind::TarGz);

        let linux_arm = asset_candidates("linux", "aarch64");
        assert_eq!(linux_arm[0].name, "arduino-cli_1.6.0_Linux_ARM64.tar.gz");

        let mac = asset_candidates("macos", "aarch64");
        assert_eq!(mac[0].name, "arduino-cli_1.6.0_macOS_ARM64.tar.gz");
        assert_eq!(mac[1].name, "arduino-cli_1.6.0_macOS_64bit.tar.gz");

        let windows = asset_candidates("windows", "x86_64");
        assert_eq!(windows[0].name, "arduino-cli_1.6.0_Windows_64bit.zip");
        assert_eq!(windows[0].kind, ArchiveKind::Zip);

        assert!(asset_candidates("solaris", "x86_64").is_empty());
        assert!(asset_candidates("linux", "mips").is_empty());
    }

    #[test]
    fn parsea_checksums_oficiales() {
        let text = "\
eccb6f09812f1f97ff034925b7d037624394653237d436a12512dbf33925c30b  arduino-cli_1.6.0-1_amd64.deb
321ee249f44c78053234fec0f36101092bc6021bf24939e15e0cc2449c5ed250  arduino-cli_1.6.0_Linux_64bit.tar.gz
f85f37f0a50f1742ce18851469d8c66ecb1b624e809525722e96ba4eb82ba183 *arduino-cli_1.6.0_Linux_ARMv6.tar.gz
no-es-un-hash  archivo.txt
";

        let checksums = parse_checksums(text);
        assert_eq!(checksums.len(), 3);
        assert_eq!(
            checksums
                .get("arduino-cli_1.6.0_Linux_64bit.tar.gz")
                .unwrap(),
            "321ee249f44c78053234fec0f36101092bc6021bf24939e15e0cc2449c5ed250"
        );
        assert!(checksums.contains_key("arduino-cli_1.6.0_Linux_ARMv6.tar.gz"));
        assert!(!checksums.contains_key("archivo.txt"));
    }

    #[test]
    fn verifica_sha256_de_un_archivo() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("binario");
        std::fs::write(&path, b"contenido").unwrap();

        let hash = sha256_file(&path).unwrap();
        assert_eq!(
            hash,
            "6f9566ef46386b8cf372671cb9eddff5488256eff5ea5fb99e1041c3e27082bf"
        );

        let dir2 = tempfile::tempdir().unwrap();
        let path2 = dir2.path().join("otro");
        std::fs::write(&path2, b"otro contenido").unwrap();
        assert_ne!(sha256_file(&path2).unwrap(), hash);
    }

    #[test]
    fn extrae_binario_desde_tar_gz() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("cli.tar.gz");
        let destination = dir.path().join("arduino-cli");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let encoder = GzEncoder::new(file, Compression::default());
            let mut builder = tar::Builder::new(encoder);

            let contents = b"binario";
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, "arduino-cli", &contents[..])
                .unwrap();
            builder.finish().unwrap();
        }

        extract_binary(&archive_path, &destination, ArchiveKind::TarGz).unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"binario");
    }

    #[test]
    fn extrae_binario_desde_zip() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("cli.zip");
        let destination = dir.path().join("arduino-cli.exe");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file("arduino-cli.exe", options).unwrap();
            writer.write_all(b"binario").unwrap();
            writer.finish().unwrap();
        }

        extract_binary(&archive_path, &destination, ArchiveKind::Zip).unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"binario");
    }

    #[test]
    fn falla_si_el_archivo_no_contiene_el_binario() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = dir.path().join("cli.zip");
        let destination = dir.path().join("arduino-cli");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            writer
                .start_file("otro.txt", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"x").unwrap();
            writer.finish().unwrap();
        }

        assert!(matches!(
            extract_binary(&archive_path, &destination, ArchiveKind::Zip),
            Err(AppError::CliDownload(_))
        ));
    }

    #[tokio::test]
    #[ignore = "descarga real de arduino-cli desde downloads.arduino.cc"]
    async fn instala_cli_real() {
        use crate::models::AppConfig;
        use crate::state::{AppPaths, AppState};

        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path().to_path_buf());
        paths.ensure().unwrap();
        let state = AppState::new(paths, AppConfig::default());
        let events = Channel::<JobEvent>::new(|_| Ok(()));

        let info = install(&state, events).await.unwrap();
        assert_eq!(info.source, CliSource::Managed);
        assert!(state.paths.cli_bin.is_file());
        assert!(arduino::version_at_least(
            &info.version,
            arduino::REQUIRED_VERSION
        ));
        assert_eq!(
            state.config.read().await.cli_version.as_deref(),
            Some(info.version.as_str())
        );
    }
}
