---
tags: [kor, contratos, ipc, tauri]
---

# Contratos Tauri

Frontera única entre Vue y Rust. Si el código y este documento difieren, se
actualiza el documento junto con el cambio de código (ver ADR-012 y
[[Convenciones y workflow]]).

Convenciones:

- Nombres de comandos en `snake_case` (Tauri los expone tal cual).
- Nombres de campos en `snake_case`; el frontend los consume igual.
- Todos los comandos devuelven `Result<T, AppError>`.
- Los flujos largos reciben un `Channel<JobEvent>` y devuelven un `JobId`.
- Fechas en ISO-8601 UTC; rutas como `String`. En compilación/carga,
  `sketch_path` acepta un UUID de sketch o una ruta absoluta.

## 1. Tipos base

### `AppError`

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum AppError {
    #[error("arduino-cli no está disponible")]
    CliNotFound,
    #[error("fallo al descargar arduino-cli: {0}")]
    CliDownload(String),
    #[error("checksum inválido para arduino-cli")]
    CliChecksumMismatch,
    #[error("versión de arduino-cli no soportada: {0}")]
    CliUnsupported(String),
    #[error("no se detectaron placas")]
    BoardNotFound,
    #[error("puerto inválido: {0}")]
    InvalidPort(String),
    #[error("FQBN inválido: {0}")]
    InvalidFqbn(String),
    #[error("compilación fallida")]
    CompileFailed { stdout: String, stderr: String, diagnostics: Vec<Diagnostic> },
    #[error("carga fallida")]
    UploadFailed { stdout: String, stderr: String },
    #[error("operación cancelada")]
    Cancelled,
    #[error("job no encontrado: {0}")]
    JobNotFound(String),
    #[error("sketch no encontrado: {0}")]
    SketchNotFound(String),
    #[error("ruta fuera del área permitida")]
    PathNotAllowed,
    #[error("error de entrada/salida: {0}")]
    Io(String),
    #[error("error de red: {0}")]
    Network(String),
    #[error("configuración inválida: {0}")]
    InvalidConfig(String),
    #[error("error interno: {0}")]
    Internal(String),
}
```

Forma en TypeScript:

```ts
export type AppErrorKind =
  | "cli_not_found" | "cli_download" | "cli_checksum_mismatch"
  | "cli_unsupported" | "board_not_found" | "invalid_port"
  | "invalid_fqbn" | "compile_failed" | "upload_failed" | "cancelled"
  | "job_not_found" | "sketch_not_found" | "path_not_allowed"
  | "io" | "network" | "invalid_config" | "internal";

export type AppError = {
  kind: AppErrorKind;
  data?: unknown;
};
```

### `JobEvent`

```rust
#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum JobEvent {
    Started { job_id: String, kind: JobKind },
    Progress { job_id: String, stage: String, percent: Option<u8> },
    Output { job_id: String, stream: OutputStream, chunk: String },
    Finished { job_id: String, success: bool },
    Failed { job_id: String, error: AppError },
}
```

```ts
export type JobEvent =
  | { type: "started"; payload: { job_id: string; kind: JobKind } }
  | { type: "progress"; payload: { job_id: string; stage: string; percent?: number } }
  | { type: "output"; payload: { job_id: string; stream: "stdout" | "stderr"; chunk: string } }
  | { type: "finished"; payload: { job_id: string; success: boolean } }
  | { type: "failed"; payload: { job_id: string; error: AppError } };
```

## 2. Configuración

### `AppConfig`

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub cli_path: Option<String>,
    pub cli_version: Option<String>,
    pub theme: Theme,                 // "light" | "dark" | "system"
    pub last_fqbn: Option<String>,
    pub last_port: Option<String>,
    pub autosave: bool,
    pub autosave_delay_ms: u32,       // default 800
    pub log_level: String,            // "error" | "warn" | "info" | "debug"
    pub max_recent_sketches: u32,     // default 20
}
```

### Comandos

