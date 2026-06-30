# CLAUDE.md — onnda

Instrucciones de proyecto para agentes. El handoff vivo y detallado está en **`.planning/RESUME.md`** (léelo al empezar). El log tarea-a-tarea de la última tanda está en `.superpowers/sdd/progress.md` (gitignored).

## Qué es
**onnda** — app macOS de **dictado de voz 100% local**. Stack: **Tauri 2 (Rust) + SvelteKit/Svelte 5 (runes)**. Motores ASR: whisper.cpp (whisper-rs, Metal) y Apple SpeechAnalyzer (sidecar Swift). Alt+Space push-to-talk, auto-paste donde está el cursor. El binario/crate se llama `onnda`/`onnda_lib`; la marca es onnda (tagline "voice to text"). El repo en GitHub se llama `onnda` (renombrado desde `voz-local`).

- **Rama de trabajo actual: `feat/accounts-metrics`** (NO mergeada). `main` tiene el milestone previo.
- La UI de la app está **en inglés**. Login **obligatorio** (cuentas locales).

## Cómo correr (SIEMPRE iterar en dev)
```bash
npm run tauri dev    # binario: target/debug/onnda
npm run check        # svelte-check + tsc (mantener 0/0)
cargo check --manifest-path src-tauri/Cargo.toml
cargo test  --manifest-path src-tauri/Cargo.toml
```
- **Itera en dev, NO compiles release en cada cambio** (`tauri build` solo cuando una feature está lista y el usuario lo pide).
- HMR del frontend recarga solo. Cambios en Rust → el watcher recompila y reinicia.
- Reinstalar la app desde /Applications **resetea permisos** macOS (firma adhoc cambia). No reinstalar en cada iteración.
- En dev, macOS atribuye Accesibilidad al **terminal** que lanza la app, no al binario.

## Convenciones de trabajo
- **Responde al usuario en español.** Comentarios de código: español. UI de la app: **inglés**.
- No pidas permiso para cada paso; haz el cambio y sigue. Prueba antes de decir "listo".
- Commits: mensajes claros estilo conventional; terminar con la línea Co-Authored-By que pida el harness.

## GOTCHAS (causan fallos difíciles de depurar — NO re-romper)
1. **Tokens de diseño legacy vs reales.** `src/lib/styles/tokens.css` tiene un *shim* de back-compat que mapea tokens viejos a los nuevos, PERO **`--accent` → verde de la racha (`--dot-on`)**. Construir UI nueva con `--accent`/`--muted`/`--faint` = se ve verde/off-brand. **Usa los reales:** `--text --text-muted --surface --bg --shell --nav-active-bg`(CTA NEGRA)`--nav-active-ink --danger --dot-grid --font-serif`(Goudy)`--font-sans`(Helvetica Neue)`--r-nav`(8)`--r-card`(16)`--s1..--s10`. Sistema = **flat monocromo B&N**; destructivos NO en rojo (tag sutil). Tema vía `[data-theme]` en `<html>`.
2. **Aptabase requiere runtime Tokio.** El plugin hace `tokio::spawn` en su setup (hilo main sin reactor) → panic "no reactor running" → la app se cierra al arrancar. `lib.rs run()` entra un runtime tokio multi-thread para toda la vida de la app. **No lo quites.**
3. **Métricas desde contextos genéricos.** `EventTracker` del plugin solo es Wry-concreto → no se llama `analytics::track` desde funciones `<R: Runtime>`. **Patrón emit-forward:** Rust emite evento `analytics-event` (solo conteos, jamás texto), el frontend lo reenvía a `track_event`. El guard opt-in (`analytics_enabled`, default false) vive en Rust.
4. **Paths por perfil.** Historial/settings/recordings van por `crate::accounts::profile_dir(&app)` (= `app_data_dir/profiles/<current_account_id>/`, o el root legacy si no hay sesión). `models/` es **global**. Cualquier código nuevo que lea/borre datos de usuario DEBE usar `profile_dir` (no `app_data_dir()` crudo).
5. **Atajos globales re-entrantes cuelgan la app.** No registrar/desregistrar un atajo global desde el callback de otro (deadlock en tauri-plugin-global-shortcut). Escape usa un NSEvent monitor pasivo (`escape.rs`).
6. **Tray icon embebido.** `src-tauri/icons/tray_idle.png` va con `include_bytes!` en `lib.rs` (`icon_as_template(true)` = macOS lo recolorea). Tras cambiar el png, `touch src-tauri/src/lib.rs` para forzar recompilación. Para regenerar desde SVG no hay PIL/ImageMagick global: usar venv + `cairosvg` (con `DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/lib`) + Pillow → template 36×36 (32 contenido + 2px inset, opaco→negro).

## Mapa rápido (Rust `src-tauri/src/`)
`lib.rs` (builder, plugins, tray, runtime tokio) · `accounts.rs` (cuentas locales argon2 + perfiles) · `analytics.rs` (Aptabase opt-in) · `commands.rs` (state machine grabación) · `recording.rs` (`stop_and_transcribe_internal`, `transcribe_file`) · `settings.rs` `history.rs` (van por `profile_dir`) · `data_mgmt.rs` · `whisper_backend.rs` `speech_backend.rs` `backend.rs` · `shortcut.rs` `escape.rs` `notch.rs` `audio.rs` (FFT bandas) · `replacements.rs` `learn.rs` (auto-aprendizaje).
Frontend: `src/routes/+page.svelte` (shell + onboarding + gate) · `src/lib/sections/*` (Home/Transcripciones/Importar/Diccionario/Ajustes/Auth) · `src/lib/components/ui/*` (atomic) · `src/routes/widget/+page.svelte` (visualizer WebGL del notch).

## Pendientes externos (los hace el usuario)
- **Aptabase:** crear cuenta → `APTABASE_APP_KEY` (sin key, métricas = no-op).
- **Vercel:** deploy `vercel-subscribe/` + KV → pegar URL en `src/lib/subscribe.ts`.
- **Producto:** bundle identifier ya es `com.onnda.app` (data local migrada). Mergear la rama cuando se apruebe.
