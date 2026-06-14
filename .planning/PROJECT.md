# Voz Local

## What This Is

App macOS 100% local de dictado por voz. Tauri 2 (Rust) + SvelteKit. Whisper.cpp vía whisper-rs para transcripción. Alt+Space para push-to-talk, auto-paste donde esté el cursor. El usuario la usa a diario para dictar en Claude Code y otras apps.

## Core Value

Dictado por voz rápido, privado (100% local) y siempre disponible vía atajo global — el texto aparece donde está el cursor sin fricción.

## Requirements

### Validated

<!-- Shipped y en uso real -->

- ✓ Push-to-talk Alt+Space + auto-paste — v1.x
- ✓ Pipeline de transcripción streaming agnóstico al modelo (Whisper.cpp) — v1.x
- ✓ Widget del notch con visualizer WebGL espectral (FFT → onda) — v1.7.0
- ✓ Cancelar grabación con Escape (monitor global NSEvent) — v1.7.0
- ✓ Historial de transcripciones (lista) — v1.x
- ✓ Diccionario de palabras personalizadas (input de texto, `custom_words`) — v1.x

### Active

<!-- Milestone actual: rediseño del panel de configuración estilo WhisprFlow -->

- [ ] Panel lateral tipo "home + settings" inspirado en la UI de WhisprFlow
- [ ] Toggle de sonidos (escucha / para / cancela)
- [ ] Toggle de pause-media (pausa la música al empezar a escuchar)
- [ ] Selector de lenguaje (automático o manual)
- [ ] Launch at login
- [ ] Panel de permisos (micrófono + accesibilidad) con estado y acción
- [ ] Auto-learn from corrections
- [ ] Editor de hotkeys (atajo de escuchar/grabar)
- [ ] Toggle push-to-talk vs mantener
- [ ] Selector de modelos ampliado (~3 Whisper + Parakeet, con versión)
- [ ] Check for updates
- [ ] Gestión de datos (abrir carpeta de cache, borrar)
- [ ] Transcripción por archivos (upload → transcripción, tabla de históricos)
- [ ] Vista de todas las transcripciones (evolución del historial)
- [ ] Diccionario como lista de items editable (no input plano)

### Out of Scope

- Cambiar el motor de ASR por defecto en este milestone — el motor es elección del usuario en el selector; la infra debe ser óptima con cualquiera.
- Cloud / cuentas / sync — la app es 100% local por diseño.

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
| GSD ligero (sin new-project completo) para este milestone | El proyecto ya está maduro y mapeado en memoria; el usuario quiere planear esta fase sin ceremonia | — Pending |
| UI inspirada en WhisprFlow | Referencia de UX que el usuario quiere emular para el panel | — Pending |

---
*Last updated: 2026-06-14 — bootstrap GSD ligero para milestone de rediseño de settings*
