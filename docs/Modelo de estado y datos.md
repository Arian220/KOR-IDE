---
tags: [kor, estado, datos, persistencia]
---

# Modelo de estado y datos

Complementa [[Arquitectura]] y [[Contratos Tauri]]. Define qué vive en Rust, qué
en Vue y dónde se persiste.

## 1. Principio

- **Rust**: fuente de verdad de configuración, sketches, jobs y rutas.
- **Vue (Pinia)**: réplica reactiva para renderizar; se hidrata al arrancar y se
  actualiza con respuestas y eventos.

## 2. Estado del backend (`AppState`)

```rust
pub struct AppState {
    pub config: tokio::sync::RwLock<AppConfig>,
    pub cli: tokio::sync::RwLock<Option<CliInfo>>,
    pub jobs: std::sync::Mutex<std::collections::HashMap<JobId, JobHandle>>,
    pub paths: AppPaths,
}

pub struct JobHandle {
    pub kind: JobKind,
    pub cancel: tokio_util::sync::CancellationToken,
    pub started_at: std::time::Instant,
}

pub enum JobKind {
    Compile,
    Upload,
    InstallCore,
    InstallLibrary,
    DownloadCli,
}
```

### Máquina de estados de un job

```text
Created → Running → (Succeeded | Failed | Cancelled)
```

Reglas:

- No se admite un segundo `Compile`/`Upload` simultáneo (se devuelve
  `AppError::Internal("operación en curso")` o se encola; decisión: rechazar).
- `cancel_job` es idempotente; si ya terminó, devuelve `false`.
- Al cerrar la app se cancelan los jobs activos y se espera su finalización.

## 3. Estado del frontend (Pinia)

| Store | Contenido | Persistencia |
|---|---|---|
| `config` | tema, FQBN y puerto recientes, autosave | Rust (`config.json`) |
| `editor` | pestañas abiertas, archivo activo, cursor, dirty | Rust (sketches) + sesión UI |
| `boards` | última detección, placa/puerto seleccionado | efímero |
| `jobs` | job activo, estado, progreso, consola | efímero |
| `sketches` | índice de sketches | Rust (`index.json`) |

> **Transición (fases 2–3):** todavía no existen Pinia ni el backend Rust. La
> configuración y la sesión se persisten con un adaptador temporal en
> `src/api/storage.ts` sobre `localStorage` (`kor.config.v1`,
> `kor.session.v1`). En la fase 4 se sustituye por los comandos Tauri; la UI no
> cambia.

Reglas:

- El autoguardado definitivo va a Rust. Durante las fases 2–3 se usa
  `localStorage` como adaptador temporal (ver nota).
- La selección de puerto se pierde al reiniciar (se vuelve a detectar).
- La consola conserva la salida de la última operación hasta que se inicia otra.

## 4. Rutas de datos

`<app_data>` = Tauri `app_data_dir()` (ADR-005):

| SO | Ruta |
|---|---|
| Windows | `%APPDATA%\com.loopi.kor` |
| macOS | `~/Library/Application Support/com.loopi.kor` |
| Linux | `~/.local/share/com.loopi.kor` |

```text
<app_data>/
├── config/
│   └── config.json
├── sketches/
│   ├── index.json
│   └── <uuid>/
│       ├── sketch.ino
│       └── meta.json
├── bin/
│   └── arduino-cli[.exe]
├── cache/
│   └── arduino-cli-1.6.0-<os>-<arch>.tar.gz
└── logs/
    ├── kor.log
    └── kor.log.1
```

### `config.json`

```json
{
  "cli_path": null,
  "cli_version": "1.6.0",
  "theme": "system",
  "last_fqbn": "arduino:avr:uno",
  "last_port": null,
  "autosave": true,
  "autosave_delay_ms": 800,
  "log_level": "info",
  "max_recent_sketches": 20
}
```

### `sketches/index.json`

```json
{
  "version": 1,
  "sketches": [
    {
      "id": "3f1c2a9e-8b4d-4c1a-9f2e-1a2b3c4d5e6f",
      "name": "blink",
      "updated_at": "2026-11-15T21:00:00Z",
      "fqbn": "esp32:esp32:esp32doit-devkit-v1"
    }
  ]
}
```

### `sketches/<uuid>/meta.json`

```json
{
  "id": "3f1c2a9e-...",
  "name": "blink",
  "fqbn": "esp32:esp32:esp32doit-devkit-v1",
  "created_at": "2026-11-15T20:00:00Z",
  "updated_at": "2026-11-15T21:00:00Z"
}
```

## 5. Escritura atómica y recuperación

- Escribir en `*.tmp` en el mismo directorio, `fsync` y `rename`.
- Al arrancar, si existe un `*.tmp` huérfano, se elimina.
- Si `index.json` está corrupto, se reconstruye escaneando carpetas `<uuid>/`.
- Los sketches temporales de compilación van a `<app_data>/cache/build/<job_id>/`
  y se eliminan al terminar el job (éxito, error o cancelación).

## 6. Limpieza

| Elemento | Momento | Acción |
|---|---|---|
| `cache/build/<job_id>` | fin del job | eliminar |
| `cache/*.tar.gz` | tras instalar CLI | eliminar |
| `*.tmp` | arranque y fin de escritura | eliminar |
| Logs antiguos | al superar N MB | rotar y borrar el más antiguo |
| Sketches | solo acción del usuario | no se borran automáticamente |

## 7. Migración desde el estado actual

- El actual `temp_sketch/temp_sketch.ino` no se migra: era efímero y no debía
  estar versionado.
- Al primer arranque de KOR no hay sketches previos; se crea uno vacío llamado
  `blink` con una plantilla mínima.
- El contenido que el usuario tuviera en el editor de Electron no se puede
  recuperar salvo que se exporte manualmente antes de la migración.

## 8. Estado de configuración del CLI

```text
Sin CLI
  └─(instalar)→ Descargando → Verificando → Listo
  └─(ruta manual válida)→ Listo
Listo
  └─(versión < 1.6.0)→ Incompatible
  └─(binario borrado)→ Sin CLI
```

La UI muestra el estado y permite reinstalar o cambiar la ruta. Detalle de
seguridad en [[Seguridad]].

## 9. Convenciones de datos

- UTF-8 sin BOM; finales de línea `\n` en archivos nuevos.
- JSON con 2 espacios, orden de claves estable cuando sea posible.
- IDs: UUID v4.
- Fechas: ISO-8601 UTC (`2026-11-15T21:00:00Z`).
- Rutas serializadas como `String`; en Rust se usan `PathBuf` internamente.
