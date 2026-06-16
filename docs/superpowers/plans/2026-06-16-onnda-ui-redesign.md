# onnda UI Redesign — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the dark-glass "Voz Local" UI with the flat/minimal "onnda" design system from Figma, built atomically (tokens → primitives → molecules → organisms), with light/dark/auto theming, and rebuild the Home screen pixel-perfect.

**Architecture:** Semantic CSS custom properties switch by `data-theme` on `<html>`. A small primitives layer (Svelte 5 components) consumes those tokens. Pure logic (theme resolution, streak grid) lives in `.svelte.ts` rune modules. Home composes the molecules. Other sections are migrated in a follow-up plan.

**Tech Stack:** SvelteKit (static adapter) + Svelte 5 runes, Tauri 2, TypeScript, plain CSS custom properties, `@fontsource/goudy-bookletter-1911`, Iconoir SVGs (inlined).

**Verification approach:** This is a presentation-layer redesign in a codebase whose frontend has **no JS test runner** (all unit tests are Rust/cargo). Introducing a JS test framework is out of scope. Verification per task is therefore: (a) `npm run check` → 0 errors (types + Svelte), and (b) visual comparison against the Figma screenshots in `npm run tauri dev`. Pure-logic helpers are written as small exported functions and spot-verified in dev. Reference screenshots: light = Figma node `43:1623`, dark = `43:1914`.

---

## File structure

**New files**
- `src/lib/brand.ts` — brand name constant.
- `src/lib/stores/theme.svelte.ts` — theme mode store + resolution + persistence.
- `src/lib/stores/userName.svelte.ts` — user display name store (onboarding will populate).
- `src/lib/components/ui/Icon.svelte` — Iconoir SVG renderer (atom).
- `src/lib/components/ui/Wordmark.svelte` — brand wordmark + tagline (atom).
- `src/lib/components/ui/SectionLabel.svelte` — tracked uppercase label (atom).
- `src/lib/components/ui/StatNumber.svelte` — big bold number (atom).
- `src/lib/components/ui/Card.svelte` — surface card (molecule).
- `src/lib/components/ui/StatCard.svelte` — number + label card (molecule).
- `src/lib/components/ui/NavItem.svelte` — sidebar nav item (molecule).
- `src/lib/components/ui/StreakCard.svelte` — streak card + 7×4 dot grid (molecule).
- `src/lib/components/ui/FeedbackBanner.svelte` — feedback banner (molecule).
- `src/lib/streak.ts` — pure helper computing the 28-cell activity grid.

**Modified files**
- `package.json` — add `@fontsource/goudy-bookletter-1911`.
- `src/app.html` — FOUC-safe theme bootstrap script + title.
- `src/lib/styles/tokens.css` — full rewrite (semantic light/dark, base-8, typography).
- `src/lib/components/Sidebar.svelte` — refactor to use `Wordmark` + `NavItem`.
- `src/lib/sections/Home.svelte` — rebuild pixel-perfect with molecules.
- `src/lib/sections/Ajustes.svelte` — add theme selector.
- `src/routes/+page.svelte` — import fontsource; shell bg already token-driven.
- `src-tauri/tauri.conf.json` — window title → brand.

---

## Task 1: Bundle the Goudy serif + FOUC-safe theme bootstrap

**Files:**
- Modify: `package.json` (add dependency)
- Modify: `src/routes/+page.svelte` (import font)
- Modify: `src/app.html` (theme bootstrap + title)

- [ ] **Step 1: Install the font package**

Run:
```bash
npm i @fontsource/goudy-bookletter-1911
```
Expected: package added to `dependencies`, woff2 files under `node_modules/@fontsource/...`.

- [ ] **Step 2: Import the font once at the app shell**

In `src/routes/+page.svelte`, add this import directly under the existing `import "$lib/styles/tokens.css";` line:
```ts
  import "@fontsource/goudy-bookletter-1911";
```

- [ ] **Step 3: Add FOUC-safe theme bootstrap + real title to `src/app.html`**

Replace the `<title>` line and add a bootstrap script in `<head>` so the theme is set before first paint (no flash). New `src/app.html`:
```html
<!doctype html>
<html lang="es">
  <head>
    <meta charset="utf-8" />
    <link rel="icon" href="%sveltekit.assets%/favicon.png" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>onnda</title>
    <script>
      // Apply persisted theme before paint to avoid a flash. Mirrors theme.svelte.ts.
      (function () {
        try {
          var m = localStorage.getItem("onnda.theme") || "auto";
          var dark = m === "dark" || (m === "auto" &&
            window.matchMedia("(prefers-color-scheme: dark)").matches);
          document.documentElement.setAttribute("data-theme", dark ? "dark" : "light");
        } catch (e) {
          document.documentElement.setAttribute("data-theme", "light");
        }
      })();
    </script>
    %sveltekit.head%
  </head>
  <style>html,body{background:transparent!important;margin:0;padding:0}</style>
  <body style="background:transparent;margin:0;padding:0" data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
```

- [ ] **Step 4: Verify build still compiles**

