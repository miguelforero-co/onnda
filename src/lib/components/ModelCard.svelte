<script lang="ts">
  import type { ModelInfo, DownloadProgress } from "$lib/types";
  // Extracted from +page.svelte model rows (L404-431) + dl-bar CSS (L749-760).
  // States: Installed (--blue badge), Downloading (dl-bar + %), Descargar (link-btn),
  // Selected (1px coral ring), comingSoon (muted/disabled, "Próximamente", NO coral).
  let {
    model,
    selected = false,
    progress,
    error,
    comingSoon = false,
    onDownload,
    onSelect,
  }: {
    model: ModelInfo;
    selected?: boolean;
    progress?: DownloadProgress | undefined;
    error?: string | undefined;
    comingSoon?: boolean;
    onDownload?: (id: string) => void;
    onSelect?: (id: string) => void;
  } = $props();

  const selectable = $derived(model.downloaded && !comingSoon);
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="model-card"
  class:selected={selected && !comingSoon}
  class:coming-soon={comingSoon}
  role={selectable ? "button" : "presentation"}
  tabindex={selectable ? 0 : -1}
  onclick={() => { if (selectable) onSelect?.(model.id); }}
  onkeydown={(e) => { if (selectable && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); onSelect?.(model.id); } }}
>
  <div class="model-info">
    <strong>{model.name}</strong>
    <span>{model.size_mb > 0 ? `${model.size_mb} MB` : "En el dispositivo · sin descarga · más rápido"}</span>
    {#if comingSoon}<span class="sub">Disponible en una próxima versión.</span>{/if}
  </div>

  <div class="model-action">
    {#if comingSoon}
      <span class="badge soon">Próximamente</span>
    {:else if model.downloaded && selected}
      <span class="badge active">Activo</span>
    {:else if model.downloaded}
      <span class="badge installed">{model.size_mb > 0 ? "Instalado" : "Nativo"}</span>
    {:else if progress}
      <div class="dl-bar-wrap"><div class="dl-bar" style="width:{progress.percent}%"></div></div>
      <span class="dl-pct">{Math.round(progress.percent)}%</span>
    {:else}
      <button class="link-btn" onclick={(e) => { e.stopPropagation(); onDownload?.(model.id); }}>Descargar</button>
    {/if}
  </div>

  {#if error}<p class="dl-error">{error}</p>{/if}
</div>

<style>
  .model-card {
    position: relative;
    display: flex; align-items: center; gap: 12px; flex-wrap: wrap;
    background: var(--glass-fill);
    -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--line);
    border-radius: var(--r);
    box-shadow: var(--glass-edge), var(--sh-2);
    padding: 10px 14px; min-height: 42px;
    transition: transform .16s var(--ease-soft), background .16s, border-color .16s, box-shadow .16s;
  }
  .model-card[role="button"]:hover {
    transform: translateY(-1px);
    background: var(--glass-fill-hi);
    border-color: var(--line-strong);
    box-shadow: var(--glass-edge), var(--sh-3);
  }
  /* Selected: a 1px iridescent ring (padding-box solid over border-box conic),
     no layout shift since the border stays 1px. Only the selected card earns it. */
  .model-card.selected {
    border-color: transparent;
    background:
      linear-gradient(var(--elev-2), var(--elev-2)) padding-box,
      var(--iris-ring) border-box;
    box-shadow: var(--glass-edge), 0 0 24px -8px rgba(180,140,252,0.4), var(--sh-2);
  }
  .model-card.selected:hover { transform: translateY(-1px); }
  @media (prefers-reduced-motion: no-preference) {
    .model-card.selected { animation: ring-drift var(--ring-dur) linear infinite; }
    @keyframes ring-drift { to { --iris-angle: 480deg; } }
  }
  /* Solid coral ring fallback for engines without @property animation support. */
  @supports not (background: paint(something)) {
    .model-card.selected { border: 1.5px solid var(--accent); background: var(--elev-2); box-shadow: var(--glass-edge), var(--sh-2); }
  }
  @media (prefers-contrast: more) {
    .model-card.selected { border: 1.5px solid var(--accent); background: var(--elev-2); box-shadow: none; }
  }
  .model-card.coming-soon { opacity: .55; cursor: default; }
  .model-card[role="button"] { cursor: pointer; }

  .model-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .model-info strong { font-size: 13px; font-weight: 450; color: var(--text); }
  .model-info span { font-size: 11px; color: var(--muted); }
  .model-info .sub { font-size: 11px; color: var(--muted); }

  .model-action { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }

  .badge {
    font-size: 11px; font-weight: 450; border-radius: 20px;
    padding: 2px 9px; color: var(--muted); background: rgba(255,255,255,.07);
  }
  .badge.installed { color: var(--blue); background: rgba(127,200,255,.14); }
  .badge.active { color: var(--accent); background: rgba(255,106,61,.16); font-weight: 600; }
  .badge.soon { color: var(--faint); background: rgba(255,255,255,.05); }

  .link-btn {
    background: none; border: none; padding: 4px 0;
    font-size: 12px; font-weight: 450; color: var(--coral);
    cursor: pointer; text-decoration: none;
  }
  .link-btn:hover { opacity: .75; }

  .dl-bar-wrap {
    width: 80px; height: 4px; background: rgba(255,255,255,.10);
    border-radius: 2px; overflow: hidden;
  }
  .dl-bar {
    height: 100%; background: var(--iris-ramp); background-size: 200% 100%;
    border-radius: 2px; transition: width .3s linear;
  }
  .dl-pct { font-size: 11px; color: var(--faint); width: 30px; text-align: right; }
  .dl-error { font-size: 11px; color: var(--coral); width: 100%; padding-bottom: 6px; }
</style>
