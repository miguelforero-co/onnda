<script lang="ts">
  import type { View } from "$lib/types";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  // The whole rail is a window drag handle (the title bar is hidden). The native
  // title bar's transparent container eats events in the top ~28px, so a tiny top
  // strip alone isn't reliably reachable — dragging from the large sidebar body is.
  // Nav clicks pass through (we skip the drag when the target is a nav item).
  function railDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest(".nav-item")) return;
    getCurrentWindow().startDragging().catch(() => {});
  }
  // Fixed 200px nav rail (--panel surface, 1px var(--line) right border).
  // Top = wordmark; below = nav items. Active = --text on --bg pill
  // (mirror .tab.on), radius 6px. NO coral on inactive nav (UI-SPEC).
  // Version + build live in Ajustes → Actualizaciones, not here.
  let {
    view = $bindable<View>("home"),
  }: {
    view?: View;
  } = $props();

  const items: { id: View; label: string }[] = [
    { id: "home",           label: "Home" },
    { id: "transcripciones", label: "Transcripciones" },
    { id: "importar",       label: "Importar" },
    { id: "diccionario",    label: "Diccionario" },
    { id: "ajustes",        label: "Ajustes" },
  ];
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<aside class="sidebar" onmousedown={railDrag}>
  <!-- Brand area sits below the floating macOS traffic lights (title bar hidden,
       Overlay style). The whole rail is a window drag handle (see railDrag). -->
  <div class="brand">
    <span class="wordmark">Voz Local</span>
  </div>

  <nav class="nav">
    {#each items as it}
      <button class="nav-item" class:on={view === it.id} onclick={() => (view = it.id)}>
        <span class="icon">
          {#if it.id === "home"}
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2.5 6.5 8 2l5.5 4.5" /><path d="M3.5 6.5V13h9V6.5" />
            </svg>
          {:else if it.id === "transcripciones"}
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="5" y1="4.5" x2="13" y2="4.5" /><line x1="5" y1="8" x2="13" y2="8" /><line x1="5" y1="11.5" x2="10" y2="11.5" />
              <circle cx="2.6" cy="4.5" r=".6" fill="currentColor" stroke="none" /><circle cx="2.6" cy="8" r=".6" fill="currentColor" stroke="none" /><circle cx="2.6" cy="11.5" r=".6" fill="currentColor" stroke="none" />
            </svg>
          {:else if it.id === "importar"}
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M8 2v7" /><path d="M5 5l3-3 3 3" /><path d="M2.5 9.5v2.5a1.5 1.5 0 0 0 1.5 1.5h8a1.5 1.5 0 0 0 1.5-1.5V9.5" />
            </svg>
          {:else if it.id === "diccionario"}
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 3h7a2 2 0 0 1 2 2v8H5a2 2 0 0 1-2-2V3Z" /><path d="M3 11a2 2 0 0 1 2-2h7" />
            </svg>
          {:else}
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="8" cy="8" r="2.2" /><path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M12.6 3.4l-1.4 1.4M4.8 11.2l-1.4 1.4" />
            </svg>
          {/if}
        </span>
        <span class="label">{it.label}</span>
      </button>
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    width: 200px; flex-shrink: 0;
    background: var(--glass-fill);
    -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
    border-right: 1px solid var(--line);
    box-shadow: var(--glass-edge);
    display: flex; flex-direction: column;
    padding: 16px 12px;
    gap: 18px;
  }

  .brand { display: flex; flex-direction: column; gap: 2px; padding: 4px 8px; }
  /* Wordmark = brushed chrome (Igloo cue) at display size, solid off-white fallback. */
  .wordmark {
    font-size: 14px; font-weight: 700; letter-spacing: -.02em;
    background: var(--chrome);
    -webkit-background-clip: text; background-clip: text;
    color: var(--text);
    -webkit-text-fill-color: transparent;
  }
  .wordmark::selection { -webkit-text-fill-color: var(--text); color: var(--text); }
  @media (prefers-contrast: more) {
    .wordmark { background: none; -webkit-text-fill-color: var(--text); color: var(--text); }
  }

  .nav { display: flex; flex-direction: column; gap: 2px; }

  .nav-item {
    position: relative;
    display: flex; align-items: center; gap: 10px;
    height: 34px; padding: 0 10px 0 14px;
    background: none; border: none; border-radius: var(--r-sm);
    color: var(--faint); cursor: pointer;
    transition: color .14s, background .14s;
    text-align: left; width: 100%;
  }
  .nav-item:hover { color: var(--muted); background: rgba(255,255,255,0.04); }
  /* Active = off-white text on a faint glass pill + an iridescent left rail
     (the notch ramp gone vertical) with a soft glow — the one accent in the rail. */
  .nav-item.on { color: var(--text); background: rgba(255,255,255,0.06); }
  .nav-item.on::before {
    content: ""; position: absolute; left: 4px; top: 7px; bottom: 7px;
    width: 3px; border-radius: 3px; pointer-events: none;
    background: linear-gradient(180deg, var(--iris-1), var(--iris-3) 50%, var(--iris-5));
    box-shadow: 0 0 10px -1px rgba(180,140,252,0.6);
  }
  @media (prefers-contrast: more) {
    .nav-item.on::before { background: var(--accent); box-shadow: none; }
  }

  .icon { display: flex; flex-shrink: 0; }
  .icon svg { width: 16px; height: 16px; }
  .label { font-size: 13px; font-weight: 450; }
</style>