| Comando | Entrada | Salida | Descripción |
|---|---|---|---|
| `get_config` | — | `AppConfig` | Lee configuración (crea defaults si no existe). |
| `save_config` | `config: AppConfig` | `AppConfig` | Valida y persiste de forma atómica. |
| `set_cli_path` | `path: Option<String>` | `CliStatus` | Fija/limpia ruta manual y valida el binario y la versión mínima. |
| `get_cli_status` | — | `CliStatus` | Indica si hay CLI, versión y origen. |
| `install_cli` | `channel: Channel<JobEvent>` | `CliInfo` | Descarga, verifica e instala v1.6.0. |

Reglas del instalador:

- Versión fijada: `1.6.0`. Assets desde
  `https://downloads.arduino.cc/arduino-cli/arduino-cli_1.6.0_<SO>_<ARCH>.<ext>`
  (mismo origen que el script oficial de Arduino).
- Checksums desde
  `https://github.com/arduino/arduino-cli/releases/download/v1.6.0/1.6.0-checksums.txt`.
  Si el SHA-256 no coincide se borra el temporal y se devuelve
  `cli_checksum_mismatch`; el binario nunca se ejecuta antes de verificar.
- Arquitecturas: `x86_64` y `ARM64`. En macOS ARM64 se intenta el asset `64bit`
  como respaldo si el nativo no está disponible.
- Emite `started`, `progress` (`descargando`, `verificando checksum`,
  `extrayendo`, `instalando`, `completado`), `output` y `finished`/`failed`.
  `job_id` permite llamar a `cancel_job` durante la descarga.
- Instalación atómica en `<app_data>/cli/arduino-cli[.exe]`; en Unix con permisos
  `0755`. Tras instalar limpia `cli_path` manual para que se use el binario
  gestionado y registra la versión en `AppConfig.cli_version`.
- Versiones manuales `>= 1.6.0` se aceptan (con aviso en logs si son distintas);
  una versión anterior devuelve `cli_unsupported`.

```rust
#[derive(Clone, Debug, serde::Serialize)]
pub struct CliInfo {
    pub path: String,
    pub version: String,
    pub source: CliSource, // "managed" | "manual" | "path"
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct CliStatus {
    pub available: bool,
    pub info: Option<CliInfo>,
    pub required_version: String,
}
```

## 3. Detección de placas

| Comando | Entrada | Salida |
|---|---|---|
| `detect_boards` | — | `Vec<BoardInfo>` |

```rust
#[derive(Clone, Debug, serde::Serialize)]
pub struct BoardInfo {
    pub port: PortInfo,
    pub matching_boards: Vec<BoardCandidate>,
    pub protocol: Option<String>,
    pub protocol_label: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PortInfo {
    pub address: String,          // "COM5", "/dev/ttyACM0"
    pub label: Option<String>,
    pub protocol: Option<String>,
    pub protocol_label: Option<String>,
    pub properties: std::collections::HashMap<String, String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct BoardCandidate {
    pub name: String,
    pub fqbn: Option<String>,
}
```

Reglas:

- Si `detect_boards` devuelve vacío, la UI muestra guía. No es un error del
  comando; es un resultado válido.
- El puerto se valida en Rust antes de subir con
  `^(COM\d+)$` (Windows) o `^/dev/tty[A-Za-z0-9._-]+$` (Unix).

## 4. Compilación

### `CompileRequest`

```rust
#[derive(Clone, Debug, serde::Deserialize)]
pub struct CompileRequest {
    pub sketch_path: String,       // UUID de sketch o ruta absoluta dentro de <app_data>/sketches
    pub fqbn: String,              // "arduino:avr:uno", "esp32:esp32:esp32doit-devkit-v1"
    pub build_properties: Vec<String>, // key=value opcionales
    pub warnings: Option<String>,  // "none" | "default" | "all"
    pub clean: bool,
}
```

### `CompileResult`

