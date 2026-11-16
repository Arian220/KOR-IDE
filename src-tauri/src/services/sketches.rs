use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use chrono::{DateTime, SecondsFormat, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infra::fs::{read_json, write_atomic, write_json};
use crate::models::{AppError, Sketch, SketchSummary};
use crate::services::arduino;
use crate::state::AppPaths;

const MAX_NAME_CHARS: usize = 120;
const META_FILE: &str = "sketch.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SketchMeta {
    id: String,
    name: String,
    #[serde(default)]
    fqbn: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

pub fn list(paths: &AppPaths) -> Result<Vec<SketchSummary>, AppError> {
    let mut summaries = Vec::new();

    for entry in fs::read_dir(&paths.sketches)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if let Some(summary) = summary_for(&path)? {
            summaries.push(summary);
        }
    }

    summaries.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(summaries)
}

pub fn read(paths: &AppPaths, id: &str) -> Result<Sketch, AppError> {
    let dir = sketch_dir(paths, id)?;
    let ino = find_ino(&dir, id)?;
    let content = fs::read_to_string(&ino)?;
    let meta = read_meta(&dir);
    let name = meta
        .as_ref()
        .map(|value| value.name.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| id.to_string());
    let fqbn = meta.as_ref().and_then(|value| value.fqbn.clone());
    let file_timestamp = file_timestamp(&ino);
    let updated = meta
        .as_ref()
        .and_then(|value| value.updated_at.as_deref())
        .and_then(parse_timestamp)
        .map(|value| value.max(file_timestamp))
        .unwrap_or(file_timestamp);

    Ok(Sketch {
        id: id.to_string(),
        name,
        content,
        fqbn,
        updated_at: iso_from_timestamp(updated),
    })
}

pub fn save(paths: &AppPaths, sketch: &Sketch) -> Result<SketchSummary, AppError> {
    let dir = sketch_dir(paths, &sketch.id)?;
    let name = sanitize_name(&sketch.name)?;
    if let Some(fqbn) = &sketch.fqbn {
        arduino::validate_fqbn(fqbn)?;
    }

    let ino = match find_ino(&dir, &sketch.id) {
        Ok(path) => path,
        Err(AppError::SketchNotFound(_)) => paths.sketch_ino(&sketch.id),
        Err(err) => return Err(err),
    };

    write_atomic(&ino, sketch.content.as_bytes())?;

    let updated_at = now_iso();
    let meta = SketchMeta {
        id: sketch.id.clone(),
        name: name.clone(),
        fqbn: sketch.fqbn.clone(),
        updated_at: Some(updated_at.clone()),
    };
    write_json(&dir.join(META_FILE), &meta)?;

    Ok(SketchSummary {
        id: sketch.id.clone(),
        name,
        updated_at,
        fqbn: sketch.fqbn.clone(),
    })
}

pub fn create(paths: &AppPaths, name: &str) -> Result<Sketch, AppError> {
    let name = sanitize_name(name)?;
    let id = Uuid::new_v4().to_string();
    let dir = paths.sketch_dir(&id);

    let result = (|| -> Result<Sketch, AppError> {
        fs::create_dir_all(&dir)?;
        let content = "void setup() {\n}\n\nvoid loop() {\n}\n";
        let ino = paths.sketch_ino(&id);
        write_atomic(&ino, content.as_bytes())?;

        let updated_at = now_iso();
        let meta = SketchMeta {
            id: id.clone(),
            name: name.clone(),
            fqbn: None,
            updated_at: Some(updated_at.clone()),
        };
        write_json(&dir.join(META_FILE), &meta)?;

        Ok(Sketch {
            id,
            name,
            content: content.to_string(),
            fqbn: None,
            updated_at,
        })
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&dir);
    }

    result
}

pub fn rename(paths: &AppPaths, id: &str, name: &str) -> Result<SketchSummary, AppError> {
    let name = sanitize_name(name)?;
    let current = read(paths, id)?;
    let dir = sketch_dir(paths, id)?;
    let updated_at = now_iso();
    let meta = SketchMeta {
        id: id.to_string(),
        name: name.clone(),
        fqbn: current.fqbn.clone(),
        updated_at: Some(updated_at.clone()),
    };
    write_json(&dir.join(META_FILE), &meta)?;

    Ok(SketchSummary {
        id: id.to_string(),
        name,
        updated_at,
        fqbn: current.fqbn,
    })
}

pub fn delete(paths: &AppPaths, id: &str) -> Result<bool, AppError> {
    match sketch_dir(paths, id) {
        Ok(dir) => {
            fs::remove_dir_all(dir)?;
            Ok(true)
        }
        Err(AppError::SketchNotFound(_)) => Ok(false),
        Err(err) => Err(err),
    }
}

pub fn export(paths: &AppPaths, id: &str, target_path: &str) -> Result<String, AppError> {
    let sketch = read(paths, id)?;
    let target = PathBuf::from(target_path);
    if !target.is_absolute() {
        return Err(AppError::PathNotAllowed);
    }

    let destination = if target.is_dir() {
        target.join(format!("{}.ino", sanitize_name(&sketch.name)?))
    } else {
        target
    };

    let parent = destination.parent().ok_or(AppError::PathNotAllowed)?;
    if !parent.is_dir() {
        return Err(AppError::PathNotAllowed);
    }

    write_atomic(&destination, sketch.content.as_bytes())?;
    Ok(destination.display().to_string())
}

