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

## 2. Flujo de arranque: Splash → Hub → IDE

La aplicación usa `vue-router` con tres rutas: `/` (Splash), `/hub` (Hub) y
`/ide` (IDE). El arranque usa **dos ventanas nativas** para respetar el tamaño
de cada pantalla:

| Ventana | Etiqueta | Tamaño | Contenido |
|---|---|---|---|
| Splash | `splash` | 600×400 fijo | Pantalla de carga (`/`) |
| Principal | `main` | 1200×760 (mín. 900×600) | Hub (`/#/hub`) e IDE (`/#/ide`) |

En Wayland el compositor ignora `set_position`/`center`, así que no se
redimensiona la ventana del splash: al terminar la carga, Rust crea la ventana
principal con su tamaño final (comando `open_main_window`) y cierra el splash.
Esto también evita el salto de posición al crecer en X11, Windows y macOS.

### 2.1 Splash

- Fondo con degradado (sin imágenes) y texto: **KOR**, **by Loop-i** y el estado
  «Cargando proyecto…».
- No muestra datos ni botones: solo carga en segundo plano y pasa al Hub.
- Chequeos: `arduino-cli`, placas conectadas, preferencias (`kor.config.v1`),
  proyectos recientes y actualizaciones (con límite de 2.5 s).
- El resultado se guarda en `localStorage` (`kor.boot.v1`) para que el Hub lo
  muestre al instante sin repetir la detección de placas.

### 2.2 Hub

- Header propio mínimo (logotipo, menú **Ejecutar → Diagnóstico completo** y
  controles de ventana), sin la barra de menús completa del IDE.
- Panel izquierdo: **Nuevo proyecto** y **Abrir editor**. Aviso de actualización
  si existe.
- Panel derecho: proyectos recientes con nombre, placa (`fqbn`), tipo
  (`Código`/`Blocky`) y fecha.
- **Nuevo proyecto** abre un modal con nombre, placa y tipo. El tipo *Blocky*
  aparece deshabilitado como «Próximamente».
- **Diagnóstico completo** (`Ctrl+Shift+I`) repite los chequeos del splash.

### 2.3 Navegación

- El splash se muestra siempre al abrir la app; al terminar redirige al Hub.
- Desde el IDE se vuelve al Hub con **Archivo → Volver al Hub**.
- No existe retorno al splash.

## 3. Estructura de pantalla

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
- La terminal queda al pie del panel derecho, debajo del editor, y puede
  abrirse o cerrarse desde el menú Terminal.
- Las barras de memoria viven al pie del panel izquierdo y permanecen
  visibles con 0 % hasta que el CLI reporte datos.
- El diagrama SVG conserva la interacción actual (selección de pines y modo).
- Las pestañas muestran archivo activo, indicador de cambios y cierre. Una
  pestaña sin `sketchId` es un borrador y se distingue de un archivo guardado con
  cambios pendientes.
- La barra de estado puede mostrar «Listo», «Compilando», «Cargando»,
  «Sin placa», «CLI no disponible», el estado de guardado y la sesión
  autoguardada.

### 3.1 Menú principal

