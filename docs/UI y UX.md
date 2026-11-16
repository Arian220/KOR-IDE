---
tags: [kor, ui, ux, vue]
---

# UI y UX

Guías: `ui-to-vue`, `vue-patterns`, `make-interfaces-feel-better`,
`design-system`. La UI actual se conserva en diseño y comportamiento
(ADR-003), pero se reimplementa en Vue 3.

## 1. Principios

1. **Continuidad**: el usuario reconoce la misma pantalla y flujo que el IDE
   actual.
2. **Estado visible**: siempre se sabe si se está compilando, cargando,
   detectando o en error.
3. **Acción segura**: los botones se deshabilitan durante operaciones y no se
   reporta éxito falso.
4. **Mensajes en español**, claros y accionables.
5. **Sin sorpresas**: no se añaden funciones no acordadas.

## 2. Estructura de pantalla

Diseño de referencia: `docs/img/pantallaprincipal.svg` (ADR-016).

```text
┌───────────────────────────────────────────────────────────────┐
│ KOR  Archivo Editar Placas Ejecutar Terminal Ventana Acerca   │
│                                    Tema [Sistema]  [−] [□] [×]│
├───────────────────────────────────────────────────────────────┤
│ Diagrama Arduino UNO (SVG)   │ Pestañas [Detectar][Compilar]   │
│ - pines interactivos         │          [Subir a la placa]     │
│ - leyenda                    ├────────────────────────────────┤
│ - Salida/Entrada/PWM         │ Selector de placa / puerto     │
│                              ├────────────────────────────────┤
│                              │ Editor CodeMirror 6            │
│                              ├────────────────────────────────┤
│ Memoria Flash/SRAM (siempre) │ Terminal                       │
│ 0% hasta compilar            │ stdout/stderr · Copiar · Limpiar│
├───────────────────────────────────────────────────────────────┤
│ Barra de estado                                               │
└───────────────────────────────────────────────────────────────┘
```

- La ventana no usa la barra de título nativa (`decorations: false`); el
  header propio actúa como zona de arrastre y expone minimizar, maximizar y
  cerrar.
- El header integra el logotipo, los menús, el selector de tema y los
  controles de ventana.
- Los botones de acción viven en la cabecera del editor, a la derecha de las
  pestañas.
- La terminal queda fija al pie del panel derecho, debajo del editor.
- Las barras de memoria viven al pie del panel izquierdo y permanecen
  visibles con 0 % hasta que el CLI reporte datos.
- El diagrama SVG conserva la interacción actual (selección de pines y modo).
- Las pestañas muestran archivo activo, indicador de cambios y cierre. Una
  pestaña sin `sketchId` es un borrador y se distingue de un archivo guardado con
  cambios pendientes.
- La barra de estado puede mostrar «Listo», «Compilando», «Cargando»,
  «Sin placa», «CLI no disponible», el estado de guardado y la sesión
  autoguardada.

### 2.1 Menú principal

| Menú | Acciones |
|---|---|
| Archivo | Nuevo archivo (`Ctrl+N`), Abrir sketch… (`Ctrl+O`), Guardar (`Ctrl+S`), Guardar como… (`Ctrl+Shift+S`), Guardar todo (`Ctrl+Alt+S`), Cerrar pestaña (`Ctrl+W`), Salir |
| Editar | Renombrar pestaña, copiar código |
| Placas | Detectar placas, subir a la placa |
| Ejecutar | Compilar, cancelar operación |
| Terminal | Copiar salida, limpiar terminal |
| Ventana | Tema (sistema/oscuro/claro), minimizar, maximizar, cerrar |
| Acerca de | Información de la versión |

Los menús se cierran con `Escape` o al hacer clic fuera. Las acciones no
disponibles se muestran deshabilitadas. Los atajos se muestran en una columna a
la derecha de cada acción.

## 3. Flujos

### 3.1 Compilar

1. Usuario pulsa **Compilar**.
2. Las pestañas guardadas se sincronizan con su sketch; los borradores usan el
   sketch gestionado `KOR Sketch`.
3. Botón → «Compilando…» y deshabilitado; se inicia el job.
4. La terminal se limpia al iniciar (P-02) y recibe salida en streaming.
5. Al terminar:
   - Éxito: terminal con salida, diagnósticos resaltados, memoria actualizada.
   - Error: terminal en rojo, diagnósticos con enlace a línea, botón habilitado.

La memoria Flash/SRAM se toma de `CompileResult.memory` (JSON estructurado del
CLI); si no está disponible, se usa el parseo de texto como respaldo.

### 3.2 Subir

1. Pulsa **Subir**.
2. Se detectan placas (RF-08).
3. 0 placas → guía. 1 placa → continúa. >1 → selector (RF-10).
4. Se muestra «Subiendo a <puerto>…».
5. Resultado: éxito o error con salida completa.

### 3.3 Cancelar

- Durante un job aparece **Cancelar**.
- Al pulsar: se solicita `cancel_job`, se muestra «Cancelando…», y al final
  «Operación cancelada».

### 3.4 Detectar placas

- Botón explícito; no sondeo automático (RF-08).
- Muestra puerto, modelo, FQBN y VID/PID cuando exista (RF-09).
- Sin placa: guía con pasos (cable, drivers, permisos, CLI).

### 3.5 Archivos y sketches

