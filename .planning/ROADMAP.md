# Roadmap: Voz Local

## Overview

Rediseño del panel de configuración de Voz Local hacia un panel lateral tipo "home + settings" inspirado en la UI de WhisprFlow. El trabajo se divide en 3 fases: (1) el rediseño completo del shell de UI + todos los settings/secciones que son UI o pegamento de macOS (la mayor parte, shippable por sí sola), (2) integrar Parakeet como motor ASR seleccionable (vía FluidAudio/ANE — motor nuevo, research pesado), y (3) auto-learn from corrections (feature ML completa). La UI de la Fase 1 ya deja lugar para Parakeet y auto-learn.

## Phases

- [ ] **Phase 1: Rediseño del panel (sidebar Home+Settings, estilo WhisprFlow)** - Shell de UI + todos los settings + archivos + diccionario como items
- [ ] **Phase 2: Parakeet como motor seleccionable** - Backend ASR adicional vía FluidAudio/ANE, integrado al selector
- [ ] **Phase 3: Auto-learn from corrections** - Aprender de las correcciones del usuario para mejorar transcripciones

## Phase Details

### Phase 1: Rediseño del panel (sidebar Home+Settings, estilo WhisprFlow)
**Goal**: La app pasa de header+tabs a un panel lateral navegable (Home + Settings + secciones) inspirado en WhisprFlow, con todos los settings nuevos funcionando (sonidos, pause-media, lenguaje, launch-at-login, permisos, hotkey-recorder, modo del atajo, selector de modelos Whisper ampliado, check-for-updates, gestión de datos), transcripción por archivos con históricos, y el diccionario como lista de items editable.
**Depends on**: Nothing (first phase)
**Requirements**: Shell sidebar + Sonidos + Pause-media + Lenguaje + Launch-at-login + Panel de permisos + Hotkey recorder + Modo atajo + Modelos Whisper ampliados + Check-for-updates + Gestión de datos + Transcripción por archivos + Vista de todas las transcripciones + Diccionario como items
**Success Criteria** (what must be TRUE):
  1. La app abre un panel lateral con navegación entre Home, Transcripciones, Diccionario y Ajustes (look & feel inspirado en WhisprFlow, paleta actual conservada).
  2. El usuario puede activar/desactivar sonidos (escucha/para/cancela) y oírlos al ocurrir cada evento.
  3. El usuario puede activar pause-media y la música se pausa al empezar a escuchar (y reanuda al terminar).
  4. Desde Ajustes: elegir lenguaje (auto/manual), modo del atajo (push-to-talk/mantener), grabar el hotkey con un capturador de teclas, elegir entre ~3+ modelos Whisper con versión, ver/activar launch-at-login, ver estado de permisos (mic+a11y) y abrirlos, check-for-updates, y gestión de datos (abrir carpeta de cache / borrar).
  5. El usuario puede subir un archivo de audio y obtener su transcripción, que queda registrada junto al resto de transcripciones.
  6. La vista de Transcripciones lista todas (dictado + archivo), con reproducción/borrado.
  7. El diccionario permite agregar/editar/borrar palabras como items (no textarea plano), preservando compatibilidad con el initial_prompt de Whisper.
**Plans**: 9 plans en 6 waves

Plans:
- [x] 01-01-PLAN.md — Foundation de datos: AppSettings (sonidos×3, pause_media, dictionary) + migración + HistoryEntry source
- [x] 01-02-PLAN.md — Build/window: ventana 880×640, deps dialog/fs/updater/symphonia/sha2, capabilities, stubs de módulos
- [x] 01-03-PLAN.md — Shell de UI: sidebar 200px + extracción de tokens y componentes (Toggle/Row/PermissionRow/ModelCard/HotkeyRecorder) + stubs de secciones
- [x] 01-04-PLAN.md — Hooks nativos: sounds.rs (NSSound) + media_pause.rs (CGEvent) en el state machine + catálogo de modelos ampliado
- [x] 01-05-PLAN.md — Transcripción por archivos: audio_decode.rs (symphonia) + comando transcribe_file → history source="file"
- [x] 01-06-PLAN.md — Gestión de datos (reveal/clear) + check-for-updates (con fallback check-only)
- [x] 01-07-PLAN.md — Sección Ajustes: hotkey, sonidos, pause-media, lenguaje, launch-at-login, permisos, modelos+Parakeet, updates, datos
- [x] 01-08-PLAN.md — Secciones Home + Transcripciones (filtro+upload) + Diccionario (items)
- [ ] 01-09-PLAN.md — Integración final + checklist de verificación manual (7 criterios)

### Phase 2: Parakeet como motor seleccionable
**Goal**: Parakeet (vía FluidAudio/ANE) queda disponible como motor ASR seleccionable en el selector de modelos, implementando el trait `TranscriptionBackend`, sin romper el pipeline existente de Whisper.
**Depends on**: Phase 1
**Requirements**: Backend Parakeet + integración al selector de modelos de la Fase 1 + descarga/gestión del modelo
**Success Criteria** (what must be TRUE):
  1. El usuario puede seleccionar Parakeet en el selector de modelos y dictar con él.
  2. El cambio de motor no rompe Whisper ni el pipeline de streaming.
**Plans**: TBD

Plans:
- [ ] 02-01: TBD

### Phase 3: Auto-learn from corrections
**Goal**: La app aprende de las correcciones del usuario (ediciones a transcripciones) para mejorar el reconocimiento futuro, alimentando el diccionario/sesgo de reconocimiento.
**Depends on**: Phase 1
**Requirements**: Captura de correcciones + mecanismo de aprendizaje + integración con diccionario
**Success Criteria** (what must be TRUE):
  1. Cuando el usuario corrige una transcripción, la app captura la corrección.
  2. Las correcciones mejoran transcripciones futuras (vía diccionario/initial_prompt o reemplazos).
**Plans**: TBD

Plans:
- [ ] 03-01: TBD

## Progress

**Execution Order:** 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Rediseño del panel | 0/9 | Not started | - |
| 2. Parakeet motor seleccionable | 0/TBD | Not started | - |
| 3. Auto-learn from corrections | 0/TBD | Not started | - |