Run: `npm run check`
Expected: 0 errors, 0 warnings (font import is side-effect only).

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json src/routes/+page.svelte src/app.html
git commit -m "feat(ui): bundle Goudy serif + FOUC-safe theme bootstrap"
```

---

## Task 2: Rewrite the token system (semantic light/dark, base-8, typography)

**Files:**
- Modify: `src/lib/styles/tokens.css` (full replacement)

- [ ] **Step 1: Replace `src/lib/styles/tokens.css` entirely**

```css
/* ── onnda — design tokens (single source of truth) ──
   Flat/minimal system from Figma vf94mZpQRKSH4G3GVZgN3j (nodes 43:1623 light,
   43:1914 dark). Semantic vars switch by [data-theme] on <html>. The widget
   (src/routes/widget/+page.svelte) does NOT import this. All spacing base-8. */

:root,
:root[data-theme="light"] {
  color-scheme: light;

  /* Surfaces */
  --bg:           #d6d8d7;   /* window, sidebar, content                       */
  --surface:      #e6e6e6;   /* cards                                          */
  --surface-ink:  #181818;   /* feedback banner (dark on light)               */

  /* Text */
  --text:         #2b2b2b;   /* headings, numbers                             */
  --text-muted:   #979797;   /* card labels                                  */
  --text-section: #020202;   /* section labels (SUMMARY/ACTIVITY)            */
  --text-on-ink:  #e6e6e6;   /* text on --surface-ink                        */

  /* Navigation */
  --nav-active-bg:  #020202;
  --nav-active-ink: #e6e6e6;
  --nav-ink:        #2e2e2e;

  /* Wordmark */
  --wordmark-tag: #000000;   /* "voice to text" tagline                       */

  /* Streak dots */
  --dot-on:    #4f7d5f;      /* active day (muted green)                       */
  --dot-off:   #c4c6c5;      /* empty day                                     */
  --dot-today: #2b2b2b;      /* today ring                                    */
}

:root[data-theme="dark"] {
  color-scheme: dark;

  --bg:           #181818;
  --surface:      #222222;
  --surface-ink:  #222222;

  --text:         #e5e5e5;
  --text-muted:   #c9c9c9;
  --text-section: #b4b4b4;
  --text-on-ink:  #e6e6e6;

  --nav-active-bg:  #e1e1e1;
  --nav-active-ink: #393939;
  --nav-ink:        #f1f1f1;

  --wordmark-tag: #ffffff;

  --dot-on:    #6fe6ce;
  --dot-off:   #3a3a3a;
  --dot-today: #e5e5e5;
}

:root {
  /* Spacing — base-8 (4 is the only half-step) */
  --s1: 4px;  --s2: 8px;  --s3: 12px; --s4: 16px;
  --s6: 24px; --s8: 32px; --s10: 40px;

  /* Radii */
  --r-card: 16px; --r-nav: 8px; --r-window: 16px;

  /* Type families */
  --font-serif: "Goudy Bookletter 1911", Georgia, "Times New Roman", serif;
  --font-sans:  "Helvetica Neue", -apple-system, "SF Pro Text", system-ui, sans-serif;

  /* Layout constants (Figma) */
  --sidebar-w: 245px;
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: var(--font-sans);
  background: var(--bg);
  color: var(--text);
  font-size: 14px;
  -webkit-font-smoothing: antialiased;
  overflow: hidden;
  height: 100vh;
}
```

- [ ] **Step 2: Verify it compiles and the app still loads**

Run: `npm run check`
Expected: 0 errors. (Components still referencing old vars like `--glass-fill` will now fall back to unset/inherit — they get fixed in later tasks; the app may look broken mid-plan, which is expected.)

- [ ] **Step 3: Commit**

```bash
git add src/lib/styles/tokens.css
git commit -m "feat(ui): rewrite tokens — flat onnda system, light/dark, base-8"
```

---

## Task 3: Brand module

**Files:**
- Create: `src/lib/brand.ts`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Create `src/lib/brand.ts`**

```ts
// Single source of truth for the product name. Change here to rebrand.
export const BRAND = "onnda";
export const BRAND_TAGLINE = "voice to text";
```

- [ ] **Step 2: Update the Tauri window title to the brand**

In `src-tauri/tauri.conf.json`, find the main window's `"title"` field and set it to `"onnda"`. (If `productName` exists at the top level, set it to `"onnda"` too. Leave `identifier`/bundle id unchanged.)

- [ ] **Step 3: Verify**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/brand.ts src-tauri/tauri.conf.json
git commit -m "feat(ui): brand module (onnda) + window title"
```

---

## Task 4: Theme store

**Files:**
- Create: `src/lib/stores/theme.svelte.ts`

- [ ] **Step 1: Create `src/lib/stores/theme.svelte.ts`**

