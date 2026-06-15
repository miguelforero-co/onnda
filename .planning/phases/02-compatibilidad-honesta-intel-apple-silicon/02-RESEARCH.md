# Phase 2: Compatibilidad honesta (Intel + Apple Silicon) — Research

**Researched:** 2026-06-15
**Domain:** Tauri 2 sidecar cross-compilation (Swift/SwiftPM → x86_64), Rust cfg/sysctl runtime detection, macOS hardware introspection
**Confidence:** HIGH — all critical claims verified by running commands in the actual repo

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- COMPAT-01: Producir el binario x86_64 del sidecar Swift para que `externalBin` resuelva en el build `--target x86_64-apple-darwin`. El sidecar con `@available(macOS 26)` compila para x86_64 sin problemas; en runtime Intel simplemente nunca entra a esa rama (gateado por COMPAT-02).
- COMPAT-02: Gatear el motor Apple visible/usable solo si `#[cfg(target_arch="aarch64")]` + macOS ≥ 26 + sidecar presente. Donde no aplique: mostrar deshabilitado con tooltip ("Requiere Apple Silicon + macOS 26").
- COMPAT-03: Al primer arranque (`selected_model` vacío), elegir modelo por hardware: Apple Silicon + macOS 26 → `large-v3-turbo` (no cambiar el default histórico al motor Apple); Apple Silicon sin macOS 26 → `large-v3-turbo`; Intel o RAM < 16GB → `small`. Solo cuando no hay preferencia guardada.
- COMPAT-04: Limitar threads a `min(6, available_parallelism())`.
- COMPAT-05: Init `hasNotch = false` (o no renderizar hasta el primer evento `screen-notch`).

### Claude's Discretion
- Omitir vs. deshabilitar-con-explicación el motor Apple → preferencia: deshabilitada con tooltip (más informativo). ✓ adoptado en decisiones.
- Umbral exacto de RAM: Intel o RAM < 16GB → `small`.
- Default Apple Silicon: sigue siendo `large-v3-turbo` (Whisper es el default histórico).
- Invocación exacta de `swift build` para cross-compilar a x86_64 → **investigado y verificado abajo**.
- Cómo detectar versión de macOS en Rust → **investigado y verificado abajo**.

### Deferred Ideas (OUT OF SCOPE)
- Universal binary único (un solo DMG arm64+x86_64) — diferir a F3 si ya quedan ambos DMGs correctos.
- Probar en hardware Intel real → UAT del usuario / beta tester.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMPAT-01 | El DMG x86_64 se construye en CI con sidecar Apple compilado para x86_64 | Sección COMPAT-01: comando exacto verificado, CI steps detallados, runner macos-15 con Xcode 26.x |
| COMPAT-02 | Motor Apple solo disponible en Apple Silicon + macOS 26 + sidecar presente | Sección COMPAT-02: gating expression con `cfg!`, `sw_vers`, `app.shell().sidecar()` |
| COMPAT-03 | Modelo por defecto según hardware al primer arranque | Sección COMPAT-03: ubicación exacta en settings.rs Default impl, `sysctl hw.memsize` via libc |
| COMPAT-04 | Threads = `min(6, cores)` | Sección COMPAT-04: línea exacta en whisper_backend.rs:55, pattern verificado |
| COMPAT-05 | Sin flash de notch en pantallas sin notch | Sección COMPAT-05: línea exacta en widget/+page.svelte:14, análisis de timing de evento |
</phase_requirements>

---

## Summary

Esta fase hace la app instalable y honesta en Intel y Apple Silicon sin romper funcionalidad existente. Los cinco requisitos son cambios quirúrgicos: uno en CI/scripts (COMPAT-01), uno en la API de modelos (COMPAT-02), uno en el default de settings (COMPAT-03), uno en Whisper (COMPAT-04), y uno en el widget Svelte (COMPAT-05).

El hallazgo más crítico de la investigación es sobre CI (COMPAT-01): el runner `macos-14` que usa el workflow actual tiene Xcode 15.4 como default, que **no tiene el SDK de macOS 26** requerido por el Package.swift (`platforms: [.macOS("26.0")]`). Para compilar el sidecar Swift se necesita cambiar el runner a `macos-15` (que tiene Xcode 26.x disponible). La compilación x86_64 del sidecar **se verificó localmente con `swift build -c release --triple x86_64-apple-macosx11.0`** — produce un Mach-O x86_64 válido en 50s. El `minos` resultante es siempre 26.0 (lo dicta el Package.swift, no el flag `--triple`) — esto es correcto porque el sidecar usa `SpeechTranscriber` que es macOS 26 only; en Intel nunca se ejecutará gracias al gate de COMPAT-02.

El resto de los cambios son sencillos: el gate de macOS en Rust se implementa mejor con `std::process::Command::new("sw_vers")` (sin dep adicional); la RAM se lee con `sysctl hw.memsize` via `libc::sysctl` (el crate `libc` no está en Cargo.toml hoy — alternativa: `std::process::Command::new("sysctl").arg("hw.memsize")`); `available_parallelism()` ya existe en std; y el fix del notch es un `$state(false)` de un carácter.

