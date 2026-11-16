---
tags: [kor, distribucion, empaquetado, release]
---

# Distribución

Ver ADR-008. Solo se distribuye por **releases de GitHub**; sin tiendas, sin
auto-update y sin versión portable (Q60–Q62).

## 1. Plataformas y formatos

| SO | Arquitectura | Formatos |
|---|---|---|
| Windows 10/11 | x86_64 | `.msi`, `.exe` (NSIS) |
| macOS | x86_64 | `.dmg` |
| Linux | x86_64 | `.deb`, `.rpm`, `.AppImage` |

- Linux objetivo: Debian/Ubuntu, Fedora/openSUSE, Arch y NixOS (vía paquete o
  AppImage).
- Sin ARM64 en esta etapa (Q7).

## 2. Nombres de artefacto

```text
kor_<version>_x64-setup.exe
kor_<version>_x64_en-US.msi
kor_<version>_x64.dmg
kor_<version>_amd64.deb
kor-<version>-1.x86_64.rpm
kor_<version>_amd64.AppImage
SHA256SUMS.txt
```

- `version` en SemVer (p. ej. `0.1.0`).
- Cada release incluye `SHA256SUMS.txt` con todos los artefactos.

## 3. Checksums

```bash
sha256sum kor_0.1.0_amd64.deb >> SHA256SUMS.txt
```

- Publicar y verificar antes de anunciar el release.
- El usuario puede comprobar:

```bash
sha256sum -c SHA256SUMS.txt
```

- En Windows: `certutil -hashfile <archivo> SHA256`.

## 4. Firma de código

- **No** en esta etapa (ADR-007): se publican checksums, no firma.
- macOS: el `.dmg` sin notarizar activa Gatekeeper. Documentar:
  1. Clic derecho → Abrir, o
  2. `xattr -d com.apple.quarantine /Applications/KOR.app`
- Windows: SmartScreen puede advertir; documentar «Más información → Ejecutar
  de todas formas».
- Evolución futura: Authenticode (Windows) y Developer ID + notarización
  (macOS).

## 5. `arduino-cli`

- **No** se incluye en el instalador.
- La app lo descarga y verifica al primer arranque (ADR-004, [[Seguridad]]).
- Si no hay red, se muestra guía y se permite ruta manual.
- El README documenta:
  - descarga automática;
  - cómo elegir una ruta manual;
  - versión mínima 1.6.0;
  - gestión de cores y librerías desde el IDE.

## 6. Proceso de release (manual, sin CI)

1. Actualizar `version` en `package.json`, `src-tauri/Cargo.toml` y
   `tauri.conf.json`.
2. Actualizar `README.md` y notas del release.
3. Ejecutar pruebas locales ([[Estrategia de pruebas]]).
4. Compilar en cada SO objetivo.
5. Generar checksums.
6. Crear el tag `v<version>` y el release en GitHub.
7. Adjuntar artefactos + `SHA256SUMS.txt`.
8. Verificar una instalación limpia en cada SO.

## 7. Instalación limpia (verificación)

- Sin Node, sin Electron, sin `arduino-cli` preinstalado.
- Abrir KOR en cada SO.
- Aceptar/realizar la descarga del CLI.
- Detectar placa, compilar `blink` y cargarlo.
- Confirmar que los datos se guardan en el directorio de datos del usuario
  ([[Modelo de estado y datos]]).
- Desinstalar y comprobar que no quedan procesos ni archivos bloqueados.

## 8. Notas de plataforma

### Windows
- Instalador por usuario (sin admin) si es posible.
- Rutas con espacios deben funcionar (Program Files).
- Antivirus puede tardar en confiar en el binario sin firma.

### macOS
- Mínimo soportado: macOS 12 (P-04 confirmada).
- `.dmg` arrastrar a `/Applications`.
- Gatekeeper: ver sección 4.
- Datos en `~/Library/Application Support/com.loopi.kor`.

### Linux
- `.deb`/`.rpm` con dependencias del WebView (`webkit2gtk`).
- AppImage autocontenido para Arch/NixOS.
- Datos en `~/.local/share/com.loopi.kor`.
- Permisos de puerto serie: grupo `dialout`/`uucp`; documentar.

## 9. Versionado

- SemVer: `MAJOR.MINOR.PATCH`.
- Cambios incompatibles de contrato Tauri → `MAJOR`.
- Cambios de CLI soportado se documentan en el release.

## 10. Rollback

- Mantener el release anterior disponible.
- Si un release falla en un SO, marcar como pre-release y publicar corrección.
- No hay auto-update; el usuario descarga manualmente.

## 11. Documentación para el usuario

El `README.md` final debe incluir:

- Qué es KOR y requisitos.
- Instalación por SO.
- Primer arranque y descarga de `arduino-cli`.
- Aviso de Gatekeeper/SmartScreen.
- Cómo compilar y subir.
- Dónde se guardan los sketches.
- Licencia MIT.
