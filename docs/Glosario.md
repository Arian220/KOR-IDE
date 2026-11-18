---
tags: [kor, glosario]
---

# Glosario

| Término | Definición |
|---|---|
| **KOR** | Nombre de la aplicación (antes «Mi IDE Arduino» / Loopi IDE). |
| **Splash** | Pantalla de carga que se muestra al abrir KOR; ejecuta los chequeos iniciales y abre el Hub. |
| **Hub** | Pantalla de inicio de trabajo con acciones y proyectos recientes; sustituye a la apertura directa del editor. |
| **Ventana principal** | Ventana nativa `main` (Hub/IDE); se crea al terminar el Splash con tamaño 1200×760. |
| **Diagnóstico completo** | Acción (`Ctrl+Shift+I`) que repite los chequeos del Splash desde el Hub o el IDE. |
| **FQBN** | Fully Qualified Board Name; identifica placa y core, p. ej. `arduino:avr:uno`. |
| **Sketch** | Proyecto de Arduino; carpeta con un `.ino` del mismo nombre. |
| **Borrador** | Pestaña abierta con `sketchId = null`; aún no se ha guardado como sketch en Rust. |
| **Guardar como** | Acción que crea un sketch nuevo con el nombre indicado y vincula la pestaña a él. |
| **Sketch gestionado** | Sketch interno `KOR Sketch` usado para compilar o subir pestañas que todavía son borradores. |
| **Flash** | Memoria de programa del microcontrolador; se reporta junto a SRAM tras compilar. |
| **Core** | Paquete de soporte de una familia de placas (p. ej. `esp32:esp32`). |
| **CLI** | `arduino-cli`, herramienta de línea de comandos que compila y carga. |
| **Job** | Operación asíncrona (compilar, subir, instalar) con id y cancelación. |
| **Canal / Channel** | Mecanismo de Tauri para enviar eventos de un comando al frontend. |
| **Capability** | Permiso que Tauri concede al WebView. |
| **CSP** | Content Security Policy; restringe qué recursos puede cargar la UI. |
| **`app_data_dir`** | Directorio de datos por usuario que Tauri resuelve según el SO. |
| **ADR** | Architecture Decision Record; registro de decisiones. |
| **MOC** | Map of Content; nota índice del baúl Obsidian. |
| **Build dual** | Etapa temporal con Electron y Tauri conviviendo. |
| **Blink** | Sketch mínimo de parpadeo usado como prueba de humo. |
| **TDD** | Test-Driven Development; prueba primero, luego implementación. |
| **E2E** | Prueba de extremo a extremo de la interfaz. |

## Siglas

| Sigla | Significado |
|---|---|
| RF | Requisito funcional |
| RNF | Requisito no funcional |
| IPC | Inter-Process Communication (Vue ↔ Rust en Tauri) |
| SRAM | Memoria estática del microcontrolador |
| OTA | Over-The-Air (no soportado por ahora) |
| VID/PID | Vendor ID / Product ID de USB |
| SHA-256 | Algoritmo de hash usado para verificar integridad |
| NSIS | Instalador de Windows |
| MSI | Paquete de instalación de Windows |
| DMG | Imagen de disco de macOS |
| AppImage | Formato portable de Linux |
