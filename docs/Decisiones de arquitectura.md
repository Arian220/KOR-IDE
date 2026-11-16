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
  SHA-256 y se guarda en `<app_data>/bin`. Se permite configurar una ruta manual.
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
- **Decisión:** configuración, jobs y sketches viven en Rust. Vue mantiene
  stores Pinia sincronizadas mediante respuestas de comandos y eventos. No hay
  escritura directa de archivos desde el frontend.
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

---

## Decisiones confirmadas (antes pendientes)

Confirmadas por el responsable humano el 2026-11-15. Se consideran cerradas.

| ID | Tema | Decisión |
|---|---|---|
| P-01 | Rutas de datos | Usar `app_data_dir()` (ADR-005). |
| P-02 | «Borrar al finalizar» (Q27) | La salida se limpia al iniciar una nueva operación, no al terminar; se conserva el resultado para revisión. |
| P-03 | Barras Flash/SRAM (Q22) | Mostrarlas cuando el CLI reporte datos; no bloquean la compilación. |
| P-04 | macOS mínimo | Soportar macOS 12 en adelante. |
| P-05 | `.dmg` sin notarizar | Aceptar el aviso de Gatekeeper y documentarlo. |
| P-06 | Alcance de cores/librerías en v1 | Instalar y listar; no resolver conflictos de versiones. |
| P-07 | Autoguardado | Guardar en `<app_data>/sketches`, no en Documentos. |
| P-08 | Versionado de `docs/` | Quitar `docs/` de `.gitignore` para versionar el baúl. |
| P-09 | Nombre del paquete | `kor` / `com.loopi.kor`. |
| P-10 | Nivel de logs por defecto | `info` en release, `debug` en desarrollo. |

Estas decisiones se reflejan en [[Requisitos]], [[Distribución]],
[[Convenciones y workflow]] y en la implementación de la fase 1.