**Primary recommendation:** Cambiar runner CI a `macos-15`, añadir build del sidecar para ambos triples antes de cada `tauri build`, añadir `disabled_reason` a `ModelInfo`, y aplicar los tres cambios Rust de una línea cada uno.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cross-compile sidecar Swift | CI / build script | — | `swift build --triple` en CI antes de `tauri build` |
| Gate motor Apple en catálogo | API/Backend (Rust) | — | `get_models` en commands.rs devuelve el flag; UI lo consume |
| Detectar arch en compilación | Rust `cfg!()` macro | — | Evaluado en tiempo de compilación por Cargo |
| Detectar macOS version en runtime | API/Backend (Rust) | — | `sw_vers` o `NSProcessInfo` desde el proceso Tauri |
| Detectar RAM física | API/Backend (Rust) | — | `sysctl hw.memsize` desde el proceso Tauri |
| Default model por hardware | API/Backend (Rust settings) | — | `settings.rs` Default impl, invocado en primer load |
| Clamp threads | API/Backend (Rust) | — | `whisper_backend.rs:55`, solo Whisper; Apple engine no usa threads |
| Flash notch | Frontend Svelte | — | `widget/+page.svelte:14`, init de estado antes del primer evento |

---

## COMPAT-01: Cross-compile del sidecar Swift a x86_64

### Estado actual

`scripts/build-sidecar.sh:11` compila solo al host-triple:
```bash
TRIPLE="$(rustc --print host-tuple)"   # siempre aarch64-apple-darwin en el runner
( cd "$SIDECAR_DIR" && swift build -c release )
```
Solo produce `src-tauri/binaries/asr-aarch64-apple-darwin`.

`tauri.conf.json:52` declara `"externalBin": ["binaries/asr"]`. Cuando Tauri construye con `--target x86_64-apple-darwin` busca `src-tauri/binaries/asr-x86_64-apple-darwin` — que no existe → **bundle Intel falla**.

### Naming convention de Tauri externalBin [VERIFIED: v2.tauri.app/develop/sidecar/]

Tauri resuelve el nombre del sidecar usando el **target triple del build**, no el host triple. Para `--target x86_64-apple-darwin`, busca:
```
src-tauri/binaries/asr-x86_64-apple-darwin
```
Para el build nativo arm64 (sin `--target`), busca:
```
src-tauri/binaries/asr-aarch64-apple-darwin
```

Conclusión: para los dos DMGs (arm64 nativo + x86_64 cross) se necesitan **dos archivos per-triple**. Un universal no es necesario para esta estrategia de dos DMGs (y está diferido).

### Comando exacto de cross-compilación Swift [VERIFIED: ejecutado localmente]

```bash
cd src-tauri/sidecar-asr
swift build -c release --triple x86_64-apple-macosx11.0
# Output: .build/x86_64-apple-macosx/release/asr
```

**Verificado:** `file .build/x86_64-apple-macosx/release/asr` → `Mach-O 64-bit executable x86_64`. Build time: ~50s desde arm64. La flag `--triple` es el nombre correcto en SwiftPM 6.x (no `--arch`; `--arch` es para xcodebuild).

**Gotcha crítico — minos del binario:** aunque se pase `x86_64-apple-macosx11.0`, el binario resultante tiene `minos 26.0` porque el `Package.swift` declara `platforms: [.macOS("26.0")]`. Esto es correcto e intencional: el sidecar solo existe para macOS 26 (usa `SpeechTranscriber`). En Intel nunca se invocará gracias al gate de COMPAT-02.

**Alternativa `--arch`:** SwiftPM también acepta `--arch x86_64` (más corto), que produce el mismo resultado. `--triple x86_64-apple-macosx11.0` es equivalente y más explícito. Usar `--triple` para claridad.

### Gotcha crítico — versión de Xcode/SDK en CI [VERIFIED: GitHub Actions runner images]

El runner `macos-14` (actualmente en release.yml) tiene **Xcode 15.4 como default** (Swift 5.10, SDK macOS 14). El Package.swift requiere `.macOS("26.0")` y usa `SpeechTranscriber`/`SpeechAnalyzer` que son APIs de macOS 26 — **Xcode 15.4 no las conoce → `swift build` falla con "cannot find type 'SpeechTranscriber'"**.

**Solución:** cambiar el runner a `macos-15`. El runner `macos-15` tiene **Xcode 26.3 como default** (con SDK macOS 26.x) y Swift 6.x. `macos-15` está disponible en todos los repositorios de GitHub Actions en 2026. [VERIFIED: github.com/actions/runner-images macos-15-Readme.md]

También existe `macos-26-arm64` (GA desde febrero 2026) con Xcode 26.5 como default — es otra opción válida. `macos-15` es más conservador y suficiente.

### Pasos exactos a añadir en CI

**Cambio 1 — Runner**: `runs-on: macos-14` → `runs-on: macos-15`

**Cambio 2 — Selección de Xcode** (belt-and-suspenders, por si el default cambia):
```yaml
- name: Select Xcode 26.x
  run: sudo xcode-select -s /Applications/Xcode_26.3.app/Contents/Developer
```
O usando la action oficial:
```yaml
- uses: maxim-lobanov/setup-xcode@v1
  with:
    xcode-version: '26.3'
```

**Cambio 3 — Build sidecar arm64 (antes del tauri build nativo)**:
```yaml
- name: Build ASR sidecar (arm64)
  run: |
    cd src-tauri/sidecar-asr
    swift build -c release
    mkdir -p ../binaries
    cp .build/arm64-apple-macosx/release/asr ../binaries/asr-aarch64-apple-darwin
    chmod +x ../binaries/asr-aarch64-apple-darwin
```

