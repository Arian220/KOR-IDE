---
tags: [kor, pruebas, calidad]
---

# Estrategia de pruebas

Guías: `rust-testing`, `e2e-testing`, `security-review`. Cobertura mínima Rust:
**75 %** (Q56, RNF-05). Sin CI por ahora (ADR-010).

Estado actual (2026-11-16): 35 pruebas unitarias Rust en `services` e
`infra`, más 3 pruebas ignoradas que usan red o el CLI real (`instala_cli_real`,
`lista_cores_real`, `lista_librerias_real`). `cargo fmt --check`,
`cargo clippy --all-targets`, `pnpm typecheck` y `pnpm build` pasan. El frontend
todavía no tiene Vitest configurado (`pnpm test` es un stub) y E2E Playwright
está pendiente.

## 1. Pirámide

```text
        Manual / smoke por SO (placa ESP32)
      E2E Playwright (UI + IPC simulado)
    Integración Rust (arduino-cli real, opcional)
  Unitarias Rust + Unitarias Vue (Vitest)
```

## 2. Pruebas unitarias Rust

Ubicación: módulo `#[cfg(test)]` junto al código y `tests/` para integración.

Qué cubrir:

| Área | Casos |
|---|---|
| Construcción de argumentos | `compile`, `upload`, `board list`, install core; rutas con espacios; FQBN inválido; puerto inválido; resolución de UUID a ruta absoluta. |
| Parseo de `board list` | JSON válido, vacío, placa sin FQBN, múltiples placas, JSON malformado. |
| Parseo de compilación | éxito, error, warnings, memoria (`text`/`data` en JSON), fallback de texto en inglés y español. |
| Errores | mapeo a `AppError`, mensajes estables, `kind` correcto. |
| Rutas | resolución de UUID, rutas absolutas, `PathNotAllowed`, traversal (`../`). |
| Configuración | defaults, merge, validación, escritura atómica, configuración corrupta. |
| Sketches | crear/leer/guardar/renombrar/eliminar/exportar, `.ino` existente, carpeta sin `.ino`, nombres sanitizados. |
| Instalador CLI | selección de asset por SO/arch, checksum ok/mismatch, versión no soportada. |
| Jobs | cancelación, job inexistente, doble compilación. |

Herramientas:

- `rstest` para casos parametrizados.
- `proptest` para invariantes (p. ej., round-trip de config, puertos válidos).
- `mockall` solo cuando la dependencia externa lo justifique; preferir
  integración con fixtures.

Ejemplo de nombres:

```rust
#[rstest]
#[case("COM3", true)]
#[case("COM", false)]
#[case("/dev/ttyUSB0", true)]
#[case("../../etc/passwd", false)]
fn valida_puerto(#[case] port: &str, #[case] valid: bool) { /* ... */ }
```

## 3. Integración Rust

- Archivo: `tests/arduino_cli_test.rs`.
- Gated con `KOR_TEST_ARDUINO_CLI=1` y ruta opcional en `KOR_TEST_CLI_PATH`.
- Sin CLI o sin placa, los tests se omiten (`#[ignore]` o retorno temprano).
- Cubre: `board list --format json`, compilación de un `blink` para el FQBN
  disponible, error de compilación forzado y detección sin placa.
- Nunca descarga el CLI en tests; se usa el instalado o la ruta indicada.

## 4. Pruebas unitarias Vue

- Pendiente configurar Vitest + Vue Test Utils; `pnpm test` aún es un stub.
- Deberán cubrir composables y componentes: `useEditor` (sesión y borradores),
  `useSketchFiles` (guardar/guardar como/guardar todo/abrir), `useArduino`
  (flujo, memoria y errores), `useTheme` y `api/errors`.
- `parsearMemoria` debe probarse con JSON, texto en inglés y texto en español.
- Componentes base y de editor con `data-testid`.

## 5. E2E con Playwright

- `e2e/` con Page Object Model (`e2e-pages/`).
- Se ejecuta contra el dev server de Vite con `src/api` en modo mock
  (`VITE_MOCK_IPC=1`), sin Tauri.
