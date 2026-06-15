# Dark Design System — Glassy + Iridescent (Voz Local)

**Phase:** 01 — Rediseño del panel de configuración (estilo WhisprFlow), **DARK redirection**
**Goal:** A NEW visual direction for the SvelteKit main window: a dark, glassy, heavily-iridescent identity that reads as premium and bold — explicitly NOT a tuned native-Mac app. Plain CSS custom properties, no Tailwind. This **overrides** the prior "warm paper / no dark mode" decision (CONTEXT D-04) and supersedes `01-DESIGN-SYSTEM-RESEARCH.md` for the main window. The notch widget is unchanged.

**Method note:** Every reference below was downloaded and *viewed* as an image, and the hex values in the token set are **sampled from those pixels** (Python/PIL `getpixel`), not invented. The iridescent ramp also carries the real notch shader hues. Sources + viewed image URLs are in §6.

---

## 1. Direction

Voz Local goes **dark, glassy, and iridescent** — a near-black cool-violet canvas with floating frosted-glass panels, hairline light edges, and a single oil-slick/holographic accent that glows rather than shouts. The base is almost black with a faint violet-blue tint (think Raycast's `#070507` and Linear's `#161617`), text is high-contrast off-white, and surfaces are translucent dark glass lifted off the background by a 1px top highlight and a soft outer shadow. Iridescence — the notch's warm→white→cool thin-film ramp, broadened toward Stripe's amber→pink→violet→aqua spectrum — appears only as **glow, edges, and accents**: a slow aurora mesh behind the shell, a holographic sheen sweeping the Dictar hero, an iridescent hairline ring on the selected card, the active-nav rail, and the wordmark. The restraint that keeps it legible: **iridescence never sits under body text.** All reading text is solid off-white on dark glass at ≥4.5:1; color is spent on one expressive moment per screen (Raycast/Geist rule), everything else is monochrome dark glass with a hairline. The feel is "a holographic foil edge catching light on smoked glass," not a rainbow skin.

---

## 2. Reference teardown (each image viewed, hex sampled)

**R1 — Raycast, "Your shortcut to everything"** (`/tmp/ref-4-raycast.png`)
*The single most on-target reference.* Near-black field (`#070507`) with **deep-crimson aurora streaks** raking diagonally from the upper-right, and a large **frosted-glass command palette** floating center with a faintly glowing warm edge (`#320A0D` fill, `#41090B`–`#280708` border) and light-gray UI text. Steal: the **dark-glass panel on an aurora-streaked black**, a *single* hued aurora (not a full rainbow) bleeding behind a neutral frosted surface, and the bright hairline panel edge that separates glass from void.

**R2 — Linear homepage + redesign hero** (`/tmp/ref-1-linear.jpg`, `/tmp/ref-10-linear-redesign-conv.png`)
Pure near-black vertical gradient (`#070507` top → `#2B2D33` bottom) with crisp white mark; the redesign hero shows a `#161617` surface with a **thin brighter hairline border** (`#1E1E1F`) and faint construction guide-lines. Steal: the **monochrome dark base + 1px elevation hairline** as the default surface treatment, and "contrast as the design" — almost no color, all depth from value steps (`#070507`→`#0E0F12`→`#161617`→`#1E1E1F`).

**R3 — Stripe holographic wash** (`/tmp/ref-12-stripe.jpg`)
A full **oil-slick spectrum**: amber `#FB9006` → orange `#FFA206` → pink `#FF7FA5`/`#FF8FE3` → violet `#B48CFC` → periwinkle `#CDC2FE`, swept as soft brushed light. Steal: the exact **iridescent ramp** — this is the broadened, on-trend version of the notch's coral→white→blue, and the source for `--iris-*` stops. On dark we use it at low opacity as glow/mesh, never as a flat fill behind text.

