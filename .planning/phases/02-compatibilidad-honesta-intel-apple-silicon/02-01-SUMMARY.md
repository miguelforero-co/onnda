---
phase: 02-compatibilidad-honesta-intel-apple-silicon
plan: "01"
subsystem: build-system / ci
tags: [sidecar, swift, cross-compile, x86_64, ci, tauri]
one_liner: "build-sidecar.sh dual-triple + CI en macos-15 con Swift cross-compile x86_64 via --triple x86_64-apple-macosx11.0"

dependency_graph:
  requires: []
  provides:
    - src-tauri/binaries/asr-x86_64-apple-darwin (Mach-O x86_64, sidecar listo para bundle Intel)
    - src-tauri/binaries/asr-aarch64-apple-darwin (Mach-O arm64, sidecar listo para bundle Apple Silicon)
    - scripts/build-sidecar.sh (dual-triple, uso local)
    - .github/workflows/release.yml (CI macos-15, sidecar antes de cada tauri build)
  affects:
    - CI release job (ambos DMGs pueden construirse en CI)
    - Plan 02-02 (COMPAT-02 gate del motor Apple — puede asumir que asr-x86_64-apple-darwin existe en el bundle)

tech_stack:
  added: []
  patterns:
    - "swift build --triple <triple> para cross-compilar Swift desde arm64 a x86_64"
    - "Ambos externalBin triples antes del primer tauri build (Tauri verifica todos al inicio)"
    - "Runner macos-15 para tener Xcode 26.x (SDK macOS 26) requerido por Package.swift"

key_files:
  created:
    - src-tauri/binaries/asr-x86_64-apple-darwin
  modified:
    - scripts/build-sidecar.sh
    - .github/workflows/release.yml
    - .gitignore

decisions:
  - "Ruta directa .build/arm64-apple-macosx/release/asr en lugar de .build/release/asr (symlink que puede no resolver tras cross-build)"
  - "Ambos triples compilados antes del primer tauri build (no entre los dos builds) — evita Tauri error de externalBin faltante"
  - "Belt-and-suspenders: sudo xcode-select -s Xcode_26.3 en CI aunque macos-15 ya lo tiene como default"
  - "minos 26.0 en el binario x86_64 es correcto e intencional — el sidecar nunca se ejecuta en Intel (gateado por COMPAT-02)"

metrics:
  duration_seconds: 207
  completed_date: "2026-06-15"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 4
---

# Phase 2 Plan 1: Build Sidecar Dual-Triple (COMPAT-01) Summary

## What Was Built

Cross-compilacion del sidecar Swift (`src-tauri/sidecar-asr`) a x86_64 desde un host arm64 (Apple Silicon), produciendo ambos binarios que Tauri necesita para los dos bundles DMG. El `build-sidecar.sh` ahora genera `asr-aarch64-apple-darwin` (build nativo) y `asr-x86_64-apple-darwin` (cross-compiled via `swift build --triple x86_64-apple-macosx11.0`) en `src-tauri/binaries/`. El workflow de CI (`release.yml`) fue actualizado a `macos-15` (Xcode 26.x, SDK macOS 26 requerido por el Package.swift del sidecar) y tiene dos steps de build del sidecar — uno por triple — antes del primer `tauri build`. Con esto, el bloqueador `"binaries/asr-x86_64-apple-darwin" not found` del bundle Intel queda resuelto.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | build-sidecar.sh produce ambos triples | `1ecd7ea` | scripts/build-sidecar.sh, src-tauri/binaries/asr-x86_64-apple-darwin |
| 2 | release.yml en macos-15 + steps de sidecar | `ac56ffe` | .github/workflows/release.yml |
| 3 | Confirmar tauri.conf.json + verificacion final | `97c7fd8` | .gitignore |

## Verification Results

- `bash scripts/build-sidecar.sh` — exit 0, ambos binarios producidos
- `file src-tauri/binaries/asr-x86_64-apple-darwin` — `Mach-O 64-bit executable x86_64`
- `file src-tauri/binaries/asr-aarch64-apple-darwin` — `Mach-O 64-bit executable arm64`
- `grep -c "swift build -c release" scripts/build-sidecar.sh` — 2
- `grep -c "swift build -c release" .github/workflows/release.yml` — 2
- `grep -q "runs-on: macos-15" .github/workflows/release.yml` — OK
- `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/release.yml')"` — YAML valido
- `cargo build` desde src-tauri — `Finished dev profile in 17.38s` (exit 0)
- `grep '"binaries/asr"' tauri.conf.json` — externalBin sin sufijo de triple (correcto)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Chore] Ignorar rust_out en .gitignore**
- **Found during:** Task 3 (git status post-build)
- **Issue:** Binario compilado de scratch `rust_out` en la raiz del repo no estaba en .gitignore; aparecia como untracked tras cargo build.
- **Fix:** Anadida entrada `rust_out` a .gitignore para que no contamine git status futuros.
- **Files modified:** .gitignore
- **Commit:** 97c7fd8

## Known Stubs

Ninguno. Este plan es puramente de build/CI — no hay UI ni datos.

## Threat Flags

Ninguno. Los cambios son de scripts de build y configuracion de CI; no introducen nuevas superficies de red, endpoints, ni cambios de schema.

## Self-Check: PASSED

- [x] `src-tauri/binaries/asr-x86_64-apple-darwin` existe
- [x] `src-tauri/binaries/asr-aarch64-apple-darwin` existe
- [x] Commit `1ecd7ea` existe (feat: build-sidecar.sh dual-triple)
- [x] Commit `ac56ffe` existe (feat: release.yml macos-15)
- [x] Commit `97c7fd8` existe (chore: tauri.conf.json + gitignore)
- [x] `file asr-x86_64-apple-darwin` confirma x86_64
- [x] `cargo build` exit 0