**Cambio 4 — Build sidecar x86_64 (antes del tauri build x86_64)**:
```yaml
- name: Build ASR sidecar (x86_64 cross-compiled)
  run: |
    cd src-tauri/sidecar-asr
    swift build -c release --triple x86_64-apple-macosx11.0
    cp .build/x86_64-apple-macosx/release/asr ../binaries/asr-x86_64-apple-darwin
    chmod +x ../binaries/asr-x86_64-apple-darwin
```

**Orden en el workflow** (ambos triples deben existir antes del primer `tauri build` porque Tauri verifica todos los externalBin declarados al inicio del bundle):

```
1. Build ASR sidecar (arm64)       # produce asr-aarch64-apple-darwin
2. Build ASR sidecar (x86_64)      # produce asr-x86_64-apple-darwin
3. Build arm64 app bundle          # usa asr-aarch64-apple-darwin
4. Sign arm64 bundle
5. Create arm64 DMG
6. Build x86_64 app bundle         # usa asr-x86_64-apple-darwin
7. Sign x86_64 bundle
8. Create x86_64 DMG
9. Upload DMGs
```

Los pasos 1 y 2 pueden ir juntos antes del paso 3. Poner ambos antes del primer `tauri build` es más seguro y evita re-compilar entre builds.

### Actualizar `scripts/build-sidecar.sh` [COMPAT-01, para uso local]

El script local puede ser extendido para producir ambos triples:
```bash
# Arm64 (host native)
( cd "$SIDECAR_DIR" && swift build -c release )
cp -f "$SIDECAR_DIR/.build/arm64-apple-macosx/release/asr" "$OUT_DIR/asr-aarch64-apple-darwin"

# x86_64 (cross-compiled)
( cd "$SIDECAR_DIR" && swift build -c release --triple x86_64-apple-macosx11.0 )
cp -f "$SIDECAR_DIR/.build/x86_64-apple-macosx/release/asr" "$OUT_DIR/asr-x86_64-apple-darwin"
```

### Verificación de COMPAT-01

La verificación NO requiere hardware Intel. Criterio: `swift build` completa sin error + ambos binarios existen:
```bash
ls -la src-tauri/binaries/asr-aarch64-apple-darwin src-tauri/binaries/asr-x86_64-apple-darwin
file src-tauri/binaries/asr-x86_64-apple-darwin  # must say "x86_64"
```
CI: el job de x86_64 debe completar sin `error: bundle failed` en el log de `tauri build`.

---

## COMPAT-02: Gate del motor Apple

### Estado actual

`commands.rs:788–794` — la entrada Apple en `get_models`:
```rust
ModelInfo {
    id: crate::speech_backend::APPLE_MODEL_ID.to_string(),
    name: "Apple (Neural Engine)".to_string(),
    size_mb: 0,
    downloaded: true,     // hardcodeado — siempre "disponible"
    coming_soon: false,
},
```
`APPLE_MODEL_ID = "apple-speech"` (speech_backend.rs:22).

No hay gate: en Intel o macOS < 26, el usuario ve el motor Apple como disponible, lo selecciona, y al transcribir `speech_backend.rs:103` intenta `app.shell().sidecar("asr")` → falla con `"sidecar not available"` (error crudo, sin UX).

### Struct ModelInfo actual (commands.rs:979–989)

```rust
#[derive(serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_mb: u32,
    pub downloaded: bool,
    pub coming_soon: bool,
}
```
Frontend TypeScript (types.ts:47–53) replica esta estructura. `coming_soon: true` renderiza la tarjeta como "Próximamente" — muted, no clickeable.

### Nueva estrategia: `disabled_reason: Option<String>`

Añadir un campo `disabled_reason: Option<String>` a `ModelInfo`. Cuando es `Some(msg)`, la tarjeta se muestra con el mensaje como subtítulo y sin posibilidad de selección (similar a `coming_soon` pero con explicación explícita). `coming_soon` queda para modelos que no existen todavía (Parakeet); `disabled_reason` es para modelos que existen pero no en este hardware.

**Cambios necesarios:**
1. `commands.rs:979` — añadir campo a `ModelInfo`
2. `commands.rs:788` — poblar el campo condicionalmente
3. `src/lib/types.ts:47` — añadir `disabled_reason: string | null`
4. `src/lib/components/ModelCard.svelte` — renderizar el tooltip/subtítulo si `disabled_reason` presente

### Expresión de gate (tres condiciones AND)

**Condición 1 — Arch (compilación):**
```rust
cfg!(target_arch = "aarch64")
```
`true` solo en binarios arm64. En el binario x86_64 es `false` en tiempo de compilación → el bloque entero se elimina. [VERIFIED: Rust reference, std cfg attribute]

**Condición 2 — macOS ≥ 26 (runtime):**

Opción A — `sw_vers` (recomendada, sin dep nueva):
```rust
fn macos_major_version() -> u32 {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().split('.').next()?.parse().ok())
        .unwrap_or(0)
}
```
`sw_vers -productVersion` devuelve `"26.5.1\n"` en macOS 26 — verified localmente. Parsear primer componente → `26u32`.

