---
tags: [kor, requisitos]
---

# Requisitos

Derivado de [[PREGUNTAS_ARQUITECTURA]]. Cada requisito tiene un identificador
estable para trazabilidad con [[Estrategia de pruebas]] y [[Contratos Tauri]].

## 1. Identidad del producto

| Campo | Valor |
|---|---|
| Nombre comercial | **KOR** |
| Empresa | Loop-i (herramienta interna, no se aceptan contribuciones externas) |
| Licencia | MIT |
| Plataforma | Escritorio (Windows, macOS, Linux) |
| Idiomas | Solo español |
| Gestor de paquetes | **pnpm** (oficial) |
| Stack | Tauri 2.12.4 + Rust + Vue 3 + TypeScript + Vite |
| Repositorio | GitHub (solo descarga directa de releases) |

## 2. Requisitos funcionales (RF)

### 2.1 Editor y proyectos

| ID | Requisito | Origen |
|---|---|---|
| RF-01 | Editor de código C/C++ basado en CodeMirror 6, empaquetado localmente (sin `esm.sh`). | Q31 |
| RF-02 | El editor debe ofrecer resaltado, numeración, autocompletado básico y tema claro/oscuro. | Q31, Q35 |
| RF-03 | La interfaz se reimplementa en Vue 3 y evoluciona según `docs/img/pantallaprincipal.svg`: header con menús sin barra nativa, acciones en el editor y terminal inferior (ADR-016). | Q29, Q30 |
| RF-04 | Generación de código a partir de pines seleccionados en el diagrama SVG (salida, entrada, PWM), conservando la interacción actual. | Q36 |
| RF-05 | Pestañas para múltiples archivos/sketches abiertos. | Q33 |
| RF-06 | Autoguardado y persistencia del código entre sesiones. | Q32 |
| RF-07 | Gestión local de sketches: crear, abrir, guardar, renombrar y eliminar. | Q32, Q33 |

### 2.2 Integración con `arduino-cli`

| ID | Requisito | Origen |
|---|---|---|
| RF-08 | Detección de placas solo al pulsar un botón (no sondeo periódico). | Q18 |
| RF-09 | Mostrar información completa de cada placa: puerto, FQBN(s) candidatas, modelo, VID/PID cuando exista. | Q19 |
| RF-10 | Si hay varias placas, mostrar selector al usuario antes de compilar/cargar. | Q17 |
| RF-11 | Si no se detectan placas, mostrar guía de diagnóstico (cable, drivers, permisos). | Q20 |
| RF-12 | Compilación con `arduino-cli` usando salida estructurada (`--format json` / `--json` según versión). | Q24 |
| RF-13 | Mostrar salida completa de compilación y carga. | Q22 |
| RF-14 | Resaltar en el editor los `warnings` y errores con archivo:línea:columna. | Q22, Q23 |
| RF-15 | Mantener visibles los indicadores de Flash y SRAM en el panel izquierdo; actualizarlos cuando el CLI los reporte (0 % sin datos). | Q22 (parcial) |
| RF-16 | Compilar sin subir y subir sin recompilar. | Q25 |
| RF-17 | Cancelar compilación o carga en curso. | Q26 |
| RF-18 | Soporte de configuración avanzada de carga: FQBN, programador, baudrate, verificación. | Q28 |
| RF-19 | Configuración de la ruta del CLI desde la interfaz, con persistencia. | Q12 |
| RF-20 | Descarga automática de `arduino-cli` al primer arranque si no está disponible. | Q10, Q13 |
| RF-21 | Verificación de integridad del binario descargado (SHA-256) y selección por SO/arquitectura. | Q53 |
| RF-22 | Gestión de cores/placas: listar, instalar y configurar. | Q14, Q15 |
| RF-23 | Gestión de librerías (Library Manager) dentro del IDE. | Q16 |
| RF-24 | Soporte de USB únicamente; sin OTA por ahora. | Q21 |

### 2.3 Placas objetivo

