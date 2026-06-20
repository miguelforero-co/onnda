# Handoff — onnda (ex Voz Local) · 2026-06-20

Estado para retomar tras resetear contexto. App macOS de dictado 100% local (Tauri 2 + Svelte 5 + Whisper/Apple Speech).

## Dónde está todo
- **Repo:** `github.com/miguelforero-co/voz-local` (**PÚBLICO** — decisión informada del usuario). El repo sigue llamándose `voz-local` aunque la marca es **onnda**.
- **Rama activa:** `feat/onnda-ui-redesign` (TODO el rediseño + rebrand). **No mergeada a `main`.** `main` ya está pusheado (milestone previo).
- Spec/plan del rediseño: `docs/superpowers/specs/2026-06-16-onnda-ui-redesign-design.md` y `docs/superpowers/plans/2026-06-16-onnda-ui-redesign.md`.

## Qué se hizo (esta tanda)
1. **Rebrand Voz Local → onnda** en todo lo visible (binario/crate `onnda`/`onnda_lib`, Info.plist, release.yml, updater owner→miguelforero-co, package.json, UI, README reescrito). `cargo check` + `npm check` verdes.
2. **Rediseño UI completo "onnda"** (flat B&N): `tokens.css` reescrito (light/dark vía `[data-theme]`, store de tema light/dark/auto), atomic design en `src/lib/components/ui/`, shell con base negra + costura 2px + textura de puntos, las 5 pantallas migradas, fuente Goudy (@fontsource) + iconos Iconoir.
3. **Features:** banner de Accesibilidad + comando `request_accessibility` (prompt que registra la app en la lista); slider de sensibilidad de la animación del mic (`mic_sensitivity`).
4. **Bug fix:** "Abrir carpeta de datos" → `open -a Finder` (el data dir termina en `.app`).
5. **Push** de `main` + `feat/onnda-ui-redesign` al remoto.

## Decisiones / pendientes (retomar aquí)
1. **Bundle `identifier`** sigue `com.vozlocal.app`. Cambiarlo a `com.onnda.app` = rebrand puro pero pierde la data de la app instalada (historial/modelos) + resetea permisos → hacerlo solo CON migración, o dejarlo. **Esperando decisión.**
2. **#21 Auth + nombre de usuario** — Home dice "Hey," sin nombre. Resolver capturando el nombre en onboarding. Decisión de producto: ¿cuenta/sign-in (Google/Apple) o solo "¿cómo te llamas?". **Esperando decisión.**
3. **#10 Rediseñar onboarding + alertas/mensajes** al sistema onnda (el banner a11y funciona pero no gusta el look). **#9** quitar el shim de tokens (`tokens.css`) cuando esto quede — hoy solo lo usan onboarding + banners en `+page.svelte`.
4. **design-tweaks:** skill instalado a nivel proyecto (`.claude/skills/design-tweaks/`). Hay un tweaker `dev/settings-page.html` para el layout de Ajustes; el usuario lo estaba ajustando — **falta que pegue el JSON/prompt** para aplicarlo.
5. **F3 firma+notarización** (cuenta Apple Developer) sigue bloqueada → necesaria para el primer release como onnda. **F4 métricas** (Aptabase) bloqueada.

## Dev
```bash
npm run tauri dev          # binario ahora: target/debug/onnda
npm run check              # tipos + svelte (0/0)
cargo check --manifest-path src-tauri/Cargo.toml
```
- **Permiso de Accesibilidad en dev:** macOS lo atribuye al **terminal** (Ghostty), no al binario. Recompilar Rust NO lo pierde (está en el terminal). En otro Mac, otorgar Accesibilidad al terminal que lanza la app.
- Iterar en dev (no compilar release cada cambio). No pedir permiso para cada paso (ver feedback del usuario en memoria).
