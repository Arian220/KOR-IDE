---
tags:
  - kor
  - diseno
  - tokens
  - design-system
---

# Design System

Guía: skill `design-system`. Objetivo: conservar la identidad visual actual
(ADR-003) y formalizarla en tokens para los temas claro y oscuro. Los valores
base se extraen de `index.html`.

## 1. Principios visuales

- Estética de IDE: superficie oscura, alto contraste en el código, acentos
  funcionales.
- El color comunica estado (azul = compilar, naranja = subir, verde = éxito,
  rojo = error).
- Jerarquía por superficie y borde, no por gradientes ni glassmorphism.
- Sin decoración que compita con el código.

## 2. Tokens de color

### Tema oscuro (base actual)

| Token | Valor | Uso |
|---|---|---|
| `--kor-bg` | `#1e1e1e` | Fondo de app |
| `--kor-surface` | `#252526` | Paneles y tarjetas |
| `--kor-surface-2` | `#2d2d2d` | Controles y subpaneles |
| `--kor-surface-3` | `#333333` | Tooltips y elevación |
| `--kor-border` | `#555555` | Bordes |
| `--kor-border-subtle` | `#444444` | Separadores |
| `--kor-text` | `#d4d4d4` | Texto principal |
| `--kor-text-strong` | `#ffffff` | Texto destacado |
| `--kor-text-muted` | `#a0a0a0` | Texto secundario |
| `--kor-accent` | `#007acc` | Acción primaria (compilar) |
| `--kor-accent-hover` | `#005999` | Hover primario |
| `--kor-run` | `#e67e22` | Acción de carga (subir) |
| `--kor-run-hover` | `#d35400` | Hover de carga |
| `--kor-success` | `#4fc08d` | Éxito / memoria OK |
| `--kor-danger` | `#e74c3c` | Error / memoria alta |
| `--kor-danger-text` | `#ff5555` | Texto de error |
| `--kor-warning` | `#e6b450` | Advertencias |
| `--kor-console-bg` | `#111111` | Consola |
| `--kor-focus` | `#00ffcc` | Anillo de foco |

### Tema claro

| Token | Valor | Uso |
|---|---|---|
| `--kor-bg` | `#f5f6f7` | Fondo de app |
| `--kor-surface` | `#ffffff` | Paneles |
| `--kor-surface-2` | `#eceef0` | Controles |
| `--kor-surface-3` | `#e2e5e8` | Elevación |
| `--kor-border` | `#c9ced3` | Bordes |
| `--kor-border-subtle` | `#dde1e4` | Separadores |
| `--kor-text` | `#1f2328` | Texto principal |
| `--kor-text-strong` | `#0b0d0f` | Texto destacado |
| `--kor-text-muted` | `#5b626a` | Texto secundario |
| `--kor-accent` | `#007acc` | Acción primaria |
| `--kor-accent-hover` | `#005999` | Hover |
| `--kor-run` | `#d35400` | Acción de carga |
| `--kor-run-hover` | `#a84300` | Hover |
| `--kor-success` | `#2e9e6b` | Éxito |
| `--kor-danger` | `#c0392b` | Error |
| `--kor-warning` | `#b7791f` | Advertencias |
| `--kor-console-bg` | `#f0f1f2` | Consola |

Contraste objetivo: ≥ 4.5:1 en texto normal y ≥ 3:1 en texto grande/UI.

## 3. Tipografía

| Token | Valor | Uso |
|---|---|---|
| `--kor-font-ui` | `"Segoe UI", Tahoma, Geneva, Verdana, sans-serif` | Interfaz |
| `--kor-font-mono` | `"Courier New", Courier, monospace` | Editor y consola |
| `--kor-text-xs` | `12px` | Metadatos |
| `--kor-text-sm` | `13px` | Controles |
| `--kor-text-md` | `14px` | Base |
| `--kor-text-lg` | `16px` | Títulos de panel |
| `--kor-text-xl` | `20px` | Título de app |
| `--kor-leading` | `1.5` | Interlineado |
| `--kor-weight-normal` | `400` | Base |
| `--kor-weight-medium` | `500` | Énfasis |
| `--kor-weight-bold` | `600` | Títulos |

- Números dinámicos (memoria, porcentajes) con `font-variant-numeric:
  tabular-nums`.
- `text-wrap: balance` en títulos cortos; `pretty` en textos cortos.