```ts
// Theme store (Svelte 5 runes). mode = user choice; resolved = actual theme applied.
// Persisted to localStorage under "onnda.theme" (key mirrored in app.html bootstrap).
export type ThemeMode = "light" | "dark" | "auto";

const KEY = "onnda.theme";

function readMode(): ThemeMode {
  if (typeof localStorage === "undefined") return "auto";
  const v = localStorage.getItem(KEY);
  return v === "light" || v === "dark" || v === "auto" ? v : "auto";
}

function systemDark(): boolean {
  return typeof window !== "undefined" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches;
}

class ThemeStore {
  mode = $state<ThemeMode>(readMode());
  #systemDark = $state<boolean>(systemDark());

  // The theme actually shown: "auto" follows the OS.
  resolved = $derived<"light" | "dark">(
    this.mode === "auto" ? (this.#systemDark ? "dark" : "light") : this.mode
  );

  constructor() {
    if (typeof window !== "undefined") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      mq.addEventListener("change", (e) => { this.#systemDark = e.matches; });
    }
    // Keep <html data-theme> and storage in sync with resolved/mode.
    $effect.root(() => {
      $effect(() => {
        document.documentElement.setAttribute("data-theme", this.resolved);
      });
      $effect(() => {
        try { localStorage.setItem(KEY, this.mode); } catch { /* ignore */ }
      });
    });
  }

  set(mode: ThemeMode) { this.mode = mode; }
}

export const theme = new ThemeStore();
```

- [ ] **Step 2: Verify**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/theme.svelte.ts
git commit -m "feat(ui): theme store (light/dark/auto) with persistence"
```

---

## Task 5: User name store

**Files:**
- Create: `src/lib/stores/userName.svelte.ts`

- [ ] **Step 1: Create `src/lib/stores/userName.svelte.ts`**

```ts
// Display name for the Home greeting. Will be populated by onboarding (sign-in,
// or "¿cómo debería llamarte?"). Persisted under "onnda.userName". Empty until set.
const KEY = "onnda.userName";

class UserNameStore {
  value = $state<string>(typeof localStorage !== "undefined"
    ? (localStorage.getItem(KEY) ?? "")
    : "");

  constructor() {
    $effect.root(() => {
      $effect(() => {
        try { localStorage.setItem(KEY, this.value); } catch { /* ignore */ }
      });
    });
  }

  set(name: string) { this.value = name.trim(); }
}

export const userName = new UserNameStore();
```

- [ ] **Step 2: Verify**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/userName.svelte.ts
git commit -m "feat(ui): userName store (onboarding will populate)"
```

---

## Task 6: Icon atom (Iconoir SVGs inlined)

**Files:**
- Create: `src/lib/components/ui/Icon.svelte`

- [ ] **Step 1: Create `src/lib/components/ui/Icon.svelte`**

Stroke-based Iconoir icons (regular, 24px grid, `currentColor`). Names map to nav: `home`, `list`, `page-plus` (Transcribe Files), `book`, `frame-tool` (Settings).

```svelte
<script module lang="ts">
  // Inlined Iconoir (MIT) regular icons. Names map to nav destinations.
  // Exported from <script module> so other components can `import { type IconName }`.
  export type IconName = "home" | "list" | "page-plus" | "book" | "frame-tool";
</script>

<script lang="ts">
  // currentColor + configurable size.
  let { name, size = 24 }: { name: IconName; size?: number } = $props();
</script>

<svg
  width={size} height={size} viewBox="0 0 24 24"
  fill="none" stroke="currentColor" stroke-width="1.5"
  stroke-linecap="round" stroke-linejoin="round"
  xmlns="http://www.w3.org/2000/svg" aria-hidden="true"
>
  {#if name === "home"}
    <path d="M2 8L11.7317 3.13416C11.9006 3.04971 12.0994 3.0497 12.2683 3.13416L22 8" />
    <path d="M20 11V19C20 20.1046 19.1046 21 18 21H6C4.89543 21 4 20.1046 4 19V11" />
  {:else if name === "list"}
    <path d="M8 6L20 6" />
    <path d="M4 6.01L4.01 5.99889" />
    <path d="M4 12.01L4.01 11.9989" />
    <path d="M4 18.01L4.01 17.9989" />
    <path d="M8 12L20 12" />
    <path d="M8 18L20 18" />
  {:else if name === "page-plus"}
    <path d="M4 12V2.6C4 2.26863 4.26863 2 4.6 2H16.2515C16.4106 2 16.5632 2.06321 16.6757 2.17574L19.8243 5.32426C19.9368 5.43679 20 5.5894 20 5.74853V21.4C20 21.7314 19.7314 22 19.4 22H11" />
    <path d="M16 2V5.4C16 5.73137 16.2686 6 16.6 6H20" />
    <path d="M1.99219 19H4.99219M7.99219 19H4.99219M4.99219 19V16M4.99219 19V22" />
  {:else if name === "book"}
    <path d="M4 19V5C4 3.89543 4.89543 3 6 3H19.4C19.7314 3 20 3.26863 20 3.6V16.7143" />
    <path d="M6 17L20 17" />
    <path d="M6 21L20 21" />
    <path d="M6 21C4.89543 21 4 20.1046 4 19C4 17.8954 4.89543 17 6 17" />
    <path d="M9 7L15 7" />
  {:else if name === "frame-tool"}
    <path d="M2 7H3M2 17H3M21 7H22M21 17H22M17 3V2M7 3V2M17 22V21M7 22V21M18 6.6V17.4C18 17.7314 17.7314 18 17.4 18H6.6C6.26863 18 6 17.7314 6 17.4V6.6C6 6.26863 6.26863 6 6.6 6H17.4C17.7314 6 18 6.26863 18 6.6Z" />
  {/if}
</svg>
```

