---
tags: [kor, adr, decisiones]
---

# Decisiones de arquitectura (ADR)

Registro de decisiones tomadas a partir de [[PREGUNTAS_ARQUITECTURA]]. Formato
ligero: contexto, decisión, consecuencias. Las decisiones **Aceptadas** guían la
implementación. Las decisiones antes pendientes fueron confirmadas por el
responsable humano el 2026-11-15 y se incorporan al cuerpo de decisiones.

---

## ADR-001 — Tauri 2 + Vue 3 (TS) en lugar de Electron

- **Estado:** Aceptada
- **Contexto:** se busca eliminar Node/Chromium, reducir tamaño, mejorar
  seguridad y distribución multiplataforma sin reescribir toda la UI en Rust.
- **Decisión:** usar Tauri v2.12.4 con backend Rust y frontend Vue 3 +
  TypeScript + Vite. El editor seguirá siendo CodeMirror 6.
- **Consecuencias:** se conserva la UI reimplementándola en Vue; el WebView es
  del sistema, con diferencias entre SO que deben probarse.

## ADR-002 — pnpm como gestor oficial

- **Estado:** Aceptada
- **Contexto:** coexisten `package-lock.json` y `pnpm-lock.yaml`.
- **Decisión:** pnpm es el único gestor. Se elimina `package-lock.json` al
  migrar dependencias.
- **Consecuencias:** comandos y documentación usan `pnpm`. No se mezclan
  instalaciones npm/pnpm.

## ADR-003 — Conservar la UI actual, reimplementada en Vue

- **Estado:** Aceptada
- **Contexto:** las respuestas indican «se conserva tal cual» (Q29) y «migrar a
  Vue» (Q30).
- **Decisión:** se conserva el diseño, la disposición y el comportamiento; la
  implementación se reescribe en Vue 3 con Composition API y componentes. No se
  mantiene el HTML/JS inline original.
- **Consecuencias:** se usa el skill `ui-to-vue` para la primera conversión y
  `vue-patterns` para la estructura final.

## ADR-004 — `arduino-cli` se descarga automáticamente y se fija en v1.6.0

- **Estado:** Aceptada
- **Contexto:** el usuario elige descarga automática (Q10, Q13) y versión mínima
  v1.6.0 (Q11).
- **Decisión:** al primer arranque, si no hay CLI configurado, se descarga el
  binario oficial para el SO/arquitectura desde GitHub Releases, se verifica
  SHA-256 y se guarda en `<app_data>/cli`. Se permite configurar una ruta manual.
- **Consecuencias:** requiere red en el primer uso; sin red y sin CLI, la app
  muestra guía y permite ruta manual. La versión queda fijada hasta actualizar
  el contrato.

## ADR-005 — Datos de usuario en el directorio estándar por SO

- **Estado:** Aceptada (corrige la respuesta original)
- **Contexto:** la respuesta indicaba `C:\Documentos`, `home` y `/Applications`.
  Escribir datos junto al ejecutable o en `/Applications` falla por permisos y
  rompe el modelo de instalación.
- **Decisión:** usar `app_data_dir()` de Tauri:
  - Windows: `%APPDATA%\com.loopi.kor`
  - macOS: `~/Library/Application Support/com.loopi.kor`
  - Linux: `~/.local/share/com.loopi.kor`
- **Consecuencias:** rutas estables y escribibles; se documenta en
  [[Modelo de estado y datos]]. Los sketches no se guardan en Documentos salvo
  que el usuario elija exportar.

## ADR-006 — Logs en el directorio de datos, no en el de instalación

- **Estado:** Aceptada (corrige la respuesta original)
- **Contexto:** «en la carpeta del programa» no es escribible en instalaciones
  de Windows/macOS/Linux.
- **Decisión:** `logs/` dentro de `<app_data>`, con `tracing` y rotación.
- **Consecuencias:** los logs sobreviven a actualizaciones y no requieren
  permisos de administrador.

## ADR-007 — Integridad por SHA-256 y firma como conceptos separados

- **Estado:** Aceptada (aclara la respuesta original)
- **Contexto:** la respuesta «firma con shasum256» mezcla checksum y firma de
  código.
- **Decisión:** SHA-256 se usa para verificar descargas e integridad de
  artefactos. La firma/notarización de instaladores es un paso separado que en
  esta etapa **no** se implementa (ver ADR-008).
- **Consecuencias:** se publican checksums en cada release; macOS mostrará aviso
  de Gatekeeper hasta que se firme.

## ADR-008 — Formatos de distribución por SO

- **Estado:** Aceptada con reserva
- **Decisión:**
  - Windows: `.msi` y `.exe` (NSIS).
  - macOS: `.dmg` (sin firma/notarización en esta etapa).
  - Linux: `.deb`, `.rpm`, `.AppImage`.
  - Distribución: solo releases de GitHub.