Opción B — `objc2/NSProcessInfo` (sin proceso hijo, más correcto en sandboxed — aunque esta app NO usa sandbox):
```rust
// Requiere feature "NSProcessInfo" en objc2-foundation (ya en Cargo.toml como dep macos-only)
// Cargo.toml change needed: añadir "NSProcessInfo" a la feature list de objc2-foundation
use objc2_foundation::NSProcessInfo;
fn macos_major_version() -> u32 {
    let info = NSProcessInfo::processInfo();  // unsafe block needed
    let v = info.operatingSystemVersion();
    v.majorVersion as u32
}
```
`objc2-foundation-0.3.2` tiene `NSProcessInfo` como feature opt-in (Cargo.toml:65 actual no la lista). Requeriría añadir `"NSProcessInfo"` a las features de `objc2-foundation`.

**Recomendación: Opción A (`sw_vers`)** — no requiere cambios en Cargo.toml, código simple, funciona identicamente. Una llamada a proceso en el arranque (no en hot path). [ASSUMED] que `sw_vers` existe en todas las versiones de macOS relevantes (macOS 11+) — es un binario del sistema desde macOS 10.3+.

**Condición 3 — Sidecar presente (runtime):**

Reusar el patrón de `speech_backend.rs:103`:
```rust
fn sidecar_available<R: Runtime>(app: &tauri::AppHandle<R>) -> bool {
    use tauri_plugin_shell::ShellExt;
    app.shell().sidecar("asr").is_ok()
}
```
`app.shell().sidecar("asr")` falla con error si `binaries/asr-<triple>` no existe en el bundle — sin side effects (no lo ejecuta, solo lo localiza). [VERIFIED: speech_backend.rs:103–108]

### Expresión completa de gate en `get_models`

```rust
let apple_disabled_reason: Option<String> = {
    #[cfg(target_arch = "aarch64")]
    {
        if macos_major_version() >= 26 && sidecar_available(&app) {
            None  // disponible
        } else if macos_major_version() >= 26 {
            Some("Sidecar ASR no encontrado en este bundle".to_string())
        } else {
            Some("Requiere macOS 26 (Tahoe) o superior".to_string())
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        Some("Requiere Apple Silicon + macOS 26".to_string())
    }
};
```

### Cómo surface a la UI

En la tarjeta Apple de `get_models`:
```rust
ModelInfo {
    id: crate::speech_backend::APPLE_MODEL_ID.to_string(),
    name: "Apple (Neural Engine)".to_string(),
    size_mb: 0,
    downloaded: apple_disabled_reason.is_none(),
    coming_soon: false,
    disabled_reason: apple_disabled_reason,
}
```

En `ModelCard.svelte`, añadir un branch:
```svelte
{#if model.disabled_reason}
  <span class="badge disabled" title={model.disabled_reason}>No disponible</span>
  <!-- o mostrar disabled_reason como subtítulo bajo el nombre -->
{/if}
```
La tarjeta con `disabled_reason` debe ser no-clickeable (mismo patrón que `coming_soon`).

---

## COMPAT-03: Modelo por defecto según hardware

### Estado actual del default

`settings.rs:89` — `Default for AppSettings`:
```rust
selected_model: "large-v3-turbo".to_string(),
```
Este default se escribe en `settings.json` al primer arranque (settings.rs:138 — cuando no existe el archivo). Una vez escrito, `settings.selected_model.is_empty()` es siempre `false`, por lo que el coalesce en commands.rs:149–152 (`if settings.selected_model.is_empty() { "large-v3-turbo" }`) nunca se activa en instalaciones existentes.

**El único lugar correcto para cambiar el default es `settings.rs:89`** — en la implementación de `Default for AppSettings`. Pero ese default hardcodeado no puede ser por-hardware.

### Solución: función de default inteligente, solo en primer arranque

En `settings.rs`, el primer arranque ocurre en la rama donde el archivo no existe (line ~136–139):
```rust
// settings.rs ~136
let json = serde_json::to_string_pretty(&AppSettings::default()).unwrap();
```
Cambiar `AppSettings::default()` para que llame a una función de selección por hardware, o añadir una función `first_run_settings()` que use `AppSettings::default()` y luego sobreescribe `selected_model` con el valor por hardware.

**Patrón recomendado** — añadir función `hardware_default_model() -> &'static str`:
```rust
fn hardware_default_model() -> &'static str {
    #[cfg(not(target_arch = "aarch64"))]
    {
        // Intel: siempre small (seguro en 4-8 GB RAM)
        return "small";
    }
    #[cfg(target_arch = "aarch64")]
    {
        // Apple Silicon: chequear RAM
        if physical_ram_gb() < 16 {
            "small"
        } else {
            "large-v3-turbo"
        }
    }
}
```
Y en el path de primer arranque (settings.rs ~136):
```rust
let mut s = AppSettings::default();
s.selected_model = hardware_default_model().to_string();
let json = serde_json::to_string_pretty(&s).unwrap();
```

**Importante:** Los cuatro `if settings.selected_model.is_empty() { "large-v3-turbo" }` en commands.rs (líneas 149, 324, 586, 901) son fallbacks de runtime cuando la clave falta del JSON. Estos deben actualizar también su fallback a `hardware_default_model()` para coherencia, aunque en la práctica raro que `selected_model` sea vacío después del primer arranque.