fn summary_for(dir: &Path) -> Result<Option<SketchSummary>, AppError> {
    let Some(id) = dir.file_name().and_then(|value| value.to_str()) else {
        return Ok(None);
    };

    if ensure_valid_id(id).is_err() {
        return Ok(None);
    }

    let ino = match find_ino(dir, id) {
        Ok(path) => path,
        Err(AppError::SketchNotFound(_)) => return Ok(None),
        Err(err) => return Err(err),
    };

    let meta = read_meta(dir);
    let name = meta
        .as_ref()
        .map(|value| value.name.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| id.to_string());
    let fqbn = meta.as_ref().and_then(|value| value.fqbn.clone());
    let file_timestamp = file_timestamp(&ino);
    let updated = meta
        .as_ref()
        .and_then(|value| value.updated_at.as_deref())
        .and_then(parse_timestamp)
        .map(|value| value.max(file_timestamp))
        .unwrap_or(file_timestamp);

    Ok(Some(SketchSummary {
        id: id.to_string(),
        name,
        updated_at: iso_from_timestamp(updated),
        fqbn,
    }))
}

fn sketch_dir(paths: &AppPaths, id: &str) -> Result<PathBuf, AppError> {
    ensure_valid_id(id)?;
    let dir = paths.sketch_dir(id);
    if !dir.is_dir() {
        return Err(AppError::SketchNotFound(id.to_string()));
    }
    paths.ensure_inside_sketches(&dir)
}

fn ensure_valid_id(id: &str) -> Result<(), AppError> {
    let re = Regex::new(r"^[a-f0-9-]{36}$")?;
    if !re.is_match(id) || Uuid::parse_str(id).is_err() {
        return Err(AppError::SketchNotFound(id.to_string()));
    }
    Ok(())
}

fn find_ino(dir: &Path, id: &str) -> Result<PathBuf, AppError> {
    let preferred = dir.join(format!("{id}.ino"));
    if preferred.is_file() {
        return Ok(preferred);
    }

    let mut found = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let is_ino = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("ino"))
            .unwrap_or(false);
        if is_ino && path.is_file() {
            found.push(path);
        }
    }

    found.sort();
    found
        .into_iter()
        .next()
        .ok_or_else(|| AppError::SketchNotFound(id.to_string()))
}

fn read_meta(dir: &Path) -> Option<SketchMeta> {
    read_json::<SketchMeta>(&dir.join(META_FILE)).ok()
}

fn sanitize_name(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidConfig(
            "el nombre del sketch no puede estar vacío".to_string(),
        ));
    }

    if trimmed.chars().count() > MAX_NAME_CHARS {
        return Err(AppError::InvalidConfig(format!(
            "el nombre del sketch no puede superar {MAX_NAME_CHARS} caracteres"
        )));
    }

    let mut sanitized: String = trimmed
        .chars()
        .map(|value| {
            if value.is_control()
                || matches!(value, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
            {
                '_'
            } else {
                value
            }
        })
        .collect();

    sanitized = sanitized.trim_matches(['.', ' ']).to_string();

    if sanitized.is_empty() {
        sanitized = "sketch".to_string();
    }

    if is_windows_reserved(&sanitized) {
        sanitized = format!("_{sanitized}");
    }

    if sanitized.chars().count() > MAX_NAME_CHARS {
        sanitized = sanitized.chars().take(MAX_NAME_CHARS).collect();
        sanitized = sanitized.trim_end_matches(['.', ' ']).to_string();
    }

    Ok(sanitized)
}

fn is_windows_reserved(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn parse_timestamp(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.timestamp())
}

fn file_timestamp(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_secs() as i64)
        .unwrap_or_else(|| Utc::now().timestamp())
}

fn iso_from_timestamp(timestamp: i64) -> String {
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(Utc::now)
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, AppPaths) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::new(dir.path().to_path_buf());
        paths.ensure().unwrap();
        (dir, paths)
    }

    #[test]
    fn crea_lee_guarda_y_lista() {
        let (_dir, paths) = setup();
        let created = create(&paths, "Blink LED").unwrap();
        assert_eq!(created.name, "Blink LED");

        let read = read(&paths, &created.id).unwrap();
        assert_eq!(read.content, created.content);

        let mut sketch = read;
        sketch.content = "void setup() { pinMode(13, OUTPUT); }\n".to_string();
        sketch.fqbn = Some("arduino:avr:uno".to_string());
        let summary = save(&paths, &sketch).unwrap();
        assert_eq!(summary.fqbn.as_deref(), Some("arduino:avr:uno"));

        let list = list(&paths).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);
        assert_eq!(list[0].name, "Blink LED");
    }

    #[test]
    fn renombra_exporta_y_elimina() {
        let (dir, paths) = setup();
        let sketch = create(&paths, "Blink").unwrap();

        let renamed = rename(&paths, &sketch.id, "Blink:LED").unwrap();
        assert_eq!(renamed.name, "Blink_LED");

        let export_dir = dir.path().join("export");
        fs::create_dir(&export_dir).unwrap();
        let destination = export(&paths, &sketch.id, export_dir.to_str().unwrap()).unwrap();
        assert!(Path::new(&destination).is_file());
        assert!(destination.ends_with("Blink_LED.ino"));

        assert!(delete(&paths, &sketch.id).unwrap());
        assert!(list(&paths).unwrap().is_empty());
        assert!(!delete(&paths, &sketch.id).unwrap());
    }

    #[test]
    fn rechaza_ids_invalidos() {
        let (_dir, paths) = setup();
        assert!(read(&paths, "../config").is_err());
        assert!(read(&paths, "no-es-un-uuid").is_err());
    }

    #[test]
    fn sanitiza_nombres_peligrosos() {
        assert_eq!(sanitize_name("  Blink:LED  ").unwrap(), "Blink_LED");
        assert_eq!(sanitize_name("../x").unwrap(), "_x");
        assert_eq!(sanitize_name("CON").unwrap(), "_CON");
        assert!(sanitize_name("   ").is_err());
    }
}
