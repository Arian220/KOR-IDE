use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::models::AppError;

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("archivo");
    let tmp = path.with_file_name(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()));

    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    #[cfg(target_os = "windows")]
    {
        let _ = fs::remove_file(path);
    }

    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = fs::remove_file(&tmp);
            Err(err.into())
        }
    }
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|err| AppError::InvalidConfig(format!("{}: {err}", path.display())))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), AppError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    write_atomic(path, &bytes)
}

pub fn ensure_inside(base: &Path, candidate: &Path) -> Result<PathBuf, AppError> {
    let base = base.canonicalize()?;
    let candidate = candidate.canonicalize().map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            AppError::SketchNotFound(candidate.display().to_string())
        } else {
            AppError::Io(err.to_string())
        }
    })?;

    if candidate.starts_with(&base) {
        Ok(candidate)
    } else {
        Err(AppError::PathNotAllowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escribe_archivo_de_forma_atomica() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        write_json(&path, &serde_json::json!({ "ok": true })).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"ok\": true"));
    }

    #[test]
    fn rechaza_rutas_fuera_del_area() {
        let dir = tempfile::tempdir().unwrap();
        let inside = dir.path().join("sketches");
        let outside = dir.path().join("fuera.txt");
        fs::create_dir(&inside).unwrap();
        fs::write(&outside, "x").unwrap();
        assert!(ensure_inside(&inside, &outside).is_err());
    }

    #[test]
    fn acepta_rutas_dentro_del_area() {
        let dir = tempfile::tempdir().unwrap();
        let inside = dir.path().join("sketches");
        fs::create_dir(&inside).unwrap();
        let sketch = inside.join("Blink");
        fs::create_dir(&sketch).unwrap();
        let ino = sketch.join("Blink.ino");
        fs::write(&ino, "void setup() {}").unwrap();
        assert_eq!(
            ensure_inside(&inside, &ino).unwrap(),
            ino.canonicalize().unwrap()
        );
    }
}