- Flujos:
  1. Cargar la app y ver el editor.
  2. Generar código de pin (salida, entrada, PWM).
  3. Compilar con éxito y ver memoria/consola.
  4. Compilar con error y ver diagnóstico.
  5. Detectar 0 placas → guía; 1 placa → continuar; 2 placas → selector.
  6. Cancelar una compilación.
  7. Guardar un borrador (diálogo de nombre), Guardar como y Guardar todo.
  8. Abrir un sketch guardado y reutilizar la pestaña si ya está abierta.
  9. Restauración de pestañas, `sketchId` y `dirty` al reiniciar.
  10. Tema claro/oscuro.
- Atributos `data-testid` estables; no depender de texto con emojis.

Configuración base del skill `e2e-testing`: HTML + JUnit + JSON reporter, trace
on-first-retry, screenshots/video on failure.

## 6. Prueba de humo manual por SO

Matriz obligatoria antes de release:

| Escenario | Windows | macOS | Linux |
|---|---|---|---|
| Abrir app e instalar CLI | [ ] | [ ] | [ ] |
| Detectar placa ESP32 | [ ] | [ ] | [ ] |
| Compilar `blink` | [ ] | [ ] | [ ] |
| Cargar `blink` | [ ] | [ ] | [ ] |
| Error de compilación visible | [ ] | [ ] | [ ] |
| Sin placa → guía | [ ] | [ ] | [ ] |
| Ruta con espacios | [ ] | [ ] | [ ] |
| CLI no instalado → descarga | [ ] | [ ] | [ ] |

Hardware disponible: ESP32 DevKit (Q57). Para AVR (UNO/Nano/Mega) se valida con
salidas de CLI y se documenta la limitación hasta disponer de placas.

Sketch de referencia (`blink`):

```cpp
void setup() { pinMode(LED_BUILTIN, OUTPUT); }
void loop() {
  digitalWrite(LED_BUILTIN, HIGH); delay(500);
  digitalWrite(LED_BUILTIN, LOW);  delay(500);
}
```

## 7. Cobertura

```bash
cargo llvm-cov --fail-under-lines 75
cargo llvm-cov --html
```

Excluir de cobertura: `main.rs` (arranque), FFI, código generado.
Objetivos: lógica crítica 100 %, API pública 90 %+, general 75 %+.

## 8. Casos de error obligatorios

- CLI no instalado, binario borrado, versión < 1.6.0.
- Descarga interrumpida y checksum inválido.
- Sin permisos de escritura en `app_data`.
- Placa desconectada durante la carga.
- Puerto ocupado o inválido.
- Sketch con ruta con espacios y caracteres no ASCII.
- Timeout de compilación y cancelación.
- JSON de `arduino-cli` inesperado o vacío.

## 9. Fixtures

```text
tests/
├── fixtures/
│   ├── board_list_empty.json
│   ├── board_list_esp32.json
│   ├── board_list_multiple.json
│   ├── compile_ok.json
│   ├── compile_ok_memory.json
│   ├── compile_error.json
│   └── compile_warnings.json
└── arduino_cli_test.rs
```

## 10. Comandos locales

```bash
pnpm test                 # stub; Vitest pendiente de configurar
# pnpm test:e2e           # pendiente de configurar con Playwright
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo llvm-cov --manifest-path src-tauri/Cargo.toml --fail-under-lines 75
```

## 11. Trazabilidad requisitos → pruebas

| Requisito | Prueba |
|---|---|
| RF-01/02 | Vue Editor + E2E tema |
| RF-04 | Unit `useEditor` + E2E generación de pines |
| RF-05/06/07 | Unit `useEditor`/`useSketchFiles` + E2E guardado, borradores y restauración |
| RF-08–11 | Unit parser + E2E selector/guía |
| RF-12–14 | Unit parser + integración + E2E consola |
| RF-15 | Unit `memory_from_json`/`memory_from_text` + Unit `parsearMemoria` + E2E barras |
| RF-16–18 | Unit args + integración |
| RF-19–21 | Unit installer + smoke descarga |
| RF-22/23 | Unit cores/libraries |
| RF-29/30 | Unit `useJobs` + E2E cancelación |
| RF-31 | Unit errores + E2E error |
| RNF-05 | `cargo llvm-cov` |
| RNF-07/08 | Revisión `security-review` + tests de validación |