- **Reserva:** el `.dmg` sin notarizar requiere abrir con clic derecho o
  `xattr -d com.apple.quarantine`. Se documenta en [[Distribución]].

## ADR-009 — Renombrar el producto a KOR

- **Estado:** Aceptada
- **Decisión:** nombre visible **KOR**; identificador de bundle
  `com.loopi.kor`; crate `kor` y paquete npm `kor`. Se actualiza la identidad
  visual de forma incremental sin romper la UI actual.
- **Consecuencias:** main.js/título/README se actualizan al final de la
  migración.

## ADR-010 — Sin CI en esta etapa

- **Estado:** Aceptada
- **Contexto:** Q55 indica que no se requiere CI por ahora.
- **Decisión:** las pruebas se ejecutan en local con un comando documentado. El
  diseño de pruebas debe permitir añadir CI después sin cambios.
- **Consecuencias:** riesgo de regresiones si no se ejecutan localmente; se
  documenta en [[Convenciones y workflow]].

## ADR-011 — Build dual temporal con Electron

- **Estado:** Aceptada
- **Decisión:** durante las fases 1–4 se mantiene Electron como respaldo; la
  fase 5 elimina Electron, `main.js` y `preload.js`.
- **Consecuencias:** el repositorio tendrá dos entrypoints temporales; se debe
  evitar duplicar lógica de negocio en JavaScript.

## ADR-012 — Contrato de errores tipado y streaming por canales

- **Estado:** Aceptada
- **Decisión:** todos los comandos devuelven `Result<T, AppError>` con un enum
  serializable (`#[serde(tag = "kind")]`). El progreso y la salida de procesos se
  envían por `tauri::ipc::Channel` asociado al job.
- **Consecuencias:** el frontend puede discriminar errores por `kind` y mostrar
  mensajes accionables; ver [[Contratos Tauri]].

## ADR-013 — Rust es dueño del estado; Vue mantiene una réplica

- **Estado:** Aceptada
- **Decisión:** configuración, jobs y sketches viven en Rust. Vue mantiene una
  réplica reactiva en composables (los stores Pinia siguen planificados) que se
  sincroniza mediante respuestas de comandos y eventos. No hay escritura directa
  de archivos desde el frontend.
- **Consecuencias:** una sola fuente de verdad; el frontend es reemplazable sin
  perder datos.

## ADR-014 — Uso de skills del proyecto

- **Estado:** Aceptada
- **Decisión:** los skills de `skills/` se usan como guía obligatoria en las
  áreas correspondientes (ver [[PLAN_MIGRACION_RUST]] y
  [[Convenciones y workflow]]).
- **Consecuencias:** revisiones de código deben contrastarse con el skill del
  área.

## ADR-015 — Soporte de placas por fases

- **Estado:** Aceptada
- **Contexto:** se piden AVR, terceros, ESP32/ESP8266/RP2040, pero solo hay una
  placa física ESP32 para pruebas.
- **Decisión:** fase inicial con el CLI genérico y FQBN configurable (sin
  hardcodear UNO); validación funcional con ESP32; AVR y el resto se validan con
  salidas de CLI y pruebas manuales cuando haya hardware.
- **Consecuencias:** el catálogo de placas depende de los cores instalados.

## ADR-016 — Header tipo VS Code y terminal inferior

- **Estado:** Aceptada
- **Contexto:** la pantalla principal evoluciona a partir de
  `docs/img/pantallaprincipal.svg`. Se pide eliminar la barra de título nativa,
  agrupar la identidad y el menú en un header propio, mover las acciones a la
  cabecera del editor y añadir una terminal al pie del editor.
- **Decisión:** la ventana Tauri usa `decorations: false`; el header custom
  incluye logotipo, menús (Archivo, Editar, Placas, Ejecutar, Terminal,
  Ventana y Acerca de), selector de tema y controles de ventana. Las acciones
  Detectar/Compilar/Subir/Cancelar viven a la derecha de las pestañas. La
  terminal sustituye a la consola embebida y permanece visible bajo el editor.
  Las barras de memoria se mueven al pie del panel izquierdo y permanecen
  visibles aunque aún no haya datos del CLI.
- **Consecuencias:** el header es la zona de arrastre y requiere la capacidad
  `core:window:allow-start-dragging`; los controles de ventana necesitan
  `core:window:allow-minimize`, `allow-toggle-maximize` y `allow-close`. El
  respaldo Electron usa `frame: false` e IPC temporal. Se actualizan
  [[UI y UX]], [[Design System]] y [[Estructura del proyecto]].


## ADR-017 — Sketches con UUID, metadatos y guardado explícito