- [ ] **Step 2: Verify**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/Icon.svelte
git commit -m "feat(ui): Icon atom — inlined Iconoir nav icons"
```

---

## Task 7: Wordmark atom

**Files:**
- Create: `src/lib/components/ui/Wordmark.svelte`

- [ ] **Step 1: Create `src/lib/components/ui/Wordmark.svelte`**

Figma: "onnda" serif 36 (`--text`) + "voice to text" sans light 12 (`--wordmark-tag`), tagline baseline-aligned near the top-right of the wordmark.

```svelte
<script lang="ts">
  import { BRAND, BRAND_TAGLINE } from "$lib/brand";
</script>

<div class="wordmark">
  <span class="name">{BRAND}</span>
  <span class="tag">{BRAND_TAGLINE}</span>
</div>

<style>
  .wordmark {
    display: inline-flex;
    align-items: flex-start;
    gap: 2px;
    line-height: 1;
  }
  .name {
    font-family: var(--font-serif);
    font-size: 36px;
    font-weight: 400;
    color: var(--text);
  }
  .tag {
    font-family: var(--font-sans);
    font-weight: 300;
    font-size: 12px;
    color: var(--wordmark-tag);
    margin-top: 11px; /* Figma: tagline offset from wordmark top */
  }
</style>
```

- [ ] **Step 2: Verify**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/Wordmark.svelte
git commit -m "feat(ui): Wordmark atom (onnda · voice to text)"
```

---

## Task 8: SectionLabel atom

**Files:**
- Create: `src/lib/components/ui/SectionLabel.svelte`

- [ ] **Step 1: Create `src/lib/components/ui/SectionLabel.svelte`**

Figma: 12px, uppercase, letter-spacing +6.48px, color `--text-section`.

```svelte
<script lang="ts">
  let { text }: { text: string } = $props();
</script>

<p class="section-label">{text}</p>

<style>
  .section-label {
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 400;
    line-height: 1.1;
    letter-spacing: 6.48px;
    text-transform: uppercase;
    color: var(--text-section);
  }
</style>
```

- [ ] **Step 2: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/SectionLabel.svelte
git commit -m "feat(ui): SectionLabel atom"
```

---

## Task 9: StatNumber atom

**Files:**
- Create: `src/lib/components/ui/StatNumber.svelte`

- [ ] **Step 1: Create `src/lib/components/ui/StatNumber.svelte`**

Figma: 24px bold, tracking -0.48px, `--text`.

```svelte
<script lang="ts">
  let { value }: { value: string | number } = $props();
</script>

<span class="stat-number">{value}</span>

<style>
  .stat-number {
    font-family: var(--font-sans);
    font-size: 24px;
    font-weight: 700;
    line-height: 1.1;
    letter-spacing: -0.48px;
    color: var(--text);
  }
</style>
```

- [ ] **Step 2: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/StatNumber.svelte
git commit -m "feat(ui): StatNumber atom"
```

---

## Task 10: Card molecule

**Files:**
- Create: `src/lib/components/ui/Card.svelte`

- [ ] **Step 1: Create `src/lib/components/ui/Card.svelte`**

Figma: `--surface`, radius 16, padding 16, column, gap 8. Accepts children + optional `width`/`grow`.

```svelte
<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    children,
    width,
    grow = false,
  }: { children: Snippet; width?: string; grow?: boolean } = $props();
</script>

<div class="card" class:grow style={width ? `width:${width}` : ""}>
  {@render children()}
</div>

<style>
  .card {
    background: var(--surface);
    border-radius: var(--r-card);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .card.grow { flex: 1 0 0; min-width: 0; }
</style>
```

- [ ] **Step 2: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/Card.svelte
git commit -m "feat(ui): Card molecule"
```

---

## Task 11: StatCard molecule

**Files:**
- Create: `src/lib/components/ui/StatCard.svelte`

- [ ] **Step 1: Create `src/lib/components/ui/StatCard.svelte`**

```svelte
<script lang="ts">
  import Card from "./Card.svelte";
  import StatNumber from "./StatNumber.svelte";
  let {
    value,
    label,
    grow = true,
    width,
  }: { value: string | number; label: string; grow?: boolean; width?: string } = $props();
</script>

<Card {grow} {width}>
  <StatNumber {value} />
  <span class="label">{label}</span>
</Card>

<style>
  .label {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 400;
    line-height: 1.1;
    color: var(--text-muted);
  }
</style>
```

- [ ] **Step 2: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/StatCard.svelte
git commit -m "feat(ui): StatCard molecule"
```

---

## Task 12: NavItem molecule

**Files:**
- Create: `src/lib/components/ui/NavItem.svelte`

- [ ] **Step 1: Create `src/lib/components/ui/NavItem.svelte`**