| ID | Requisito | Origen |
|---|---|---|
| RF-25 | Soporte de varias placas AVR (UNO, Nano, Mega). | Q2 |
| RF-26 | Soporte de cores de terceros y placas personalizadas. | Q2 |
| RF-27 | Soporte de ESP32, ESP8266 y RP2040. | Q2 |
| RF-28 | El FQBN por defecto es configurable; la compilación no debe estar fijada a `arduino:avr:uno`. | Plan original |

### 2.4 Experiencia de ejecución

| ID | Requisito | Origen |
|---|---|---|
| RF-29 | Progreso de compilación/carga en tiempo real mediante eventos o canales Tauri. | Q46 |
| RF-30 | La salida de la operación actual se limpia al iniciar una nueva operación. | Q27 (interpretación) |
| RF-31 | Los errores se muestran de forma explícita; nunca se reporta éxito si el proceso falló. | Plan original |
| RF-32 | Funcionamiento híbrido: las funciones locales operan sin internet; las descargas de CLI/cores requieren red. | Q5 |

## 3. Requisitos no funcionales (RNF)

| ID | Requisito | Origen |
|---|---|---|
| RNF-01 | Eliminar Electron y el runtime Node/Chromium del artefacto final. | Q3 |
| RNF-02 | Reducir tamaño del instalador y consumo de recursos frente a Electron. | Q3 |
| RNF-03 | Arquitectura x86_64 para Windows 10/11, macOS y Linux (`.deb`, `.rpm`, Arch, openSUSE, NixOS). | Q6, Q7 |
| RNF-04 | Solo escritorio; sin soporte táctil ni móvil. | Q9 |
| RNF-05 | Cobertura de pruebas mínima del **75 %** en Rust. | Q56 |
| RNF-06 | Inicio de la aplicación en menos de 2 s en hardware de referencia (objetivo). | Q3 |
| RNF-07 | Sin dependencias remotas en tiempo de ejecución (CSP estricta). | Q50 |
| RNF-08 | Toda operación de proceso usa `Command` con argumentos separados; sin shell. | Plan original |
| RNF-09 | Datos de usuario en el directorio de datos de la aplicación (no junto al binario). | Q44, Q45 |
| RNF-10 | Accesibilidad no requerida en esta versión. | Q37 |
| RNF-11 | Sin CI por ahora; pruebas locales ejecutables con un comando. | Q55 |
| RNF-12 | Sin actualización automática en esta versión. | Q61 |

## 4. Restricciones

- El único hardware físico disponible para pruebas es una placa **ESP32 DevKit**.
  La paridad AVR se validará con salidas de CLI y pruebas manuales cuando haya
  hardware.
- macOS: `.dmg` sin notarización en esta etapa (ver [[Distribución]] y
  [[Decisiones de arquitectura]]); Gatekeeper puede requerir apertura manual.
- La documentación vive en `docs/` (baúl Obsidian) y se versiona en Git
  (P-08 confirmada).

## 5. Fuera de alcance (por ahora)

- Actualizaciones automáticas.
- Sincronización en la nube o cuentas de usuario.
- Programación OTA / conexión por red.
- Internacionalización.
- Publicación en tiendas de aplicaciones.
- Soporte móvil o tablet.
- Contribuciones externas.

## 6. Decisiones confirmadas

Todas las decisiones antes pendientes fueron confirmadas el 2026-11-15 y se
detallan en [[Decisiones de arquitectura]]:

1. Datos de usuario en `app_data_dir()` por SO (ADR-005), no en Documentos ni
   `/Applications`.
2. La salida de procesos se limpia al iniciar una nueva operación; el resultado
   de la última operación se conserva para revisión (P-02).
3. Las barras de Flash/SRAM se muestran cuando el CLI reporta datos (P-03).
4. Se acepta el `.dmg` sin firma/notarización y se documenta Gatekeeper (P-05).
5. macOS mínimo soportado: 12 (P-04).
6. Cores/librerías: instalar y listar, sin resolución de conflictos (P-06).
7. Autoguardado en `<app_data>/sketches` (P-07).
8. `docs/` se versiona en Git (P-08).
9. Paquete `kor`, bundle `com.loopi.kor` (P-09).
10. Logs `info` en release y `debug` en desarrollo (P-10).