- **Nuevo archivo**: crea una pestaña borrador (`sketchId = null`) con nombre
  temporal y contenido vacío.
- **Abrir sketch…**: lista los sketches guardados en Rust y reutiliza la pestaña
  si el sketch ya está abierto.
- **Guardar**:
  - Archivo guardado: sincroniza contenido y nombre con su sketch.
  - Borrador: abre un diálogo para elegir nombre; al confirmar se crea el
    sketch y la pestaña deja de ser borrador.
- **Guardar como…**: crea un sketch nuevo con el nombre indicado y vincula la
  pestaña al nuevo `sketchId`.
- **Guardar todo**: guarda todas las pestañas; los borradores conservan su
  nombre actual.
- **Cerrar pestaña**: si hay cambios sin guardar, pide confirmación antes de
  descartarlos.
- Al iniciar la app se restaura la sesión (pestañas, `sketchId`, contenido y
  estado `dirty`) desde `localStorage`; el contenido definitivo vive en Rust
  solo cuando el usuario guarda.

## 4. Estados de UI

| Estado | Elemento | Comportamiento |
|---|---|---|
| Cargando app | pantalla | Esqueleto/spinner breve |
| CLI ausente | barra | Aviso + «Descargar» / «Elegir ruta» |
| Sin placa | selector | Guía, no bloquea compilar |
| Compilando | botones | Deshabilitar compilar/subir; mostrar cancelar |
| Error compilación | terminal + editor | Rojo, diagnósticos clicables |
| Error carga | terminal | Rojo con detalle |
| Cancelado | terminal | Mensaje neutro |
| Borrador sin guardar | pestaña | Indicador ● y título «Borrador sin guardar» |
| Cambios sin guardar | pestaña | Indicador ● y título «Cambios sin guardar» |
| Tema | raíz | Claro/oscuro/sistema |

## 5. Editor

- CodeMirror 6 local (`codemirror`, `@codemirror/lang-cpp`,
  `@codemirror/theme-one-dark`).
- Numeración, resaltado C++, búsqueda, autocompletado básico.
- Marcadores de diagnóstico: subrayado y gutter; clic lleva a línea/columna.
- Autoguardado de sesión con debounce (~800 ms) en `localStorage` para restaurar
  pestañas y cursor; el guardado en sketches de Rust es explícito mediante
  Archivo → Guardar / Guardar como / Guardar todo.

## 6. Terminal

- Ocupa el pie del panel del editor y permanece visible con un mensaje de
  espera mientras no hay actividad.
- Muestra stdout y stderr con prefijo o color distinto.
- Streaming en tiempo real (RF-29).
- Auto-scroll con opción de detener si el usuario sube.
- Límite de memoria (p. ej. 5000 líneas) con recorte del inicio.
- Acciones «Copiar» y «Limpiar terminal» en la cabecera; también accesibles
  desde el menú Terminal.
- Los diagnósticos se muestran además como lista con enlace al editor.

## 7. Temas y tokens

- Tokens en [[Design System]].
- Tema claro/oscuro/sistema persistido en configuración.
- Contraste mínimo 4.5:1 para texto normal.
- Colores de estado: éxito verde, error rojo, advertencia ámbar.

## 8. Accesibilidad

No es requisito en esta versión (Q37). Aun así:

- Controles con `aria-label` cuando el texto no sea visible.
- Foco visible.
- No depender solo del color para estados.

## 9. Responsive

- Solo escritorio (Q9). La ventana mínima debe mantener el editor usable
  (p. ej. 900×600).
- Paneles redimensionables si el diseño actual lo permite; si no, se conserva.

## 10. Microinteracciones

Aplicar `make-interfaces-feel-better`:

- Hit areas ≥ 40×40 px.
- Transiciones explícitas (no `transition: all`).
- Enter/exit suaves; press `scale(0.96)`.
- `font-variant-numeric: tabular-nums` en memoria/porcentajes.
- Sombras y bordes con propósito.

## 11. Componentes principales

| Componente | Responsabilidad |
|---|---|
| `IdePage.vue` | Composición general y layout |
| `AppMenuBar.vue` | Header sin barra nativa, menús, tema y controles de ventana |
| `EditorTabs.vue` | Pestañas y archivo activo |
| `CodeEditor.vue` | CodeMirror y diagnósticos |
| `ArduinoUnoSvg.vue` | Diagrama interactivo y generación de código |
| `BoardSelector.vue` | Detección, lista y selección |
| `MemoryBars.vue` | Flash/SRAM |
| `ConsoleOutput.vue` | Terminal inferior: streaming, copiar, limpiar |
| `ActionBar.vue` | Detectar/Compilar/Subir/Cancelar |
| `AppDialog.vue` | Diálogo modal base para confirmaciones y avisos |
| `AppPrompt.vue` | Diálogo modal para pedir un nombre (Guardar como) |
| `SketchPicker.vue` | Selector modal de sketches guardados |
| `AppToast.vue` | Avisos no bloqueantes |
| `useSketchFiles.ts` | Guardar, guardar como, guardar todo, listar y abrir sketches |
| `useWindowControls.ts` | Puente Tauri/Electron para minimizar, maximizar y cerrar |

## 12. Textos

- Español neutro, sin tecnicismos innecesarios.
- Errores: qué pasó + qué hacer.
- Evitar emojis en la UI nueva salvo que ya existan y se conserven por
  continuidad.