Figma: row, gap 12, px 12 py 8, radius 8. Active = `--nav-active-bg` pill, icon+text `--nav-active-ink`. Inactive = text `--nav-ink`. Keep `class="nav-item"` so the Sidebar `railDrag` skip selector keeps working.

```svelte
<script lang="ts">
  import Icon, { type IconName } from "./Icon.svelte";
  let {
    icon,
    label,
    active = false,
    onclick,
  }: { icon: IconName; label: string; active?: boolean; onclick: () => void } = $props();
</script>

<button class="nav-item" class:active {onclick}>
  <span class="ic"><Icon name={icon} size={24} /></span>
  <span class="label">{label}</span>
</button>

<style>
  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--s3);            /* 12px */
    width: 100%;
    padding: var(--s2) var(--s3); /* 8 / 12 */
    border: none;
    border-radius: var(--r-nav);
    background: transparent;
    color: var(--nav-ink);
    cursor: pointer;
    text-align: left;
    transition: background .12s, color .12s;
  }
  .nav-item:hover { background: rgba(127,127,127,0.10); }
  .nav-item.active { background: var(--nav-active-bg); color: var(--nav-active-ink); }
  .nav-item.active:hover { background: var(--nav-active-bg); }
  .ic { display: flex; flex-shrink: 0; }
  .label {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 400;
  }
</style>
```

- [ ] **Step 2: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/NavItem.svelte
git commit -m "feat(ui): NavItem molecule"
```

---

## Task 13: Streak grid helper + StreakCard molecule

**Files:**
- Create: `src/lib/streak.ts`
- Create: `src/lib/components/ui/StreakCard.svelte`

- [ ] **Step 1: Create the pure grid helper `src/lib/streak.ts`**

28 cells (4 rows × 7 cols) ending **today** in the last (bottom-right) cell. Each cell = a day; `active` if that day had ≥1 transcription. `today` flags the final cell.

```ts
// Pure helper for the Streak dot grid. Produces 28 cells (4 rows x 7 cols),
// oldest first, last cell = today. A cell is active if that calendar day has
// at least one timestamp in `timestampsMs`.
export interface StreakCell {
  active: boolean;
  today: boolean;
}

export function buildStreakGrid(timestampsMs: number[], now: number = Date.now()): StreakCell[] {
  const days = new Set<string>();
  for (const ts of timestampsMs) days.add(new Date(ts).toDateString());

  const cells: StreakCell[] = [];
  const base = new Date(now);
  for (let i = 27; i >= 0; i--) {
    const d = new Date(base);
    d.setDate(base.getDate() - i);
    cells.push({ active: days.has(d.toDateString()), today: i === 0 });
  }
  return cells;
}
```

- [ ] **Step 2: Create `src/lib/components/ui/StreakCard.svelte`**

Figma: 280px wide card; header row "Streak" (bold 24) + "N days" (bold 16, right); below, the 4×7 dot grid filling the remaining height. Active dots `--dot-on`, empty `--dot-off`, today gets a ring.

```svelte
<script lang="ts">
  import Card from "./Card.svelte";
  import StatNumber from "./StatNumber.svelte";
  import { buildStreakGrid } from "$lib/streak";

  let {
    timestampsMs,
    streakDays,
  }: { timestampsMs: number[]; streakDays: number } = $props();

  const cells = $derived(buildStreakGrid(timestampsMs));
  const daysLabel = $derived(streakDays === 1 ? "1 day" : `${streakDays} days`);
</script>

