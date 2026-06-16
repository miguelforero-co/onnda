# Rediseño de UI "onnda" — sistema atómico + tema light/dark

**Fecha:** 2026-06-16
**Estado:** propuesto (pendiente de revisión del usuario)
**Fuente de diseño:** Figma `vf94mZpQRKSH4G3GVZgN3j`, nodos `43:1623` (light) y `43:1914` (dark) — pantalla Home.

## Objetivo

Reemplazar el lenguaje visual actual (dark-glass iridiscente, marca "Voz Local") por
el sistema flat/minimal del Figma, montado como **atomic design** (tokens → primitivos →
moléculas → organismos) para que un cambio en el sistema se propague a toda la app.
Entregar **Home pixel-perfect** primero como rebanada vertical, validarla, y luego propagar
el sistema a las otras 4 secciones.

**Prioridad #1: pixel-perfect contra el Figma.** Todos los espaciados, tamaños y
distancias en base-8.

## Decisiones tomadas (con el usuario)

1. **Marca → "onnda"** (rebrand total). El nombre vive en una sola variable
   (`src/lib/brand.ts`) para cambiarlo fácil. Wordmark, títulos de ventana y
   `tauri.conf.json` (productName) lo consumen. Bundle id se mantiene estable.
2. **Tema → light / dark / auto**, elegible manualmente en Ajustes (como apps nativas).
   `auto` sigue `prefers-color-scheme`. Persistido entre sesiones.
3. **Arquitectura → atomic design.** Se migra la app a tokens + capa de componentes.
4. **Fuentes → empaquetar Goudy Bookletter 1911** localmente (`@font-face`).
   Helvetica Neue del sistema con fallback.
5. **Iconos → Iconoir** (MIT). SVG fuente del paquete core, inline en un átomo `Icon`
   (control de color vía `currentColor`). No se usa el wrapper `svelte-iconoir` (v0.15,
   riesgo con Svelte 5).

## Fuera de alcance

- El widget del notch (`src/routes/widget/+page.svelte`) — tiene estilo propio y NO
  importa `tokens.css`. No se toca.
- Lógica de negocio (grabación, ASR, paste, modelos). Solo capa de presentación.

## Sistema de diseño (tokens)

`src/lib/styles/tokens.css` se reescribe. Variables **semánticas** que cambian por tema
vía `:root[data-theme="light"]` / `[data-theme="dark"]`.

### Colores

| Token | Light | Dark | Uso |
|---|---|---|---|
| `--bg` | `#d6d8d7` | `#181818` | ventana, sidebar, contenido |
| `--surface` | `#e6e6e6` | `#222222` | cards |
| `--surface-ink` | `#181818` | `#222222` | banner de feedback |
| `--text` | `#2b2b2b` | `#e5e5e5` | títulos, números |
| `--text-muted` | `#979797` | `#c9c9c9` | labels de card |
| `--text-section` | `#020202` | `#b4b4b4` | labels de sección (SUMMARY…) |
| `--text-on-ink` | `#e6e6e6` | `#e6e6e6` | texto sobre banner oscuro |
| `--nav-active-bg` | `#020202` | `#e1e1e1` | pill nav activo |
| `--nav-active-ink` | `#e6e6e6` | `#393939` | texto nav activo |
| `--nav-ink` | `#2e2e2e` | `#f1f1f1` | texto nav inactivo |
| `--wordmark-tag` | `#000000` | `#ffffff` | tagline "voice to text" |

### Espaciado (base-8)

`--s1: 4px · --s2: 8px · --s3: 12px · --s4: 16px · --s6: 24px · --s8: 32px · --s10: 40px`

(Los valores del Figma como `11.351`, `6.811`, `9.081`, `13.621` vienen de un escalado del
frame ≈0.908×; se normalizan a `12 / 8 / 8 / 16`.)

### Radios

`--r-card: 16px · --r-nav: 8px · --r-window: 16px`

### Tipografía

- `--font-serif: "Goudy Bookletter 1911", Georgia, serif` (empaquetada)
- `--font-sans: "Helvetica Neue", -apple-system, system-ui, sans-serif`

| Rol | Familia | Tamaño | Peso | Tracking |
|---|---|---|---|---|
| Wordmark | serif | 36px | regular | — |
| Heading ("Hey Miguel,") | serif | 24px | regular | — |
| Section label | sans | 12px | regular | `+6.48px` (uppercase) |
| Stat number | sans | 24px | bold | `-0.48px` |
| Stat label / nav / banner | sans | 14px | regular | — |
| Tagline | sans | 12px | light | — |

## Capa atómica (`src/lib/components/ui/`)

