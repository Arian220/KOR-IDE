---
tags: [kor, estructura, carpetas]
---

# Estructura del proyecto

## 1. Estado actual (fase 2)

```text
loopiIDE/
├── docs/                          # baúl Obsidian
├── skills/                        # guías de ingeniería (local, ignorado)
├── main.js                        # Electron temporal (se elimina en fase 6)
├── preload.js                     # puente window.api temporal
├── index.html                     # entrypoint Vite + tema pre-mount
├── src/
│   ├── api/                       # client.ts, storage.ts, errors.ts
│   ├── components/
│   │   ├── action/                # ActionBar.vue
│   │   ├── base/                  # AppButton.vue, AppToast.vue
│   │   ├── board/                 # BoardSelector.vue
│   │   ├── console/               # ConsoleOutput.vue
│   │   ├── diagram/               # ArduinoUnoSvg.vue, pinCoordinates.ts
│   │   ├── editor/                # CodeEditor.vue, EditorTabs.vue
│   │   ├── layout/                # AppMenuBar.vue
│   │   ├── memory/                # MemoryBars.vue
│   │   └── status/                # StatusBar.vue
│   ├── composables/               # useArduino, useDiagram, useEditor,
│   │                              # useTheme, useToast, useWindowControls
│   ├── pages/IdePage.vue
│   ├── types/                     # app.ts, contract.ts, legacy.d.ts
│   ├── utils/                     # codegen.ts, memory.ts
│   └── assets/                    # arduino-uno.svg, styles/
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── vite.config.mts
├── tsconfig.json
└── dist/                          # build (ignorado)
```

`temp_sketch/` y `package-lock.json` ya no existen.

## 2. Estado objetivo

```text
kor/
├── docs/                          # baúl Obsidian
├── skills/                        # skills de ingeniería (local)
├── src/                           # frontend Vue
│   ├── main.ts
│   ├── App.vue
│   ├── api/                       # único punto de contacto con el backend
│   │   ├── client.ts
│   │   ├── storage.ts             # adaptador temporal (fases 2–3)
│   │   ├── commands.ts
│   │   ├── events.ts
│   │   └── errors.ts
│   ├── components/
│   │   ├── action/                # ActionBar.vue
│   │   ├── base/                  # Button, IconButton, Bar, Tabs, Toast
│   │   ├── editor/                # CodeEditor.vue, EditorTabs.vue
│   │   ├── board/                 # BoardSelector.vue, BoardCard.vue
│   │   ├── console/               # ConsoleOutput.vue (terminal inferior)
│   │   ├── diagram/               # ArduinoUnoSvg.vue
│   │   ├── layout/                # AppMenuBar.vue
│   │   ├── memory/                # MemoryBars.vue
│   │   ├── status/                # StatusBar.vue
│   │   └── sketch/                # SketchList.vue
│   ├── composables/
│   │   ├── useArduino.ts
│   │   ├── useDiagram.ts
│   │   ├── useEditor.ts
│   │   ├── useJobs.ts
│   │   ├── useSketches.ts
│   │   ├── useTheme.ts
│   │   ├── useToast.ts
│   │   ├── useWindowControls.ts
│   │   └── useConfig.ts
│   ├── stores/
│   │   ├── config.ts
│   │   ├── editor.ts
│   │   ├── boards.ts
│   │   ├── jobs.ts
│   │   └── sketches.ts
│   ├── types/
│   │   ├── app.ts                 # estado de UI (tema, pestañas, cursor)
│   │   ├── contract.ts            # espejo de modelos Rust
│   │   ├── legacy.d.ts            # window.api temporal
│   │   └── index.ts
│   ├── utils/
│   │   ├── codegen.ts
│   │   └── memory.ts
│   ├── assets/
│   │   ├── arduino-uno.svg
│   │   └── styles/
│   │       ├── tokens.css
│   │       └── main.css
│   └── pages/
│       └── IdePage.vue
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── state.rs
│       ├── models/
│       │   ├── mod.rs
│       │   ├── config.rs
│       │   ├── error.rs
│       │   ├── board.rs
│       │   ├── compile.rs
│       │   ├── upload.rs
│       │   ├── sketch.rs
│       │   ├── core.rs
│       │   └── library.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── config.rs
│       │   ├── cli.rs
│       │   ├── boards.rs
│       │   ├── compile.rs
│       │   ├── upload.rs
│       │   ├── sketches.rs
│       │   └── cores.rs
│       ├── services/
│       │   ├── mod.rs
│       │   ├── arduino.rs
│       │   ├── jobs.rs
│       │   ├── config.rs
│       │   ├── sketches.rs
│       │   └── cli_installer.rs
│       └── infra/
│           ├── mod.rs
│           ├── process.rs
│           ├── fs.rs
│           ├── download.rs
│           └── logging.rs
├── tests/                         # integración Rust (opcional CLI real)
├── e2e/                           # Playwright
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml          # onlyBuiltDependencies (electron)
├── vite.config.mts
├── tsconfig.json
├── index.html                     # entrypoint mínimo de Vite
├── README.md
└── LICENSE
```

## 3. Correspondencia legado → objetivo

| Legado | Destino | Fase |
|---|---|---|
| `index.html` (UI + script) | `src/` Vue + `src/pages/IdePage.vue` | 1–2 |
| `main.js` | `src-tauri/src/main.rs` + `tauri.conf.json` | 3 |
| `preload.js` | `src-tauri/src/commands/*` + `src/api/*` | 3–4 |
| `arduino-uno.svg` | `src/assets/arduino-uno.svg` | 1 |
| `temp_sketch/` | `<app_data>/sketches` + `cache/build` | 3–4 |
| `window.api` | `invoke`/`Channel` vía `src/api` | 4 |
| `localStorage` (fase 2) | `config.json` + sketches en Rust | 4 |
| Imports `esm.sh` | dependencias pnpm (`codemirror`, `@codemirror/lang-cpp`, `@codemirror/theme-one-dark`) | 1 |

## 4. Reglas de dependencias

```text
commands  → services → infra
commands  → models
services  → models
infra     → models
models    → (sin dependencias de app)
```

- `infra` no conoce `commands` ni Tauri.
- `services` no importa `tauri::*`.
- `commands` solo usa Tauri para IPC y `State`.
- El frontend no importa nada de `src-tauri`.

## 5. Convenciones de nombres

| Elemento | Convención |
|---|---|
| Componentes Vue | `PascalCase.vue` |
| Composables | `useCamelCase.ts` |
| Stores | `useXxxStore` (archivo `xxx.ts`) |
| Módulos Rust | `snake_case.rs` |
| Comandos Tauri | `snake_case` |
| Tipos TS | `PascalCase` |
| Archivos de test | `*_test.rs` / `*.spec.ts` |

Detalle en [[Convenciones y workflow]].
