---
tags: [kor, estado, datos, persistencia]
---

# Modelo de estado y datos

Complementa [[Arquitectura]] y [[Contratos Tauri]]. Define qué vive en Rust, qué
en Vue y dónde se persiste.

## 1. Principio

- **Rust**: fuente de verdad de configuración, sketches, jobs y rutas.
- **Vue (composables)**: réplica reactiva para renderizar; se hidrata al
  arrancar y se actualiza con respuestas y eventos.

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

## 3. Estado del frontend

La UI usa Composition API con composables; todavía no hay stores Pinia.

| Composable | Contenido | Persistencia |
|---|---|---|
| `useEditor` | pestañas abiertas, pestaña activa, cursor, `dirty`, `sketchId` | `localStorage` (`kor.session.v1`); contenido definitivo en Rust al guardar |
| `useSketchFiles` | lista de sketches, carga y errores de guardado/exportación | Rust (`list_sketches`, `read_sketch`, `save_sketch`, `create_sketch`) |
| `useArduino` | estado del CLI, placas, puerto seleccionado, job, consola y memoria | efímero |
| `useTheme` | tema claro/oscuro/sistema | `localStorage` (`kor.config.v1`, adaptador temporal) |
| `useToast` | avisos no bloqueantes | efímero |

Reglas:

- `EditorTab` contiene `id` (identificador de pestaña), `sketchId` (sketch en
  Rust o `null`), `name`, `content`, `cursor` y `dirty`.
- `sketchId === null` significa **borrador**. Guardar un borrador pide nombre y
  crea el sketch; «Guardar como» crea uno nuevo; «Guardar todo» guarda todas las
  pestañas.
- La sesión restaura hasta 50 pestañas con su `sketchId` y `dirty`; el contenido
  se conserva aunque la app se cierre, pero el guardado definitivo en Rust es
  explícito (Archivo → Guardar).
- La selección de puerto se pierde al reiniciar (se vuelve a detectar).
- La consola conserva la salida de la última operación hasta que se inicia otra.
- Rust es dueño de configuración, sketches, jobs y rutas. La migración completa
  de la configuración de UI a Rust sigue pendiente: hoy el tema se persiste en
  `localStorage` con el adaptador temporal.

## 4. Rutas de datos

`<app_data>` = Tauri `app_data_dir()` (ADR-005):

| SO | Ruta |
|---|---|
| Windows | `%APPDATA%\com.loopi.kor` |
| macOS | `~/Library/Application Support/com.loopi.kor` |
| Linux | `~/.local/share/com.loopi.kor` |

```text
<app_data>/
├── config.json
├── sketches/
│   └── <uuid>/
│       ├── <uuid>.ino
│       └── sketch.json
├── cli/
│   └── arduino-cli[.exe]
├── logs/
│   ├── kor.log
│   └── kor.log.1
└── tmp/
```

### `config.json`

```json
{
  "cli_path": null,
  "cli_version": null,
  "theme": "system",
  "last_fqbn": null,
  "last_port": null,
  "autosave": true,
  "autosave_delay_ms": 800,
  "log_level": "info",
  "max_recent_sketches": 20
}
```

### `sketches/<uuid>/sketch.json`

```json
{
  "id": "3f1c2a9e-8b4d-4c1a-9f2e-1a2b3c4d5e6f",
  "name": "blink",
  "fqbn": "esp32:esp32:esp32doit-devkit-v1",
  "updated_at": "2026-11-15T21:00:00Z"
}
```

El `.ino` se guarda como `<uuid>.ino`. Si la carpeta ya contiene otro `.ino`
creado manualmente, `read_sketch` lo usa como contenido y `save_sketch` escribe
sobre él. El nombre visible (`name`) no cambia el nombre de la carpeta ni del
archivo.

No existe `index.json`: `list_sketches` recorre `<app_data>/sketches`, ignora
carpetas sin un `.ino` válido, ordena por `updated_at` descendente y sintetiza el
índice en cada llamada.

## 5. Escritura atómica y recuperación

- Cada archivo se escribe primero en `.<nombre>.<uuid>.tmp` dentro del mismo
  directorio, con `fsync`, y luego se renombra al destino.
- En Windows se elimina el destino antes del `rename` (best effort).
- Si la configuración está corrupta, se renombra a `config.json.bad` y se
  regenera con los valores por defecto.
- No hay índice que reconstruir: la lista de sketches se obtiene escaneando las
  carpetas. Una carpeta sin `.ino` se omite.
- `tmp/` está reservado para archivos temporales de la app. `arduino-cli` usa su
  propio directorio de build/cache, no el de la app.

## 6. Limpieza

| Elemento | Momento | Acción |
|---|---|---|
| `*.tmp` de escritura | fin de la escritura o error | eliminar (best effort) |
| `config.json.bad` | al regenerar la configuración | conservar para diagnóstico |
| `kor.log` | al superar 5 MB en el arranque | rotar a `kor.log.1` |
| Sketches | solo acción del usuario | no se borran automáticamente |

## 7. Migración desde el estado actual

- El antiguo `temp_sketch/temp_sketch.ino` no se migra: era efímero y no debía
  estar versionado.
- Al primer arranque no hay sketches en Rust. La UI restaura (o crea) una pestaña
  `blink.ino` con un texto de espera; esa pestaña es un borrador hasta que el
  usuario guarde.
- Al compilar o subir un borrador se crea o reutiliza el sketch gestionado
  `KOR Sketch` como almacenamiento temporal.
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