<Card width="280px">
  <div class="head">
    <StatNumber value="Streak" />
    <span class="days">{daysLabel}</span>
  </div>
  <div class="grid">
    {#each cells as c}
      <span class="dot" class:on={c.active} class:today={c.today}></span>
    {/each}
  </div>
</Card>

<style>
  .head { display: flex; align-items: center; justify-content: space-between; }
  .days {
    font-family: var(--font-sans);
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.32px;
    color: var(--text);
  }
  .grid {
    flex: 1 0 0;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    grid-template-rows: repeat(4, 1fr);
    align-items: center;
    justify-items: start;
    gap: var(--s2);
    min-height: 96px;
  }
  .dot {
    width: 8px; height: 8px; border-radius: 50%;
    background: var(--dot-off);
  }
  .dot.on { background: var(--dot-on); }
  .dot.today {
    background: transparent;
    box-shadow: 0 0 0 1.5px var(--dot-today);
  }
  .dot.today.on { background: var(--dot-on); }
</style>
```

- [ ] **Step 3: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 4: Spot-check the grid logic in dev**

In `npm run tauri dev`, open Home with existing history; confirm the bottom-right dot is ringed (today) and active days render green. (No JS test runner; logic is pure and small.)

- [ ] **Step 5: Commit**

```bash
git add src/lib/streak.ts src/lib/components/ui/StreakCard.svelte
git commit -m "feat(ui): StreakCard molecule + pure 28-cell grid helper"
```

---

## Task 14: FeedbackBanner molecule

**Files:**
- Create: `src/lib/components/ui/FeedbackBanner.svelte`

- [ ] **Step 1: Create `src/lib/components/ui/FeedbackBanner.svelte`**

Figma: `--surface-ink` bg, radius 16, padding 16, text `--text-on-ink` 14px line-height 1.1. Opens a feedback link via Tauri opener (the project already depends on `@tauri-apps/plugin-opener`).

```svelte
<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { BRAND } from "$lib/brand";

  const FEEDBACK_URL = "https://onnda.app/feedback"; // TODO-LINK: confirm real URL with user before ship
  function open() { openUrl(FEEDBACK_URL).catch(() => {}); }
</script>

<button class="banner" onclick={open}>
  Thanks for using {BRAND}, a lot of love went into building it. If you want to
  leave feedback or request a feature click here
</button>

<style>
  .banner {
    width: 100%;
    text-align: left;
    background: var(--surface-ink);
    color: var(--text-on-ink);
    border: none;
    border-radius: var(--r-card);
    padding: var(--s4);
    font-family: var(--font-sans);
    font-size: 14px;
    line-height: 1.1;
    cursor: pointer;
  }
</style>
```

> **Note for executor:** `FEEDBACK_URL` is the one value not in the Figma. It is marked `TODO-LINK` — surface it to the user during execution review and replace before shipping. Do not leave it unconfirmed at merge.

- [ ] **Step 2: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/FeedbackBanner.svelte
git commit -m "feat(ui): FeedbackBanner molecule"
```

---

## Task 15: Refactor Sidebar to the new system

**Files:**
- Modify: `src/lib/components/Sidebar.svelte` (full replacement)

- [ ] **Step 1: Replace `src/lib/components/Sidebar.svelte`**

Keep `railDrag` and the `view` bindable prop. Use `Wordmark` + `NavItem`. Width 245px, bg `--bg`, no right border (Figma has no visible divider; the content area sits flush). Brand at left 24 / top ~51 (top padding clears traffic lights). Nav gap 16, items full width. Labels are English per Figma.

```svelte
<script lang="ts">
  import type { View } from "$lib/types";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Wordmark from "$lib/components/ui/Wordmark.svelte";
  import NavItem from "$lib/components/ui/NavItem.svelte";
  import type { IconName } from "$lib/components/ui/Icon.svelte";

  // The whole rail is a window drag handle (title bar hidden). Nav clicks pass
  // through (we skip the drag when the target is a .nav-item).
  function railDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest(".nav-item")) return;
    getCurrentWindow().startDragging().catch(() => {});
  }

  let { view = $bindable<View>("home") }: { view?: View } = $props();

  const items: { id: View; label: string; icon: IconName }[] = [
    { id: "home",            label: "Home",            icon: "home" },
    { id: "transcripciones", label: "Transcriptions",  icon: "list" },
    { id: "importar",        label: "Transcribe Files", icon: "page-plus" },
    { id: "diccionario",     label: "Dictionary",      icon: "book" },
    { id: "ajustes",         label: "Settings",        icon: "frame-tool" },
  ];
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<aside class="sidebar" onmousedown={railDrag}>
  <div class="brand"><Wordmark /></div>
  <nav class="nav">
    {#each items as it}
      <NavItem
        icon={it.icon}
        label={it.label}
        active={view === it.id}
        onclick={() => (view = it.id)}
      />
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-w);
    flex-shrink: 0;
    background: var(--bg);
    display: flex;
    flex-direction: column;
    /* top clears the floating macOS traffic lights; left 24 per Figma */
    padding: 51px 24px 24px;
    gap: var(--s6);   /* 24px between brand and nav */
  }
  .brand { display: flex; }
  .nav { display: flex; flex-direction: column; gap: var(--s4); } /* 16px */
</style>
```

- [ ] **Step 2: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 3: Visual check** — `npm run tauri dev`: sidebar matches Figma (wordmark, 5 nav items, Home active pill) in both themes.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Sidebar.svelte
git commit -m "feat(ui): refactor Sidebar to Wordmark + NavItem atoms"
```

---

## Task 16: Rebuild Home pixel-perfect

**Files:**
- Modify: `src/lib/sections/Home.svelte` (replace template + styles; keep the `$derived` data logic)

- [ ] **Step 1: Replace `src/lib/sections/Home.svelte`**

Keep all existing data derivations (totalCount, fileCount, totalWords, weekWords, savedMinutes, streak) and the `fmt*` helpers; **remove** the now-unused `latest`/`fmtTime` peek and `fmtStreak` (StreakCard owns its label). Add `userName`. Compose with molecules. Layout: content padding left/right 40, top 81; outer column gap 32; SUMMARY block (label + 2 StatCards, gap 16); ACTIVITY block (label + row: StreakCard 280 + right column gap 16 [row of 2 StatCards + wide StatCard]); FeedbackBanner pinned at bottom.

```svelte
<script lang="ts">
  import type { HistoryEntry } from "$lib/types";
  import { userName } from "$lib/stores/userName.svelte";
  import SectionLabel from "$lib/components/ui/SectionLabel.svelte";
  import StatCard from "$lib/components/ui/StatCard.svelte";
  import StreakCard from "$lib/components/ui/StreakCard.svelte";
  import FeedbackBanner from "$lib/components/ui/FeedbackBanner.svelte";

  let { history }: { history: HistoryEntry[] } = $props();

  const totalCount = $derived(history.length);
  const fileCount = $derived(history.filter((h) => h.source === "file").length);

  function wordCount(t: string): number {
    const s = t.trim();
    return s ? s.split(/\s+/).length : 0;
  }
  const totalWords = $derived(history.reduce((s, h) => s + wordCount(h.text), 0));
  const weekWords = $derived.by(() => {
    const weekAgo = Date.now() - 7 * 24 * 3600 * 1000;
    return history.filter((h) => h.timestamp_ms >= weekAgo)
      .reduce((s, h) => s + wordCount(h.text), 0);
  });
  const savedMinutes = $derived.by(() => {
    let saved = 0;
    for (const h of history) {
      const typeMin = wordCount(h.text) / 40;
      const spokenMin = (h.duration_secs || 0) / 60;
      saved += Math.max(0, typeMin - spokenMin);
    }
    return saved;
  });
  const streak = $derived.by(() => {
    const days = new Set(history.map((h) => new Date(h.timestamp_ms).toDateString()));
    const d = new Date();
    if (!days.has(d.toDateString())) d.setDate(d.getDate() - 1);
    let s = 0;
    while (days.has(d.toDateString())) { s++; d.setDate(d.getDate() - 1); }
    return s;
  });

  const timestamps = $derived(history.map((h) => h.timestamp_ms));
  const greeting = $derived(userName.value ? `Hey ${userName.value},` : "Hey,");

  function fmtWords(n: number): string { return n.toLocaleString("es"); }
  function fmtSaved(min: number): string {
    if (min < 1) return "—";
    if (min < 60) return `${Math.round(min)} min`;
    const h = Math.floor(min / 60), m = Math.round(min % 60);
    return m > 0 ? `${h}h ${m} min` : `${h}h`;
  }
</script>

<div class="home">
  <div class="content">
    <h1 class="greeting">{greeting}</h1>

    <section class="block">
      <SectionLabel text="Summary" />
      <div class="row">
        <StatCard value={totalCount} label="Total transcriptions" />
        <StatCard value={fileCount} label="Files transcribed" />
      </div>
    </section>

    <section class="block">
      <SectionLabel text="Activity" />
      <div class="activity">
        <StreakCard timestampsMs={timestamps} streakDays={streak} />
        <div class="activity-right">
          <div class="row">
            <StatCard value={fmtWords(totalWords)} label="Dictated words total" />
            <StatCard value={fmtWords(weekWords)} label="Words this week" />
          </div>
          <StatCard value={fmtSaved(savedMinutes)} label="Time saved from typing" grow={false} />
        </div>
      </div>
    </section>
  </div>

  <div class="banner-wrap"><FeedbackBanner /></div>
</div>

<style>
  .home {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 81px var(--s10) var(--s10);  /* top 81, sides/bottom 40 */
    overflow: hidden;
  }
  .content { display: flex; flex-direction: column; gap: var(--s8); } /* 32 */
  .greeting {
    font-family: var(--font-serif);
    font-size: 24px;
    font-weight: 400;
    color: var(--text);
  }
  .block { display: flex; flex-direction: column; gap: var(--s2); } /* 8 */
  .row { display: flex; gap: var(--s4); align-items: stretch; }      /* 16 */
  .activity { display: flex; gap: var(--s4); align-items: stretch; } /* 16 */
  .activity-right {
    flex: 1 0 0; min-width: 0;
    display: flex; flex-direction: column; gap: var(--s4);
  }
  .banner-wrap { padding-top: var(--s8); }
</style>
```

- [ ] **Step 2: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 3: Visual check vs Figma**

`npm run tauri dev` → Home. Compare against light (`43:1623`) and dark (`43:1914`): greeting serif, SUMMARY two cards, ACTIVITY streak 280 + 3 cards, banner bottom. Verify spacing reads as base-8.

- [ ] **Step 4: Commit**

```bash
git add src/lib/sections/Home.svelte
git commit -m "feat(ui): rebuild Home pixel-perfect with atomic molecules"
```

---

## Task 17: Theme selector in Ajustes

**Files:**
- Modify: `src/lib/sections/Ajustes.svelte`

- [ ] **Step 1: Import the theme store**

At the top of the `<script>` in `src/lib/sections/Ajustes.svelte`, add:
```ts
  import { theme, type ThemeMode } from "$lib/stores/theme.svelte";
  const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
    { value: "light", label: "Claro" },
    { value: "dark",  label: "Oscuro" },
    { value: "auto",  label: "Automático" },
  ];
