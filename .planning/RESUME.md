# Handoff — onnda · 2026-06-27

App macOS de dictado 100% local (Tauri 2 + Svelte 5 + Whisper/Apple Speech). Este doc es para retomar tras limpiar contexto.

## Estado / rama
- **Rama activa: `feat/accounts-metrics`** (sacada de `feat/onnda-ui-redesign`). **NO mergeada.**
- Repo: `github.com/miguelforero-co/voz-local` (público; el repo se llama voz-local aunque la marca es onnda).
- App corre en dev, **en inglés**, abre en **pantalla de login** (login obligatorio ahora).
- `cargo check` + `npm run check` = 0/0. 61 tests Rust verdes.
- Specs/plans: `docs/superpowers/{specs,plans}/2026-06-26-*`. Historial tarea-a-tarea (SDD): `.superpowers/sdd/progress.md` (gitignored).

## Qué se construyó (esta tanda)
1. **Plan A — Métricas Aptabase (desbloquea F4):** analítica anónima **opt-in** (`analytics_enabled` default false; guard único en `src-tauri/src/analytics.rs`). Eventos: app_launched, transcription_completed (dictado+archivo), model_downloaded, engine_changed (solo cambio real whisper↔apple), correction_learned. **Nunca** envía texto, solo conteos.
2. **Plan B — Cuentas locales + perfiles:** login email/contraseña/nombre, **argon2id**, 100% local sin backend (`accounts.rs` + `accounts.json`). Datos por perfil bajo `app_data_dir/profiles/<id>/` (settings/history/recordings; **`models/` global**). 1ª cuenta **reclama** data legacy del root. Frontend: `src/lib/auth.svelte.ts` + gate en `+page.svelte` con `src/lib/sections/Auth.svelte`. Saludo con nombre (#21 resuelto), sección Cuenta en Ajustes.
3. **Plan C (anexo) — Captar emails:** función Vercel standalone en `vercel-subscribe/` (KV) + `src/lib/subscribe.ts` (opt-in fire-and-forget en signup; URL es placeholder hasta deploy).
4. **App entera a inglés**; **rediseño del login** a tokens onnda reales; **tray icon** = logo onnda (template).

## Ronda de fixes UI en curso (uno por uno con el usuario)
- **#1 Tray icon — HECHO.** `src-tauri/icons/tray_idle.png` regenerado del SVG `~/Downloads/onddaa.svg` → template 36×36 (32 contenido + 2px inset, opaco→negro). Es `include_bytes!` en lib.rs → tras cambiar el png, `touch src-tauri/src/lib.rs` para recompilar. Regenerar: venv scratchpad + `cairosvg` (con `DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/lib`) → 256px → Pillow a 36×36.
- (siguen más fixes que dirá el usuario)

## GOTCHAS críticos (no re-romper)
- **Tokens de diseño:** pantallas nuevas se hicieron con tokens LEGACY que el shim mapea mal → **`--accent` = verde de la racha** (el botón verde que el usuario odió). Usar SIEMPRE los reales: `--text --text-muted --surface --bg --nav-active-bg`(CTA negra)`--nav-active-ink --danger --dot-grid --font-serif --font-sans --r-nav --r-card --s1..--s10`. Si algo se ve verde/genérico → tiene tokens legacy. Auth + onboarding ya corregidos.
- **Aptabase crashea sin runtime Tokio:** el plugin hace `tokio::spawn` en su setup (hilo main, sin reactor) → panic "no reactor running" → app se cierra al arrancar. Fix en `lib.rs run()`: crear y `enter()` un runtime tokio multi-thread para toda la vida de la app. **NO quitar.**
- **Métricas desde contextos genéricos `<R>`:** el `EventTracker` del plugin solo es Wry-concreto → no se puede llamar `analytics::track` desde funciones genéricas. Patrón **emit-forward**: Rust emite evento `analytics-event` (sin contenido) y el frontend lo reenvía a `track_event`.
- **Paths por perfil:** todo lo de historial/settings/recordings DEBE ir por `crate::accounts::profile_dir(&app)`. (`data_mgmt.rs` ya corregido; antes borraba el root vacío.)

## Caveats aceptados del login local (v1)
No portátil entre Macs · reset de contraseña local (sin verificación por correo) · sin cifrado por perfil.

## Dev
```bash
npm run tauri dev          # binario target/debug/onnda; login obligatorio
npm run check              # 0/0
cargo check --manifest-path src-tauri/Cargo.toml
```

## Pendientes externos (usuario)
1. Aptabase: cuenta + `APTABASE_APP_KEY` (sin key, métricas = no-op).
2. Vercel: deploy `vercel-subscribe/` + provisionar KV → pegar URL en `src/lib/subscribe.ts`.

## Pendiente de producto
- **#1 Bundle identifier** sigue `com.vozlocal.app` (cambiar a `com.onnda.app` solo CON migración de data, o dejarlo).
- Mergear `feat/accounts-metrics` cuando el usuario apruebe.
