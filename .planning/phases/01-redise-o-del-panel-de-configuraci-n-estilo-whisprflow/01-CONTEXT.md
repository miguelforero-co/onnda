# Phase 1: Rediseño del panel de configuración (estilo WhisprFlow) - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning

> El usuario delegó las decisiones de esta fase explícitamente ("haz lo tuyo, godspeed"). Todas las decisiones abajo son discreción de Claude con criterio de ingeniería, ancladas en el código existente y en la referencia de WhisprFlow. Downstream (researcher/planner) puede refinar HOW, pero el QUÉ y los defaults están fijados aquí.

<domain>
## Phase Boundary

Rediseñar la app principal (`src/routes/+page.svelte`, hoy 480×600 con header+tabs) a un **panel lateral navegable estilo WhisprFlow**: sidebar con secciones Home / Transcripciones / Diccionario / Ajustes. Incluye TODOS los settings nuevos que son UI o pegamento de macOS, la transcripción por archivos, la vista unificada de transcripciones y el diccionario como lista de items.

**FUERA de esta fase (movido a fases propias):**
- **Parakeet como motor ASR** → Phase 2 (motor nuevo vía FluidAudio/ANE, research pesado). En Fase 1 el selector de modelos solo deja *lugar visual* para él.
- **Auto-learn from corrections** → Phase 3 (feature ML completa). En Fase 1 solo se sientan las bases (diccionario como items + transcripciones editables si aplica).
</domain>

<decisions>
## Implementation Decisions

### Shell de UI (sidebar)
- **D-01:** Layout = **sidebar izquierda fija** (~200px) con ítems de navegación (icono + label) + área de contenido a la derecha. Reemplaza el header+tabs actual.
- **D-02:** Secciones de navegación: **Home**, **Transcripciones**, **Diccionario**, **Ajustes**. (El onboarding sigue siendo un flujo aparte que precede al shell.)
- **D-03:** **Home** = hub de dictado: estado/affordance de la acción de dictar (atajo actual visible), accesos rápidos y las transcripciones recientes. Es la primera pantalla tras el onboarding.
- **D-04:** **Conservar la paleta y materiales actuales** (beige `--bg`, panel `--panel`, coral `--coral`, etc.) — NO migrar a dark mode ni rehacer el sistema de color. Inspiración WhisprFlow = estructura/UX (sidebar, densidad, secciones), no su cromática.
- **D-05:** **Ventana** crece para acomodar el sidebar: objetivo **~880×640**, `resizable: true`. Actualizar `tauri.conf.json` (requiere relanzar la app — ver GOTCHA del proyecto).
- **D-06:** Reutilizar los componentes/estilos existentes (rows, toggles, badges, secciones) — extraer a componentes Svelte reutilizables donde reduzca duplicación.

### Ajustes — settings nuevos
- **D-07:** **Sonidos** = 3 toggles independientes (al **escuchar**, al **parar/transcribir**, al **cancelar**). Reproducir sonidos cortos sutiles. Implementación preferida: **nativo macOS (NSSound vía objc2)** disparado desde Rust en cada evento del state machine (`commands.rs`), porque el widget/main puede estar oculto y el Audio API del frontend no es fiable ahí. Assets: sonidos breves discretos (bundlear 3 archivos pequeños, o usar sonidos del sistema). Persistir 3 flags en `AppSettings`.
- **D-08:** **Pause-media** = 1 toggle. Al **empezar a escuchar**, pausar la reproducción multimedia del sistema; al **terminar** (transcribir/cancelar), reanudar. Implementación preferida: enviar la **media key Play/Pause** (NX_KEYTYPE_PLAY vía objc2/CGEvent) — solo togglear si había algo sonando. El researcher debe confirmar la vía más robusta (media key vs MediaRemote/AppleScript) y el manejo de "reanudar". Persistir flag.
- **D-09:** **Panel de permisos en Ajustes**: sección dedicada que muestra estado de **Micrófono** y **Accesibilidad** (reutilizar las filas del onboarding) con acción "Abrir ajustes" y polling de estado en vivo (ya existe `checkPerms`).
- **D-10:** **Launch at login**: el campo `autostart` ya existe en settings. Verificar que esté **realmente cableado** a `tauri-plugin-autostart`; si no, cablearlo (enable/disable al cambiar el toggle).
- **D-11:** **Hotkey recorder**: reemplazar el `<input type=text>` del atajo por un **capturador de teclas** (click → "presiona la combinación" → captura modificadores+tecla → valida → guarda con `shortcutChanged: true`). Mantener `Alt+Space` como default.
- **D-12:** **Modo del atajo** (push-to-talk vs mantener): ya existe el toggle `push_to_talk`. Conservar, ubicar en la sección de grabación del nuevo layout.
- **D-13:** **Selector de modelos Whisper ampliado**: ofrecer **base, small, medium, large-v3-turbo** (con tamaño/versión y estado descargado), cada uno descargable (ya existe `download_model` + progreso). Agregar las URLs/IDs faltantes (small, medium) en `commands.rs`. Mostrar **Parakeet como tarjeta "Próximamente / Phase 2"** (no funcional aún) para que el selector ya tenga el lugar.
- **D-14:** **Check for updates**: botón "Buscar actualizaciones" + (opcional) toggle de auto-check. Implementación preferida: **tauri-plugin-updater** contra releases de GitHub (`emeforero/voz-local`). El researcher debe confirmar viabilidad del endpoint/firma; si la infra de releases no está lista, entregar al menos UI + comando de check que reporte "estás al día / hay versión X".
- **D-15:** **Gestión de datos**: sección con (a) **abrir carpeta de cache** (revelar `app_data_dir`/models en Finder), (b) **borrar datos** con confirmación — separar: borrar historial/audios, borrar modelos descargados. Comandos nuevos en Rust.

