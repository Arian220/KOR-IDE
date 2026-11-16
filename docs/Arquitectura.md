---
tags: [kor, arquitectura]
---

# Arquitectura

## 1. Resumen

KOR es una aplicación de escritorio construida con **Tauri 2.12.4**. El backend
está en **Rust** y asume todas las operaciones nativas: proceso de `arduino-cli`,
sistema de archivos, descarga y verificación de binarios, configuración, registro
de logs y gestión de trabajos. El frontend es una SPA **Vue 3 + TypeScript +
Vite**, empaquetada dentro del WebView del sistema; conserva la experiencia
visual del IDE actual y usa **CodeMirror 6** como editor.

Regla de oro: **el frontend no toca el sistema operativo directamente**. Solo
invoca comandos Tauri y escucha eventos; no se habilitan plugins de `fs` ni
`shell` en el WebView.

## 2. Principios

1. **Frontera única**: toda capacidad nativa pasa por [[Contratos Tauri]].
2. **Tipos en la frontera**: parsear y validar en Rust; el frontend recibe datos
   tipados, no cadenas crudas.
3. **Errores explícitos**: enums serializables, nunca `unwrap()` en producción.
4. **Operaciones cancelables**: cada compilación/carga es un `Job` con token de
   cancelación.
5. **Estado mínimo compartido**: Rust es dueño de configuración y trabajos; Vue
   mantiene una réplica reactiva para la UI.
6. **Seguridad por defecto**: capacidades mínimas, CSP estricta, sin cargas
   remotas, argumentos separados.
7. **Idiomático**: seguir [[Convenciones y workflow]] y los skills de Rust/Vue.

## 3. Diagrama de contexto

```text
┌──────────────────────────────────────────────────────────────┐
│                        KOR (Tauri)                           │
│                                                              │
│  ┌────────────────────┐        IPC tipado        ┌─────────┐  │
│  │  WebView (Vue 3)   │  invoke / Channel/Event  │  Rust   │  │
│  │  CodeMirror, SVG   │ ◄──────────────────────► │ Backend │  │
│  └────────────────────┘                          └────┬────┘  │
└───────────────────────────────────────────────────────┼──────┘
                                                        │
                     ┌──────────────────────────────────┼──────────────────┐
                     │                                  │                  │
              ┌──────▼──────┐                  ┌────────▼───────┐  ┌───────▼────────┐
              │ arduino-cli │                  │  Sistema de    │  │  Red (HTTPS)   │
              │ (proceso)   │                  │  archivos      │  │  descarga CLI  │
              └─────────────┘                  └────────────────┘  └────────────────┘
```

## 4. Capas

### 4.1 Frontend (Vue)

```text
src/
├── pages/         # composición de pantalla del IDE
├── components/    # editor, consola, selector de placa, diagrama SVG, barras
├── composables/   # useArduino, useEditor, useSketches, useJobs, useConfig
├── stores/        # Pinia: config, editor/tabs, boards, jobs, sketches
├── api/           # cliente de comandos Tauri (adaptador único)
├── types/         # tipos espejo de los contratos Rust
└── assets/        # estilos, tokens, iconos
```

- **`api/`** es el único módulo que importa `@tauri-apps/api`. El resto nunca
  llama `invoke` directamente.
- **Pinia** gestiona estado global. Los composables encapsulan flujos.
- **CodeMirror** se instala por pnpm; no se usan CDN ni `esm.sh`.

Detalle en [[Estructura del proyecto]] y [[UI y UX]].

### 4.2 Backend Rust

```text
src-tauri/src/
├── main.rs            # arranque Tauri, registro de comandos, estado
├── lib.rs             # re-exports y setup
├── commands/          # capa IPC: validación superficial + llamada a services
├── services/          # lógica de dominio (arduino, jobs, config, sketches)
├── infra/             # procesos, archivos, descarga, logging
├── models/            # tipos serializables (DTO) y enums de error
└── state.rs           # AppState compartido
```

- **`commands/`** no contiene lógica de negocio; convierte DTO ↔ dominio y
  devuelve `Result<T, AppError>`.
- **`services/`** es testeable sin Tauri. Aquí viven la construcción de
  argumentos, el parseo de salidas y la orquestación de jobs.
- **`infra/`** encapsula `tokio::process`, `std::fs`, descarga HTTP y logging.
- **`models/`** define los tipos del contrato con `serde`.

Detalle en [[Contratos Tauri]] y [[Modelo de estado y datos]].

## 5. Flujos principales

### 5.1 Compilar

```text
Usuario → Vue (Compilar)
  → api.compile() → invoke("compile", { request })
    → commands::compile
      → services::arduino::compile
        → infra::process::run_arduino_cli(args, job_id)
          ├─ emite Channel: output(chunk) / progress(stage)
          └─ al terminar: parsea --format json
      → CompileResult (diagnósticos, memoria, stdout/stderr)
  → Vue actualiza consola, resalta errores, barras de memoria
```

