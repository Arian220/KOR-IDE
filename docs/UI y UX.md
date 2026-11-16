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

```text
┌───────────────────────────────────────────────────────────────┐
│ Barra superior: KOR · [Detectar placas] [Compilar] [Subir]    │
├───────────────────────────────────────────────────────────────┤
│ Pestañas: blink.ino × | motor.ino × | +                       │
├───────────────────────────────┬───────────────────────────────┤
│                               │  Diagrama Arduino UNO (SVG)   │
│  Editor CodeMirror 6          │  - pines interactivos         │
│                               │  - leyenda                    │
│                               │  [Salida/Entrada/PWM]         │
├───────────────────────────────┴───────────────────────────────┤
│ Selector de placa / puerto (cuando aplica)                    │
├───────────────────────────────────────────────────────────────┤
│ Memoria: Flash ▓▓▓░░ 32% · SRAM ▓▓░░░ 18% (si hay datos)      │
├───────────────────────────────────────────────────────────────┤
│ Consola de salida (stdout/stderr, diagnósticos)               │
└───────────────────────────────────────────────────────────────┘
```

- El diagrama SVG conserva la interacción actual (selección de pines y modo).
- Las pestañas muestran archivo activo, indicador `dirty` y cierre.
- La barra de estado puede mostrar «Listo», «Compilando», «Cargando»,
  «Sin placa», «CLI no disponible».

## 3. Flujos

### 3.1 Compilar

1. Usuario pulsa **Compilar**.
2. Botón → «Compilando…» y deshabilitado; se inicia el job.
3. La consola se limpia al iniciar (P-02) y recibe salida en streaming.
4. Al terminar:
   - Éxito: consola con salida, diagnósticos resaltados, memoria si existe.
   - Error: consola en rojo, diagnósticos con enlace a línea, botón habilitado.

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

## 4. Estados de UI

| Estado | Elemento | Comportamiento |
|---|---|---|
| Cargando app | pantalla | Esqueleto/spinner breve |
| CLI ausente | barra | Aviso + «Descargar» / «Elegir ruta» |
| Sin placa | selector | Guía, no bloquea compilar |
| Compilando | botones | Deshabilitar compilar/subir; mostrar cancelar |
| Error compilación | consola + editor | Rojo, diagnósticos clicables |
| Error carga | consola | Rojo con detalle |
| Cancelado | consola | Mensaje neutro |
| Sketch sin guardar | pestaña | Punto/asterisco |
| Tema | raíz | Claro/oscuro/sistema |

## 5. Editor

- CodeMirror 6 local (`codemirror`, `@codemirror/lang-cpp`,
  `@codemirror/theme-one-dark`).
- Numeración, resaltado C++, búsqueda, autocompletado básico.
- Marcadores de diagnóstico: subrayado y gutter; clic lleva a línea/columna.
- Autoguardado con debounce (~800 ms) y guardado al cerrar pestaña.
- Restauración de pestañas y cursor al abrir la app.

## 6. Consola

- Muestra stdout y stderr con prefijo o color distinto.
- Streaming en tiempo real (RF-29).
- Auto-scroll con opción de detener si el usuario sube.
- Límite de memoria (p. ej. 5000 líneas) con recorte del inicio.
- Botón «Copiar» y «Limpiar».
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
| `EditorTabs.vue` | Pestañas y archivo activo |
| `CodeEditor.vue` | CodeMirror y diagnósticos |
| `ArduinoUnoSvg.vue` | Diagrama interactivo y generación de código |
| `BoardSelector.vue` | Detección, lista y selección |
| `MemoryBars.vue` | Flash/SRAM |
| `ConsoleOutput.vue` | Streaming, copiar, limpiar |
| `ActionBar.vue` | Detectar/Compilar/Subir/Cancelar |
| `AppToast.vue` | Avisos no bloqueantes |

## 12. Textos

- Español neutro, sin tecnicismos innecesarios.
- Errores: qué pasó + qué hacer.
- Evitar emojis en la UI nueva salvo que ya existan y se conserven por
  continuidad.
