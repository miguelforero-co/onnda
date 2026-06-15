# Design System Research — Iridescent / Metallic over Warm Paper

**Phase:** 01 — Rediseño del panel de configuración (estilo WhisprFlow)
**Goal:** Translate the visual language of the notch's WebGL thin-film wave into a cohesive, restrained CSS design system for the SvelteKit settings/main window — plain CSS custom properties, no Tailwind, no component lib.

---

## 0. Source aesthetic — what the notch actually does (extracted from code)

Read from `src/routes/widget/+page.svelte`. The translation must be faithful to these real values, not a generic "rainbow."

**The thin-film LUT (256×1 texture) is built from 5 hex stops per mode:**

| Mode | Palette stops (the real LUT ramp) | Feel |
|---|---|---|
| `speaking` (`SPK_PAL`) | `#ff6524` → `#e29746` → `#ffffff` → `#bfe9ff` → `#3171c4` | warm coral/amber → **white specular core** → icy blue → deep blue |
| `processing` (`PRC_PAL`) | `#000000` → `#ff6f0f` → `#ffb3b3` → `#bfe9ff` → `#9ec8ff` | black → orange → pink → icy blue (calmer) |

**How the iridescence is produced (fragment shader, `idx` = LUT lookup coordinate 0..1):**
```
idx = idxBase
    + 0.5 * n * disp          // n = signed distance across wave THICKNESS → thin-film dispersion
    + pos * (xm - 0.5)        // horizontal position shift
    + flow * (fbm - 0.5)      // fbm flow-noise perturbation (oil-slick turbulence)
    + height * (cy - center)  // wave height shifts the hue
    + drift * sin(time*0.25)  // slow global hue DRIFT over time
```
Key takeaways for the CSS translation:
- The hue is **not** angle-on-a-color-wheel; it is a **fixed warm→white→cool ramp** sampled by thickness/position/flow. That is exactly an **oil-slick / thin-film** signature: coral and amber on one edge, an icy-blue on the other, a **bright specular white where they meet**.
- `spec` adds `vec3(1.0) * pow(core,3.0)` → a **hot white specular highlight** along the wave crest. This is the "metallic" cue we steal for sheen sweeps and chrome edges.
- `drift = sin(time*0.25)` is a **very slow** hue drift. Our CSS motion must be equally slow (8–20s loops), never fast/glittery.
- `pGamma` + `bright` + `pSat` shape contrast/saturation — the petroleum hues are **muted**, not neon. Match that: low-chroma iridescence.

**Current warm productivity palette** (`src/lib/styles/tokens.css`, locked per CONTEXT D-04):
`--bg #F4F0EB` · `--panel #FDFCFA` · `--text #1C1917` · `--muted #78716C` · `--faint #A8A29E` · `--line rgba(0,0,0,.07)` · `--coral #E85535` · `--amber #F4AA6A` · `--blue #7B9BD2` · `--r 9px`. Font: SF / `-apple-system`, 13px base.

The design system **evolves** this: warm paper stays the substrate and all body text stays solid; iridescence/metal is an **accent layer** on focal surfaces only.

---

## 1. Design principle

**Iridescence and metal are a thin accent layer painted only on focal, brand, and active surfaces — never on the reading surface.** The warm paper base (`--bg #F4F0EB`, `--panel #FDFCFA`, `--text #1C1917`) carries all legibility and is unchanged; body and UI text remain solid tokens meeting WCAG AA (≥4.5:1). On top of that calm canvas we apply the notch's thin-film signature — a muted warm→white→cool oil-slick ramp with a hot specular core — to exactly the elements that deserve a "premium moment": the primary **Dictar** hero/button (metallic body + a slow specular sheen sweep), the **active** nav item (iridescent hairline ring or rail), the **wordmark** (clamped iridescent display text at ≥24px where 3:1 applies), and a barely-perceptible aurora wash behind the shell. Following the cross-app rule distilled from Linear, Vercel/Geist, Raycast, Arc and Family: **one expressive gradient per screen, the rest neutral and high-contrast**, every effect on a pointer-transparent layer, every animation slow (8–20s), GPU-cheap (`transform`/`opacity`/`background-position`), and gated behind `prefers-reduced-motion`, `prefers-reduced-transparency`, and `prefers-contrast`. The result reads as "iridescent/metallic foil pressed into warm paper," not a glittery skin.