### Transcripciones (archivos + vista unificada)
- **D-16:** **Modelo de datos unificado**: NO crear dos tablas separadas. Extender el store de historial con un campo **`source`** (`"dictation" | "file"`). La vista "Transcripciones" lista todas con filtro opcional por origen. (El usuario habló de "una tabla con los históricos" de archivos y "todas las transcripciones" — se cubren ambas con un store unificado + filtro.)
- **D-17:** **Transcripción por archivos**: botón **Upload** (tauri dialog) → decodificar el audio a PCM mono 16kHz → correr por el `TranscriptionBackend` existente → guardar como entrada con `source:"file"` + nombre de archivo original. Formatos objetivo: **wav, mp3, m4a** (decodificar con `symphonia`; reusar `resample()` existente). Mostrar progreso/estado.
- **D-18:** La vista de Transcripciones reutiliza el render del historial actual (reproducir audio, borrar, timestamp, duración) + filtro por origen + (nice-to-have) copiar al portapapeles.

### Diccionario como items
- **D-19:** Convertir `custom_words` (string separado por comas) en una **lista de items editable** (agregar con input, editar inline, borrar). UI tipo chips/filas.
- **D-20:** **Compatibilidad backend**: mantener el contrato con Whisper (initial_prompt). Almacenar como `Vec<String>` en settings con **migración** desde el string por comas; el backend sigue recibiendo el texto unido. Considerar agregar el campo nuevo con `#[serde(default)]` y derivarlo del viejo si está vacío.
- **D-21:** **Solo palabras** en Fase 1 (no reemplazos `a→b`). Los reemplazos/correcciones dirigidas pertenecen a **Phase 3 (auto-learn)**.

### Claude's Discretion
El usuario delegó toda la fase. Donde no se fijó un default explícito arriba (microcopy exacto en español, iconografía del sidebar, animaciones/transiciones, orden fino de secciones, elección final de assets de sonido), **Claude decide** siguiendo la paleta y el tono actuales. El researcher/planner resuelve las vías técnicas marcadas como "confirmar" (pause-media, updater).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No hay specs/ADRs externos formales en el repo. Las referencias canónicas son el código y los docs de planeación:

### Código existente (anclas de la fase)
- `src/routes/+page.svelte` — UI actual (header+tabs, onboarding, secciones de settings, historial) y el design system (paleta, toggles, rows, badges) a conservar/extraer.
- `src/routes/widget/+page.svelte` — widget del notch (NO se toca en esta fase, pero comparte eventos).
- `src-tauri/src/settings.rs` — `AppSettings` (struct a extender) + cache + load/save.
- `src-tauri/src/commands.rs` — state machine de grabación, `get_models`/`download_model` (modelos + URLs), `open_*_settings`, `test_paste`, `get_history`/`delete_history_entry`/`get_recording_audio`, `ModelInfo`.
- `src-tauri/src/history.rs` — store de historial (a extender con `source`).
- `src-tauri/src/transcription.rs` — `resample()`, `correct_words()` (reusar para archivos).
- `src-tauri/src/backend.rs` — trait `TranscriptionBackend` + `TranscribeOpts` (usar para transcripción por archivos).
- `src-tauri/src/mic_permission.rs` — checks de permisos (reusar en panel de Ajustes).
- `src-tauri/tauri.conf.json` — tamaño/flags de ventana `main` (a actualizar) + capabilities/permissions de plugins.
- `src-tauri/src/lib.rs` — registro de comandos invoke_handler (agregar comandos nuevos).