```rust
#[derive(Clone, Debug, serde::Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub diagnostics: Vec<Diagnostic>,
    pub memory: Option<MemoryUsage>,
    pub build_dir: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Diagnostic {
    pub severity: Severity,        // "error" | "warning" | "note"
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct MemoryUsage {
    pub flash_used: u64,
    pub flash_max: u64,
    pub flash_percent: f32,
    pub sram_used: u64,
    pub sram_max: u64,
    pub sram_percent: f32,
}
```

| Comando | Entrada | Salida |
|---|---|---|
| `compile` | `request: CompileRequest, events: Channel<JobEvent>` | `CompileResult` |
| `cancel_job` | `job_id: String` | `bool` |

Notas:

- El comando devuelve el resultado final; `events` entrega streaming.
- Si la compilación falla, se devuelve `Err(AppError::CompileFailed { .. })`
  **y** se emite `JobEvent::Failed` para quien solo escuche el canal.
- Se usa `--format json` (o `--json` si la v1.6.0 lo requiere) y se conserva
  stdout/stderr crudos.
- `sketch_path` puede ser un UUID o una ruta absoluta. Rust la resuelve a una
  ruta absoluta lógica y valida que quede dentro de `<app_data>/sketches` antes
  de invocar al CLI; el valor que recibe `arduino-cli` nunca es un UUID relativo.
- `memory` se obtiene del `builder_result.executable_sections_size` del JSON
  (`text`/`data`) o, como respaldo, del texto en inglés o español. El frontend
  debe consumir este campo y no reparsear el `stdout`.
- Si el JSON trae `compiler_out`/`compiler_err`, `stdout`/`stderr` contienen ese
  texto del compilador; el JSON crudo no se expone en el resultado.

## 5. Carga

### `UploadRequest`

```rust
#[derive(Clone, Debug, serde::Deserialize)]
pub struct UploadRequest {
    pub sketch_path: String,       // UUID de sketch o ruta absoluta dentro de <app_data>/sketches
    pub fqbn: String,
    pub port: String,
    pub compile_first: bool,       // false = subir sin recompilar
    pub verify: bool,
    pub programmer: Option<String>,
    pub baudrate: Option<u32>,
    pub build_properties: Vec<String>,
}
```

| Comando | Entrada | Salida |
|---|---|---|
| `upload` | `request: UploadRequest, events: Channel<JobEvent>` | `UploadResult` |
| `cancel_job` | `job_id: String` | `bool` |

```rust
#[derive(Clone, Debug, serde::Serialize)]
pub struct UploadResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub port: String,
    pub fqbn: String,
}
```

`upload` también resuelve `sketch_path` a una ruta absoluta antes de invocar al
CLI. Cuando `compile_first = true`, la salida de compilación incluye el texto de
memoria; el frontend lo usa como respaldo para actualizar las barras porque
`UploadResult` no tiene un campo `memory` estructurado.

## 6. Sketches

