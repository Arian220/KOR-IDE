---
tags: [kor, indice, moc]
---

# KOR — Segundo cerebro del proyecto

Este baúl de Obsidian concentra toda la documentación de **KOR** (antes «Mi IDE
Arduino» / Loopi IDE): requisitos, arquitectura, decisiones, contratos, pruebas,
seguridad y operación. Es la fuente de verdad previa a cualquier línea de código.

## Navegación

### Producto y requisitos
- [[Requisitos]] — alcance, requisitos funcionales y no funcionales, criterios de aceptación.
- [[UI y UX]] — flujos, pantallas, editor, pestañas, temas y comportamiento.
- [[Design System]] — tokens, tipografía, espaciado y auditoría visual.
- [[Glosario]] — vocabulario del dominio y siglas.

### Arquitectura
- [[Arquitectura]] — visión general, capas, componentes y flujos.
- [[Decisiones de arquitectura]] — ADRs y decisiones confirmadas.
- [[Contratos Tauri]] — comandos, eventos, tipos y errores entre Vue y Rust.
- [[Modelo de estado y datos]] — estado compartido, configuración y rutas de datos.
- [[Estructura del proyecto]] — organización de carpetas Rust/Vue.

### Plan y ejecución
- [[PLAN_MIGRACION_RUST]] — plan por fases con criterios de salida.
- [[Convenciones y workflow]] — estilo, ramas, commits, uso de skills.

### Calidad y seguridad
- [[Estrategia de pruebas]] — unitarias, integración, E2E y cobertura.
- [[Seguridad]] — modelo de amenazas, capacidades, CSP y verificación del CLI.

### Operación
- [[Distribución]] — empaquetado, formatos, checksums y firma.
- [[Entorno de desarrollo]] — herramientas, instalación y comandos.

## Documentos fuente
- [[PREGUNTAS_ARQUITECTURA]] — cuestionario respondido que originó estas decisiones.

## Reglas del baúl
1. Toda decisión técnica relevante se registra en [[Decisiones de arquitectura]].
2. Todo cambio de alcance se refleja primero en [[Requisitos]].
3. Los contratos entre Vue y Rust viven en [[Contratos Tauri]]; si el código y el
   contrato difieren, el contrato manda hasta que se actualice explícitamente.
4. Los documentos usan enlaces `[[...]]` para mantener la red de conocimiento.
5. `docs/` se versiona en Git (decisión P-08 confirmada); el baúl forma parte
   del repositorio.
