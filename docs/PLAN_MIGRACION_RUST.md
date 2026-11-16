---
tags: [kor, plan, migracion, rust, tauri]
---

# Plan de migración a Rust / Tauri

> Documento de ejecución. Las decisiones están en
> [[Decisiones de arquitectura]]; los requisitos en [[Requisitos]]; la
> arquitectura en [[Arquitectura]].

## 1. Objetivo

Eliminar Electron y trasladar las capacidades nativas a **Rust** con **Tauri
2.12.4**, conservando la experiencia del IDE Arduino actual pero reimplementada
en **Vue 3 + TypeScript**. Al finalizar, el artefacto distribuible no incluye
Node ni Chromium, funciona en Windows, macOS y Linux, y gestiona la detección de
placas, compilación, memoria y carga mediante `arduino-cli` v1.6.0.

## 2. Alcance acordado

- Backend Rust + frontend Vue (ADR-001, ADR-003).
- Placas AVR, de terceros y ESP32/ESP8266/RP2040 con FQBN configurable
  (RF-25 a RF-28).
- Descarga automática y verificada de `arduino-cli` (ADR-004).
- Gestión de cores y librerías (RF-22, RF-23).
- Múltiples pestañas y autoguardado (RF-05, RF-06).
- Distribución por releases de GitHub en `.msi`, `.exe`, `.dmg`, `.deb`, `.rpm`
  y `.AppImage` (ADR-008).
- Sin CI, sin auto-update y sin OTA en esta etapa (ADR-010, RNF-12, RF-24).

## 3. Datos de partida confirmados

- `main.js` (36 líneas) crea la ventana Electron con `webSecurity: false` y
  `openDevTools()` siempre activo.
- `preload.js` (71 líneas) ejecuta `arduino-cli` con `exec`, ruta Windows fija
  `C:\ArduinoCLI\arduino-cli.exe` y target fijo `arduino:avr:uno`.
- `index.html` (527 líneas) contiene toda la UI y ~310 líneas de script inline;
  importa CodeMirror desde `esm.sh`.
- `temp_sketch/` guarda el sketch efímero junto al ejecutable.
- Existen `package-lock.json` y `pnpm-lock.yaml`; el oficial es pnpm (ADR-002).
- No hay pruebas automatizadas.

## 4. Skills del proyecto y cuándo usarlos

Los skills viven en `skills/` y son de uso obligatorio en su área.

| Skill | Uso en la migración | Fases |
|---|---|---|
| `rust-patterns` | Estructura de módulos, errores con `thiserror`, concurrencia, `Arc<RwLock>`, ownership. | 3, 4 |
| `rust-testing` | TDD, unitarias con `rstest`/`proptest`, integración, cobertura 75 % con `cargo llvm-cov`. | 3, 5 |
| `vue-patterns` | Composition API, `<script setup>`, Pinia Setup Stores, composables y router. | 2, 4 |
| `ui-to-vue` | Conversión inicial del `index.html` actual a componentes Vue. | 2 |
| `design-system` | Tokens de color/tipografía/espaciado, tema claro/oscuro, auditoría visual. | 2 |
| `make-interfaces-feel-better` | Pulido de hit areas, transiciones, sombras, tipografía y estados. | 2, 4 |
| `frontend-patterns` | Patrones conceptuales de composición y estado (adaptados; es React-first). | 2 |
| `e2e-testing` | Playwright, Page Object Model, configuración y artefactos. | 5 |
| `security-review` | Checklist de validación, CSP, secretos y dependencias. | 3, 5 |
| `team-builder` | Opcional: revisión paralela de arquitectura/seguridad/UX. | 0, 5 |

Regla: antes de implementar un área, leer el skill correspondiente y contrastar
la revisión con su checklist.

## 5. Fases

### Fase 0 — Documentación y cierre del alcance (esta entrega)

- Crear el baúl de Obsidian con requisitos, arquitectura, ADRs, contratos,
  modelo de datos, pruebas, seguridad, UI, distribución y entorno.
- Confirmar las decisiones de [[Decisiones de arquitectura]] (P-01 a P-10).
- Revisar el plan con `team-builder` si se desea (Arquitectura + Seguridad + UX).

**Criterio de salida:** documentos aprobados por el responsable humano (Q68).

### Fase 1 — Preparación del workspace

- Confirmar pnpm; eliminar `package-lock.json`; no mezclar gestores (ADR-002).
- Inicializar Vite + Vue 3 + TypeScript en `src/` con `<script setup>`.
- Instalar dependencias locales: `codemirror`, `@codemirror/lang-cpp`,
  `@codemirror/theme-one-dark`; eliminar imports de `esm.sh`.
