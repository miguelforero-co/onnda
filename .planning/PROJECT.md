# Voz Local

## What This Is

App macOS 100% local de dictado por voz. Tauri 2 (Rust) + SvelteKit. Whisper.cpp vía whisper-rs para transcripción. Alt+Space para push-to-talk, auto-paste donde esté el cursor. El usuario la usa a diario para dictar en Claude Code y otras apps.

## Core Value

Dictado por voz rápido, privado (100% local) y siempre disponible vía atajo global — el texto aparece donde está el cursor sin fricción.

## Current Milestone: v2.0 Camino al lanzamiento público

**Goal:** Dejar Voz Local lista para lanzarse al público como descarga directa firmada+notarizada y open source, funcionando bien en Intel y Apple Silicon, gratis.

**Target features:**
- Blindaje de producción (cero crashes en ruta crítica, integridad+offline del modelo, logging, fallos visibles)
- Compatibilidad honesta Intel + Apple Silicon (build x86_64, motor Apple gateado a Silicon, modelo por defecto según hardware)
- Firma Developer ID + notarización + repo público (updater real, LICENSE MIT, higiene OSS)
- Métricas y crash reporting opt-in que respetan el "100% local" (Aptabase + Sentry/GlitchTip, solo conteos)
- Pulido (CI en PRs, tests de rutas críticas, partir god-file commands.rs)

**Key context:** Descarga directa (NO App Store — el sandbox rompe el auto-paste). Gratis/OSS sin pagos en v1. Neural Engine se mantiene para Apple Silicon. Diagnóstico completo en `research/LAUNCH-DIAGNOSIS.md`.

## Requirements

### Validated

<!-- Shipped y en uso real -->

- ✓ Push-to-talk Alt+Space + auto-paste — v1.x
- ✓ Pipeline de transcripción streaming agnóstico al modelo (Whisper.cpp) — v1.x
- ✓ Widget del notch con visualizer WebGL espectral (FFT → onda) — v1.7.0
- ✓ Cancelar grabación con Escape (monitor global NSEvent) — v1.7.0
- ✓ Historial de transcripciones (lista) — v1.x
- ✓ Diccionario de palabras personalizadas (input de texto, `custom_words`) — v1.x

### Validated (milestone v1.0 — rediseño + motores + aprendizaje)

- ✓ Panel lateral "home + settings" estilo WhisprFlow — v1.0
- ✓ Motor Apple SpeechAnalyzer seleccionable (sidecar Swift, Apple Silicon) — v1.0
- ✓ Auto-learn from corrections + reemplazos deterministas/snippets — v1.0
- ✓ Transcripción por archivos, historial unificado, diccionario editable, stats de uso — v1.0
- ✓ Ajustes completos (sonidos, pause-media, idioma, launch-at-login, permisos, modelos, updates, datos) — v1.0

### Active

<!-- Milestone v2.0: Camino al lanzamiento público (descarga directa + OSS, Intel+Silicon, gratis) -->
<!-- Requirements detallados en REQUIREMENTS.md; diagnóstico en research/LAUNCH-DIAGNOSIS.md -->

- [ ] Blindaje de producción: cero crashes en ruta crítica, integridad+offline del modelo, logging, fallos visibles
- [ ] Compatibilidad honesta: build Intel x86_64, motor Apple gateado, modelo por defecto según hardware
- [ ] Firma + notarización + repo público (Developer ID, updater real, LICENSE, higiene OSS)
- [ ] Métricas y crash reporting opt-in (Aptabase + Sentry/GlitchTip, solo conteos, jamás contenido)
- [ ] Pulido: CI en PRs, tests de rutas críticas, partir god-file commands.rs

### Out of Scope (milestone v2.0)

- **Mac App Store** — incompatible con el App Sandbox por el auto-paste (Accessibility/CGEvent). Distribución = descarga directa firmada+notarizada.
- **Cuentas / backend propio / sync** — la app es 100% local; no aportan valor en v1, solo fricción y responsabilidad.
- **Monetización / pagos / licencias** — v2.0 es gratis + OSS. Polar/llaves Ed25519 diferido a milestone futuro.
- **Cambiar el motor de ASR por defecto** — el motor es elección del usuario; la infra debe ser óptima con cualquiera.
- **Tácticas de lanzamiento profundas** (Product Hunt/HN/Homebrew cask/pricing) — research diferido, no bloquea.

## Context

- Stack: Tauri 2 (Rust) + SvelteKit. Frontend principal en `src/routes/+page.svelte`; widget en `src/routes/widget/+page.svelte`.
- Módulos Rust en `src-tauri/src/`: `audio.rs`, `whisper_backend.rs`, `backend.rs` (trait `TranscriptionBackend`), `commands.rs`, `shortcut.rs`, `history.rs`, `settings.rs`, `mic_permission.rs`, `notch.rs`, `escape.rs`, `vad.rs`, `streaming.rs`, `transcription.rs`.
- Modelo actual: `ggml-large-v3-turbo.bin` en `~/Library/Application Support/com.vozlocal.app/models/`.
- Inferencia Whisper tiene costo casi fijo ~1.7s (padding a 30s). El salto de velocidad real sería cambiar el motor (Parakeet/ANE vía FluidAudio) — fuera de scope este milestone.
- Iterar SIEMPRE en `npm run tauri dev` (log en `/tmp/voz-local-dev.log`); solo `tauri build` cuando una feature esté terminada y el usuario lo pida.

## Constraints

- **Tech stack**: Tauri 2 + SvelteKit + Rust — mantener; no introducir frameworks pesados de UI sin justificación.
- **Privacidad**: todo procesamiento local; nada sale del equipo.
- **Plataforma**: macOS primario (APIs nativas: NSEvent, notch, permisos). Mantener build de Intel en CI.
- **Permisos macOS**: reinstalar la app resetea permisos (firma adhoc cambia) — evitar reinstalar en cada iteración.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| GSD ligero (sin new-project completo) para este milestone | El proyecto ya está maduro y mapeado en memoria; el usuario quiere planear esta fase sin ceremonia | ✓ v1.0 |
| UI inspirada en WhisprFlow | Referencia de UX que el usuario quiere emular para el panel | ✓ v1.0 |
| Distribución por descarga directa, NO App Store | El auto-paste (Accessibility/CGEvent) es incompatible con el App Sandbox que exige el App Store; toda la categoría distribuye así | — v2.0 |
| Soportar Intel + Apple Silicon, Neural Engine solo en Silicon | El usuario quiere ambos públicos; el ANE no existe en Intel (físico), pero no se le quita a Silicon | — v2.0 |
| Gratis + OSS en v1, sin cuentas ni backend | Adopción y feedback primero; sin valor en cuentas para una utilidad local, solo fricción/responsabilidad | — v2.0 |
| Métricas opt-in, solo conteos jamás contenido | El brand es "tu voz nunca sale del Mac"; cualquier telemetría debe ser opt-in y anónima (Aptabase + Sentry/GlitchTip) | — v2.0 |

---
*Last updated: 2026-06-15 — inicio milestone v2.0 Camino al lanzamiento público*