---

## 2. Evolved token set — add to `tokens.css`

Keep the existing `:root` block intact. Append the following. All iridescent ramps are **derived from the notch LUT hues** and kept low-chroma to match the shader's muted petroleum look.

```css
:root {
  /* ── existing warm base (UNCHANGED) ──
     --bg #F4F0EB · --panel #FDFCFA · --text #1C1917 · --muted #78716C
     --faint #A8A29E · --line rgba(0,0,0,.07) · --coral #E85535
     --amber #F4AA6A · --blue #7B9BD2 · --r 9px                          */

  /* ── 2.1 Thin-film accent ramp (derived from the notch SPK_PAL LUT) ──
     warm coral/amber → white specular → icy blue → deep blue.
     These are the canonical iridescent stops for the whole app.        */
  --film-1: #FF6524;   /* coral edge   (LUT stop 0) */
  --film-2: #E29746;   /* amber        (LUT stop 1) */
  --film-3: #FFFFFF;   /* specular core(LUT stop 2) */
  --film-4: #BFE9FF;   /* icy blue     (LUT stop 3) */
  --film-5: #3171C4;   /* deep blue    (LUT stop 4) */

  /* Toned-down, app-friendly variants (lower chroma so they sit on paper,
     matching the shader's pSat/bright/gamma muting). Use these on UI. */
  --film-warm: #F0894A;   /* coral+amber blended, softened   */
  --film-cool: #8FC4E8;   /* icy/deep blue blended, softened */

  /* ── 2.2 Iridescent gradients (the reusable recipes) ──
     OKLCH-aware where it matters; sRGB hex fallbacks inline.           */
  --iris-sheen:  linear-gradient(105deg,
                   var(--film-warm) 0%, var(--film-2) 22%,
                   #FFFFFF 48%, var(--film-4) 72%, var(--film-cool) 100%);
  --iris-ring:   conic-gradient(from var(--iris-angle, 120deg) in oklch shorter hue,
                   var(--film-warm), var(--film-4), var(--film-cool),
                   var(--film-2), var(--film-warm));
  /* Aurora wash for the shell background — extremely low opacity blobs */
  --aurora-warm: radial-gradient(60% 80% at 18% 0%,
                   rgba(255,101,36,0.10), transparent 70%);
  --aurora-cool: radial-gradient(55% 75% at 92% 12%,
                   rgba(49,113,196,0.08), transparent 72%);

  /* ── 2.3 Metallic surface tokens (chrome/embossed body + warm metal) ──
     Hard mid-band reversal = the "metal snap"; warm-tinted for paper UI. */
  --metal-warm: linear-gradient(180deg,
                   #FFF7EF 0%, #F4D9C2 26%, #C98A5E 50%,  /* hard horizon */
                   #F4D9C2 74%, #FFF7EF 100%);
  --metal-chrome: linear-gradient(180deg,
                   #FFFFFF 0%, #D9DCE1 25%, #9AA0A8 45%,
                   #4A4F57 50%, #9AA0A8 55%, #E8EBEF 80%, #FFFFFF 100%);
  --emboss-hi:  inset 0 1px 0 rgba(255,255,255,0.75);
  --emboss-lo:  inset 0 -1px 2px rgba(0,0,0,0.14);

  /* ── 2.4 Glass tokens (warm-tinted; light-mode safe) ──
     Film opacity raised to ~55% because pure transparency dies on a
     light bg. saturate() keeps colours from going gray.                */
  --glass-film:   rgba(255, 250, 245, 0.55);
  --glass-blur:   saturate(180%) blur(16px);
  --glass-border: 1px solid rgba(255, 255, 255, 0.6);
  --glass-edge:   inset 0 1px 0 rgba(255,255,255,0.7);
  --glass-shadow: 0 8px 24px rgba(60, 50, 40, 0.12);

  /* ── 2.5 Sheen / specular tokens (the moving highlight band) ──       */
  --sheen-band: linear-gradient(120deg,
                   rgba(255,255,255,0) 30%,
                   rgba(255,255,255,0.55) 50%,
                   rgba(255,255,255,0) 70%);

  /* ── 2.6 Subtle film-grain (SVG feTurbulence, GPU-cheap, no asset) ── */
  --grain: url("data:image/svg+xml,%3Csvg viewBox='0 0 250 250' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");

  /* ── 2.7 Motion tokens (slow, matching the shader's sin(time*0.25)) ── */
  --drift-slow: 16s;   /* hue / aurora drift  */
  --sheen-dur:  2.2s;  /* one sheen sweep     */
}

/* Registered custom property so the conic ring angle is actually animatable.
   Baseline across Chrome/Safari/Edge; Firefox 128+. */
@property --iris-angle {
  syntax: "<angle>";
  inherits: false;
  initial-value: 120deg;
}

/* ── Global accessibility fallbacks: flatten everything to solid/static ── */
@media (prefers-contrast: more) {
  :root { --glass-film: var(--panel); }
}
@media (prefers-reduced-transparency: reduce) {
  :root { --glass-film: var(--panel); --glass-blur: none; }
}
```