- Mover `arduino-uno.svg` a `src/assets/` y el entrypoint a `index.html` mínimo.
- Retirar `temp_sketch/` del repositorio y añadirlo a `.gitignore` si aún no lo
  está.
- Extraer la lógica del script inline a módulos Vue/composables y definir el
  adaptador `src/api` (aún con implementación Electron).
- Contraste con `ui-to-vue` y `vue-patterns`.

**Criterio de salida:** la UI compila con Vite, conserva todas sus funciones y
no depende directamente de `window.api` fuera de `src/api`.

### Fase 2 — Migración de la interfaz a Vue

- Convertir el diseño actual en componentes (editor, diagrama, consola,
  selector de placa, barras, pestañas) preservando el aspecto (ADR-003).
- Implementar pestañas, autoguardado y persistencia (RF-04, RF-05, RF-06).
- Aplicar `design-system` para tokens y tema claro/oscuro (RF-02, Q35).
- Aplicar `make-interfaces-feel-better` para hit areas, transiciones y foco.
- No añadir funciones nuevas fuera de las acordadas.

**Criterio de salida:** paridad visual y funcional con el IDE actual; la UI
funciona con el adaptador Electron.

> En el entorno de desarrollo actual no hay `arduino-cli` instalado. La UI
> muestra el error «arduino-cli no instalado» en estado, selector y consola, sin
> bloquear el resto de la interfaz. La descarga automática queda pendiente dentro
> de la fase 3.

### Fase 3 — Backend Rust y aplicación Tauri

- Inicializar `src-tauri` (Tauri 2.12.4) con la estructura
  `commands/`, `services/`, `infra/`, `models/` (ADR-001, [[Estructura del proyecto]]).
- Implementar `AppError` y los DTO de [[Contratos Tauri]].
- Implementar `services::arduino` con construcción de argumentos separados
  (`tokio::process::Command`), sin shell ni interpolación (RNF-08, `rust-patterns`).
- Implementar `detect_boards`, `compile`, `upload`, `cancel_job` con streaming
  por `Channel` (RF-08 a RF-18, RF-29).
- Implementar `services::cli_installer`: descarga de v1.6.0, selección por SO,
  verificación SHA-256 y ruta manual (RF-19 a RF-21, ADR-004).
- Implementar configuración, sketches y gestión de cores/librerías
  (RF-07, RF-19, RF-22, RF-23).
- Usar `app_data_dir()` para datos, caché y logs (ADR-005, ADR-006).
- Aplicar `security-review` y [[Seguridad]]: capacidades mínimas, CSP, sin
  plugins `fs`/`shell`, validación de puertos y rutas.
- Los procesos largos usan `tokio` y `spawn_blocking` para FS (ADR-013).

> **Progreso (2026-11-16):** scaffold `src-tauri` con Tauri 2.12.5, iconos,
> CSP/capabilities, `AppError` y DTOs, `infra::process` sin shell,
> `services::arduino` (`detect_boards`, `compile`, `upload`, `cancel_job`),
> configuración y sketches (crear/leer/guardar/renombrar/borrar/exportar).
> `cargo build`, 19 tests unitarios, `cargo fmt --check` y
> `cargo clippy --all-targets -- -D warnings` pasan. Smoke Tauri verificado en
> dev (Vite) y release (assets embebidos): la UI invoca `get_cli_status` por IPC
> y recibe `available=false` sin `arduino-cli`. Pendiente: `cli_installer`,
> cores/librerías y extracción opcional de `services/jobs.rs`.

**Criterio de salida:** cada comando funciona sin la UI y devuelve errores
tipados; los tests unitarios de `services` pasan.

### Fase 4 — Integración Vue ↔ Rust y paridad

- Cambiar `src/api` a `invoke`/`Channel` de Tauri (RF-29).
- Mostrar consola completa, diagnósticos con enlace a línea y barras de memoria
  (RF-13 a RF-15, P-03).
- Implementar cancelación real y limpieza de salida al iniciar una operación
  (RF-17, RF-30, P-02).
- Selector de placa cuando hay varias; guía cuando no hay ninguna
  (RF-09 a RF-11).
- Validar FQBN/puerto y opciones avanzadas de carga (RF-18, RF-24).
- Eliminar rutas y supuestos de Windows; confirmar en los 3 SO (RNF-03).
- Nunca reportar éxito si el proceso falló (RF-31).

**Criterio de salida:** flujos principales funcionan con Tauri, sin Electron
activo, en Windows, macOS y Linux.

### Fase 5 — Pruebas y calidad

- Unitarias Rust: argumentos CLI, parseo de JSON, errores, rutas, puertos
  (`rust-patterns`, `rust-testing`).
- Integración con `arduino-cli` real, gated por variable de entorno; fixtures
  para salidas.