## 4. Espaciado y radios

Escala de 4 px:

| Token | Valor |
|---|---|
| `--kor-space-1` | `4px` |
| `--kor-space-2` | `8px` |
| `--kor-space-3` | `12px` |
| `--kor-space-4` | `16px` |
| `--kor-space-5` | `20px` |
| `--kor-space-6` | `24px` |
| `--kor-titlebar-height` | `40px` |
| `--kor-terminal-height` | `200px` |
| `--kor-radius-sm` | `3px` |
| `--kor-radius-md` | `4px` |
| `--kor-radius-lg` | `8px` |
| `--kor-radius-pill` | `999px` |

Regla de radio concéntrico: `radio_exterior = radio_interior + padding`.

## 5. Sombras y bordes

| Token | Valor |
|---|---|
| `--kor-shadow-sm` | `0 1px 2px rgba(0,0,0,.3)` |
| `--kor-shadow-md` | `0 4px 6px rgba(0,0,0,.5)` |
| `--kor-shadow-focus` | `0 0 0 2px var(--kor-focus)` |

- Bordes para separación; sombras para elevación (tooltip, modal, menú).
- Foco siempre visible en controles.

## 6. Movimiento

| Token | Valor |
|---|---|
| `--kor-dur-fast` | `120ms` |
| `--kor-dur-base` | `150ms` |
| `--kor-dur-slow` | `220ms` |
| `--kor-ease-out` | `cubic-bezier(.2,.8,.2,1)` |

- Transiciones explícitas (`background-color`, `transform`, `box-shadow`).
- Prohibido `transition: all`.
- Press `scale(0.96)` en botones; respetar `prefers-reduced-motion`.

## 7. Componentes base

| Componente | Especificación |
|---|---|
| `Button` | Altura 32 px, padding `6px 12px`, radio 4 px, foco visible, disabled 50 % |
| `IconButton` | Hit area ≥ 40×40 px, icono 18 px, centrado óptico |
| `Tabs` | Altura 36 px, activo con borde inferior de acento, cierre 16 px |
| `Panel` | Fondo `surface`, radio 8 px, padding 20 px, borde sutil |
| `Bar` | Altura 10 px, fondo `surface-3`, relleno acento/verde/rojo |
| `MenuBar` | Altura `--kor-titlebar-height`, fondo `surface`, borde inferior `surface-3`, zona de arrastre |
| `Menu` | Trigger de altura completa, dropdown `surface-2`, borde `border`, sombra `md`, ítems 30 px |
| `WindowControl` | Hit area 40×40 px, icono 10 px, hover `surface-3`, cierre hover `danger` |
| `Terminal` | Altura `--kor-terminal-height`, fondo `console-bg`, cabecera 34 px, mono 13 px |
| `Toast` | Fondo `surface-3`, sombra `md`, auto-cierre 4 s |
| `Select` | Altura 32 px, fondo `surface-2`, borde `border` |

## 8. Estados

- Hover: oscurecer/aclarar 10 %.
- Active: `scale(0.96)` y cambio de tono.
- Focus: anillo de 2 px con `--kor-focus`.
- Disabled: opacidad 50 %, cursor `not-allowed`.
- Loading: spinner inline y texto de estado.

## 9. Tema

- `data-theme="dark" | "light"` en `<html>`.
- `system` sigue `prefers-color-scheme`.
- Persistido en `AppConfig.theme`.
- Sin parpadeo: aplicar tema antes de montar la app.

## 10. Checklist de auditoría

- [ ] Colores usados provienen de tokens, no hex sueltos.
- [ ] Jerarquía tipográfica clara y consistente.
- [ ] Espaciado en múltiplos de 4 px.
- [ ] Componentes similares lucen similares.
- [ ] Tema claro completo (no solo invertir colores).
- [ ] Transiciones con propósito, sin `transition: all`.
- [ ] Contraste ≥ 4.5:1.
- [ ] Hit areas ≥ 40×40 px.
- [ ] Estados hover/active/focus/disabled/loading definidos.
- [ ] Sin patrones genéricos (gradientes gratuitos, glassmorphism).

## 11. Salidas esperadas (skill `design-system`)

- `src/assets/tokens.css` con los tokens anteriores.
- `DESIGN.md` es este documento.
- `design-preview.html` opcional para revisar tokens antes de implementar.