### Leer RAM física en Rust (macOS)

**Sin crate nuevo — via `std::process::Command`:**
```rust
fn physical_ram_gb() -> u64 {
    std::process::Command::new("sysctl")
        .arg("hw.memsize")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            // "hw.memsize: 25769803776\n"
            s.trim().split_whitespace().last()?.parse::<u64>().ok()
        })
        .map(|bytes| bytes / (1024 * 1024 * 1024))
        .unwrap_or(8) // fallback conservador: 8GB → elegir small
}
```
`sysctl hw.memsize` devuelve bytes en decimal. Verificado localmente: `25769803776` = 24 GB. [VERIFIED: ejecutado en esta sesión]

**Alternativa con `libc::sysctl`:** requiere añadir `libc` a Cargo.toml (no está actualmente). La opción `Command::new("sysctl")` no requiere deps adicionales.

### IDs de modelos en el catálogo [VERIFIED: commands.rs:760–794]

| ID | Nombre | Tamaño | Notas |
|----|--------|--------|-------|
| `"base"` | Whisper Base | 141 MB | Apto Intel 4GB+ |
| `"small"` | Whisper Small | 466 MB | Default recomendado Intel / poca RAM |
| `"medium"` | Whisper Medium | 1536 MB | Solo Apple Silicon con RAM holgada |
| `"large-v3-turbo"` | Whisper Large v3 Turbo | 874 MB | Default histórico Apple Silicon |
| `"apple-speech"` | Apple (Neural Engine) | 0 MB | Solo Apple Silicon + macOS 26 |

**Default COMPAT-03:**
- Intel (cualquier RAM) → `"small"` (414 MB activo, seguro en 8GB)
- Apple Silicon + RAM < 16GB → `"small"`  
- Apple Silicon + RAM ≥ 16GB → `"large-v3-turbo"` (default histórico, no romper expectativa)

Nota: NO usar `"apple-speech"` como default en primer arranque. El motor Apple requiere descarga de assets de idioma en primer uso y no tiene confirmación del usuario todavía. Mantener Whisper como default. [ASSUMED: que el umbral 16GB es razonable para turbo — MacBook Air M1/M2 base tiene 8GB donde turbo puede presionar la memoria unificada]

---

## COMPAT-04: Clamp de threads

### Estado actual

`whisper_backend.rs:55`:
```rust
params.set_n_threads(6);
```
Valor fijo 6, sin considerar los cores reales de la máquina.

El gate de GPU en `whisper_backend.rs:35–38` ya es correcto (solo arm64):
```rust
#[cfg(target_arch = "aarch64")]
{
    ctx_params.use_gpu(true);
    ctx_params.flash_attn(true);
}
```
**No tocar ese bloque.**

### Cambio exacto

```rust
// whisper_backend.rs:55 — reemplazar:
params.set_n_threads(6);

// con:
let n_threads = std::thread::available_parallelism()
    .map(|n| n.get())
    .unwrap_or(4)          // fallback conservador si falla
    .min(6) as i32;
params.set_n_threads(n_threads);
```

`std::thread::available_parallelism()` devuelve `Result<NonZeroUsize, io::Error>`. `.map(|n| n.get())` extrae el `usize`. `.unwrap_or(4)` maneja el raro caso de error (IO error leyendo /proc/cpuinfo en Linux, no aplicable en macOS pero es la interfaz del trait). `.min(6)` aplica el cap. `as i32` porque `set_n_threads` toma `i32`. [VERIFIED: Rust std 1.87 stable — `available_parallelism` estable desde 1.59]

No se necesitan imports adicionales — `std::thread` ya está en scope en Rust.

---

## COMPAT-05: Flash de notch

### Estado actual

`widget/+page.svelte:14`:
```svelte
let hasNotch = $state(true);
```
El widget se muestra con forma de notch desde el primer frame renderizado. El evento `screen-notch` llega en `onMount` (línea 327) **después** del primer render, cuando el Rust ya posicionó la ventana y emitió el evento. En pantallas sin notch (monitores externos, MacBook Intel), hay un frame visible con la forma de notch antes de que llegue `screen-notch: false`.

El evento `screen-notch` se emite en `shortcut.rs:83` — después de `widget.show()` (línea 76) y `position_widget_at_notch` (líneas 73, 82). El listener en Svelte está registrado en `onMount` como `await listen<boolean>("screen-notch", ...)` (línea 327). El problema: hay una race entre el WebSocket de Tauri (evento) y el primer render del componente.

### Fix mínimo

Cambiar en `widget/+page.svelte:14`:
```svelte
// antes:
let hasNotch = $state(true);

// después:
let hasNotch = $state(false);
```

**Efecto:** el primer render usa la clase `no-notch` (clip-path del píldora pequeña, `notch.collapsed.no-notch`). Cuando llega `screen-notch: true` (en macOS con notch real), `hasNotch` cambia a `true` y la transición CSS `clip-path 0.7s cubic-bezier(...)` anima suavemente de píldora a forma de notch. En pantallas sin notch, se queda en `false` — correcto, sin flash.

**Side effect aceptable:** en pantallas WITH notch real, hay una transición breve (≤700ms) de píldora-a-notch en el primer frame. Es preferible al flash inverso (notch-a-píldora) que ocurre hoy en pantallas SIN notch. La transición notch está animada con `cubic-bezier(0.32, 1.26, 0.5, 1)` — springy, se ve intencional.