- E2E de UI con Playwright (`e2e-testing`), con IPC simulado y Page Objects.
- Smoke manual por SO con la placa ESP32 y un sketch `blink` (Q57, Q58).
- Cobertura mínima 75 % (`cargo llvm-cov --fail-under-lines 75`, RNF-05).
- Revisión de seguridad final con `security-review`.

**Criterio de salida:** suite verde y criterios de aceptación de [[Requisitos]].

### Fase 6 — Eliminar Electron y publicar

- Quitar `electron` de `package.json` y del lockfile.
- Retirar `main.js`, `preload.js` y scripts/config de Electron.
- Ajustar `tauri.conf.json` para los formatos de ADR-008.
- Generar artefactos y checksums SHA-256 por release.
- Documentar instalación de `arduino-cli` (aunque sea automática) y apertura del
  `.dmg` sin notarizar.
- Actualizar README, nombre KOR e identidad visual (ADR-009).
- Verificar instalación limpia sin Node ni Electron.

**Criterio de salida:** release de GitHub válido en los 3 SO.

## 6. Estado de implementación

| Fase | Estado | Notas |
|---|---|---|
| 0. Documentación | Completada | Baúl en `docs/`; P-01 a P-10 confirmadas. |
| 1. Workspace | Completada | pnpm; Vite 8 + Vue 3.5 + TS; CodeMirror local; UI extraída a `src/`; adaptador `src/api`; SVG en `src/assets`; `temp_sketch/` retirado; `package-lock.json` eliminado. |
| 2. UI Vue | Completada | Componentes de acción, pestañas, selector de placa, consola, memoria, toasts y estado; autoguardado y cursor con adaptador temporal `localStorage`; temas claro/oscuro/sistema. |
| 3. Backend Rust/Tauri | En curso | Scaffold Tauri, `AppError`/DTOs, proceso sin shell, boards/compile/upload/cancel, config, CLI y sketches; 19 tests, clippy/fmt y smoke Tauri. Pendiente installer, cores/librerías y `jobs.rs` opcional. |
| 4. Integración | En curso | `invoke`/`Channel` reales, estado CLI, compilar/subir/cancelar, sketch gestionado y errores tipados. Pendiente diagnósticos con enlace a línea, opciones avanzadas de carga, UI de sketches/cores/librerías y verificación en 3 SO. |
| 5. Pruebas | Pendiente | Cobertura 75 %. |
| 6. Release | Pendiente | Eliminar Electron y publicar. |

## 7. Orden de entrega

1. Fase 0: documentación aprobada.
2. Fase 1: workspace con pnpm + Vite + Vue y adaptador.
3. Fase 2: UI Vue con paridad.
4. Fase 3: backend Rust/Tauri.
5. Fase 4: integración y paridad funcional.
6. Fase 5: pruebas y cobertura.
7. Fase 6: eliminación de Electron y release.

## 8. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| «Migrar a Rust» implicaba reescribir la UI | ADR-001/003: Tauri conserva la UI reimplementada en Vue. |
| Formatos de `arduino-cli` varían | Fijar v1.6.0, `--format json`, fixtures y fallback de parseo. |
| Descarga del CLI falla por red/proxy | Ruta manual + mensaje accionable (RF-19). |
| macOS bloquea el `.dmg` sin notarizar | Documentar apertura manual; firmar en el futuro (ADR-008). |
| Diferencia de WebView entre SO | Pruebas de humo en los 3 SO; evitar APIs no portables. |
| Compilaciones largas congelan la UI | `tokio::process`, streaming y cancelación. |
| Pérdida de datos en sketches | Escritura atómica, índice reconstruible y autoguardado. |
| `docs/` fuera de Git | Resuelto: P-08 confirmada, se versiona el baúl. |
| Umbral de 75 % difícil al inicio | TDD desde `services`; excluir FFI/generado. |

## 9. Criterios de aceptación globales

- [ ] El artefacto no incluye Electron ni Node.
- [ ] Detecta placas y muestra info completa; guía si no hay ninguna.
- [ ] Compila con FQBN configurable y muestra salida/diagnósticos.
- [ ] Cancela compilación y carga.
- [ ] Descarga `arduino-cli` v1.6.0 verificando SHA-256, con ruta manual.
- [ ] Guarda y restaura sketches con autoguardado.
- [ ] Tema claro/oscuro y diagrama interactivo conservados.
- [ ] Funciona en Windows, macOS y Linux.
- [ ] Cobertura Rust ≥ 75 %.
- [ ] Sin cargas remotas en runtime y con CSP estricta.

## 10. Definición de terminado

La migración se considera completa cuando el distribuible no incluye Electron,
los flujos Arduino acordados pasan las pruebas, la configuración funciona desde
una instalación limpia y la documentación del baúl refleja el estado final.
