# Diagnóstico de lanzamiento — Voz Local v2.0

Investigación previa al milestone "Camino al lanzamiento público" (2026-06-15).
Generado por 4 agentes en paralelo (codebase / compatibilidad / distribución / métricas).
Este documento es la **fuente de verdad** para planear las fases. Cada finding trae `file:line`.

## Decisiones estratégicas tomadas (con el usuario)

| Tema | Decisión | Implicación |
|---|---|---|
| **Distribución** | Descarga directa firmada + notarizada + OSS. **NO** App Store. | El auto-paste (Accessibility/CGEvent) es incompatible con el App Sandbox que exige el App Store. Toda la categoría (Wispr Flow, Superwhisper, MacWhisper) distribuye así. Conservamos todas las features. Costo: $99/año Apple Developer. |
| **Hardware** | Soportar Intel **Y** Apple Silicon. Neural Engine (motor Apple) se mantiene **solo** para Silicon; Intel corre Whisper. | Hay que arreglar el build x86_64 y gatear el motor Apple correctamente. No existe ANE en Intel (físico). |
| **Monetización** | Gratis + OSS por ahora. Sin pagos/licencias/cuentas en v1. | No construir backend, ni cuentas, ni Polar/llaves en este milestone. |

## F1 — Blindaje de producción (no crashear en público)

- **🔴 BLOCKER** `audio.rs:73,75` — `.expect("failed to build/start input stream")`. Mic desconectado / bloqueado por otra app / denegado tras encontrar device → **crash a mitad de grabación**. `start_recording_internal` (commands.rs:100) ya espera `Result<(),String>`. Convertir a `?`.
- **🔴 BLOCKER** `whisper_backend.rs:25,50,51` — `MODEL_CACHE.lock().unwrap()`. Un panic con el lock tomado **envenena el mutex** y toda transcripción posterior paniquea en la ruta crítica (streaming loop commands.rs:222, tail ~378). Usar `parking_lot::Mutex` o `unwrap_or_else(|e| e.into_inner())`.
- **🔴 BLOCKER** descarga de modelo (commands.rs:785–834) — 1.5GB de HuggingFace **sin verificar SHA256** (dep `sha2` declarada pero NUNCA usada — promesa falsa), URL `.../main/...` flotante (sin pin a commit), y **app inútil offline en primer arranque**. Pinear URL a commit SHA + verificar checksum contra hashes fijados + UX de primer-arranque-offline.
- **🟠 HIGH** fallos silenciosos — streaming (commands.rs:~238) solo procesa `if let Ok(Ok(t))`, avanza el marcador igual → segmento fallido **se descarta sin avisar**. Tail (~397–413) con fallback parcial silencioso. Emitir error al usuario.
- **🟠 HIGH** sin observabilidad — 8 `eprintln!` que se evaporan al lanzar desde Finder. Sin log en disco, sin crash reporting. Añadir `tauri-plugin-log`/`tracing` con archivo rotativo en app-data dir.

## F2 — Compatibilidad honesta (Intel + Silicon)

- **🔴** sidecar Apple es solo `asr-aarch64-apple-darwin`; no hay build x86_64. `scripts/build-sidecar.sh:11` compila solo el host-triple. `tauri.conf.json:52` `externalBin:["binaries/asr"]` → el build `--target x86_64-apple-darwin` (release.yml:65) busca `asr-x86_64-apple-darwin` que **no existe → el bundle Intel falla**. Opciones: (a) build x86_64 del sidecar Swift, o (b) hacer el sidecar opcional para que su ausencia no rompa el bundle Intel.
- **🔴 UX** `commands.rs:774–781` (`get_models`) — entrada Apple hardcodeada `downloaded:true, coming_soon:false` sin gate de arch/OS. Usuario Intel o pre-macOS-26 la ve lista y **falla con error crudo**. Gatear: `#[cfg(target_arch="aarch64")]` + `majorVersion>=26` + sidecar presente; si no, omitir o disabled con tooltip.
- **Hechos web confirmados:** Apple SpeechAnalyzer = macOS 26 (Tahoe) + Apple Silicon (ANE), no existe en Intel. whisper.cpp Metal NO acelera Intel (el código ya hace CPU-only en Intel por `#[cfg(target_arch="aarch64")]` en whisper_backend.rs:35–38 y Cargo.toml:55–56 — correcto). large-v3 necesita ~3-4GB activos; en Intel 8GB es inusable.
- **🟠** sin selección de modelo por hardware — a todos se les pone el más pesado. Detectar arch + RAM (`hw.memsize`): Intel/<16GB → `small`/`base`; Silicon → turbo o motor Apple. Limitar `set_n_threads(6)` (whisper_backend.rs:55) a cores reales.
- **🟡** `widget/+page.svelte:14` `hasNotch = $state(true)` → flash de forma-notch un frame en pantallas sin notch. Init `false` o esperar primer evento `screen-notch`. (El fallback de no-notch en sí YA funciona bien — confirmado.)
- Floor de OS: declarado `minimumSystemVersion:11.0` (tauri.conf.json:60). Notch (`safeAreaInsets`) es macOS 12+; degrada a "sin notch" en 11. Motor Apple = 26+. Documentar floors reales.