| Comando | Entrada | Salida |
|---|---|---|
| `list_sketches` | — | `Vec<SketchSummary>` |
| `read_sketch` | `id: String` | `Sketch` |
| `save_sketch` | `sketch: Sketch` | `SketchSummary` |
| `create_sketch` | `name: String` | `Sketch` |
| `rename_sketch` | `id: String, name: String` | `SketchSummary` |
| `delete_sketch` | `id: String` | `bool` |
| `export_sketch` | `id: String, target_path: String` | `String` |

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Sketch {
    pub id: String,
    pub name: String,
    pub content: String,
    pub fqbn: Option<String>,
    pub updated_at: String,        // ISO-8601 UTC
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct SketchSummary {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub fqbn: Option<String>,
}
```

Reglas:

- `id` generado por Rust (UUID v4) y validado como `[a-f0-9-]{36}`.
- `name` limitado a 120 caracteres; se sanitiza para nombre de archivo.
- Disposición en disco:
  `<app_data>/sketches/<id>/<id>.ino` y
  `<app_data>/sketches/<id>/sketch.json` con `id`, `name`, `fqbn` y
  `updated_at`. No existe `index.json`; `list_sketches` escanea las carpetas,
  ignora las que no tienen `.ino` y ordena por `updated_at`.
- `create_sketch` crea la carpeta, un `.ino` con la plantilla mínima y
  `sketch.json`. `save_sketch` escribe el contenido en el `.ino` existente y
  actualiza el nombre y la fecha en `sketch.json`.
- `export_sketch` exige una ruta destino absoluta cuyo directorio padre exista;
  si el destino es un directorio, escribe `<name>.ino` dentro.
- Todas las rutas se validan para quedar dentro de `<app_data>/sketches`. Si no,
  `PathNotAllowed`.
- Escritura atómica: escribir `.<nombre>.<uuid>.tmp` y renombrar.

## 7. Cores y librerías

| Comando | Entrada | Salida |
|---|---|---|
| `list_cores` | — | `Vec<CoreInfo>` |
| `install_core` | `id: String, events: Channel<JobEvent>` | `CoreInfo` |
| `uninstall_core` | `id: String` | `bool` |
| `list_libraries` | `query: Option<String>` | `Vec<LibraryInfo>` |
| `install_library` | `name: String, version: Option<String>, events: Channel<JobEvent>` | `LibraryInfo` |
| `uninstall_library` | `name: String` | `bool` |

```rust
#[derive(Clone, Debug, serde::Serialize)]
pub struct CoreInfo {
    pub id: String,          // "arduino:avr"
    pub installed: bool,
    pub version: Option<String>,
    pub latest_version: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct LibraryInfo {
    pub name: String,
    pub installed_version: Option<String>,
    pub latest_version: Option<String>,
    pub author: Option<String>,
    pub sentence: Option<String>,
}
```

Reglas:

- `list_cores` usa `core list --format json`; `installed` se deduce de
  `installed_version`.
- `install_core`/`install_library` emiten `JobEvent` y ocupan un job exclusivo:
  no conviven con `compile`, `upload`, `install_cli` ni otra instalación.
- `list_libraries(None)` lista las instaladas con `lib list` y deja
  `latest_version = None`; `list_libraries(Some(query))` usa `lib search`, marca
  `latest_version` con la última release y mezcla `installed_version` de las
  librerías instaladas.
- `install_library` acepta `name` y `version` opcional; construye
  `name@version` (sin shell) y devuelve la entrada instalada.
- `uninstall_core`/`uninstall_library` devuelven `true` si el CLI termina bien.
- Validación: core `^[A-Za-z0-9_-]{1,64}:[A-Za-z0-9_-]{1,64}$`; librería
  `^[A-Za-z0-9][A-Za-z0-9 ._+@-]{0,127}$`; versión
  `^[A-Za-z0-9][A-Za-z0-9.+_-]{0,63}$`. Los fallos del CLI se devuelven como
  `internal` con el `stderr` recortado.

## 8. Errores en el frontend

`api/` traduce cada `AppError.kind` a un mensaje en español y una acción
sugerida:

| kind | Mensaje UI | Acción |
|---|---|---|
| `cli_not_found` | «No se encontró arduino-cli.» | Botón «Descargar» / «Elegir ruta» |
| `cli_checksum_mismatch` | «El binario descargado no es válido.» | Reintentar descarga |
| `board_not_found` | «No se detectó ninguna placa.» | Abrir guía |
| `invalid_port` | «El puerto seleccionado no es válido.» | Volver a detectar |
| `compile_failed` | «La compilación falló.» | Mostrar consola y diagnósticos |
| `upload_failed` | «No se pudo cargar el programa.» | Mostrar consola |
| `cancelled` | «Operación cancelada.» | — |
| `path_not_allowed` | «Ruta no permitida.» | — |
| `io` / `internal` | «Error interno.» | Registrar y mostrar detalle |

## 9. Compatibilidad con Electron (transitorio)

Mientras exista el build dual, `api/` detecta el entorno:

```ts
const isTauri = "__TAURI_INTERNALS__" in window;
```

- En Tauri usa `invoke` y `Channel`.
- En Electron usa `window.api` (legado) y adapta la forma de los datos.
- Al eliminar Electron se borra la rama legada.
