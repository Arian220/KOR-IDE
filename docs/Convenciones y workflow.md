---
tags: [kor, convenciones, workflow]
---

# Convenciones y workflow

## 1. Flujo de trabajo

- Rama de trabajo: **`main` directamente** (Q66). No se exige rama de migración.
- Build dual Electron/Tauri durante las fases 1–4; Electron se elimina en fase 6
  (ADR-011).
- Antes de implementar un área, leer el skill correspondiente (ver
  [[PLAN_MIGRACION_RUST]]).
- Toda decisión técnica nueva se anota en [[Decisiones de arquitectura]].
- Toda capacidad o supuesto de datos se refleja en [[Requisitos]] y
  [[Contratos Tauri]].
- No se toca código sin un requisito o decisión que lo respalde.

## 2. Estilo de commits

Formato: `tipo(alcance): descripción en imperativo`

Tipos: `feat`, `fix`, `refactor`, `test`, `docs`, `build`, `chore`.

Ejemplos:

```text
feat(src-tauri): agregar comando detect_boards
test(src-tauri): cubrir parseo de board list vacio
docs(docs): registrar decision de rutas de datos
```

- Commits pequeños y con un propósito.
- No mezclar refactor con cambio de comportamiento.
- No commitear `node_modules/`, `dist/`, `target/` ni artefactos.

## 3. Rust

Guía: skill `rust-patterns`.

### Estilo

- `cargo fmt` obligatorio; `cargo clippy -- -D warnings` sin advertencias.
- `snake_case` para funciones/módulos; `CamelCase` para tipos.
- `pub(crate)` por defecto; exponer solo lo necesario.
- Sin `unwrap()`/`expect()` en producción; sí en tests.
- Errores de biblioteca con `thiserror`; contexto con `anyhow` solo en binarios.
- Enums exhaustivos sin `_` en lógica de negocio.
- Sin `unsafe` salvo FFI justificada con comentario `// SAFETY:`.

### Async y procesos

- `tokio::process::Command` con argumentos separados.
- Nunca `sh -c` ni concatenación de comandos.
- FS bloqueante en `spawn_blocking`.
- Cancelación con `CancellationToken` por job.
- No bloquear el runtime con `std::thread::sleep`.

### Organización

```text
commands/  → services/ → infra/
```

- `services` no importa `tauri`.
- `infra` no importa `commands`.
- DTO en `models`, serializables con `serde`.

## 4. TypeScript / Vue

Guía: skill `vue-patterns`.

- Vue 3.5+, `<script setup lang="ts">`, Composition API.
- Componentes `PascalCase.vue`; composables `useXxx.ts`; stores Setup Store.
- Props tipadas con `defineProps<Props>()`; no mutar props.
- `v-for` con `key` estable (id, no índice).
- `v-if` y `v-for` nunca en el mismo elemento; usar `computed`.
- Estilos `scoped`; tokens CSS para color/espaciado ([[Design System]]).
- No `v-html` con contenido de usuario.
- Acceso a Tauri solo desde `src/api`; el resto usa comandos tipados.
- Sin `any`; sin `@ts-ignore` sin justificación.

### Estructura de SFC

1. imports
2. props/emits/slots
3. composables
4. estado local
5. computed
6. métodos
7. watchers
8. lifecycle

## 5. Documentación

- El baúl `docs/` es la fuente de verdad ([[Inicio]]).
- Enlaces internos con `[[...]]`.
- Fechas ISO-8601; unidades explícitas.
- Cada documento nuevo se enlaza desde el índice.
- `docs/` se versiona en Git (P-08 confirmada).

## 6. Seguridad

Guía: skill `security-review` y [[Seguridad]].

- Sin secretos en el repositorio.
- Validar toda entrada del frontend en Rust.
- Rutas resueltas y validadas dentro de `<app_data>`.
- Sin cargas remotas en runtime (CSP).
- Dependencias auditadas con `pnpm audit` y `cargo audit`.

## 7. Pruebas

Guía: [[Estrategia de pruebas]].

- TDD en `services` (RED → GREEN → REFACTOR).
- Cobertura Rust ≥ 75 %.
- Tests de UI con `data-testid`.
- Antes de cerrar una tarea: `pnpm test`, `cargo test`, `cargo clippy`,
  `cargo fmt --check`.

## 8. Checklist antes de cerrar una tarea

- [ ] Requisito/ADR asociado.
- [ ] Código formateado y sin warnings de clippy.
- [ ] Pruebas nuevas o actualizadas.
- [ ] Contrato actualizado si cambió la IPC.
- [ ] Documento del baúl actualizado si cambió el comportamiento.
- [ ] Sin secretos ni rutas locales hardcodeadas.
- [ ] Sin `esm.sh` ni CDNs.
- [ ] Funciona en el SO de desarrollo; se anota lo pendiente para los demás.

## 9. Skills: regla de uso

| Área | Skill obligatorio |
|---|---|
| Rust (estructura, errores, concurrencia) | `rust-patterns` |
| Pruebas Rust | `rust-testing` |
| Componentes y estado Vue | `vue-patterns` |
| Conversión de UI existente | `ui-to-vue` |
| Tokens y tema | `design-system` |
| Pulido visual | `make-interfaces-feel-better` |
| Patrones frontend conceptuales | `frontend-patterns` |
| E2E | `e2e-testing` |
| Seguridad | `security-review` |

## 10. Herramientas

- Node LTS + pnpm (versión documentada en [[Entorno de desarrollo]]).
- Rust stable + `rustfmt`, `clippy`, `cargo-llvm-cov`.
- Tauri CLI 2.x.
- `arduino-cli` v1.6.0 (o descargado por la app).