- **Estado:** Aceptada
- **Contexto:** la UI necesita varios archivos con nombre visible, distinguir
  borradores y compilar tanto archivos guardados como pestañas nuevas.
- **Decisión:** cada sketch vive en `<app_data>/sketches/<uuid>/`; el contenido
  se escribe en `<uuid>.ino` (o en el primer `.ino` existente) y los metadatos en
  `sketch.json` (`id`, `name`, `fqbn`, `updated_at`). El `id` es el UUID; el
  nombre visible solo vive en los metadatos. `list_sketches` escanea las carpetas
  y ordena por `updated_at`; no hay `index.json`. En el frontend, una pestaña
  nueva tiene `sketchId = null` y se marca como borrador. «Guardar» en un
  borrador pide nombre y crea el sketch; «Guardar como» crea un sketch nuevo;
  «Guardar todo» guarda todas las pestañas. La sesión (`kor.session.v1`)
  restaura contenido, `sketchId` y `dirty` desde `localStorage`, pero el
  contenido definitivo se persiste en Rust al guardar.
- **Consecuencias:** el nombre en disco es el UUID; para compartir el archivo se
  usa `export_sketch`. No se reconstruye ningún índice: una carpeta sin `.ino`
  se ignora. Compilar un borrador usa el sketch gestionado `KOR Sketch`;
  compilar un archivo guardado sincroniza y usa su propio `sketchId`.

## ADR-018 — Memoria Flash/SRAM desde el resultado estructurado

- **Estado:** Aceptada
- **Contexto:** el backend ya devolvía `CompileResult.memory`, pero el frontend
  lo descartaba e intentaba leer la memoria del `stdout` JSON con expresiones de
  texto, por lo que las barras quedaban en 0.
- **Decisión:** `compile` parsea la salida `--format json` o de texto y devuelve
  `CompileResult.memory` (`MemoryUsage`). El frontend mapea ese objeto a
  `MemoryState` y usa `parsearMemoria` solo como respaldo para `upload` o
  respuestas sin memoria. El respaldo entiende inglés/español y JSON.
- **Consecuencias:** una sola fuente de verdad para la memoria, independiente del
  idioma del CLI. Se mantienen pruebas de parseo JSON y de texto. `UploadResult`
  no incluye memoria: se obtiene del texto de compilación de `compile --upload`.

## ADR-019 — El backend resuelve la ruta absoluta del sketch antes del CLI

- **Estado:** Aceptada
- **Contexto:** al compilar con un UUID de sketch, `arduino-cli` recibía el UUID
  relativo y lo resolvía contra el directorio de trabajo (`src-tauri`), lo que
  producía «no such file or directory».
- **Decisión:** `validate_compile_request` y `validate_upload_request` devuelven
  un `PathBuf` con la ruta lógica absoluta; `compile` y `upload` reemplazan
  `request.sketch_path` antes de construir los argumentos. Se acepta un UUID o
  una ruta absoluta; ambas deben quedar dentro de `<app_data>/sketches`. No se
  canonicaliza para no introducir el prefijo `\\?\` en Windows.
- **Consecuencias:** `arduino-cli` recibe siempre una ruta absoluta
  independiente del cwd. La resolución está cubierta por
  `resuelve_la_ruta_del_sketch_antes_de_compilar`.

---

## Decisiones confirmadas (antes pendientes)

Confirmadas por el responsable humano el 2026-11-15. Se consideran cerradas.

| ID | Tema | Decisión |
|---|---|---|
| P-01 | Rutas de datos | Usar `app_data_dir()` (ADR-005). |
| P-02 | «Borrar al finalizar» (Q27) | La salida se limpia al iniciar una nueva operación, no al terminar; se conserva el resultado para revisión. |
| P-03 | Barras Flash/SRAM (Q22) | Mantenerlas visibles en el panel izquierdo; se actualizan cuando el CLI reporte datos y no bloquean la compilación. |
| P-04 | macOS mínimo | Soportar macOS 12 en adelante. |
| P-05 | `.dmg` sin notarizar | Aceptar el aviso de Gatekeeper y documentarlo. |
| P-06 | Alcance de cores/librerías en v1 | Instalar y listar; no resolver conflictos de versiones. |
| P-07 | Autoguardado | Los sketches se guardan en `<app_data>/sketches`, no en Documentos; la sesión se restaura y el guardado en sketches es explícito. |
| P-08 | Versionado de `docs/` | Quitar `docs/` de `.gitignore` para versionar el baúl. |
| P-09 | Nombre del paquete | `kor` / `com.loopi.kor`. |
| P-10 | Nivel de logs por defecto | `info` en release, `debug` en desarrollo. |

Estas decisiones se reflejan en [[Requisitos]], [[Distribución]],
[[Convenciones y workflow]] y en la implementación de la fase 1.
