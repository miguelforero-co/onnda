---
name: design-tweaks
version: 1.3.0
description: |
  Universal visual tweaker. Opens an interactive dashboard-style playground where
  you can adjust any visual parameter in real time — design tokens, iconography,
  animation curves, spacing, sound parameters, color palettes, or any set of
  knobs that belongs to the current project — then export as JSON or a
  natural-language prompt with exact values to apply back to the codebase.
license: MIT
compatibility: claude-code
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Grep
  - Glob
---

# design tweaks — Universal Visual Tweaker

A self-contained visual playground with a consistent dashboard shell: sidebar nav, controls panel, live preview. What goes inside changes based on what's being built. No build step, no dependencies, runs entirely in the browser.

---

## Invocation flow — always follow this order

### Step 0 — Read the conversation

Before doing anything else, read the conversation that led here. The user's intent is almost always already established in what they've been talking about. Ask yourself:

- What are they building?
- What visual aspect are they trying to adjust?
- Are there specific parameters or values already mentioned?

If the subject is clear from the conversation (e.g., they've been discussing animation timing, or tweaking icon weights, or choosing a color palette), proceed directly to Step 2 without asking.

### Step 1 — Ask questions until context is fully established

If the conversation doesn't make the subject clear enough to build the right tweaker, ask. There is no limit on how many questions to ask — ask until you have enough to build exactly the right thing. Each question should narrow the scope:

- What visual parameter are you adjusting?
- What does the live preview need to show?
- Are there specific ranges or constraints on the values?
- What does the exported config get applied to?

Do not ask questions you can answer from the conversation. Do not ask all questions at once — ask the most important one first, then follow up if needed.

### Step 2 — Decide what to build

| User chose | Action |
|---|---|
| Design tokens | Open default explorer → `~/.claude/skills/design-tweaks/dev/explorer.html` |
| Anything else | Generate a custom tweaker → Mode 3, then open it |
| Pasted back a JSON/prompt | Apply to codebase → Mode 2 |

---

## Mode 1 — Open the default design tokens explorer

```bash
open ~/.claude/skills/design-tweaks/dev/explorer.html
```

Tell the user:
> Explorer is open. Adjust any token in the sidebar — color, type, spacing, border, shadow, motion, opacity, blur, or grid. When you're done, hit **Copy JSON** or **Copy prompt** and paste it back here.

Wait. Do not do anything else until they paste the config back.

---

## Mode 2 — Apply exported config to the codebase

Triggered when the user pastes a JSON block or a natural-language config from the explorer.

**Step 1 — Parse.** Extract all key-value pairs exactly as given. Preserve units. Don't rename keys.

**Step 2 — Find where the values live.** Search in this order:

```bash
grep -r "\-\-.*color\|--font\|--radius\|--spacing\|--duration" --include="*.css" --include="*.scss" -l .
grep -r "fontBase\|radiusBase\|colorAccent\|designTokens\|theme\s*=" --include="*.ts" --include="*.js" --include="*.tsx" -l .
find . -name "tailwind.config.*" 2>/dev/null
find . -name "tokens.json" -o -name "*.tokens.json" 2>/dev/null
```

If nothing found, ask the user where to write before touching any file.

**Step 3 — Write values.** Match to the project's existing naming convention. Don't convert formats (CSS vars stay CSS vars, JS objects stay JS objects).

**Step 4 — Confirm.** List every file touched and every token updated.

---

## Mode 3 — Generate a custom tweaker

Used when the subject is anything other than standard design tokens.

### What a custom tweaker is

A single self-contained HTML file, saved to `~/.claude/skills/design-tweaks/dev/<subject>.html`, that uses the same visual shell as `explorer.html` but has categories, controls, and a preview tailored to the subject.

### How to generate it

**Step 1 — Read the shell CSS from the existing explorer:**

```bash
head -200 ~/.claude/skills/design-tweaks/dev/explorer.html
```

Copy the CSS verbatim (everything in `<style>` up to and including the dark mode rules). Do not redesign it. This is the invariant shell.

**Step 2 — Define the state object** for this subject. Structure it as a flat or two-level JS object where each key corresponds to a tweakable parameter.

Examples:

```js
// Icon style
const S = {
  stroke: { weight: 1.5, cap: 'round', join: 'round' },
  corner: { radius: 2 },
  size: { base: 24, optical: true },
  color: { value: '#1a1917' }
}

// Animation / spring
const S = {
  timing: { fast: 120, base: 250, slow: 450 },
  easing: { curve: 'standard' },
  spring: { mass: 1, stiffness: 200, damping: 20 },
  stagger: { delay: 40, maxItems: 8 }
}

// Sound envelope
const S = {
  envelope: { attack: 0.01, decay: 0.1, sustain: 0.7, release: 0.3 },
  tone: { frequency: 440, waveform: 'sine', detune: 0 },
  reverb: { size: 0.3, wet: 0.2 },
  mix: { volume: 0.8, pan: 0 }
}
```

**Step 3 — Define categories** that map to the state keys:

```js
const cats = [
  { id: 'stroke', label: 'Stroke',  desc: 'Weight, cap, and join style' },
  { id: 'corner', label: 'Corner',  desc: 'Radius per corner' },
  { id: 'size',   label: 'Size',    desc: 'Base size and optical sizing' },
  { id: 'color',  label: 'Color',   desc: 'Stroke and fill color' }
]
```

**Step 4 — Define controls** for each category using the same helpers from the shell (`slider`, `colorSwatch`, `pills`). If the shell helpers don't cover a control type you need (e.g. a waveform picker, a grid of icon previews, a WebAudio oscillator), add it inline — but reuse the `.cg`, `.cg-label`, `.cg-val`, `.pills`, `.pill` classes from the shell CSS.

**Step 5 — Define the preview** appropriate to the subject:
- Icons → render a grid of SVG icons that update live as stroke/size/color change
- Animation → render a bouncing/sliding element that plays on click
- Sound → render an ADSR curve SVG + play button using Web Audio API
- Component options → render the actual component with the selected props applied

**Step 6 — Wire export.** The `exportJSON()` function should serialize `S` directly. The `exportPrompt()` function should write a human-readable description of the current state.

**Step 7 — Open the file:**

```bash
open ~/.claude/skills/design-tweaks/dev/<subject>.html
```

Tell the user:
> Tweaker is open for [subject]. Adjust the parameters on the left — the preview updates live. When you're done, hit **Copy JSON** or **Copy prompt** and paste it back here.

---

## Shell invariants — never change these in custom tweakers

These must be identical across all tweakers:

| Property | Value |
|---|---|
| Font | `'Plus Jakarta Sans'` from Google Fonts, weights 400/500/600/700 |
| Body background | `#e9e7e4` |
| Sidebar / topbar / controls | `#ffffff`, border `#e5e3df` |
| Preview area | `#f2f0ed` |
| Label style | Sentence case, 13px, weight 500, color `#1a1917` |
| Muted text | `#78766e` |
| Control borders | `#e5e3df` |
| Active pill | `background: #1a1917; color: #fff` |
| Toggle on-color | `#2563eb` |
| Monospace | `ui-monospace, 'SF Mono', monospace` (values/numbers only) |
| No uppercase labels | Never — not even for section dividers |
| No gradient text | Never |
| No glow/bloom effects | Never |
| Grid layout | `210px sidebar | 300px controls | 1fr preview`, `52px topbar` |

---

## Reference — design token categories (Mode 1 default)

| Category | State key | Controls |
|---|---|---|
| Color | `color` | Swatches: bg, surface, border, text, muted, accent, success, warning, error, info |
| Typography | `type` | Sliders: fontBase, lineHeight, letterSpacing; pills: scaleRatio, fontFamily, fontWeight |
| Spacing | `spacing` | Slider: unit (generates 1×–12× scale) |
| Border | `border` | Slider: radius; pills: width, style |
| Shadow | `shadow` | Pills: level; slider: opacity |
| Motion | `motion` | Sliders: fast, base, slow; pills: easing + bezier viz |
| Opacity | `opacity` | Sliders: disabled, overlay, muted |
| Effects | `effects` | Sliders: blurSm, blurMd, blurLg, backdropBlur |
| Grid | `grid` | Pills: columns; sliders: gutter, margin |

---

## Shadow string reconstruction

```
none → none
xs   → 0 1px 2px rgba(0,0,0,{o})
sm   → 0 1px 3px rgba(0,0,0,{o}), 0 1px 2px -1px rgba(0,0,0,{o})
md   → 0 4px 6px -1px rgba(0,0,0,{o}), 0 2px 4px -2px rgba(0,0,0,{o})
lg   → 0 10px 15px -3px rgba(0,0,0,{o}), 0 4px 6px -4px rgba(0,0,0,{o})
xl   → 0 20px 25px -5px rgba(0,0,0,{o}), 0 8px 10px -6px rgba(0,0,0,{o})
2xl  → 0 25px 50px -12px rgba(0,0,0,{o})
```

---

## Easing presets → cubic-bezier

| Preset | Value |
|---|---|
| linear | 0, 0, 1, 1 |
| standard | 0.2, 0, 0, 1 |
| decelerate | 0, 0, 0, 1 |
| accelerate | 0.3, 0, 1, 1 |
| emphasized | 0.05, 0.7, 0.1, 1 |
| bounce | 0.34, 1.56, 0.64, 1 |
| smooth | 0.4, 0, 0.2, 1 |

---

## Modular type scale formula

```
xs   = fontBase × ratio^-3
sm   = fontBase × ratio^-1
base = fontBase
lg   = fontBase × ratio^1
xl   = fontBase × ratio^2
2xl  = fontBase × ratio^3
3xl  = fontBase × ratio^4
4xl  = fontBase × ratio^5
```

Round to 1 decimal place (e.g. `12.4px`).

---

## Hard rules

- Read the conversation first — it is the primary signal, not the file system.
- Ask as many questions as needed to establish full context. Do not guess. Do not build the wrong tweaker.
- Never open the default explorer for a non-token subject — generate the right tweaker.
- Never open the explorer more than once per invocation unless asked.
- In apply mode, search the project for where values live before writing anything.
- Never convert the project's token format (CSS → JS, etc.) unless explicitly asked.
- If the pasted config has `"mode": "dark"`, apply to dark-mode overrides only.
- Custom tweakers must use the shell CSS verbatim — no redesigning the shell.
- Export always uses exact values (numbers with units, hex codes, specific strings). No qualitative language, no rounding, no omitting non-default values. Every parameter that is tweakable must appear in the export.
