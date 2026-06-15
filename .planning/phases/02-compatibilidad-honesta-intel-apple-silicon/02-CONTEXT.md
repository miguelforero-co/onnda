# Phase 2: Compatibilidad honesta (Intel + Apple Silicon) - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning
**Source:** Derivado de `.planning/research/LAUNCH-DIAGNOSIS.md` (sección F2) + decisión del usuario "soportar Intel Y Apple Silicon; Neural Engine se mantiene para Silicon".

<domain>
## Phase Boundary

La app se instala y funciona bien en **Intel y Apple Silicon**, sin ofrecer features que no existen en cada hardware, y eligiendo un modelo razonable por máquina. Es compatibilidad + honestidad de UI; NO toca firma/notarización (F3) ni métricas (F4).

**Dentro de scope:** que el DMG x86_64 se construya, gatear el motor Apple a donde existe, modelo por defecto según hardware, threads = cores reales, fix del flash del notch.

**Fuera de scope:** firma/notarización (F3), repo público (F3), métricas (F4), tests/refactor (F5), cambiar el motor por defecto en Apple Silicon (sigue siendo elección del usuario).
</domain>

<decisions>
## Implementation Decisions

### COMPAT-01 — El DMG de Intel (x86_64) se construye
- **Problema:** el sidecar Apple solo existe como `asr-aarch64-apple-darwin`. `tauri.conf.json:52` `externalBin:["binaries/asr"]` → el build `--target x86_64-apple-darwin` busca `asr-x86_64-apple-darwin` que no existe → el bundle Intel falla.
- **Decisión:** producir también el binario x86_64 del sidecar para que el `externalBin` resuelva en ambos targets. El sidecar Swift usa `@available(macOS 26)` → compila para x86_64 sin problema (en runtime Intel simplemente nunca entra a esa rama; el motor Apple queda gateado por COMPAT-02 de todas formas).
- `scripts/build-sidecar.sh` hoy solo compila el host-triple (`rustc --print host-tuple`). Debe generar **ambos** triples (aarch64 + x86_64) y/o un universal.
- **Claude's Discretion / a investigar:** la invocación exacta de `swift build` para cross-compilar a x86_64 desde un Mac arm64 (`--arch x86_64` / `-Xswiftc -target x86_64-apple-macosx`), vs. construir universal con `lipo`. Y cómo el CI (`.github/workflows/release.yml`) debe invocar el build del sidecar (hoy NO lo invoca) para cada DMG.

### COMPAT-02 — Motor Apple solo donde funciona
- `commands.rs:774–781` (`get_models`): la entrada Apple está hardcodeada `downloaded:true, coming_soon:false`. Gatearla: visible/usable solo si `#[cfg(target_arch="aarch64")]` **Y** la versión de macOS es ≥ 26 **Y** el sidecar `asr` está presente. Donde no aplique: omitir la card, o mostrarla deshabilitada con tooltip ("Requiere Apple Silicon + macOS 26").
- **Claude's Discretion:** omitir vs. deshabilitar-con-explicación (preferencia: deshabilitada con tooltip, más informativo). Cómo detectar la versión de macOS en Rust (objc2 `ProcessInfo.operatingSystemVersion` o `sysctl kern.osproductversion`).

### COMPAT-03 — Modelo por defecto según hardware
- Hoy el default resuelve a `large-v3-turbo` para todos (commands.rs ~155). En un Intel de 8GB es inusable.
- **Decisión:** al primer arranque (cuando `selected_model` está vacío), elegir según hardware: Apple Silicon + macOS 26 → motor Apple (`apple-speech`) o `large-v3-turbo`; Apple Silicon sin macOS 26 → `large-v3-turbo`; Intel o RAM < 16GB → `small` (o `base`). Detectar arch (`cfg!(target_arch)`) + RAM física (`sysctl hw.memsize`).
- **Claude's Discretion:** umbral exacto de RAM, y si el default Apple-Silicon es `apple-speech` o `large-v3-turbo` (Whisper sigue siendo DEFAULT histórico — no romper esa expectativa; aplicar la auto-selección solo cuando no hay preferencia guardada).

### COMPAT-04 — Threads = cores reales
- `whisper_backend.rs:55` `set_n_threads(6)` fijo, más que los cores de una máquina vieja.
- **Decisión:** limitar a `min(6, available_parallelism())` (o similar) usando `std::thread::available_parallelism()`.

### COMPAT-05 — Sin flash de forma-notch en pantallas sin notch
- `src/routes/widget/+page.svelte:14` `hasNotch = $state(true)` → un frame con forma de notch antes del primer evento `screen-notch`. El fallback de no-notch en sí YA funciona.
- **Decisión:** init `hasNotch = false` (o no renderizar la forma hasta el primer evento). Cambio cosmético mínimo.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Diagnóstico de origen (autoridad)
- `.planning/research/LAUNCH-DIAGNOSIS.md` — sección "F2 — Compatibilidad honesta" con file:line y los hechos web confirmados (SpeechAnalyzer = macOS 26 + Apple Silicon; whisper.cpp Metal solo Apple Silicon; large-v3 ~3-4GB).

### Código a modificar (leer estado actual antes de tocar)
- `scripts/build-sidecar.sh` — hoy solo host-triple (COMPAT-01)
- `.github/workflows/release.yml` — dos DMGs arch-específicos, NO invoca el build del sidecar (COMPAT-01)
- `src-tauri/tauri.conf.json` — `externalBin`, `bundle.targets`, `minimumSystemVersion` (COMPAT-01)
- `src-tauri/src/commands.rs` — `get_models` (~774, COMPAT-02), resolución de modelo por defecto (~155, COMPAT-03)
- `src-tauri/src/whisper_backend.rs` — `set_n_threads` (55, COMPAT-04), gate GPU `#[cfg(target_arch="aarch64")]` (35-38, ya correcto)
- `src-tauri/src/speech_backend.rs` — invocación del sidecar (para el check de presencia, COMPAT-02)
- `src/routes/widget/+page.svelte` — `hasNotch` init (14, COMPAT-05)

### Convenciones
- `./CLAUDE.md`; mensajes de usuario en español. Iterar en `npm run tauri dev`.
- No podemos probar en hardware Intel real — la verificación de COMPAT-01 es que el build x86_64 **produzca** el bundle sin fallar (cross-compile + presencia del sidecar x86_64), no un arranque en Intel.
</canonical_refs>

<specifics>
## Specific Ideas

- El gate de GPU en whisper_backend.rs (aarch64-only) YA es correcto — no tocar, solo confirmar.
- La auto-selección de modelo (COMPAT-03) solo aplica cuando NO hay `selected_model` guardado — respetar la elección explícita del usuario siempre.
- Para COMPAT-02, reusar la lógica de detección de sidecar que ya existe en speech_backend.rs (`app.shell().sidecar("asr")` falla con "sidecar not available" si no está).
</specifics>

<deferred>
## Deferred Ideas

- Universal binary único (un solo DMG arm64+x86_64) en vez de dos DMGs — opcional, menor; si el cross-build de COMPAT-01 ya deja ambos DMGs correctos, el universal es P2 (puede ir en F3 con la firma).
- Probar en hardware Intel real → UAT del usuario / beta tester (no tenemos Intel).
</deferred>

---

*Phase: 02-compatibilidad-honesta-intel-apple-silicon*
*Context derivado del diagnóstico 2026-06-15*
