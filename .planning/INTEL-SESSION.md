# Handoff para sesión en Mac Intel — afinar velocidad de transcripción

Lee esto + `.planning/RESUME.md` + `CLAUDE.md`. Estás en la rama **`fix/intel-cpu`** (ya pusheada).

## ✅ RESUELTO (2026-06-30, sesión en Intel real — i7-1068NG7, 4c/8t)
Diagnóstico medido con `cargo run --release --example bench` (nuevo, en `src-tauri/examples/bench.rs`):
- **El "9s" era throttling térmico/contención**, NO el build ni el algoritmo. Whisper procesa una ventana fija de ~30s → el tiempo es casi constante sin importar la duración del clip (2.4s y 8.9s de audio costaban casi lo mismo en el log).
- **Build dev == release**: `[profile.dev.package."*"] opt-level=3` SÍ propaga a la compilación CMake de whisper.cpp (`small`: 2.54s en ambos). El comentario del Cargo.toml era correcto.
- **Threads 4 vs 6: idéntico** en este chip (no afinar threads).
- Números idle (audio 8.4s, idma=es): `small` 2.69s · `small-q5_1` 2.57s · **`base-q5_1` 0.90s** · `base` 0.95s · `tiny` 0.54s.

**Cambio aplicado:** default de Intel → **`base-q5_1`** (≈2.8× más rápido que `small`, calidad muy similar, 57 MB).
- `compat.rs::hardware_default_model()` x86_64 → `"base-q5_1"`.
- `models.rs`: arm de descarga (commit `5359861…`, sha `422f1ae…`). En x86_64 la tarjeta "Whisper Base" ES `base-q5_1` (el base full-precision NO se muestra en Intel — redundante y más lento); en aarch64 "Whisper Base" sigue siendo el `base` normal.
- 57 tests verdes, `cargo check` limpio (x86_64). `npm run check` NO corrido (Node no instalado en este Mac; no se tocó frontend → sigue 0/0).

**Toolchain de este Mac Intel** (estaba limpio): Rust vía rustup (`~/.cargo`), cmake 4.3.3 en `~/.local/opt/cmake` (symlinks en `~/.local/bin`). Falta **Node** para `npm run tauri dev` y el build de release.

**Pendiente:** (1) verificación en vivo con `npm run tauri dev` (requiere instalar Node); (2) release 1.7.5 (ver abajo).

## Contexto inmediato
- onnda **v1.7.4 publicada** (arm64 + Intel). En Intel, whisper.cpp con **Metal se colgaba** (transcripción atascada "transcribing" para siempre). Fix aplicado en esta rama: **Intel usa CPU**.
  - `src-tauri/src/whisper_backend.rs`: en `#[cfg(all(target_os="macos", target_arch="x86_64"))]` → `ctx_params.use_gpu(false)`. Apple Silicon sigue `use_gpu(true)+flash_attn(true)`.
  - `src-tauri/Cargo.toml`: feature `metal` de whisper-rs SOLO en `[target.'cfg(target_arch="aarch64")']`. Intel compila sin Metal (CPU puro).
- **Estado verificado en Intel real:** ya NO se cuelga, **transcribe en ~9s con el modelo `small`**. Funciona pero es lento.

## Tarea: hacer Intel más rápido (sin colgarse)
Objetivo: bajar de ~9s a ~3-5s para dictados cortos, manteniendo precisión razonable.

**Primero diagnostica DÓNDE se van los 9s** (whisper_backend ya loguea tiempos):
```
grep -iE "ms|inference|resample|vad|loading model|transcrib" ~/Library/Logs/com.onnda.app/onnda.log | tail -20
```
- Si el grueso es **"loading model"** → solo la 1ª transcripción tras abrir es lenta (carga ~1-2s); el modelo queda cacheado (`MODEL_CACHE` en whisper_backend.rs, `ensure_loaded` solo recarga si cambia el path). Confirmar que la 2ª+ es rápida.
- Si el grueso es **inferencia** → aplicar los levers de abajo.

**Levers de velocidad (en orden de impacto):**
1. **Modelo por defecto Intel → `base`** (≈2× más rápido que `small`, bien para dictado). Cambiar en `src-tauri/src/compat.rs::hardware_default_model()`: rama `#[cfg(not(target_arch="aarch64"))]` devuelve `"small"` → probar `"base"`. (El usuario debe descargar `base` en Settings para probar; o cambiar default y onboarding lo baja.)
2. **Params** en `whisper_backend.rs` `transcribe()`: ya usa `Greedy{best_of:1}`, `n_max_text_ctx(224)`, `n_threads` = min(cores,6). En Intel con muchos cores, probar subir el cap (el assert actual es `n<=6`, líneas ~116-117) — pero whisper escala mal >8 hilos, medir. Probar `set_temperature(0.0)` / `set_single_segment` según resultados.
3. Considerar modelos **cuantizados** (q5_0/q8_0) si la lista de modelos los soporta — más rápidos en CPU. (Revisar `src-tauri/src/models.rs`.)

## Cómo iterar en ESTA Mac (Intel, nativo — sin cross-compile)
```bash
npm run tauri dev      # build nativo x86_64, hot-reload; binario target/debug/onnda
npm run check          # 0/0
cargo test --manifest-path src-tauri/Cargo.toml
```
Para medir: dicta clips cortos, mira los tiempos en `~/Library/Logs/com.onnda.app/onnda.log`. NO necesitas firmar/notarizar para probar en dev.

## Cuando quede bien → release 1.7.5 (lo hace la sesión del dev Mac o esta, con credenciales)
- Bump 1.7.5 (tauri.conf.json, Cargo.toml, package.json).
- Build **ambas** arch: `npm run tauri build` (arm64) y `--target x86_64-apple-darwin` (Intel), con `TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/onnda-updater.key)"`.
- Firma Developer ID + notarización (profile keychain `onnda-notary`, o inline `--apple-id me@miguelforero.co --password <app-pw> --team-id Z464275F27`). Si el keychain se bloquea: `security unlock-keychain ~/Library/Keychains/login.keychain-db`.
- DMG con `scripts/make-dmg.sh <app> <out>` (fondo `src-tauri/icons/dmg-background.png` + @2x).
- `latest.json` con `darwin-aarch64` + `darwin-x86_64` (firmas de los `.app.tar.gz.sig`). DMGs nombrados `onnda_1.7.5_AppleSilicon.dmg` / `onnda_1.7.5_Intel.dmg`.
- `gh release create v1.7.5 --target main` con: 2 DMG + 2 tar + 2 sig + latest.json.
- **Llave updater/firma stable:** `~/.tauri/onnda-updater.key` (esta vive en el dev Mac; si vas a releasear desde Intel, cópiala). Pubkey embebida en tauri.conf.

## Nota
Las *memorias* del proyecto viven en el `~/.claude` del dev Mac, NO se transfieren. Este archivo + RESUME.md cargan lo esencial. Branch limpia: `npm run check` 0/0, 57 tests Rust verdes.