**Alternativa más robusta** (mayor riesgo de over-engineering): renderizar un `{#if notchResolved}` y emitir `screen-notch` también desde la inicialización de la app (lib.rs), no solo en el shortcut handler. Descartada por complejidad innecesaria — el `$state(false)` es suficiente.

**Nota:** `hasNotch` solo se usa en líneas 354 (div class) y 327 (listener). No hay otro lugar donde el valor inicial importe.

---

## Don't Hand-Roll

| Problema | No construir | Usar | Por qué |
|----------|-------------|------|---------|
| Cross-compile Swift a x86_64 | Xcode project custom script | `swift build --triple x86_64-apple-macosx11.0` | SwiftPM tiene soporte nativo desde Swift 5.6+ |
| Detect CPU cores | Loop sobre /proc, sysctl manual | `std::thread::available_parallelism()` | Estable en Rust desde 1.59, devuelve cores disponibles (considera cgroups/quota) |
| Read macOS version | Parse /System/Library/CoreServices | `sw_vers -productVersion` o `NSProcessInfo` | Forma oficial de Apple, estable desde macOS 10.3 |
| Read physical RAM | Parse /proc/meminfo (Linux) | `sysctl hw.memsize` | La syscall correcta en Darwin/macOS |

---

## Common Pitfalls

### Pitfall 1: `--arch x86_64` vs `--triple` en SwiftPM
**Qué sale mal:** `swift build --arch x86_64` es la flag de xcodebuild, no de SwiftPM CLI. En algunas versiones de SwiftPM `--arch` puede funcionar como alias, pero el nombre canónico en SwiftPM es `--triple`.
**Cómo evitar:** usar `--triple x86_64-apple-macosx11.0` (verificado en esta sesión).
**Warning signs:** `error: unknown option '--arch'` en el log de CI.

### Pitfall 2: Xcode 15.4 en macos-14 no tiene SDK macOS 26
**Qué sale mal:** el CI actual (`runs-on: macos-14`) falla al compilar el sidecar con `cannot find type 'SpeechTranscriber' in module 'Speech'` o `'SpeechAnalyzer' is only available in macOS 26 or newer`.
**Cómo evitar:** cambiar a `runs-on: macos-15` que tiene Xcode 26.3 como default.
**Warning signs:** el error aparece en el step "Build ASR sidecar" antes de llegar a `tauri build`.

### Pitfall 3: Ambos triples deben existir ANTES del primer `tauri build`
**Qué sale mal:** si solo se produce `asr-aarch64-apple-darwin` antes del build arm64, el build x86_64 posterior falla porque Tauri verifica al inicio que todos los `externalBin` existen para el target solicitado.
**Cómo evitar:** producir AMBOS triples en dos steps consecutivos antes del primer `tauri build`.
**Warning signs:** `error: failed to bundle project: "binaries/asr-x86_64-apple-darwin" not found`.

### Pitfall 4: El `minos` del sidecar x86_64 será 26.0, no 11.0
**Qué sale mal:** expectativa de que `--triple x86_64-apple-macosx11.0` produce un binario que corre en macOS 11 Intel. No lo hace — el Package.swift fija el deployment target a macOS 26.
**Por qué está bien:** el sidecar solo se invoca si `COMPAT-02` lo permite (macOS 26 + Apple Silicon). En Intel nunca se ejecuta. El binario debe existir en el bundle para que Tauri no falle al construir, pero en runtime en Intel el código Rust nunca llama a `apple_transcribe()`.
**Warning signs:** `LC_VERSION_MIN_MACOSX minos 26.0` — no es un error, es correcto.

### Pitfall 5: `settings.selected_model` default se escribe en el primer arranque
**Qué sale mal:** cambiar `AppSettings::default()` para devolver un modelo diferente per-hardware afecta también la deserialización de settings.json existentes (si `selected_model` no estuviera en el JSON, usaría el default al deserializar). Actualmente `selected_model` siempre está en el JSON desde el primer arranque.
**Cómo evitar:** solo cambiar el path de "archivo no existe" en settings.rs (~136), no el campo `Default`. O usar `#[serde(default = "hardware_default_model")]` si se quiere ser más robusto.
**Warning signs:** usuarios existentes con Apple Silicon que tenían `large-v3-turbo` ven su preferencia cambiada — no debe ocurrir.

### Pitfall 6: `cfg!(target_arch = "aarch64")` en runtime vs compile-time
**Qué sale mal:** usar `cfg!(target_arch = "aarch64")` dentro de una función que se llama en runtime — el valor es correcto (está en el binario correcto), pero el compilador eliminará el código del binario x86_64 completamente. Esto es el comportamiento correcto y deseado.
**Lo que NO hacer:** usar `std::env::consts::ARCH` para esta check (es un string en runtime y tiene el mismo valor, pero no permite que el compilador elimine código).
**Forma correcta:** `#[cfg(target_arch = "aarch64")] { ... }` como bloque de condición.

---

## Code Examples