### Contexto del proyecto
- `.planning/PROJECT.md` — visión, core value, constraints (privacidad/local, macOS, no cambiar motor por defecto).
- Memoria del proyecto (sesiones previas): GOTCHAS de macOS (reinstalar resetea permisos; cambiar tamaño de ventana requiere relanzar; atajos globales re-entrantes cuelgan la app — relevante para el hotkey recorder).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Design system inline** en `+page.svelte` (`.rows`, `.row`, `.toggle/.knob`, `.sel`, `.ipt`, `.badge`, `.section-label`, `.banner`, paleta `:root`): extraer a componentes/estilos compartidos para el sidebar + páginas.
- **Filas de permisos** del onboarding (mic/a11y con dot+estado+acción): reutilizar en el panel de permisos de Ajustes.
- **Flujo de modelos** (`get_models`, `download_model`, eventos `download-progress`/`download-complete`, barras de progreso): reusar y ampliar para más modelos Whisper.
- **Historial** (`get_history`, `delete_history_entry`, `get_recording_audio`, reproducción base64, `fmtTime`/`fmtDur`): base para la vista unificada de Transcripciones.
- **`TranscriptionBackend` + `resample()` + `correct_words()`**: reusar tal cual para la transcripción por archivos.
- **`schedSave()` debounce + `save_settings`**: patrón de persistencia para todos los toggles nuevos.

### Established Patterns
- Estado con **Svelte 5 runes** (`$state`), `invoke`/`listen` de Tauri, persistencia debounced.
- Settings = un struct `AppSettings` serializado a `settings.json` en `app_data_dir`, con cache en memoria (`CACHE`) y `#[serde(default)]` para evolución no rompedora.
- Comandos Rust registrados en `lib.rs` invoke_handler; eventos emitidos desde `commands.rs`.
- macOS-specific vía `objc2` (ya usado en `notch.rs`/`escape.rs`) — patrón para sonidos (NSSound) y media key.

### Integration Points
- **Onboarding → shell**: `finishOnboarding()` cambia a la primera pantalla; ahora debe llevar al Home del sidebar.
- **Eventos del state machine** (`commands.rs`): puntos donde enganchar sonidos y pause-media (start listening / stop+transcribe / cancel).
- **`AppSettings`**: punto único para todos los flags nuevos (sonidos×3, pause_media, dictionary items, autostart ya existe).
- **`tauri.conf.json`**: tamaño de ventana + permisos de plugins nuevos (dialog para file upload, autostart, updater).
- **`lib.rs`**: registro de comandos nuevos (data mgmt, file transcription, check-updates).
</code_context>

<specifics>
## Specific Ideas

- **Referencia explícita: UI de WhisprFlow** — emular su estructura de panel lateral (sidebar con secciones), densidad y limpieza. NO copiar su paleta (conservar la beige+coral de Voz Local).
- El usuario quiere "mucho más completo" y "tipo home con settings": el Home no es solo settings, es un dashboard de dictado con las transcripciones a mano.
</specifics>

<deferred>
## Deferred Ideas

- **Parakeet como motor ASR** → **Phase 2**. En Fase 1 solo aparece como tarjeta "Próximamente" en el selector.
- **Auto-learn from corrections** → **Phase 3**. En Fase 1 se sientan bases (diccionario como items).
- **Reemplazos dirigidos en el diccionario (`a→b`)** → ligado a Phase 3 (auto-learn).
- **Dark mode / rediseño cromático** → fuera de scope; se conserva la paleta actual.
- **Copiar transcripción al portapapeles** desde la vista — nice-to-have, el planner decide si entra.
</deferred>

---

*Phase: 01-redise-o-del-panel-de-configuraci-n-estilo-whisprflow*
*Context gathered: 2026-06-14*
