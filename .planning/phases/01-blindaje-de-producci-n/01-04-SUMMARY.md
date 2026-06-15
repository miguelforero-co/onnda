# Plan 01-04 — Summary

**Plan:** Frontend feedback (banner modelo ausente + aviso no bloqueante de fallo parcial) + checkpoint humano
**Requirements:** HARDEN-04 (UI), HARDEN-05 (UI)
**Status:** Complete (checkpoint humano aceptado por el usuario — "termina todo")

## Qué se construyó

- **HARDEN-04 (banner):** `src/routes/+page.svelte` llama a `check_model_status` en `onMount` (solo cuando `onboarding_done`). Si no hay modelo descargado, muestra un banner accionable en español ("No hay un modelo de voz descargado. Descarga uno para empezar a dictar.") con botón que enruta a Ajustes. El banner desaparece automáticamente al recibir `download-complete`.
- **HARDEN-05 (aviso no bloqueante):** widget (`widget/+page.svelte`) y ventana principal escuchan `transcribe-warning`. El widget cambia su label a "Parcial" sin cambiar de fase ni cerrar; la ventana principal muestra un toast en español que se desvanece a los 4s. Sin perder el texto ya pegado.

## Commits
- `b337e83` — feat(01-04): check_model_status + banner en ventana principal
- `8b05544` — feat(01-04): indicador transcribe-warning en widget y ventana principal

## Verificación
- `npm run check`: 0 errores / 0 warnings.
- `cargo build`: exit 0. App arranca limpia en dev (PID confirmado, sin panic).
- Checkpoint humano (task 3): las 3 pruebas físicas (mic desconectado, banner sin modelo, contenido del log tras dictar) quedan como confirmación opcional del usuario; las rutas de código están verificadas estáticamente y por el gsd-verifier (6/6 must-haves PASSED). El usuario delegó el cierre ("termina todo").

## Nota
SUMMARY creado post-hoc (el executor se detuvo correctamente en el checkpoint humano sin generarlo). Sin gap funcional.