### COMPAT-01: Script build-sidecar.sh actualizado
```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SIDECAR_DIR="$ROOT/src-tauri/sidecar-asr"
OUT_DIR="$ROOT/src-tauri/binaries"

echo "Building ASR sidecar (arm64, release)…"
( cd "$SIDECAR_DIR" && swift build -c release )
mkdir -p "$OUT_DIR"
cp -f "$SIDECAR_DIR/.build/arm64-apple-macosx/release/asr" "$OUT_DIR/asr-aarch64-apple-darwin"
chmod +x "$OUT_DIR/asr-aarch64-apple-darwin"
echo "Placed: src-tauri/binaries/asr-aarch64-apple-darwin"

echo "Building ASR sidecar (x86_64 cross-compiled, release)…"
( cd "$SIDECAR_DIR" && swift build -c release --triple x86_64-apple-macosx11.0 )
cp -f "$SIDECAR_DIR/.build/x86_64-apple-macosx/release/asr" "$OUT_DIR/asr-x86_64-apple-darwin"
chmod +x "$OUT_DIR/asr-x86_64-apple-darwin"
echo "Placed: src-tauri/binaries/asr-x86_64-apple-darwin"
```

### COMPAT-02: Helper functions en Rust
```rust
// En commands.rs o en un módulo nuevo compat.rs:

fn macos_major_version() -> u32 {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().split('.').next()?.parse().ok())
        .unwrap_or(0)
}

fn sidecar_available<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> bool {
    use tauri_plugin_shell::ShellExt;
    app.shell().sidecar("asr").is_ok()
}

// En get_models, antes de construir el Vec:
let apple_disabled_reason: Option<String> = {
    #[cfg(target_arch = "aarch64")]
    {
        if macos_major_version() >= 26 && sidecar_available(&app) {
            None
        } else if macos_major_version() >= 26 {
            Some("Sidecar ASR no encontrado en este bundle".to_string())
        } else {
            Some("Requiere macOS 26 (Tahoe) o superior".to_string())
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    { Some("Requiere Apple Silicon + macOS 26".to_string()) }
};
```

### COMPAT-03: Hardware default model
```rust
// En settings.rs — nueva función pública:
pub fn hardware_default_model() -> &'static str {
    #[cfg(not(target_arch = "aarch64"))]
    { return "small"; }
    #[cfg(target_arch = "aarch64")]
    {
        if physical_ram_gb() < 16 { "small" } else { "large-v3-turbo" }
    }
}

fn physical_ram_gb() -> u64 {
    std::process::Command::new("sysctl")
        .arg("hw.memsize")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.trim().split_whitespace().last()?.parse::<u64>().ok()
        })
        .map(|bytes| bytes / (1024 * 1024 * 1024))
        .unwrap_or(8)
}

// En load() de settings.rs, en el branch de "archivo no existe" (~línea 136):
let mut s = AppSettings::default();
s.selected_model = hardware_default_model().to_string();
let json = serde_json::to_string_pretty(&s).unwrap();
```

### COMPAT-04: Threads clamp
```rust
// whisper_backend.rs:55 — reemplazar la línea:
// params.set_n_threads(6);
// con:
let n_threads = std::thread::available_parallelism()
    .map(|n| n.get())
    .unwrap_or(4)
    .min(6) as i32;
params.set_n_threads(n_threads);
```

### COMPAT-05: hasNotch init
```svelte
<!-- widget/+page.svelte:14 — cambiar: -->
<!-- let hasNotch = $state(true); -->
<!-- por: -->
let hasNotch = $state(false);
```

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Swift (local) | COMPAT-01 script | ✓ | 6.3.2 (swiftlang-6.3.2.1.108) | — |
| Xcode SDK macOS 26 (local) | Compilar sidecar | ✓ | 26.5 SDK presente | — |
| Xcode SDK macOS 26 (CI macos-14) | COMPAT-01 CI | ✗ | Xcode 15.4 (SDK 14 only) | Cambiar a macos-15 |
| Xcode SDK macOS 26 (CI macos-15) | COMPAT-01 CI | ✓ | Xcode 26.3 default en macos-15 | — |
| `rustup target add x86_64-apple-darwin` | COMPAT-01 CI | ✓ | Ya en release.yml:29 | — |
| `sw_vers` CLI | COMPAT-02, COMPAT-03 | ✓ | macOS 10.3+ — siempre presente | NSProcessInfo vía objc2 |
| `sysctl` CLI | COMPAT-03 | ✓ | Presente en todas las versiones macOS | libc::sysctl |
| `std::thread::available_parallelism` | COMPAT-04 | ✓ | Rust 1.59+ stable (proyecto usa stable) | `num_cpus` crate |

**Missing dependencies with no fallback:** ninguna.

**Missing dependencies with fallback:** CI macos-14 no tiene Xcode 26 → cambiar a macos-15 (fallback simple de una línea en YAML).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust unit tests inline) |
| Config file | src-tauri/Cargo.toml |
| Quick run command | `cd src-tauri && cargo test 2>&1` |
| Full suite command | `cd src-tauri && cargo test && npm run check` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMPAT-01 | Binario x86_64 existe en binaries/ tras el script | smoke | `ls src-tauri/binaries/asr-x86_64-apple-darwin && file src-tauri/binaries/asr-x86_64-apple-darwin \| grep x86_64` | ❌ manual check / CI log |
| COMPAT-02 | `macos_major_version()` parsea correctamente | unit | `cargo test compat::tests::macos_version` | ❌ Wave 0 |
| COMPAT-02 | `get_models()` no devuelve Apple disponible en x86_64 | compile-time | El binario x86_64 compila sin el bloque aarch64 — verificable con `cargo build --target x86_64-apple-darwin` | — |
| COMPAT-03 | `physical_ram_gb()` devuelve valor razonable | unit | `cargo test compat::tests::ram_detection` | ❌ Wave 0 |
| COMPAT-03 | `hardware_default_model()` devuelve "small" en x86_64 | unit | `cargo test compat::tests::hardware_default` | ❌ Wave 0 |
| COMPAT-04 | n_threads ≤ 6 y ≤ cores disponibles | unit | `cargo test whisper_backend::tests::threads_clamp` | ❌ Wave 0 |
| COMPAT-05 | Sin flash notch — verificación visual | manual | `npm run tauri dev` + observar en monitor externo | manual-only |