## F3 — Firma, notarización, repo público

- **🟠 HIGH** ships sin notarizar — firma ad-hoc `codesign --sign -` (release.yml); release notes instruyen `sudo xattr -cr`. Necesita Developer ID + hardened runtime + entitlements (mic + accessibility) + `notarytool` + staple en CI. `TAURI_SIGNING_PRIVATE_KEY:""` vacío.
- **🟠 HIGH** updater falso — `@tauri-apps/plugin-updater` en package.json pero SIN crate Rust, SIN bloque `updater` en tauri.conf, SIN endpoint/pubkey/permiso. La implementación real (`updater_check.rs`) solo consulta GitHub Releases API (check-only). Decidir: wirear updater real (latest.json firmado minisign en GitHub Releases) o quitar la dep JS falsa y dejar el check honesto.
- **🟠 HIGH (OSS)** `.planning/` (31 archivos) y `dev/` (7 HTMLs de juguete, ~85KB) están git-tracked → se publicarían. Gitignore o remover antes de abrir el repo.
- **🟡** version drift — `package.json:4` = 0.1.0, `Cargo.toml:3` = 1.7.0, `tauri.conf.json` = 1.7.0, user_agent "voz-local/0.1". Una sola fuente de verdad.
- LICENSE MIT (adopción máxima; el usuario eligió gratis/OSS). README de cara al público + landing.

## F4 — Medir y aprender (opt-in, respetando "100% local")

- **Analítica → Aptabase** — plugin oficial Tauri 2 (`tauri-plugin-aptabase`, Rust + `@aptabase/tauri`), anónimo por diseño, self-hosteable (AGPL server / MIT SDK). Gratis 20k eventos/mes. Capability `aptabase:allow-track-event`. **Opt-in apagado por defecto.**
- **La línea dura:** un evento es **un verbo + un número, jamás el objeto**. SÍ: activaciones, # palabras (entero), modelo usado, idioma, tipo de error, OS/arch, feature usage, paso de onboarding. NUNCA: texto/audio transcrito, snippets, paths, filenames, window titles, clipboard, IP, email, ID persistente cross-session.
- **Crashes → `tauri-plugin-sentry`** (timfish/sentry-tauri, soporta Tauri v2: panics Rust + JS + minidumps). Opt-in, `send_default_pii:false`, scrub en `before_send`. Para "100% local" literal: self-host **GlitchTip** (~$5/mes VPS, mismo DSN, swap a Sentry hosted con 1 línea).
- **Consentimiento:** opt-in (apagado) es la única postura coherente con el brand "tu voz nunca sale del Mac" + ePrivacy Art. 5(3). UX honesta en onboarding, publicar la lista de eventos.
- Legal: GDPR Recital 26 (datos anónimos fuera de scope); EDPB 2/2023 (almacenar estado en device puede requerir consentimiento aunque sea anónimo) → opt-in lo resuelve.

## F5 — Pulido

- **🟡** sin CI en PRs — único workflow es release-on-tag. Añadir build + `cargo test` + `svelte-check` en push/PR.
- **🟡** tests: `commands.rs` (909 LOC, 0 tests), `whisper_backend.rs` (0 tests), `vad.rs` (0 tests). ~32 tests existentes, ~15% cobertura, frontend 0%. Sin tests async (`#[tokio::test]`). Cubrir rutas críticas.
- **🟡** `commands.rs` god-file (909 LOC, 20 comandos + máquina de estados + paste + descarga + catálogo). Partir: `paste.rs` (clipboard+CGEvent), `models.rs` (download+catálogo), `recording.rs` (streaming loop+state).
- Lógica de resolución de modelo duplicada 4× en commands.rs (160, 331, 587, catálogo) → extraer `resolve_model_path()`.

## NO bloquea (futuro)
- Investigación profunda de tácticas de lanzamiento (Product Hunt/HN, Homebrew cask, pricing) — el agente se cortó por límite de gasto mensual de la cuenta. Re-correr cuando se reponga. No bloquea porque ya vamos por descarga directa.

## Fuentes web clave
MacStories/MacRumors/WWDC25-277 (SpeechAnalyzer macOS 26 + Apple Silicon) · whisper.cpp ggml-org (Metal solo Apple Silicon) · Aptabase tauri-plugin · TelemetryDeck anonymization · timfish/sentry-tauri · GlitchTip pricing · Tauri updater docs · EDPB Guidelines 2/2023.
