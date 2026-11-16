---
tags: [kor, seguridad, tauri]
---

# Seguridad

Guía: skill `security-review`. Aplica a las fases 3 y 5 de
[[PLAN_MIGRACION_RUST]].

## 1. Modelo de amenazas

| Activo | Amenaza | Mitigación |
|---|---|---|
| Ejecución de procesos | Inyección de comandos vía puerto/FQBN/rutas | `Command` con argumentos separados; validación con regex; sin shell |
| Sistema de archivos | Path traversal / escritura fuera de `app_data` | Canonicalizar y comprobar prefijo; IDs UUID validados |
| WebView | XSS / recursos remotos | CSP estricta; sin `v-html` con datos de usuario; sin CDNs |
| Descarga del CLI | Binario manipulado | HTTPS + versión fijada + SHA-256 oficial |
| Datos del usuario | Pérdida o corrupción | Escritura atómica; índice reconstruible |
| Logs | Fuga de información | Sin secretos; rutas y datos mínimos |
| Dependencias | Vulnerabilidades | `pnpm audit`, `cargo audit`, lockfiles fijados |

Fuera de alcance: autenticación, multiusuario, red entrante, OTA.

## 2. Capacidades Tauri

`src-tauri/capabilities/default.json` mínimo:

```json
{
  "identifier": "default",
  "description": "Capacidades mínimas de KOR",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

Reglas:

- **No** habilitar plugins `fs`, `shell`, `http` ni `process` para el WebView.
- Todo acceso a disco/procesos ocurre en comandos Rust.
- No `webSecurity: false`; no se abre DevTools en release.
- No se expone `window.__TAURI_INTERNALS__` a contenido remoto (no hay contenido
  remoto).

## 3. Content Security Policy

En `tauri.conf.json` (`app.security.csp`):

```text
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
img-src 'self' data:;
font-src 'self';
connect-src 'self' ipc: http://ipc.localhost;
object-src 'none';
base-uri 'self';
frame-ancestors 'none';
```

- `style-src 'unsafe-inline'` es deuda temporal por estilos inyectados por
  CodeMirror/Vue; documentar y revisar al actualizar dependencias.
- Sin `unsafe-eval`.
- Sin `https:` en `script-src`, `style-src` ni `connect-src` salvo el IPC local.
- Prohibido cargar `esm.sh` u otros CDNs.

## 4. Ejecución de procesos

```rust
let mut cmd = tokio::process::Command::new(&cli_path);
cmd.arg("compile")
   .arg("--fqbn").arg(&fqbn)
   .arg("--format").arg("json")
   .arg(&sketch_path);
```

Reglas:

- Nunca `sh -c`, `cmd /C` ni concatenar cadenas.
- Validar `fqbn` con `^[a-z0-9_-]+:[a-z0-9_-]+(:[a-z0-9_.-]+)?$`.
- Validar `port`:
  - Windows: `^COM\d+$`
  - macOS/Linux: `^/dev/(tty|cu)[A-Za-z0-9._-]+$`
- Las rutas se canonicalizan y deben empezar por `<app_data>/sketches`.
- Cancelación: `child.kill().await` y limpieza; nunca dejar procesos huérfanos.
- Timeout máximo configurable (p. ej. 10 min) para evitar cuelgues.

## 5. Descarga y verificación de `arduino-cli`

- Versión fijada: **1.6.0** (ADR-004).
- Origen: releases oficiales de Arduino en GitHub (`arduino/arduino-cli`).
- Selección por SO/arquitectura (x86_64):
  - Windows: `arduino-cli_1.6.0_Windows_64bit.zip`
  - macOS: `arduino-cli_1.6.0_macOS_64bit.tar.gz`
  - Linux: `arduino-cli_1.6.0_Linux_64bit.tar.gz`
- Verificación: descargar `checksums.txt` oficial, comparar SHA-256 del archivo.
- Si el checksum no coincide: borrar el archivo, `CliChecksumMismatch`, no
  ejecutar.
- Instalación atómica en `<app_data>/bin/arduino-cli[.exe]`, permisos `0755` en
  Unix.
- Sin red y sin CLI: mostrar guía y permitir ruta manual (RF-19).
- No ejecutar el binario hasta verificar.
- Registrar versión y hash en logs (`info`).

## 6. Validación de entrada

| Entrada | Validación |
|---|---|
| `fqbn` | Regex; debe existir en `board listall` antes de compilar (opcional) |
| `port` | Regex por SO; proveniente de `board list` o entrada manual validada |
| `sketch_path` | Canonicalizar; prefijo `app_data/sketches`; sin `..` |
| `sketch.id` | UUID v4 |
| `sketch.name` | 1–120 caracteres; sanitizar `/\:*?"<>|` y control |
| `baudrate` | 300–2 000 000 |
| `build_properties` | `clave=valor`; sin saltos de línea; longitud limitada |
| `target_path` (export) | Canonicalizar; permitir solo rutas elegidas por el usuario vía diálogo |

- Mensajes de error sin rutas internas ni stack traces para el usuario final.
- Logs internos pueden incluir detalle, sin secretos.

## 7. Datos y rutas

- Base: `app_data_dir()` (ADR-005).
- Sketches en `<app_data>/sketches/<uuid>/`.
- Caché y build en `<app_data>/cache/`; se limpian al terminar.
- Logs en `<app_data>/logs/` (ADR-006).
- Sin escritura junto al ejecutable ni en `/Applications`.
- Configuración con escritura atómica y validación de esquema.

## 8. Secretos y configuración

- No hay claves ni tokens en esta aplicación.
- No se deben commitear rutas locales ni datos de usuario.
- `.env` no es necesario; si se añade, se ignora en Git.

## 9. Dependencias

```bash
pnpm audit
cargo audit
```

- Lockfiles versionados (`pnpm-lock.yaml`, `Cargo.lock`).
- Pinear versiones de Tauri y de extensiones de CodeMirror.
- Revisar cambios de dependencias que inyecten estilos o scripts.

## 10. Distribución y firma

- Se publican checksums SHA-256 en cada release (ADR-007).
- macOS `.dmg` sin notarizar en esta etapa (P-05 confirmada): Gatekeeper lo
  bloquea; documentar apertura manual. Plan futuro: firmar y notarizar.
- Windows sin firma Authenticode en esta etapa; SmartScreen puede advertir.
- No se incluye binario de `arduino-cli` en el instalador; se descarga y verifica.

## 11. Checklist previo a release

- [ ] `core:default` y nada más en capabilities.
- [ ] CSP aplicada y sin `unsafe-eval`; sin cargas remotas.
- [ ] Sin `webSecurity: false` ni DevTools en release.
- [ ] Argumentos de proceso separados y validados.
- [ ] Rutas canonicalizadas dentro de `app_data`.
- [ ] SHA-256 del CLI verificado antes de ejecutar.
- [ ] `pnpm audit` y `cargo audit` sin hallazgos críticos.
- [ ] Logs sin secretos ni datos sensibles.
- [ ] Checksums publicados en el release.
