# Spec A — Métricas anónimas con Aptabase

**Fecha:** 2026-06-26
**Estado:** aprobado (brainstorming) — pendiente de plan
**Rama destino:** a definir (feature nueva, ortogonal al rediseño de UI)
**Relación:** desbloquea la fase **F4 (métricas opt-in)** del roadmap. Independiente de Spec B (cuentas) y Spec C (emails).

## Objetivo

Saber, de forma **anónima y agregada**, cuántos usuarios tiene onnda y cómo se usa: número de usuarios activos, número de transcripciones, uso en el tiempo y longitud de las transcripciones. Sin login, sin backend propio, sin enviar jamás contenido dictado.

## No-objetivos (YAGNI)

- **No** atar métricas a usuarios identificados (eso sería Spec B + backend, descartado para métricas).
- **No** enviar texto transcrito, nombres de archivo, ni ninguna PII.
- **No** construir dashboards propios: se usan los de Aptabase.

## Decisión de herramienta

**Aptabase** (`tauri-plugin-aptabase`, plugin oficial de Tauri 2, open-source, privacy-first). Ya era la elección documentada del proyecto para F4. Da usuarios activos (dispositivos anónimos únicos), conteo de eventos personalizados con propiedades, y uso en el tiempo (DAU/WAU). Cloud con free tier, o self-host.

## Postura de privacidad

- **Opt-in.** El envío está **apagado por defecto** hasta que el usuario consienta. Coherente con la marca "100% privado".
- Setting nuevo `analytics_enabled: bool` (default `false`).
- Se pregunta **una vez** en el onboarding (paso propio, copy claro: "Ayúdanos con estadísticas anónimas de uso. Nunca enviamos lo que dictas."). También togglable en Ajustes → (sección Privacidad o Datos).
- **Toda** llamada a analítica pasa por un guard central que verifica `analytics_enabled`; si está off, es no-op.

## Arquitectura

- **Rust:** módulo nuevo `src-tauri/src/analytics.rs`.
  - Inicializa el plugin con `APTABASE_APP_KEY` (ver Config).
  - Función `track(app, event: &str, props: Option<serde_json::Value>)` que: (1) lee `analytics_enabled` de settings; (2) si está on, llama al plugin; (3) si está off, retorna sin hacer nada.
  - Se registra el plugin en `lib.rs` (junto a los demás `.plugin(...)`).
- **Frontend:** helper `src/lib/analytics.ts` con `track(event, props)` que invoca un comando Tauri `track_event`, o usa el binding JS del plugin si `withGlobalTauri`. El guard vive en Rust (fuente de verdad única).

## Eventos a instrumentar

Solo conteos y propiedades numéricas/categóricas. **Nunca** texto del usuario.

| Evento | Propiedades | Dónde se dispara |
|---|---|---|
| `app_launched` | — | arranque (`lib.rs` setup) |
| `transcription_completed` | `engine` (whisper\|apple), `model`, `language`, `source` (dictation\|file), `word_count` (int), `char_count` (int), `duration_ms` (int) | al terminar una transcripción (`commands.rs` / `recording.rs`) |
| `file_imported` | `model`, `duration_ms` | al transcribir un archivo importado |
| `model_downloaded` | `model` | al completar descarga de modelo |
| `engine_changed` | `engine` | al cambiar motor en Ajustes |
| `correction_learned` | — | cuando auto-learn promueve una regla |

`word_count`/`char_count` son enteros derivados del texto **ya descartado** — solo el número, jamás el texto.

## Config / secretos

- `APTABASE_APP_KEY`: la crea el usuario en su cuenta Aptabase (**único bloqueante**; tarea de cuenta, no de código).
- La key de Aptabase **no es secreta** (va embebida en apps cliente), así que puede vivir en el repo público o leerse de env en build. Se decide en el plan; por defecto, constante en `analytics.rs` o `env!()` en build.
- Self-host de Aptabase: opción futura, solo cambia el host base. No bloquea.

## Datos / flujo

```
acción del usuario (dicta / importa / cambia motor)
  -> analytics::track(app, "evento", props)
       -> ¿analytics_enabled? --no--> no-op
                              --sí--> plugin Aptabase -> Aptabase cloud (solo conteos)
```

## Manejo de errores

- Fallo de red / Aptabase caído: el plugin no debe romper ni bloquear la UX. `track` es fire-and-forget; cualquier error se ignora (a lo sumo log en debug). La app funciona idéntica offline.

## Verificación

- `analytics_enabled=false` → 0 eventos enviados (verificable con `read_network_requests` o el dashboard vacío).
- Con opt-in on, un dictado genera exactamente un `transcription_completed` con props correctas y **sin** texto.
- `cargo check` + `npm run check` verdes.
- Revisar en Aptabase que aparezcan: usuarios, eventos, props numéricas.

## Tareas externas (bloqueantes)

1. Crear cuenta Aptabase y obtener `APTABASE_APP_KEY`.
