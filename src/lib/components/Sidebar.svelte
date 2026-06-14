<script lang="ts">
  import type { View } from "$lib/types";
  // Fixed 200px nav rail (--panel surface, 1px var(--line) right border).
  // Top = wordmark + version; below = 4 nav items. Active = --text on --bg pill
  // (mirror .tab.on), radius 6px. NO coral on inactive nav (UI-SPEC).
  let {
    view = $bindable<View>("home"),
    appVersion = "",
    buildHash = "",
  }: {
    view?: View;
    appVersion?: string;
    buildHash?: string;
  } = $props();

  const items: { id: View; label: string }[] = [
    { id: "home",           label: "Home" },
    { id: "transcripciones", label: "Transcripciones" },
    { id: "importar",       label: "Importar" },
    { id: "diccionario",    label: "Diccionario" },
    { id: "ajustes",        label: "Ajustes" },
  ];
</script>

<aside class="sidebar">
  <div class="brand">
    <span class="wordmark">Voz Local</span>
    {#if appVersion}<span class="version">v{appVersion}{#if buildHash} · {buildHash}{/if}</span>{/if}
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
    background: var(--panel);
    border-right: 1px solid var(--line);
    display: flex; flex-direction: column;
    padding: 16px 12px;
    gap: 18px;
  }

  .brand { display: flex; flex-direction: column; gap: 2px; padding: 4px 8px; }
  .wordmark { font-size: 13.5px; font-weight: 600; color: var(--text); letter-spacing: -.02em; }
  .version { font-size: 10.5px; font-weight: 450; color: var(--faint); }

  .nav { display: flex; flex-direction: column; gap: 2px; }

  .nav-item {
    position: relative;
    display: flex; align-items: center; gap: 10px;
    height: 34px; padding: 0 10px 0 14px;
    background: none; border: none; border-radius: 6px;
    color: var(--faint); cursor: pointer;
    transition: color .12s, background .12s;
    text-align: left; width: 100%;
  }
  .nav-item:hover { color: var(--muted); }
  /* Active = the ONLY nav affordance: --text on --bg pill (UI-SPEC) + a thin
     iridescent left rail (the notch ramp, vertical) as the accent garnish. */
  .nav-item.on { color: var(--text); background: var(--bg); }
  .nav-item.on::before {
    content: ""; position: absolute; left: 4px; top: 7px; bottom: 7px;
    width: 3px; border-radius: 3px;
    background: linear-gradient(180deg,
      var(--film-warm), var(--film-3) 50%, var(--film-cool));
    pointer-events: none;
  }
  @media (prefers-contrast: more) {
    .nav-item.on::before { background: var(--coral); }
  }

  .icon { display: flex; flex-shrink: 0; }
  .icon svg { width: 16px; height: 16px; }
  .label { font-size: 13px; font-weight: 450; }
</style>
