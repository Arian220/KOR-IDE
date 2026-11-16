---
tags: [kor, desarrollo, setup]
---

# Entorno de desarrollo

## 1. Requisitos

| Herramienta | Versión | Notas |
|---|---|---|
| Node.js | LTS (22+) | Solo para el frontend |
| pnpm | 9+ | Gestor oficial (ADR-002) |
| Rust | stable | `rustup` + `rustfmt` + `clippy` |
| Tauri CLI | 2.x | `pnpm tauri` o `cargo tauri` |
| `arduino-cli` | 1.6.0 | Opcional; la app lo busca en `cli/`, configuración manual y `PATH` |
| `cargo-llvm-cov` | última | Cobertura |
| Playwright | última | E2E |

## 2. Dependencias del sistema

### Linux (Debian/Ubuntu)
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

### macOS
```bash
xcode-select --install
```

Requisito mínimo de ejecución: macOS 12 (P-04 confirmada).

### Windows
- Microsoft C++ Build Tools.
- WebView2 (incluido en Windows 10/11).
- Rust MSVC toolchain.

## 3. Instalación

```bash
# pnpm
corepack enable
corepack prepare pnpm@latest --activate

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rustfmt clippy

# Dependencias del proyecto
pnpm install
```

> No usar `npm install` ni `npm ci` (ADR-002). El `package-lock.json` se elimina
> en la fase 1.
>
> `pnpm-workspace.yaml` declara `onlyBuiltDependencies: [electron]` para que
> pnpm 12 permita el postinstall de Electron. Si el binario de Electron no se
> descargó, ejecutar `node node_modules/electron/install.js`.

## 4. Comandos habituales

```bash
# Frontend
pnpm dev              # Vite dev server
pnpm build            # typecheck + build de producción
pnpm test             # stub; Vitest pendiente de configurar
# pnpm test:e2e       # pendiente de configurar con Playwright

# Tauri (desarrollo)
pnpm tauri dev
pnpm tauri build

# Rust
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo llvm-cov --manifest-path src-tauri/Cargo.toml --fail-under-lines 75

# Seguridad
pnpm audit
cargo audit
```

## 5. Variables de entorno

| Variable | Uso |
|---|---|
| `VITE_MOCK_IPC=1` | Planificado para E2E/desarrollo sin Tauri; aún no implementado |
| `KOR_TEST_ARDUINO_CLI=1` | Planificado para habilitar integración con CLI real |
| `KOR_TEST_CLI_PATH` | Planificado: ruta al `arduino-cli` de pruebas |
| `RUST_LOG=debug` | Nivel de log en desarrollo |

## 6. Flujo de desarrollo

1. Leer el requisito y el ADR correspondiente.
2. Leer el skill del área ([[Convenciones y workflow]]).
3. Escribir prueba (TDD en `services`).
4. Implementar.
5. Ejecutar `pnpm test`, `cargo test`, `cargo clippy`, `cargo fmt --check`.
6. Actualizar [[Contratos Tauri]] si cambió la IPC.
7. Actualizar el baúl si cambió el comportamiento.

## 7. Build dual (transitorio)

Durante las fases 1–4:

```bash
pnpm dev              # UI en navegador (sin backend nativo)
pnpm electron:dev     # Vite + Electron en un solo comando
pnpm electron:build   # build de Vite + Electron con dist/
pnpm tauri dev        # UI + backend Rust (desde la fase 3)
```

- Evitar duplicar lógica de negocio en JavaScript.
- El adaptador `src/api` decide entre Tauri y Electron; fuera de él no se usa
  `window.api`.
- Electron se elimina en la fase 6.

## 8. Estructura de trabajo

Ver [[Estructura del proyecto]]. Regla de dependencias:
`commands → services → infra`; el frontend solo habla con `src/api`.

## 9. Depuración

- Rust: `RUST_LOG=debug` y logs en `<app_data>/logs/kor.log`.
- Tauri: DevTools solo en desarrollo; nunca en release.
- Frontend: Vue DevTools en el navegador cuando se usa `VITE_MOCK_IPC=1`.
- Procesos: registrar argumentos (sin secretos) en nivel `debug`.

## 10. Problemas frecuentes

| Síntoma | Causa probable | Solución |
|---|---|---|
| `arduino-cli` no encontrado | No está en `cli/`, ni en la ruta manual, ni en `PATH` | Instalarlo (`~/.local/bin`, Homebrew, WinGet, Scoop, Chocolatey), fijar la ruta desde la app o esperar la descarga automática (pendiente). |
| Permiso denegado en puerto serie (Linux) | Usuario fuera de `dialout` | `sudo usermod -aG dialout $USER` y relogin |
| WebView no carga en Linux | Falta `webkit2gtk` | Instalar dependencias de la sección 2 |
| Build Tauri falla en Windows | Sin MSVC Build Tools | Instalar y reiniciar |
| `pnpm electron:dev` no abre ventana | Falta el binario de Electron | `node node_modules/electron/install.js` |
| E2E inestable | Timeouts/animaciones | Usar `data-testid` y esperas por condición |
| Cobertura < 75 % | Falta cubrir `services` | Añadir tests de parseo/argumentos |

## 11. Variables de datos en desarrollo

- `app_data_dir()` resuelve a la ruta real del usuario (ver
  [[Modelo de estado y datos]]). No hay override por variable todavía.
- El backend Tauri busca `arduino-cli` en este orden: ruta manual de la
  configuración, `<app_data>/cli/arduino-cli[.exe]` y `PATH`/ubicaciones
  conocidas por SO (`~/.local/bin`, `~/.cargo/bin`, Homebrew, WinGet, Scoop,
  Chocolatey, etc.).
- El adaptador Electron (`VITE_ARDUINO_CLI_PATH`) es temporal y se elimina con
  Electron en la fase 6.
- No commitear sketches ni logs generados.