```

- [ ] **Step 2: Add a theme section to the template**

Add near the top of the Ajustes settings list (follow the existing markup pattern in the file — wrap in whatever row/section container the file already uses). Minimal self-contained block:
```svelte
<div class="theme-row">
  <span class="theme-label">Apariencia</span>
  <div class="seg">
    {#each THEME_OPTIONS as opt}
      <button
        class="seg-btn"
        class:on={theme.mode === opt.value}
        onclick={() => theme.set(opt.value)}
      >{opt.label}</button>
    {/each}
  </div>
</div>
```

- [ ] **Step 3: Add styles (append to the file's `<style>`)**

```css
  .theme-row { display: flex; align-items: center; justify-content: space-between; gap: var(--s4); padding: var(--s2) 0; }
  .theme-label { font-family: var(--font-sans); font-size: 14px; color: var(--text); }
  .seg { display: inline-flex; background: var(--surface); border-radius: var(--r-nav); padding: 2px; gap: 2px; }
  .seg-btn {
    border: none; background: transparent; cursor: pointer;
    font-family: var(--font-sans); font-size: 13px; color: var(--text-muted);
    padding: 6px 12px; border-radius: 6px;
  }
  .seg-btn.on { background: var(--nav-active-bg); color: var(--nav-active-ink); }
```

- [ ] **Step 4: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 5: Visual check** — `npm run tauri dev` → Settings: switching Claro/Oscuro/Automático repaints the whole app live.

- [ ] **Step 6: Commit**

```bash
git add src/lib/sections/Ajustes.svelte
git commit -m "feat(ui): theme selector (light/dark/auto) in Ajustes"
```

---

## Task 18: Shell layout — content surface + remove stale glass

**Files:**
- Modify: `src/routes/+page.svelte` (shell container styles only)

- [ ] **Step 1: Inspect the shell markup**

Read `src/routes/+page.svelte`. The shell wraps `<Sidebar>` + the active section in a flex row on a window surface. The old styles reference dropped tokens (`--glass-*`, `--panel`, borders). 

- [ ] **Step 2: Make the shell flat per Figma**

Ensure the outer shell uses `background: var(--bg)`, fills the window, rounds with `--r-window`, and lays sidebar + content as a flex row where the content area scrolls. Remove any `--glass-*`, aurora, grain, and border references in the shell `<style>`. The content column background is `var(--bg)` (sidebar and content share the flat bg per Figma; cards provide the only contrast). Example shell style block (adapt selector names to the file):
```css
  .shell {
    display: flex;
    height: 100vh;
    background: var(--bg);
    border-radius: var(--r-window);
    overflow: hidden;
  }
  .main { flex: 1 0 0; min-width: 0; height: 100vh; overflow-y: auto; background: var(--bg); }
```

- [ ] **Step 3: Verify** — Run: `npm run check` → 0 errors.

- [ ] **Step 4: Visual check** — `npm run tauri dev`: window is flat `--bg`, rounded; no leftover glass/aurora; sidebar 245 + content flush. Both themes.

- [ ] **Step 5: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat(ui): flatten app shell to onnda surfaces"
```

---

## Task 19: Final pass — full visual verification both themes

**Files:** none (verification + cleanup only)

- [ ] **Step 1: Type/lint gate**

Run: `npm run check`
Expected: 0 errors, 0 warnings.

- [ ] **Step 2: Side-by-side visual diff**

`npm run tauri dev`. With Home open, compare against the Figma screenshots:
- Light (`43:1623`): bg #d6d8d7, cards #e6e6e6, greeting serif, SUMMARY/ACTIVITY tracking, streak dots, banner dark.
- Dark (`43:1914`): toggle via Settings → bg #181818, cards #222, nav active = light pill.
Check spacing reads base-8 and the sidebar is 245px.

- [ ] **Step 3: Theme persistence**

Switch to Dark, fully quit and relaunch the dev app. Expected: opens in Dark with no light flash (app.html bootstrap).

- [ ] **Step 4: Confirm the one open value**

Surface `FEEDBACK_URL` (Task 14) to the user and replace with the real link, or leave a tracked TODO if not yet known. Do not silently ship the placeholder.

- [ ] **Step 5: Final commit (if any cleanup)**

```bash
git add -A
git commit -m "chore(ui): final onnda Home redesign pass — verified light/dark"
```

---

## Self-review notes (author)

- **Spec coverage:** tokens (T2), fonts (T1), theme store+persistence+Ajustes control (T4,T17), brand var (T3), atomic layer Icon/Wordmark/SectionLabel/StatNumber/Card/StatCard/NavItem/StreakCard/FeedbackBanner (T6–T14), Sidebar organism (T15), Home pixel-perfect (T16), userName/onboarding seam (T5,T16), shell flatten (T18), dark=palette-only / light canonical (T16 single layout). All spec sections map to a task.
- **Out of scope (follow-up plan):** migrating Transcripciones / Importar / Diccionario / Ajustes (beyond the theme selector) to the new molecules. Tracked, not in this plan.
- **Type consistency:** `IconName` union shared by Icon/NavItem/Sidebar; `ThemeMode` shared by theme store/Ajustes; `buildStreakGrid` / `StreakCell` shared by streak.ts/StreakCard; store singletons `theme`, `userName`.
- **Known placeholder:** `FEEDBACK_URL` (Task 14) — the only value absent from Figma; flagged in T14 and T19-step4 for user confirmation, not left silent.