| Menú      | Acciones                                                                                                                                                                                                 |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Archivo   | Nuevo archivo (`Ctrl+N`), Abrir sketch… (`Ctrl+O`), Guardar (`Ctrl+S`), Guardar como… (`Ctrl+Shift+S`), Guardar todo (`Ctrl+Alt+S`), Cerrar pestaña (`Ctrl+W`), Salir |
| Editar    | Renombrar pestaña (`F2`), Deshacer (`Ctrl+Z`), Rehacer (`Ctrl+Shift+Z`), Cortar (`Ctrl+X`), Copiar (`Ctrl+C`), Pegar (`Ctrl+V`), Buscar (`Ctrl+F`)                                                        |
| Placas    | Detectar placas (`Ctrl+Shift+D`), Subir a la placa (`Ctrl+U`), Configurar placa (FQBN), Información de la placa                                                                                           |
| Ejecutar  | Compilar (`Ctrl+B`), Cancelar operación, Depurar (próximamente), Puntos de interrupción (próximamente)                                                                                                    |
| Terminal  | Abrir terminal (Ctrl + \`), Cerrar terminal (Ctrl + \`), Copiar salida (`Ctrl+Shift+C`), Limpiar terminal (`Ctrl+Shift+L`)                                                                                    |
| Ventana   | Tema (sistema/oscuro/claro), minimizar, maximizar, cerrar                                                                                                                                                |
| Acerca de | Información de KOR (`F1`), Documentación (GitHub), Buscar actualizaciones, Licencia                                                                                                                       |

Los menús se cierran con `Escape` o al hacer clic fuera. Las acciones no
disponibles se muestran deshabilitadas. Los atajos se muestran en una columna a
la derecha de cada acción.

### 3.2 Información de KOR

La acción **Acerca de → Información de KOR** (`F1`) abre un diálogo modal
centrado con:

- logotipo, nombre y descripción de KOR;
- versión y fecha de publicación de esa versión;
- tipo de licencia;
- stack: Tauri 2 + Rust, Vue 3 + TypeScript, Vite, CodeMirror 6 y
  `arduino-cli`;
- datos del sistema operativo: plataforma, versión, arquitectura, idioma y
  resolución de pantalla.

Los datos de la aplicación viven en `src/config/appInfo.ts`; los del sistema se
deducen en el webview y no se envían a ningún servicio.

## 4. Flujos

### 4.1 Compilar

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

### 4.2 Subir

1. Pulsa **Subir**.
2. Se detectan placas (RF-08).
3. 0 placas → guía. 1 placa → continúa. >1 → selector (RF-10).
4. Se muestra «Subiendo a <puerto>…».
5. Resultado: éxito o error con salida completa.

### 4.3 Cancelar

- Durante un job aparece **Cancelar**.
- Al pulsar: se solicita `cancel_job`, se muestra «Cancelando…», y al final
  «Operación cancelada».

### 4.4 Detectar placas

- Botón explícito; no sondeo automático (RF-08).
- Muestra puerto, modelo, FQBN y VID/PID cuando exista (RF-09).
- Sin placa: guía con pasos (cable, drivers, permisos, CLI).

### 4.5 Archivos y sketches

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

## 5. Estados de UI

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

## 6. Editor

- CodeMirror 6 local (`codemirror`, `@codemirror/lang-cpp`,
  `@codemirror/theme-one-dark`).
- Numeración, resaltado C++, búsqueda, autocompletado básico.
- Marcadores de diagnóstico: subrayado y gutter; clic lleva a línea/columna.
- Autoguardado de sesión con debounce (~800 ms) en `localStorage` para restaurar
  pestañas y cursor; el guardado en sketches de Rust es explícito mediante
  Archivo → Guardar / Guardar como / Guardar todo.

## 7. Terminal

- Ocupa el pie del panel del editor y puede abrirse o cerrarse desde el menú
  Terminal (Ctrl + \`); muestra un mensaje de espera mientras no hay actividad.
- Muestra stdout y stderr con prefijo o color distinto.
- Streaming en tiempo real (RF-29).
- Auto-scroll con opción de detener si el usuario sube.
- Límite de memoria (p. ej. 5000 líneas) con recorte del inicio.
- Acciones «Copiar» y «Limpiar terminal» en la cabecera; también accesibles
  desde el menú Terminal.
- Los diagnósticos se muestran además como lista con enlace al editor.

## 8. Temas y tokens

- Tokens en [[Design System]].
- Tema claro/oscuro/sistema persistido en configuración.
- Contraste mínimo 4.5:1 para texto normal.
- Colores de estado: éxito verde, error rojo, advertencia ámbar.

## 9. Accesibilidad

No es requisito en esta versión (Q37). Aun así:

- Controles con `aria-label` cuando el texto no sea visible.
- Foco visible.
- No depender solo del color para estados.

## 10. Responsive

- Solo escritorio (Q9). La ventana mínima debe mantener el editor usable
  (p. ej. 900×600).
- Paneles redimensionables si el diseño actual lo permite; si no, se conserva.

## 11. Microinteracciones

Aplicar `make-interfaces-feel-better`:

- Hit areas ≥ 40×40 px.
- Transiciones explícitas (no `transition: all`).
- Enter/exit suaves; press `scale(0.96)`.
- `font-variant-numeric: tabular-nums` en memoria/porcentajes.
- Sombras y bordes con propósito.

## 12. Componentes principales

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
| `AppPrompt.vue` | Diálogo modal para pedir un nombre o un FQBN |
| `SketchPicker.vue` | Selector modal de sketches guardados |
| `BoardInfoDialog.vue` | Información de la placa seleccionada |
| `AboutKorDialog.vue` | Información de KOR, versión, licencia, stack y sistema |
| `LicenseDialog.vue` | Texto de la licencia MIT |
| `AppToast.vue` | Avisos no bloqueantes |
| `useSketchFiles.ts` | Guardar, guardar como, guardar todo, listar y abrir sketches |
| `useAppInfo.ts` | Metadatos de la app y datos del sistema operativo |
| `useWindowControls.ts` | Puente Tauri/Electron para minimizar, maximizar y cerrar |
| `SplashPage.vue` | Pantalla de carga con degradado; ejecuta el arranque y abre la ventana principal. |
| `HubPage.vue` | Pantalla principal con acciones y proyectos recientes. |
| `HubMenuBar.vue` | Header mínimo del Hub y diagnóstico completo. |
| `router/index.ts` | Rutas `/`, `/hub` e `/ide` con hash history. |
| `useAppBoot.ts` | Orquesta la carga inicial y comparte el resultado con el Hub (`kor.boot.v1`). |
| `useMainWindow.ts` | Invoca `open_main_window` o usa el router como respaldo. |
| `useUpdates.ts` | Comprueba la última release de GitHub. |

## 13. Textos

- Español neutro, sin tecnicismos innecesarios.
- Errores: qué pasó + qué hacer.
- Evitar emojis en la UI nueva salvo que ya existan y se conserven por
  continuidad.

Textos fijos de arranque y Hub:

| Contexto | Texto |
|---|---|
| Splash (título) | `KOR` |
| Splash (subtítulo) | `by Loop-i` |
| Splash (estado) | `Cargando proyecto…` |
| Hub (acción) | `Nuevo proyecto` / `Abrir editor` |
| Hub (panel) | `Proyectos recientes` / `No tienes proyectos recientes todavía.` |
| Modal | `Configurar nuevo proyecto`, `Nombre del proyecto`, `Placa`, `Tipo de proyecto`, `Finalizar` |
| Tipo no disponible | `Blocky (Próximamente)` |
| Menú | `Diagnóstico completo` |
| IDE | `Volver al Hub` |