**Contrast guarantees (verified against the warm base):**
- Body/UI text uses `--text #1C1917` on `--panel`/`--bg` → far exceeds 4.5:1. Never change this.
- Iridescent **text** is allowed ONLY at display size (≥24px or ≥18.7px bold) where the WCAG bar is 3:1 — and the ramp is clamped so its lightest stop (`--film-4 #BFE9FF`) is never used as the sole fill over light. See §3 wordmark.

---

## 3. Component treatments — copy-pasteable CSS

Every animated treatment is **off by default** and opted into via `(prefers-reduced-motion: no-preference)`, per Josh Comeau's inverted-guard pattern. Decorative layers set `pointer-events: none` (Rauno's rule).

### 3.1 Primary "Dictar" hero / button — metallic body + specular sheen sweep

The hero gets the strongest treatment: a warm-metal body (the notch's coral→white→cool reflected as a vertical metal band), an embossed edge, and a slow diagonal specular sheen — the CSS analog of the shader's `pow(core,3.0)*spec` crest highlight.

```css
.dictar {
  position: relative;
  overflow: hidden;                 /* clips the sheen band */
  border: none;
  border-radius: var(--r);
  padding: 12px 20px;
  font: 600 13.5px/1 -apple-system, "SF Pro Text", system-ui, sans-serif;
  color: #fff;
  /* warm-metal body on top of the brand coral so text stays legible */
  background:
    var(--iris-sheen) 0 0 / 220% 100%,   /* faint iridescent wash */
    var(--coral);
  background-blend-mode: soft-light, normal;
  box-shadow:
    var(--emboss-hi),                /* top inset highlight  */
    var(--emboss-lo),                /* bottom inset shadow  */
    0 2px 8px rgba(232,85,53,0.28);  /* warm coral lift      */
  cursor: pointer;
  transition: transform 140ms ease, box-shadow 140ms ease;
}
.dictar:hover  { transform: translateY(-1px);
                 box-shadow: var(--emboss-hi), var(--emboss-lo),
                             0 4px 14px rgba(232,85,53,0.34); }
.dictar:active { transform: translateY(0); }
.dictar:focus-visible {
  outline: 2px solid var(--film-5);
  outline-offset: 2px;
}

/* The specular sheen band — a skewed translucent highlight that sweeps across */
.dictar::after {
  content: "";
  position: absolute; inset: 0; left: -120%;
  background: var(--sheen-band);
  transform: skewX(-18deg);
  pointer-events: none;
}
@media (prefers-reduced-motion: no-preference) {
  .dictar::after { animation: dictar-sheen var(--sheen-dur) ease-in-out infinite; }
  @keyframes dictar-sheen {
    0%, 60% { left: -120%; }     /* long pause, brief sweep — restrained */
    100%    { left: 120%; }
  }
}
```
When `prefers-reduced-motion: reduce`, the button is a clean embossed coral with no sweep — still premium, fully static.

### 3.2 Sidebar + active-nav item — iridescent rail / hairline ring

The sidebar stays neutral (warm panel). The **active** item is the focal accent: an iridescent left rail and a faint film tint. This is the "accent on active surface only" rule.

```css
.sidebar { background: var(--panel); border-right: 1px solid var(--line); }

.nav-item {
  position: relative;
  border-radius: 7px;
  padding: 7px 10px 7px 14px;
  color: var(--muted);
  transition: color 120ms ease, background 120ms ease;
}
.nav-item:hover { color: var(--text); background: rgba(0,0,0,0.035); }

.nav-item.active { color: var(--text); background: rgba(255,255,255,0.6); }
.nav-item.active::before {        /* iridescent rail = the notch ramp, vertical */
  content: "";
  position: absolute; left: 4px; top: 7px; bottom: 7px;
  width: 3px; border-radius: 3px;
  background: linear-gradient(180deg,
    var(--film-warm), var(--film-3) 50%, var(--film-cool));
  pointer-events: none;
}
```

### 3.3 Cards (ModelCard etc.) — paper card, iridescent ring on selected

Cards default to plain warm panel with a hairline. The **selected** card earns a 1px iridescent ring built with `mask-composite` (respects `border-radius`, unlike `border-image`).

```css
.card {
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: var(--r);
  padding: 14px;
  box-shadow: 0 1px 2px rgba(0,0,0,0.04);
  transition: box-shadow 140ms ease, transform 140ms ease;
}
.card:hover { box-shadow: 0 4px 14px rgba(0,0,0,0.07); transform: translateY(-1px); }

/* Selected: iridescent hairline ring (no layout shift — border already 1px) */
.card.selected {
  border-color: transparent;
  background:
    linear-gradient(var(--panel), var(--panel)) padding-box,
    var(--iris-ring) border-box;
}
@media (prefers-reduced-motion: no-preference) {
  .card.selected { animation: ring-drift var(--drift-slow) linear infinite; }
  @keyframes ring-drift { to { --iris-angle: 480deg; } }   /* 120 → 480 = one slow turn */
}
/* Fallback for engines without mask/@property edge cases: solid coral ring */
@supports not (background: paint(something)) {
  .card.selected { border: 1px solid var(--coral); background: var(--panel); }
}
```

### 3.4 Section header / wordmark — clamped iridescent display text

Iridescent text is permitted **only here** (display size, ≥24px, where WCAG's bar is 3:1). The gradient is clamped to the darker half of the film ramp so the lightest icy stop is never the sole fill. `::selection` resets to solid (Rauno's rule) so selected text stays legible.

```css
.wordmark {
  font-size: 26px;            /* ≥24px → WCAG large-text 3:1 applies */
  font-weight: 700;
  letter-spacing: -0.02em;
  /* darker-clamped film ramp: warm → deep blue, white only as a thin highlight */
  background: linear-gradient(100deg,
    var(--film-1), var(--film-2), var(--film-5)) 0 0 / 220% 100%;
  -webkit-background-clip: text;
          background-clip: text;
  color: var(--text);                       /* solid fallback */
  -webkit-text-fill-color: transparent;
}
.wordmark::selection { -webkit-text-fill-color: var(--text); color: var(--text); }
@media (prefers-reduced-motion: no-preference) {
  .wordmark { animation: wordmark-shift var(--drift-slow) linear infinite; }
  @keyframes wordmark-shift { to { background-position: 220% 0; } }
}
@media (prefers-contrast: more) {
  .wordmark { background: none; -webkit-text-fill-color: var(--text); color: var(--text); }
}
/* Section headers (small) stay SOLID — never iridescent at UI size */
.section-header { font-size: 11px; font-weight: 600; letter-spacing: 0.04em;
                  text-transform: uppercase; color: var(--faint); }
```

### 3.5 Toggles — film accent on the "on" state

Off = neutral track. On = the film ramp (warm→cool), reading as "energized," mirroring the wave going active.

```css
.toggle { width: 38px; height: 22px; border-radius: 999px;
          background: rgba(0,0,0,0.12); transition: background 160ms ease; position: relative; }
.toggle[aria-checked="true"] { background: var(--iris-sheen); background-size: 160% 100%; }
.toggle .knob {
  position: absolute; top: 2px; left: 2px; width: 18px; height: 18px;
  border-radius: 50%; background: #fff;
  box-shadow: 0 1px 2px rgba(0,0,0,0.25), var(--emboss-hi);
  transition: transform 160ms cubic-bezier(0.32,1.26,0.5,1);
}
.toggle[aria-checked="true"] .knob { transform: translateX(16px); }
```

### 3.6 Shell background — barely-perceptible aurora + grain

The whole window gets the warm base plus two extremely faint aurora blobs (warm top-left, cool top-right — the notch's two hue poles) and a 4% grain so gradients never band. All on a pointer-transparent layer.

```css
.shell { position: relative; background: var(--bg); }
.shell::before {                 /* aurora wash — the two notch poles, ~8–10% */
  content: ""; position: absolute; inset: 0; z-index: 0;
  background: var(--aurora-warm), var(--aurora-cool);
  pointer-events: none;
}
.shell::after {                  /* film grain */
  content: ""; position: absolute; inset: 0; z-index: 0;
  background-image: var(--grain);
  opacity: 0.04; mix-blend-mode: overlay; pointer-events: none;
}
.shell > * { position: relative; z-index: 1; }   /* content above decoration */

@media (prefers-reduced-motion: no-preference) {
  .shell::before { animation: aurora-drift var(--drift-slow) ease-in-out infinite alternate; }
  @keyframes aurora-drift {     /* transform/opacity only — compositor-cheap */
    to { transform: translate3d(2%, 1%, 0) scale(1.04); opacity: 0.85; }
  }
}
@media (prefers-reduced-transparency: reduce) {
  .shell::before, .shell::after { display: none; }   /* solid warm paper */
}
```

### 3.7 Optional glass — translucent toolbar/overlay only

Use glass **only** for a floating toolbar/popover that sits over content, never as a default panel (it disappears on flat light bg and costs GPU).

```css
.glass-bar {
  background: var(--glass-film);
  -webkit-backdrop-filter: var(--glass-blur);
          backdrop-filter: var(--glass-blur);
  border: var(--glass-border);
  border-radius: 14px;
  box-shadow: var(--glass-edge), var(--glass-shadow);
}
```

---

## 4. Do / Don't

**Do**
- Keep all body/UI text solid (`--text`) at ≥4.5:1. Iridescent text only at ≥24px display (3:1) with `::selection` reset and a `prefers-contrast` solid fallback.
- Budget **one expressive iridescent moment per screen** (the Dictar hero). Everything else is neutral with at most a hairline accent on the active/selected element.
- Keep motion slow (8–20s, `--drift-slow`) and rare (sheen sweeps with a long pause) — matching the shader's `sin(time*0.25)` drift.
- Animate only `transform`, `opacity`, `background-position`, and the registered `--iris-angle`. Gate every animation behind `(prefers-reduced-motion: no-preference)`; ship the static version as default.
- Set `pointer-events: none` on all decorative layers (aurora, grain, sheen, rings).
- Add the 4% SVG grain over any large gradient to prevent banding (also Rauno: prefer radial gradients for blurred fills).
- Provide solid fallbacks for `prefers-contrast: more` and `prefers-reduced-transparency: reduce`.

**Don't**
- Don't put iridescent or gradient fills behind/inside body text, table cells, form labels, or any sustained-reading surface — contrast varies across the glyph and will fail 4.5:1 at its lightest point.
- Don't use `mix-blend-mode: color-dodge` on the light/warm base — it blows out to glare. Prefer `soft-light`/`overlay` and keep opacity ≤55%.
- Don't use glass as the default panel style on this light UI — it vanishes and burns GPU; reserve it for floating overlays over content.
- Don't animate `width`/`height`/`top`/`left`/`box-shadow`/`background-color` (layout/paint each frame). Don't animate `backdrop-filter` blur.
- Don't make the iridescence neon — the notch hues are **muted** (low `pSat`, gamma-shaped). Use `--film-warm`/`--film-cool` on UI, not the raw saturated LUT stops.
- Don't leave `will-change` on permanently; add only if you observe jank, then remove.

**Performance notes**
- `transform`/`opacity` run on the compositor thread → 60fps even under JS load (web.dev). `background-position` is GPU-offloadable. Color-stop animation is paint-bound — avoid; use `@property --iris-angle` only for the small selected-card ring.
- Large `blur()`/`backdrop-filter` are expensive (Rauno); keep blur ≤16px, glass surfaces small, and never animate the blur. Pause looping animations when off-screen.
- The SVG grain is a single inline data-URI (no network asset) at 4% opacity — negligible cost.

---

## 5. Sources (cited, with one-line takeaways)

**Iridescent / holographic / thin-film CSS**
1. [web.dev — Animated Gradient Text (Adam Argyle)](https://web.dev/articles/speedy-css-tip-animated-gradient-text) — Oversize the gradient + `background-clip:text` + animate `background-position` for shimmer; basis of §3.4 wordmark.
2. [MDN — `@property`](https://developer.mozilla.org/en-US/docs/Web/CSS/@property) — Registering a typed `<angle>` is what makes a conic gradient actually animatable (the selected-card ring).
3. [pyxofy — `@property` + conic gradient animation](https://www.pyxofy.com/css-animation-property-and-conic-gradient-animation/) — `from var(--angle) in oklch` produces the oil-slick sweep without rotating the element.
4. [poke-holo by Simon Goellner (live)](https://poke-holo.simey.me/) · [repo](https://github.com/simeydotme/pokemon-cards-css) — Holographic foil = rainbow `repeating-linear-gradient` at 400% + pointer-driven `background-position` + `color-dodge`; we borrow the structure, not the loudness.
5. [Temani Afif — Gradient border with radius (`mask-composite`)](https://dev.to/afif/border-with-gradient-and-radius-387f) · [MDN mask-composite](https://developer.mozilla.org/en-US/docs/Web/CSS/mask-composite) — Two stacked masks `exclude`d leave only the border ring and respect `border-radius`; basis of §3.3.
6. [CodeTV — Animated gradient border, no JS](https://codetv.dev/blog/animated-css-gradient-border) — `padding-box` solid over `border-box` conic = rotating iridescent ring with zero masking.
7. [CSS-Tricks — mix-blend-mode almanac](https://css-tricks.com/almanac/properties/m/mix-blend-mode/) · [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/mix-blend-mode) — On light backgrounds prefer `soft-light`/`overlay`; `color-dodge` glares. Informs the Don't list.
8. [Quackit — shine-on-hover button](https://www.quackit.com/html/templates/buttons/shine_hover_effect_button.cfm) — Skewed translucent gradient pseudo-element swept via `left`/`translateX` inside `overflow:hidden`; basis of §3.1 sheen.

**Glassmorphism + metallic / chrome**
9. [Josh Comeau — Next-level frosted glass](https://www.joshwcomeau.com/css/backdrop-filter/) — Use `mask-image` (not `overflow:hidden`) to trim extended blur; `saturate(140–180%)` kills the gray mud; `pointer-events:none` on the backdrop.
10. [MDN — backdrop-filter](https://developer.mozilla.org/en-US/docs/Web/CSS/backdrop-filter) — `blur()`+`saturate()` over a translucent bg is minimum viable glass; ship the `-webkit-` prefix for Safari.
11. [CSS-Tricks — Gradient Text](https://css-tricks.com/snippets/css/gradient-text/) — Declare `background` BEFORE `background-clip` or the clip resets to `border-box`; always set a solid `color` fallback.
12. [CSS-Tricks / simurai — Brushed Metal with CSS gradients](https://css-tricks.com/brushed-metal-with-css-gradients/) — A hard mid-band reversal ("horizon") is what makes a gradient read as metal vs plastic; basis of `--metal-*`.
13. [CSS-Tricks — Grainy Gradients (Adam Argyle)](https://css-tricks.com/grainy-gradients/) · [freeCodeCamp — SVG noise data-URI](https://www.freecodecamp.org/news/grainy-css-backgrounds-using-svg-filters/) — One inline `feTurbulence` data-URI at low opacity + `mix-blend-mode:overlay` adds grain with no asset; basis of `--grain`.
14. [Apple Newsroom — Liquid Glass (WWDC 2025)](https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/) · [LogRocket — adopting Liquid Glass](https://blog.logrocket.com/ux-design/adopting-liquid-glass-examples-best-practices/) — Emulate with tinted translucency + edge specular highlight + adaptive contrast; true lensing is native-only, so glass stays a garnish.

**Restraint, motion, accessibility**
15. [Rauno Freiberg — Web Interface Guidelines](https://interfaces.rauno.me/) — Decorative gradients/glows must be `pointer-events:none`; gradient text resets on `::selection`; use radial (not scaled rectangular) gradients to avoid banding; pause off-screen loops; big blurs are slow.
16. [Linear — How we redesigned the Linear UI](https://linear.app/now/how-we-redesigned-the-linear-ui) — Neutral-first system: LCH color, collapsed to base/accent/contrast, contrast as a first-class tunable axis.
17. [Vercel — Geist design system](https://vercel.com/geist/introduction) — Spend accent color on meaning/state (ship/preview/develop); keep the canvas monochrome.
18. [Raycast design analysis](https://getdesign.md/raycast/design-md) — One expressive gradient per page maximum, for hero moments; everything else monochrome or sub-perceptible.
19. [Family (Benji Taylor)](https://benji.org/family-values) — Reserve shimmer/iridescence for rare delight moments, not persistent chrome.
20. [web.dev — Animations and performance](https://web.dev/articles/animations-and-performance) — Only `transform`/`opacity` run on the compositor thread; geometry/paint props are expensive. Drives the Performance notes.
21. [Josh Comeau — prefers-reduced-motion](https://www.joshwcomeau.com/react/prefers-reduced-motion/) — Default to static, opt into motion via `(prefers-reduced-motion: no-preference)`; degrades gracefully.
22. [MDN — prefers-reduced-transparency](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-transparency) · [prefers-contrast](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-contrast) — Swap glass for solid under reduced-transparency; flatten gradients to solid under high-contrast.
23. [WebAIM — Contrast and Color Accessibility](https://webaim.org/articles/contrast/) — AA = 4.5:1 (normal) / 3:1 (large ≥24px or ≥18.7px bold); measure gradient text at its weakest point → iridescent text is display/wordmark only.

---

## DESIGN RESEARCH COMPLETE
