# Handoff — onnda · 2026-06-30

App macOS de dictado 100% local (Tauri 2 + Svelte 5 + Whisper/Apple Speech). Lee esto al retomar.

## Estado actual
- **v1.7.1 PUBLICADA** → https://github.com/miguelforero-co/onnda/releases/tag/v1.7.1
  - Firmada Developer ID + **notarizada + stapled**. DMG bonito (logo + flecha + Applications, vía `create-dmg`).
  - **Auto-update in-app funcionando** (verificado end-to-end). Assets del release: `onnda_1.7.1_aarch64.dmg`, `onnda.app.tar.gz`, `onnda.app.tar.gz.sig`, `latest.json`.
- **`main` al día** (commit merge `05a81c5`, PR #3). Rama de trabajo `fix/onboarding-apple-robustness` ya mergeada.
- Repo: `github.com/miguelforero-co/onnda` (renombrado desde voz-local). Identifier app: `com.onnda.app`.
- `npm run check` 0/0, `cargo check` limpio, 55 tests Rust.
- **Solo se construyó arm64** (Apple Silicon). Falta build Intel x86_64.

## Lo que se hizo esta sesión (rebrand + 5 bugfixes + auto-update)
1. **Rebrand total a onnda** (sesión previa dentro de esta): identifier `com.onnda.app`, repo renombrado, logs/temp, README con punch, LICENSE MIT.
2. **Quitado sistema de cuentas/login** (argon2/perfiles) → local-first, datos en `app_data_dir` directo. Onboarding pide nombre + correo (→ Loops, opt-in).
3. **5 bugs reportados por tester, arreglados** (rama fix, en main):
   - Onboarding: botón Continue fuera de la ventana → `.ob` scroll-container + `.ob-inner` margin:auto.
   - Onboarding: selección de modelo no persistía → faltaba onSelect/selected en ModelCard + warming de Apple al seleccionarlo.
   - Notch nunca atascado: `IS_TRANSCRIBING`+`CANCEL_TRANSCRIPTION` (Escape cierra en fase transcribe) + timeout 45s en `recording.rs`.
   - Doble-paste: guard de dedup en `paste.rs::paste_text` (mismo texto <1.2s).
   - **Transcriptions crash** (causa raíz reproducida en vivo): Svelte `each_key_duplicate` por ids de historial duplicados (eran solo timestamp_ms → doble-guardado = id repetido). Fix en `history.rs`: id `{ts}-{seq}` único + dedup al guardar + **dedup por id en `load()`** (arregla data ya rota).
4. **Auto-update** (`feat/updater`): plugin `tauri-plugin-updater` + `@tauri-apps/plugin-process`. Banner "Update & restart" en `+page.svelte` (check al arrancar → download con % → relaunch). Config en `tauri.conf.json` (`plugins.updater` endpoint+pubkey, `createUpdaterArtifacts`). Ver memoria [[release-firma-notarizacion-onnda]] para el flujo de release CON updater (orden crítico: firmar/notarizar/staple el .app ANTES de regenerar el `.app.tar.gz`).

## Pendientes (orden sugerido)
1. **Probar first-run pristino**: el usuario BORRÓ todo onnda del Mac (apps, datos, permisos TCC reseteados). Va a instalar v1.7.1 a mano y probar onboarding + permisos frescos. (v1.7.0 no tiene updater → instalar v1.7.1 a mano UNA vez; de ahí, auto-update.)
2. **Build Intel x86_64** para v1.7.1: rebuild `--target x86_64-apple-darwin`, firmar+notarizar, regenerar tar+sig, añadir `darwin-x86_64` a `latest.json`, subir al release. (Sin esto, usuarios Intel no descargan/actualizan.)
3. **Arreglar la CI** (GitHub issue #2): subir `whisper-rs` 0.14→0.16 (el runner M1 de GitHub no compila el i8mm del ggml viejo; la M5 del dev lo enmascara). Validar con run de CI. Ver memoria [[ci-whisper-m1-i8mm]].
4. **Re-activar el auto-release**: `release.yml` está en `workflow_dispatch` (trigger de tags comentado) hasta arreglar la CI.

## GOTCHAS críticos (no re-romper)
- **Tokens de diseño**: usar los reales (`--text --text-muted --surface --bg --nav-active-bg`(CTA negra)`--font-serif`(Goudy)`--s1..--s10`). Los legacy `--accent/--muted/--faint/--r` mapean mal (verde/genérico).
- **each_key_duplicate**: cualquier `{#each (e.id)}` con ids repetidos crashea la página entera en Svelte 5. Mantener ids únicos + el dedup en `history::load`.
- **Releases LOCALES** (la CI está rota): construir en la Mac del dev. Flujo completo (firma Apple + updater + latest.json) en memoria [[release-firma-notarizacion-onnda]]. Notarización de Apple ahora rápida (ya conoce la app).
- **Updater key**: `~/.tauri/onnda-updater.key` (sin password). Pubkey embebida en tauri.conf. Si se pierde la llave, los updates dejan de funcionar.
- **Dev env se "wedgea"** tras muchos kill/relaunch + correr `cargo` directo junto al watcher de `tauri dev` (webview congelado). Si pasa: matar todo (`pkill -f onnda; pkill -f "tauri dev"; pkill -f vite`), `rm -rf .svelte-kit node_modules/.vite`, relanzar UNA vez.
- Para clickear la app en tests: `swift` posteando CGEvent de mouse (cliclick/Quartz no están). La app de menu-bar cuesta subirla al frente (Google Sheets u otras la tapan).

## Memorias del proyecto (persisten solas, en ~/.claude/.../memory/)
- `release-firma-notarizacion-onnda` — Developer ID, secrets, flujo firma+notarización+updater.
- `ci-whisper-m1-i8mm` — por qué la CI falla y el fix (whisper-rs 0.16).

## Dev
```bash
npm run tauri dev      # binario target/debug/onnda
npm run check          # 0/0
cargo test --manifest-path src-tauri/Cargo.toml   # 55 verdes
```
