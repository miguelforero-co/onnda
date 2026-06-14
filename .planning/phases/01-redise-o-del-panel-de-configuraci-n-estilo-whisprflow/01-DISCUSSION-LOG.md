# Phase 1: Rediseño del panel de configuración - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisiones canónicas en CONTEXT.md — este log preserva el contexto de la conversación.

**Date:** 2026-06-14
**Phase:** 1 - Rediseño del panel de configuración (estilo WhisprFlow)
**Areas discussed:** (el usuario delegó todas las decisiones)

---

## Selección de áreas a discutir

| Opción presentada | Descripción | Seleccionada |
|-------------------|-------------|--------------|
| Shell de UI (sidebar Home+Settings) | Organización del panel lateral, Home vs Settings, tamaño, paleta | — |
| Modelos + Parakeet (alcance) | Cuáles Whisper, Parakeet ahora vs después | — |
| Auto-learn from corrections | Qué significa, alcance, ahora vs después | — |
| Archivos + Transcripciones + Diccionario | Upload/formatos, tabla, items | — |

**Respuesta del usuario:** "nada tu haz lo tuyo godspeed" — delegación total de las decisiones a Claude.

**Notas:** Antes de la pregunta, Claude marcó como thinking-partner que **Parakeet** (motor ASR nuevo) y **auto-learn** (feature ML completa) son piezas pesadas, casi-fases por sí solas.

---

## Decisión estructural de Claude (al delegar el usuario)

- Dividir el milestone en **3 fases**: Fase 1 = shell de UI + todos los settings/secciones de UI+macOS (shippable), Fase 2 = Parakeet como motor seleccionable, Fase 3 = auto-learn.
- Conservar la paleta beige+coral; emular de WhisprFlow solo la estructura (sidebar) y densidad.
- Modelo de datos de transcripciones **unificado** con campo `source` (dictation|file), no dos tablas.
- Diccionario como items solo-palabras en Fase 1; reemplazos ligados a auto-learn (Fase 3).

## Claude's Discretion

Toda la fase (microcopy, iconografía, animaciones, assets de sonido, orden de secciones, vías técnicas de pause-media/updater) queda a discreción de Claude, anclada en código y paleta existentes. Detalle completo en CONTEXT.md.

## Deferred Ideas

- Parakeet (Phase 2), auto-learn (Phase 3), reemplazos en diccionario (Phase 3), dark mode (fuera de scope).