**R4 — Vercel "What will you ship?" OG** (`/tmp/ref-2-vercel.png`)
White card with a **corner iridescent prism beam** firing from the bottom-right. Sampled along the beam: mint `#B5FAE2` → aqua `#95F5D6` → teal `#67B7BB` → slate-blue `#7995B2` → mauve `#B3707E`. Steal: the **cool half of the iridescent ramp** (aqua→teal→slate) and the idea of a *single directional beam* anchored to one corner — perfect for the Dictar hero's holographic sheen and the cool pole of the shell aurora.

**R5 — Igloo Inc** (`/tmp/ref-6-igloo.jpg`)
Desaturated **cool blue-gray chrome** 3D scene (`#A7ADB9` sky, `#878D99` snow, `#BBC0C6` highlights) — premium, monochrome, no neon. Steal: the **cool-violet tint** for the dark base (don't make the black pure neutral; tint it toward blue-violet), and a restrained **chrome/silver** treatment for the wordmark/metal edges instead of loud color.

**R6 — Resend, "Email for Developers"** (`/tmp/ref-8-resend.png`)
Pure black (`#010101`) with a **dark frosted code panel** (`#050505`) that's separated from the void only by a barely-there top-edge highlight and a soft inner glow. Centered serif-ish display heading in off-white. Steal: the **"glass on true black" minimalism** — when the base is this dark, a 1px white-at-8% top edge + a faint inset glow is *all* you need to make glass read; no heavy borders.

**Synthesis:** Raycast's dark-glass-on-aurora + Linear's value-step depth + Stripe/Vercel's iridescent ramp + Igloo's cool-violet tint + Resend's restraint. Iridescence is the *accent and the light source*; the structure is monochrome dark glass.

---

## 3. Dark token set — COMPLETE replacement `:root` for `tokens.css`

This replaces the light palette wholesale. Drop-in for `src/lib/styles/tokens.css`. The widget (`src/routes/widget/+page.svelte`) does not import this and is unaffected.

```css
/* ── Voz Local — DARK glass + iridescent tokens (single source) ──
   Base sampled from Raycast/Linear/Resend; iridescent ramp from the notch
   thin-film LUT broadened with Stripe/Vercel holographic hues. */
:root {
  color-scheme: dark;

  /* ── 3.1 Backgrounds — near-black with a cool violet-blue tint (Igloo cue).
     Value ladder is the depth system (Linear): each step ~+6–10 L*.        */
  --bg:        #07070B;   /* shell void — near-black, faint violet           */
  --bg-2:      #0B0C12;   /* recessed wells / scroll track                   */
  --elev-1:    #111219;   /* base elevated surface (sidebar, panels)         */
  --elev-2:    #181A23;   /* cards on panels                                 */
  --elev-3:    #20232E;   /* hover / popover / highest surface               */

  /* ── 3.2 Text — high-contrast off-white + muted greys (WCAG on dark glass) */
  --text:      #F3F4F8;   /* primary  — ~16:1 on --elev-1                    */
  --muted:     #A6A9B6;   /* secondary— ~6.5:1 on --elev-1 (AA)              */
  --faint:     #6E7180;   /* tertiary/labels — ~3.4:1, large/uppercase only  */
  --on-accent: #0A0A0F;   /* dark text for the rare light/iridescent fill    */

  /* ── 3.3 Hairlines & strokes (light, low-alpha — the edge that lifts glass) */
  --line:        rgba(255,255,255,0.08);   /* default divider/border         */
  --line-strong: rgba(255,255,255,0.14);   /* focused / hovered edge         */
  --edge-hi:     rgba(255,255,255,0.16);   /* 1px top specular edge on glass */
  --edge-lo:     rgba(0,0,0,0.55);         /* bottom contact shadow line     */

  --r:    11px;   /* corner radius (slightly larger → modern/glassy)         */
  --r-sm: 8px;
  --r-lg: 16px;

  /* ── 3.4 Iridescent ramp — notch thin-film hues, broadened to Stripe/Vercel.
     Canonical oil-slick stops for the whole app. On dark these are LIGHT and
     SATURATED so they GLOW; opacity/blend control intensity, never raw fill. */
  --iris-1: #FF6A3D;   /* warm coral   (notch #FF6524 ↔ Stripe amber)        */
  --iris-2: #FF7FA5;   /* hot pink     (Stripe #FF7FA5)                       */
  --iris-3: #B48CFC;   /* violet       (Stripe #B48CFC)                       */
  --iris-4: #7FC8FF;   /* icy blue     (notch #BFE9FF deepened)              */
  --iris-5: #6FE6CE;   /* aqua/mint    (Vercel #95F5D6/#67B7BB)              */
  --iris-spec: #FFFFFF;/* specular core(notch white crest, spec highlight)   */

  /* The full holographic gradient (warm→pink→violet→blue→aqua, looping).    */
  --iris-ramp: linear-gradient(105deg,
    var(--iris-1) 0%, var(--iris-2) 24%, var(--iris-3) 46%,
    var(--iris-4) 70%, var(--iris-5) 100%);
  /* Conic version for animated rings (needs @property --iris-angle).        */
  --iris-ring: conic-gradient(from var(--iris-angle, 120deg) in oklch shorter hue,
    var(--iris-1), var(--iris-2), var(--iris-3),
    var(--iris-4), var(--iris-5), var(--iris-1));
  /* Specular sheen band (the moving holographic glint — Vercel beam).       */
  --iris-sheen: linear-gradient(115deg,
    transparent 28%,
    rgba(255,127,165,0.45) 42%, rgba(180,140,252,0.55) 50%,
    rgba(127,200,255,0.45) 58%,
    transparent 72%);

  /* ── 3.5 Aurora mesh — the two-ish hued blobs behind the shell.
     Low alpha so the black stays black; warm pole top-left, cool pole right. */
  --aurora-warm:   radial-gradient(60% 70% at 12% -8%,
                     rgba(255,106,61,0.20), transparent 60%);
  --aurora-violet: radial-gradient(55% 65% at 88% 6%,
                     rgba(180,140,252,0.20), transparent 62%);
  --aurora-aqua:   radial-gradient(70% 80% at 70% 110%,
                     rgba(111,230,206,0.14), transparent 60%);

  /* ── 3.6 Glass tokens — dark translucent fill + blur/saturate + edge + glow.
     Fills are LOW alpha because the backdrop is dark; saturate boosts the
     aurora bleeding through. Resend/Raycast treatment.                      */
  --glass-fill:    rgba(22, 24, 33, 0.55);   /* dark glass body (over aurora) */
  --glass-fill-hi: rgba(30, 33, 45, 0.66);   /* hover / raised glass          */
  --glass-blur:    saturate(150%) blur(20px);
  --glass-border:  1px solid var(--line);
  --glass-edge:    inset 0 1px 0 var(--edge-hi);            /* top specular   */
  --glass-glow:    inset 0 0 24px rgba(127,200,255,0.05);  /* faint inner cool*/
  --glass-shadow:  0 16px 40px -12px rgba(0,0,0,0.7),      /* depth           */
                   0 2px 8px rgba(0,0,0,0.45);

  /* ── 3.7 Accent / brand — coral stays the brand, retuned to glow on dark. */
  --accent:        #FF6A3D;   /* brand coral (notch warm pole)               */
  --accent-soft:   rgba(255,106,61,0.16);   /* tint fill for active states   */
  --accent-glow:   0 0 0 1px rgba(255,106,61,0.5),
                   0 6px 22px -4px rgba(255,106,61,0.45);

  /* ── 3.8 Chrome / metal edge (Igloo cue) — for wordmark / premium edges.  */
  --chrome: linear-gradient(180deg,
    #FFFFFF 0%, #C9CCD6 24%, #8A8E9C 49%,
    #5A5E6B 50%, #9DA1AE 74%, #EDEFF4 100%);   /* hard mid-band = metal snap  */

  /* ── 3.9 Shadows / elevation (cast shadows on dark are deep + soft).      */
  --sh-1: 0 1px 2px rgba(0,0,0,0.5);
  --sh-2: 0 8px 24px -8px rgba(0,0,0,0.6);
  --sh-3: 0 24px 60px -16px rgba(0,0,0,0.72);

  /* ── 3.10 Sheen (neutral white glint for non-iridescent surfaces).        */
  --sheen-band: linear-gradient(120deg,
    transparent 35%, rgba(255,255,255,0.14) 50%, transparent 65%);

  /* ── 3.11 Grain — SVG feTurbulence, kills banding on dark gradients.      */
  --grain: url("data:image/svg+xml,%3Csvg viewBox='0 0 250 250' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");

  /* ── 3.12 Motion — slow, matching the shader's sin(time*0.25).            */
  --drift-slow: 22s;   /* aurora / hue drift          */
  --ring-dur:   18s;   /* iridescent ring rotation    */
  --sheen-dur:  3.6s;  /* one holographic sheen sweep */
  --ease-soft:  cubic-bezier(0.32, 0.72, 0, 1);
}

/* Animatable angle for the conic iridescent ring. */
@property --iris-angle {
  syntax: "<angle>";
  inherits: false;
  initial-value: 120deg;
}

/* ── Accessibility fallbacks: flatten glass & kill color animation ── */
@media (prefers-contrast: more) {
  :root {
    --glass-fill: var(--elev-2); --glass-fill-hi: var(--elev-3);
    --line: rgba(255,255,255,0.22); --muted: #C2C5D0;
  }
}
@media (prefers-reduced-transparency: reduce) {
  :root { --glass-fill: var(--elev-2); --glass-fill-hi: var(--elev-3); --glass-blur: none; }
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: -apple-system, "SF Pro Text", system-ui, sans-serif;
  background: var(--bg);
  color: var(--text);
  font-size: 13px;
  -webkit-font-smoothing: antialiased;
  overflow: hidden;
  height: 100vh;
}
```

**Contrast guarantees (text on the darkest realistic glass, ~`#16181F`):**
- `--text #F3F4F8` → ~16:1 (AAA).
- `--muted #A6A9B6` → ~6.6:1 (AA for body).
- `--faint #6E7180` → ~3.4:1 → use ONLY for ≥16px uppercase labels / non-essential meta (AA large), never body.
- Iridescent fills carry **no body text** (see §4). Iridescent wordmark is display-size only (3:1 bar) with a solid fallback.

---

## 4. Component treatments (copy-pasteable CSS)

All motion is **off by default** and opted in via `(prefers-reduced-motion: no-preference)`. Decorative layers are `pointer-events: none`.

### 4.1 Shell — dark aurora mesh + grain (replaces `+page.svelte` `.shell`)

```css
.shell { position: relative; display: flex; height: 100vh; background: var(--bg); }

/* Aurora mesh: warm top-left, violet top-right, aqua bottom — low alpha so
   the void stays near-black. Raycast/Stripe cue. */
.shell::before {
  content: ""; position: absolute; inset: 0; z-index: 0; pointer-events: none;
  background: var(--aurora-warm), var(--aurora-violet), var(--aurora-aqua);
  filter: blur(8px);            /* melt the blob seams */
}
/* Grain over the gradient so it never bands on dark. */
.shell::after {
  content: ""; position: absolute; inset: 0; z-index: 0; pointer-events: none;
  background-image: var(--grain);
  opacity: 0.05; mix-blend-mode: overlay;
}
.shell > :global(*) { position: relative; z-index: 1; }

@media (prefers-reduced-motion: no-preference) {
  .shell::before { animation: aurora-drift var(--drift-slow) var(--ease-soft) infinite alternate; }
  @keyframes aurora-drift {
    to { transform: translate3d(2.5%, 1.5%, 0) scale(1.06); opacity: 0.85; }
  }
}
@media (prefers-reduced-transparency: reduce) { .shell::before, .shell::after { display: none; } }

.content { flex: 1; overflow-y: auto; overflow-x: hidden; background: transparent; padding: 32px; }
```

### 4.2 Sidebar — dark glass rail + iridescent active item

```css
.sidebar {
  width: 200px; flex-shrink: 0;
  display: flex; flex-direction: column; gap: 18px; padding: 16px 12px;
  background: var(--glass-fill);
  -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
  border-right: 1px solid var(--line);
  box-shadow: var(--glass-edge);
}

.nav-item {
  position: relative; display: flex; align-items: center; gap: 10px;
  height: 34px; padding: 0 10px 0 14px; width: 100%;
  background: none; border: none; border-radius: var(--r-sm);
  color: var(--faint); cursor: pointer; text-align: left;
  transition: color .14s, background .14s;
}
.nav-item:hover { color: var(--muted); background: rgba(255,255,255,0.04); }

/* Active = off-white text on a faint glass pill + iridescent left rail (the
   notch ramp gone vertical) with a soft glow. The one accent in the rail. */
.nav-item.on { color: var(--text); background: rgba(255,255,255,0.06); }
.nav-item.on::before {
  content: ""; position: absolute; left: 4px; top: 7px; bottom: 7px;
  width: 3px; border-radius: 3px; pointer-events: none;
  background: linear-gradient(180deg, var(--iris-1), var(--iris-3) 50%, var(--iris-5));
  box-shadow: 0 0 10px -1px rgba(180,140,252,0.6);
}
@media (prefers-contrast: more) { .nav-item.on::before { background: var(--accent); box-shadow: none; } }

.label { font-size: 13px; font-weight: 450; }
.icon svg { width: 16px; height: 16px; }
```

### 4.3 Dictar hero — glassy iridescent button + animated holographic sheen (the boldest moment)

Replaces `.btn-primary.dictar` in `Home.svelte`. Dark glass body, an iridescent edge ring, a slow holographic sheen sweep (the Vercel beam in motion), and a warm inner glow so it reads as the focal action.

```css
.hero {
  position: relative;
  background: var(--glass-fill);
  -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
  border: 1px solid var(--line); border-radius: var(--r-lg);
  box-shadow: var(--glass-edge), var(--glass-glow), var(--glass-shadow);
  padding: 28px; display: flex; flex-direction: column; align-items: center; gap: 12px;
}

.btn-primary.dictar {
  position: relative; overflow: hidden;
  display: inline-flex; align-items: center; gap: 8px;
  padding: 13px 26px; border: none; border-radius: var(--r);
  font: 600 13.5px/1 -apple-system, "SF Pro Text", system-ui, sans-serif;
  letter-spacing: -.01em; color: var(--text); cursor: default;
  /* dark glass core, iridescent ring drawn as a gradient border via the
     padding-box/border-box trick (respects radius, no layout shift). */
  background:
    linear-gradient(var(--elev-2), var(--elev-2)) padding-box,
    var(--iris-ramp) border-box;
  border: 1.5px solid transparent;
  box-shadow:
    inset 0 1px 0 var(--edge-hi),
    0 0 22px -6px rgba(180,140,252,0.45),
    var(--accent-glow);
  transition: transform .14s var(--ease-soft), box-shadow .14s;
}
.btn-primary.dictar svg, .dictar-label { position: relative; z-index: 2; }
.btn-primary.dictar:disabled { opacity: 1; }   /* stays premium when informational */
.btn-primary.dictar:hover:not(:disabled) { transform: translateY(-1px); }
.btn-primary.dictar:focus-visible { outline: 2px solid var(--iris-4); outline-offset: 3px; }

/* Holographic sheen sweep — Vercel's prism beam, skewed, swept across. */
.btn-primary.dictar::after {
  content: ""; position: absolute; inset: 0; left: -130%; z-index: 1;
  background: var(--iris-sheen); transform: skewX(-16deg);
  pointer-events: none; mix-blend-mode: screen;   /* glows ON dark, no glare */
}
@media (prefers-reduced-motion: no-preference) {
  .btn-primary.dictar::after { animation: dictar-sheen var(--sheen-dur) var(--ease-soft) infinite; }
  @keyframes dictar-sheen { 0%,55% { left: -130%; } 100% { left: 130%; } }
}
@media (prefers-contrast: more) {
  .btn-primary.dictar { background: var(--elev-2); border-color: var(--accent); box-shadow: none; }
  .btn-primary.dictar::after { display: none; }
}
```

### 4.4 Cards / ModelCard — dark glass panel, iridescent edge on hover/selected

```css
.card, .stat-card, .quick-card {
  position: relative;
  background: var(--glass-fill);
  -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
  border: 1px solid var(--line); border-radius: var(--r);
  box-shadow: var(--glass-edge), var(--sh-2);
  transition: transform .16s var(--ease-soft), box-shadow .16s, border-color .16s, background .16s;
}
.card:hover, .quick-card:hover {
  transform: translateY(-1px);
  background: var(--glass-fill-hi);
  border-color: var(--line-strong);
  box-shadow: var(--glass-edge), var(--sh-3);
}

/* Selected: iridescent hairline ring (gradient border, respects radius) +
   a faint matching glow. No layout shift — border stays 1px-equivalent. */
.card.selected {
  border-color: transparent;
  background:
    linear-gradient(var(--elev-2), var(--elev-2)) padding-box,
    var(--iris-ring) border-box;
  box-shadow: var(--glass-edge), 0 0 24px -8px rgba(180,140,252,0.4), var(--sh-2);
}
@media (prefers-reduced-motion: no-preference) {
  .card.selected { animation: ring-drift var(--ring-dur) linear infinite; }
  @keyframes ring-drift { to { --iris-angle: 480deg; } }
}
@media (prefers-contrast: more) {
  .card.selected { background: var(--elev-2); border: 1.5px solid var(--accent); box-shadow: none; }
}
```

### 4.5 Section headers / wordmark — chrome/iridescent display text (dark-safe)

Display-size text only (≥18px). Wordmark = brushed **chrome** (Igloo cue) by default; iridescent variant available. Small section labels stay solid.

```css
.wordmark {
  font-size: 20px; font-weight: 700; letter-spacing: -.02em;
  background: var(--chrome) 0 0 / 100% 200%;
  -webkit-background-clip: text; background-clip: text;
  color: var(--text);                       /* solid fallback */
  -webkit-text-fill-color: transparent;
}
/* Iridescent alternative (use one or the other): */
.wordmark--iris {
  background: var(--iris-ramp) 0 0 / 220% 100%;
  -webkit-background-clip: text; background-clip: text;
  -webkit-text-fill-color: transparent;
}
.wordmark::selection, .wordmark--iris::selection {
  -webkit-text-fill-color: var(--text); color: var(--text);
}
@media (prefers-reduced-motion: no-preference) {
  .wordmark--iris { animation: wordmark-shift var(--drift-slow) linear infinite; }
  @keyframes wordmark-shift { to { background-position: 220% 0; } }
}
@media (prefers-contrast: more) {
  .wordmark, .wordmark--iris { background: none; -webkit-text-fill-color: var(--text); color: var(--text); }
}

/* Page title + small labels stay SOLID — never gradient at UI size. */
.page-title  { font-size: 16px; font-weight: 600; color: var(--text); letter-spacing: -.01em; }
.section-label {
  font-size: 11px; font-weight: 600; text-transform: uppercase;
  letter-spacing: .07em; color: var(--faint);
}
```

### 4.6 Toggles — iridescent "on" track

```css
.toggle {
  position: relative; width: 38px; height: 22px; border-radius: 999px;
  background: rgba(255,255,255,0.12); border: 1px solid var(--line);
  transition: background .18s, box-shadow .18s; cursor: pointer;
}
.toggle[aria-checked="true"] {
  background: var(--iris-ramp); background-size: 160% 100%; border-color: transparent;
  box-shadow: 0 0 14px -3px rgba(180,140,252,0.6);
}
.toggle .knob {
  position: absolute; top: 2px; left: 2px; width: 18px; height: 18px; border-radius: 50%;
  background: #fff; box-shadow: 0 1px 3px rgba(0,0,0,0.6);
  transition: transform .18s var(--ease-soft);
}
.toggle[aria-checked="true"] .knob { transform: translateX(16px); }
@media (prefers-contrast: more) {
  .toggle[aria-checked="true"] { background: var(--accent); box-shadow: none; }
}
```

### 4.7 Inputs

```css
.input, input[type="text"], textarea, select {
  width: 100%; padding: 9px 12px; color: var(--text);
  background: var(--bg-2); border: 1px solid var(--line); border-radius: var(--r-sm);
  font: 13px/1.4 inherit; transition: border-color .14s, box-shadow .14s, background .14s;
}
.input::placeholder { color: var(--faint); }
.input:focus, input:focus, textarea:focus, select:focus {
  outline: none; background: var(--elev-1); border-color: transparent;
  box-shadow: 0 0 0 1px var(--iris-4), 0 0 0 4px rgba(127,200,255,0.16);   /* iridescent focus ring */
}
.kbd, kbd {
  display: inline-block; padding: 1px 6px; border-radius: 5px;
  background: rgba(255,255,255,0.08); border: 1px solid var(--line);
  font: 11px/1 inherit; color: var(--muted);
}
```

### 4.8 List rows (Transcripciones / history)

```css
.row {
  display: flex; gap: 10px; padding: 11px 12px; border-radius: var(--r-sm);
  border: 1px solid transparent; color: var(--text);
  transition: background .12s, border-color .12s;
}
.row + .row { border-top: 1px solid var(--line); border-radius: 0; }   /* hairline separators */
.row:hover { background: rgba(255,255,255,0.04); border-color: var(--line); border-radius: var(--r-sm); }
.row .meta { color: var(--faint); font-size: 11px; }
.row .body { color: var(--muted); }
```

---

## 5. Do / Don't + performance

**Do**
- Keep the base near-black with a faint violet tint (`--bg #07070B`), and build depth from the value ladder (`--elev-1/2/3`) + 1px light hairline (`--edge-hi`), not from heavy borders.
- Spend iridescence as **glow, edge, and accent only**: aurora mesh, the Dictar sheen, the selected-card ring, the active rail, focus rings, the wordmark. One expressive moment per screen.
- Use `mix-blend-mode: screen` for iridescent highlights on dark (adds light, glows). Keep aurora blob alpha ≤0.20.
- Keep all reading text solid off-white (`--text`/`--muted`) at ≥4.5:1; iridescent/chrome text is display-size only with a solid fallback and `::selection` reset.
- Animate only `transform`, `opacity`, `background-position`, and the registered `--iris-angle`. Gate every animation behind `(prefers-reduced-motion: no-preference)`; ship static as default.
- Keep glass fills low-alpha (`~0.55`) with `saturate(150%)` so the aurora bleeds through and doesn't go muddy-grey. `pointer-events: none` on every decorative layer.
- Provide `prefers-contrast: more` (flatten glass to `--elev-2`, solid accent borders) and `prefers-reduced-transparency: reduce` (drop blur + aurora) fallbacks — already in the tokens.

**Don't**
- Don't put iridescent/gradient fills behind body text, table cells, form labels, or list rows — contrast varies across the glyph and fails at the light stops.
- Don't use `mix-blend-mode: color-dodge` (glare) — use `screen`/`overlay` on dark.
- Don't make glass fills opaque or blur >24px — it kills the aurora and is GPU-expensive; never animate `backdrop-filter`.
- Don't render the iridescence neon-bright everywhere; it's a thin-film *accent*, not a skin. Keep blobs faint and the ramp on edges.
- Don't animate `width`/`height`/`top`/`left`/`box-shadow`/`background-color`/`filter`. Don't leave `will-change` on permanently.

**Performance**
- `transform`/`opacity` run on the compositor thread (60fps under JS load); `background-position` is GPU-offloadable; `@property --iris-angle` paint is cheap because the ring is a tiny border.
- `backdrop-filter` is the main cost: keep glass surfaces bounded (sidebar, hero, cards — not full-bleed), blur ≤20px, never animated. The shell aurora is two stacked radial gradients on one pseudo-element + a 5% grain data-URI (no network asset) → negligible.
- Pause the aurora/ring loops when the window is hidden (Tauri `visibilitychange`) to save battery.

---

## 6. Sources (images viewed + technique references)

**Reference images downloaded and viewed (hex sampled from these):**
- R1 Raycast — https://www.raycast.com/opengraph-image-pwu6ef.png (dark crimson aurora + frosted command glass)
- R2 Linear — https://linear.app/static/og/homepage.jpg + https://webassets.linear.app/images/ornj730p/production/90328a09e6b2e2e15c0d33cb54f7e0c0f53d3997-4112x1888.png (near-black value-step depth + hairline)
- R3 Stripe — https://images.stripeassets.com/fzn2n1nzq965/XtX984S1GJVsVOXFC7kMu/01988281e867728dfb09aa7793a6e3b9/Stripe.jpg (holographic spectrum ramp)
- R4 Vercel — https://assets.vercel.com/image/upload/contentful/image/e5382hct74si/4JmubmYDJnFtstwHbaZPev/0c3576832aae5b1a4d98c8c9f98863c3/Vercel_Home_OG.png (corner prism beam, cool aqua→slate ramp)
- R5 Igloo Inc — https://www.igloo.inc/assets/images/social.jpg (cool-violet chrome, premium monochrome)
- R6 Resend — https://resend.com/static/cover.png (glass on true black, minimal edge)
- Also viewed (not adopted): Family (light) https://family.co/preview.png, Arc (light) https://arc.net/og.png, Framer mosaic https://framerusercontent.com/images/yyBL8MFizGZKUd27rQGHp30fyc.jpg, Rauno (yellow) https://rauno.me/og4.png.

**Notch source (real iridescent hues carried in):** `src/routes/widget/+page.svelte` — `SPK_PAL = #ff6524, #e29746, #ffffff, #bfe9ff, #3171c4`; `spec` white crest; `drift = sin(time*0.25)` (slow).

**Technique references:**
- web.dev — Animated Gradient Text (oversize gradient + `background-clip:text` + animate `background-position`): https://web.dev/articles/speedy-css-tip-animated-gradient-text
- MDN — `@property` (typed `<angle>` makes conic gradients animatable): https://developer.mozilla.org/en-US/docs/Web/CSS/@property
- Temani Afif — gradient border with radius (`padding-box`/`border-box` trick): https://dev.to/afif/border-with-gradient-and-radius-387f
- Josh Comeau — next-level frosted glass (`saturate` to kill grey mud, `pointer-events:none` backdrop): https://www.joshwcomeau.com/css/backdrop-filter/
- MDN — backdrop-filter: https://developer.mozilla.org/en-US/docs/Web/CSS/backdrop-filter
- CSS-Tricks / simurai — brushed metal (hard mid-band reversal = metal): https://css-tricks.com/brushed-metal-with-css-gradients/
- CSS-Tricks — grainy gradients (inline feTurbulence data-URI): https://css-tricks.com/grainy-gradients/
- Aceternity — Aurora background pattern: https://ui.aceternity.com/components/aurora-background
- Rauno Freiberg — Web Interface Guidelines (decorative layers pointer-transparent, pause off-screen loops, big blurs slow): https://interfaces.rauno.me/
- Linear — how we redesigned the UI (value/contrast-first system): https://linear.app/now/how-we-redesigned-the-linear-ui
- Vercel Geist — spend color on meaning, keep canvas monochrome: https://vercel.com/geist/introduction
- web.dev — animations & performance (compositor-only props): https://web.dev/articles/animations-and-performance
- WebAIM — contrast (AA 4.5:1 / 3:1 large): https://webaim.org/articles/contrast/
- MDN — prefers-reduced-transparency / prefers-contrast: https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-transparency

---

## DARK DESIGN RESEARCH COMPLETE