**Átomos**
- `Icon.svelte` — renderiza un SVG de Iconoir por nombre, `size` (default 24), `currentColor`.
- `Wordmark.svelte` — "onnda" (serif 36) + "voice to text" (sans light 12). Consume `BRAND`.
- `SectionLabel.svelte` — texto uppercase con tracking +6.48px.
- `StatNumber.svelte` — número grande bold con tracking -0.48px.

**Moléculas**
- `Card.svelte` — caja `--surface`, radio 16, padding 16. Slot.
- `StatCard.svelte` — `Card` + `StatNumber` + label muted. (number, label).
- `NavItem.svelte` — icono + label; estado activo (pill). (icon, label, active, onclick).
- `StreakCard.svelte` — `Card` (280px) con título "Streak", contador "N days" y
  **grid de puntos 7×4** que refleja días con actividad (verde lleno = activo,
  contorno = hoy, gris = vacío). Driven por `history`.
- `FeedbackBanner.svelte` — banner `--surface-ink`, texto `--text-on-ink`, link a feedback.

**Organismos**
- `Sidebar.svelte` — refactor del actual: `Wordmark` + lista de `NavItem`. Mantiene el
  `railDrag` (arrastre de ventana) y el padding superior que libera los traffic lights.
- Secciones de Home compuestas con las moléculas.

Cada sección (`Home`, `Transcripciones`, `Importar`, `Diccionario`, `Ajustes`) consume estos
componentes → cambiar un token o un primitivo repinta toda la app.

## Tema (estado + persistencia)

- `src/lib/stores/theme.svelte.ts` — store con runes: `mode: "light"|"dark"|"auto"`,
  `resolved: "light"|"dark"` (derivado, escucha `prefers-color-scheme` cuando `auto`).
- Aplica `data-theme={resolved}` en `document.documentElement`.
- Persistencia: `localStorage` (simple, ya disponible en el webview). Default: `auto`.
- Control en Ajustes: selector segmentado Light / Dark / Auto.

## Home pixel-perfect (layout)

Referencia: nodo `43:1623`. Ventana 1000×726, radio 16, sidebar a la izquierda.

- **Sidebar** 245px: wordmark (left 24, top 51), nav (gap 16 entre items, item px12 py8,
  gap icono-texto 12, radio 8). Item activo = pill `--nav-active-bg`.
- **Contenido** (left 40, top 81, gap 32 entre bloques):
  - `Heading` "Hey {nombre}," (serif 24).
  - Bloque **SUMMARY** (gap 8): label + fila de 2 `StatCard` (gap 16, flex 1) →
    "Total transcriptions" (count), "Files transcribed" (file count).
  - Bloque **ACTIVITY** (gap 8): label + fila (gap 16) con `StreakCard` (280px) a la
    izquierda y, a la derecha, columna (gap 16) de: fila de 2 `StatCard`
    ("Dictated words total", "Words this week") + `StatCard` ancho
    ("Time saved from typing").
  - **FeedbackBanner** anclado abajo (left 40, ancho 689, top ~639).

Datos: reutiliza los `$derived` que ya existen en `Home.svelte` (totalCount, fileCount,
totalWords, weekWords, savedMinutes, streak). El nombre "Miguel" sale del usuario/sistema
(placeholder configurable; por ahora derivado o constante).

## Plan de propagación (post-Home)

Una vez aprobado Home, migrar `Transcripciones`, `Importar`, `Diccionario`, `Ajustes`
a los mismos primitivos y tokens. Sin Figma de referencia para ellas: se reinterpreta con
el mismo lenguaje (mismas cards, labels, espaciados). Ajustes incorpora además el selector
de tema.

## Verificación

- `npm run check` → 0 errores.
- App corre en dev (`npm run tauri dev`), Home se ve idéntico al Figma en ambos temas
  (comparación visual contra los screenshots).
- Cambiar el tema en Ajustes repinta toda la app.
- Cambiar un token (p. ej. `--surface`) se refleja en todas las cards de todas las secciones.

## Riesgos / notas

- **Pixel-perfect en tema dark**: el Figma dark tiene el layout de ACTIVITY incompleto
  (solo 2 cards y posiciones absolutas distintas al light). Se toma el **light como layout
  canónico** y el dark solo aporta la paleta. (Confirmar con el usuario si quiere respetar
  el layout absoluto del dark — se asume que no.)
- Goudy Bookletter 1911: verificar licencia de embedding (es OFL/dominio público vía
  Google Fonts → ok para empaquetar).
- El glifo de Settings (Figma "grid") puede no calzar exacto con Iconoir; swap trivial.