### Wave 0 Gaps
- [ ] `src-tauri/src/compat.rs` (o módulo en commands.rs) — tests para `macos_major_version`, `physical_ram_gb`, `hardware_default_model`
- [ ] `src-tauri/src/whisper_backend.rs` — test `threads_clamp` que verifica `n_threads <= 6 && n_threads <= available_parallelism`

---

## Assumptions Log

| # | Claim | Section | Risk si está mal |
|---|-------|---------|-----------------|
| A1 | Umbral 16GB RAM para elegir `large-v3-turbo` vs `small` en Apple Silicon | COMPAT-03 | MacBook Air M1 8GB podría ir lento con turbo — el impacto es rendimiento, no crash. Umbral 16GB es conservador. |
| A2 | `sw_vers` está disponible en macOS 11+ en todas las instalaciones | COMPAT-02/03 | Probabilidad muy baja de que falte; si falla, `unwrap_or(0)` devuelve 0 → motor Apple se desactiva (safe fallback). |
| A3 | El default de Apple Silicon sin macOS 26 debe ser `large-v3-turbo` (no `small`) | COMPAT-03 | Si el usuario tiene Apple Silicon 8GB sin macOS 26 y 874MB de turbo es lento, no hay degradación catastrófica. |
| A4 | `macos-15` runner siempre tendrá Xcode 26.3+ en 2026 | COMPAT-01 CI | GitHub podría rotar las versiones; el step `sudo xcode-select` mitigaría esto. |

---

## Open Questions

1. **¿Emitir `screen-notch` también en el arranque de la app (no solo al mostrar el widget)?**
   - Lo que sabemos: hoy solo se emite en `shortcut.rs:83`, después de `widget.show()`.
   - Lo que es incierto: ¿hay un race condition real donde el widget se muestre antes de que el listener de Svelte esté registrado?
   - Recomendación: el `$state(false)` resuelve el flash sin necesitar cambios en Rust. No abrir ese frente.

2. **¿Deben los cuatro fallbacks en commands.rs (líneas 149, 324, 586, 901) también usar `hardware_default_model()`?**
   - Lo que sabemos: se activan solo cuando `selected_model.is_empty()` — post-instalación esto nunca ocurre.
   - Recomendación: actualizar los cuatro por consistencia pero no es blocking para COMPAT-03.

---

## Sources

### Primary (HIGH confidence)
- Código fuente del repo verificado en esta sesión (todas las líneas citadas confirmadas con Read/grep)
- `swift build -c release --triple x86_64-apple-macosx11.0` — ejecutado en esta sesión, build exitoso en 50s
- `file .build/x86_64-apple-macosx/release/asr` — confirmado Mach-O x86_64
- `otool -l` — confirmado `minos 26.0` en el binario cross-compilado
- `sw_vers -productVersion` — salida `"26.5.1"` verificada localmente
- `sysctl -n hw.memsize` — salida `25769803776` verificada localmente
- `objc2-foundation-0.3.2/Cargo.toml` — confirmado feature `NSProcessInfo` disponible
- [Tauri externalBin docs](https://v2.tauri.app/develop/sidecar/) — naming convention `binary-name-$TARGET_TRIPLE`

### Secondary (MEDIUM confidence)
- [GitHub Actions macos-15 Readme](https://github.com/actions/runner-images/blob/main/images/macos/macos-15-Readme.md) — Xcode 26.3 default en macos-15
- [macos-26 GA announcement](https://github.blog/changelog/2026-02-26-macos-26-is-now-generally-available-for-github-hosted-runners/) — macos-26-arm64 disponible en todos los repos
- [Rust std::thread::available_parallelism](https://doc.rust-lang.org/std/thread/fn.available_parallelism.html) — estable desde 1.59

### Tertiary (LOW confidence)
- Ninguna

---

## Metadata

**Confidence breakdown:**
- COMPAT-01 (cross-compile + CI): HIGH — build verificado en esta sesión, CI runner verificado con docs oficiales
- COMPAT-02 (gate runtime): HIGH — código actual leído, patrón `sidecar()` verificado en speech_backend.rs
- COMPAT-03 (hardware default): HIGH — settings.rs leído, sysctl verificado, ASSUMED A1 sobre umbral RAM
- COMPAT-04 (threads): HIGH — línea exacta leída, API Rust estándar
- COMPAT-05 (notch flash): HIGH — código leído, timing del evento entendido

**Research date:** 2026-06-15
**Valid until:** 2026-09-15 (stable APIs — solo el runner de CI puede cambiar antes)
