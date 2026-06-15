# Phase 1: Blindaje de producción - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning
**Source:** Derivado de `.planning/research/LAUNCH-DIAGNOSIS.md` (auditoría de 4 agentes)

<domain>
## Phase Boundary

Esta fase hace que Voz Local **no crashee ni pierda dictado en silencio** en las rutas que un usuario público va a estrenar, y deja la app **diagnosticable** en producción. Es puramente robustez del backend Rust + feedback mínimo al usuario; NO toca el diseño visual, ni el motor ASR, ni la compatibilidad Intel (eso es Phase 2).

**Dentro de scope:** los 3 crashes de la ruta crítica (mic, mutex, descarga), integridad+offline del modelo, hacer visibles los fallos de transcripción, y logging a disco.

**Fuera de scope:** firma/notarización (Phase 3), métricas/crash-reporting remoto (Phase 4), tests/refactor (Phase 5), cualquier cambio de compatibilidad Intel/Apple (Phase 2).
</domain>

<decisions>
## Implementation Decisions

### HARDEN-01 — Mic no crashea (audio.rs)
- `audio.rs:73,75` — los `.expect("failed to build/start input stream")` se convierten en propagación de error (`?`) hacia `Result<(), String>`. La función de captura ya vive bajo `start_recording_internal` (commands.rs:100) que devuelve `Result<(),String>`.
- Mapear `cpal::BuildStreamError` y `PlayStreamError` a mensajes claros en español para el usuario (ej. "El micrófono dejó de estar disponible. Revisa que no esté en uso por otra app.").
- El caso device-not-found (audio.rs:18–24) ya está bien manejado — mantener ese patrón.

### HARDEN-02 — Un panic no mata la transcripción (whisper_backend.rs)
- Reemplazar el `std::sync::Mutex` de `MODEL_CACHE` (whisper_backend.rs:25,50,51) por `parking_lot::Mutex` (no envenena). Añadir dep `parking_lot`.
- Auditar que no queden otros `.lock().unwrap()` en la ruta de transcripción que puedan envenenar.

### HARDEN-03 — Integridad + URL pinneada del modelo (commands.rs download_model)
- Usar la dep `sha2` (ya declarada, hoy sin uso) para verificar SHA256 del archivo descargado en streaming, contra un hash esperado por modelo.
- Pinear la URL de descarga a una **revisión/commit estable** de HuggingFace (hoy `.../resolve/main/...` flotante) — fijar a un revision SHA o tag.
- En mismatch de hash: borrar el `.tmp`, no renombrar, y devolver error claro. (El patrón temp→rename ya existe.)
- **Claude's Discretion / a investigar:** de dónde obtener los SHA256 reales de cada modelo ggml (base, small, medium, large-v3-turbo) — HF API/LFS pointers. Si no hay hash confiable, fallback mínimo: verificar `content-length` == bytes escritos + pin de revisión.

### HARDEN-04 — Primer arranque offline (modelo ausente)
- Detectar al iniciar (o al primer intento de dictado) que el modelo por defecto no está descargado y/o no hay conexión, y mostrar un **estado claro y accionable** en la UI (prompt de descarga / reintentar), no un fallo críptico de carga.
- Reusar el comando/flujo de descarga existente (Ajustes→Modelos ya descarga). El error actual "Modelo no encontrado. Descárgalo en Ajustes → Modelos." existe (commands.rs:326) — mejorar para que sea proactivo en primer arranque, no solo reactivo al soltar.
- **Claude's Discretion:** dónde exactamente vive el banner (onboarding existente vs. Home vs. widget).

### HARDEN-05 — Fallos de transcripción visibles
- Hoy un segmento de streaming fallido se descarta callado (commands.rs:~238, solo procesa `if let Ok(Ok(t))`, avanza el marcador igual) y el tail tiene fallback parcial silencioso (~397–413).
- Emitir un evento de error al usuario cuando un segmento o el tail fallan, en vez de tragar el error. El caso no-segmentos ya emite `transcribe-error` — extender a los demás caminos.
- **Claude's Discretion:** UX exacta (un aviso no-bloqueante en la ventana principal y/o el widget). Mínimo: que el usuario sepa que parte del dictado no se transcribió.

### HARDEN-06 — Logging a disco
- Añadir un framework de logging (preferencia: `tauri-plugin-log` por integración Tauri 2 nativa; alternativa `tracing` + `tracing-appender`) que escriba un **archivo rotativo** en el app-data dir.
- Convertir los 8 `eprintln!` existentes (audio.rs:70, commands.rs:288/~359, escape.rs:21, shortcut.rs:22/35, whisper_backend.rs:33/85) a macros estructuradas (`log::warn!/error!` o `tracing`).
- Conservar los logs `[timing]` por etapa de whisper_backend, pero por el logger.
- **Claude's Discretion:** nivel por defecto (info), política de rotación, si exponer "abrir carpeta de logs" en Ajustes (ya hay reveal de cache).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Diagnóstico de origen (autoridad de esta fase)
- `.planning/research/LAUNCH-DIAGNOSIS.md` — sección "F1 — Blindaje de producción" tiene cada finding con file:line y la corrección.

### Código a modificar (leer estado actual antes de tocar)
- `src-tauri/src/audio.rs` — captura cpal (HARDEN-01, eprintln HARDEN-06)
- `src-tauri/src/whisper_backend.rs` — MODEL_CACHE + locks + timing logs (HARDEN-02, HARDEN-06)
- `src-tauri/src/commands.rs` — download_model (HARDEN-03), streaming loop + tail (HARDEN-05), modelo ausente (HARDEN-04), eprintln (HARDEN-06)
- `src-tauri/Cargo.toml` — deps (sha2 ya presente; añadir parking_lot + logger)
- `src/routes/+page.svelte` y `src/routes/widget/+page.svelte` — feedback de error (HARDEN-04/05)

### Convenciones del proyecto
- `./CLAUDE.md` — guías del repo
- Mensajes de usuario en **español** (la app es ES-first). Iterar en `npm run tauri dev` (ver feedback memory).
</canonical_refs>

<specifics>
## Specific Ideas

- El patrón de manejo graceful ya existe y es ejemplar en `speech_backend.rs:103–123` (sidecar Apple): todo fallo → `Result::Err` con stderr capturado. **Replicar ese estándar** en las rutas Whisper/audio.
- `reqwest` ya usa `features=["stream"]` → el SHA256 se puede calcular incrementalmente mientras se escribe el archivo, sin doble lectura.
- No introducir frameworks pesados (constraint del proyecto). parking_lot y un logger ligero están bien.
</specifics>

<deferred>
## Deferred Ideas

- Crash reporting remoto (Sentry/GlitchTip) → Phase 4 (es opt-in y necesita consentimiento). Esta fase solo deja logs LOCALES en disco.
- Tests de las rutas que se tocan → Phase 5.
- Refactor del god-file commands.rs → Phase 5 (en esta fase se modifica in-place, sin partirlo).
</deferred>

---

*Phase: 01-blindaje-de-producci-n*
*Context derivado del diagnóstico 2026-06-15*