- El sketch se escribe antes de compilar en el directorio de datos.
- La cancelación dispara `cancel_job(job_id)` y mata el proceso hijo.
- Si el CLI falla, `CompileFailed` incluye stdout/stderr y diagnósticos.

### 5.2 Detectar placas

```text
Compilar/Subir → api.detectBoards() → invoke("detect_boards")
  → services::arduino::detect_boards
    → arduino-cli board list --format json
    → Vec<BoardInfo>
  → Vue: selector si hay >1; guía si hay 0
```

### 5.3 Subir

```text
Vue (Subir) → detectBoards → selección de placa/puerto
  → api.upload({ sketch, fqbn, port, options })
    → validación de puerto (regex por SO)
    → compile + upload (o upload sin recompilar)
    → eventos de progreso y resultado
```

### 5.4 Autoguardado

- Cambios en el editor se debouncean (p. ej. 800 ms) en `useEditor`.
- Vue llama `save_sketch`; Rust escribe de forma atómica (temp + rename).
- Al abrir la app se restauran pestañas y contenido desde el índice de sketches.

## 6. Gestión de procesos y concurrencia

- Comandos Tauri de compilación/carga son `async`.
- Invocación del CLI con `tokio::process::Command` para no bloquear el runtime.
- Operaciones de FS intensivas o bloqueantes usan `spawn_blocking`.
- `AppState`:
  - `config: RwLock<AppConfig>`
  - `cli: RwLock<Option<CliInfo>>`
  - `jobs: Mutex<HashMap<JobId, CancellationToken>>`
- Un `Job` = `{ id, kind: Compile|Upload|InstallCore|InstallLibrary|DownloadCli }`.
- Solo se permite un job de compilación/carga a la vez por ventana; los jobs de
  instalación pueden coexistir con bloqueo de recursos.

## 7. Persistencia

| Dato | Ubicación | Formato |
|---|---|---|
| Configuración | `<app_data>/config/config.json` | JSON |
| Sketches | `<app_data>/sketches/<id>/` | `.ino` + metadatos |
| Índice de sketches | `<app_data>/sketches/index.json` | JSON |
| CLI descargado | `<app_data>/bin/arduino-cli[.exe]` | binario |
| Caché de descargas | `<app_data>/cache/` | temporal |
| Logs | `<app_data>/logs/kor.log` | texto JSON-lines |

`<app_data>` se resuelve con Tauri `app_data_dir()` (ver [[Modelo de estado y datos]]).

## 8. Seguridad

Resumen; detalle en [[Seguridad]].

- Capacidades Tauri mínimas (`core:default` y lo estrictamente necesario).
- Sin plugins `fs` ni `shell` expuestos al WebView.
- CSP estricta sin `unsafe-eval`; sin recursos remotos.
- Descarga de `arduino-cli` por HTTPS, versión fijada y SHA-256 verificado.
- Puerto validado con regex por SO y pasado como argumento separado.
- Rutas de sketches canonicalizadas dentro de `<app_data>/sketches`.

## 9. Observabilidad

- Logging estructurado con `tracing` + `tracing-subscriber`.
- Niveles: `error`, `warn`, `info`, `debug` (configurable).
- Rotación simple por tamaño; retención de N archivos.
- La consola de la UI muestra stdout/stderr del CLI, no logs internos.
- Los errores hacia la UI incluyen un `code` estable y un `message` en español.

## 10. Compatibilidad y transición

- Durante la migración se mantiene una compilación dual: Electron (legado) y
  Tauri (nuevo). Electron se elimina al cerrar la fase 5 del
  [[PLAN_MIGRACION_RUST]].
- El frontend Vue se construye como SPA; Electron puede cargarla temporalmente
  para comparar comportamiento.

## 11. Riesgos arquitectónicos

| Riesgo | Impacto | Mitigación |
|---|---|---|
| `arduino-cli` cambia formatos entre versiones | Parseo frágil | Fijar v1.6.0, parsear `--format json`, fixtures y pruebas |
| WebView por SO difiere (WebKitGTK / WebView2 / WKWebView) | UI inconsistente | Probar en los 3 SO; evitar APIs no soportadas |
| Descarga del CLI bloqueada por red/proxy | App inutilizable | Permitir ruta manual y mensaje claro |
| macOS sin notarización | Gatekeeper bloquea | Documentar apertura manual; firmar en el futuro |
| Compilaciones largas | UI congelada | Streaming + cancelación + `tokio::process` |
| Muchas placas/core | Crecimiento del alcance | Fases: AVR → ESP32 → resto |

## 12. Documentos relacionados

- [[Decisiones de arquitectura]]
- [[Contratos Tauri]]
- [[Modelo de estado y datos]]
- [[Estructura del proyecto]]
- [[Estrategia de pruebas]]
- [[Seguridad]]
