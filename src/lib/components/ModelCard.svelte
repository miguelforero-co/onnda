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
    <span>{model.size_mb} MB</span>
    {#if comingSoon}<span class="sub">Disponible en una próxima versión.</span>{/if}
  </div>

  <div class="model-action">
    {#if comingSoon}
      <span class="badge soon">Próximamente</span>
    {:else if model.downloaded}
      <span class="badge installed">Instalado</span>
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
    display: flex; align-items: center; gap: 12px; flex-wrap: wrap;
    background: var(--panel);
    border: 1px solid transparent;
    border-radius: var(--r);
    padding: 10px 14px; min-height: 42px;
  }
  .model-card.selected { border-color: var(--coral); }
  .model-card.coming-soon { opacity: .55; cursor: default; }
  .model-card[role="button"] { cursor: pointer; }

  .model-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .model-info strong { font-size: 13px; font-weight: 450; color: var(--text); }
  .model-info span { font-size: 11px; color: var(--faint); }
  .model-info .sub { font-size: 11px; color: var(--faint); }

  .model-action { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }

  .badge {
    font-size: 11px; font-weight: 450; border-radius: 20px;
    padding: 2px 9px; color: var(--faint); background: rgba(0,0,0,.05);
  }
  .badge.installed { color: var(--blue); background: rgba(123,155,210,.12); }
  .badge.soon { color: var(--faint); background: rgba(0,0,0,.05); }

  .link-btn {
    background: none; border: none; padding: 4px 0;
    font-size: 12px; font-weight: 450; color: var(--coral);
    cursor: pointer; text-decoration: none;
  }
  .link-btn:hover { opacity: .75; }

  .dl-bar-wrap {
    width: 80px; height: 4px; background: rgba(0,0,0,.08);
    border-radius: 2px; overflow: hidden;
  }
  .dl-bar {
    height: 100%; background: var(--coral);
    border-radius: 2px; transition: width .3s linear;
  }
  .dl-pct { font-size: 11px; color: var(--faint); width: 30px; text-align: right; }
  .dl-error { font-size: 11px; color: var(--coral); width: 100%; padding-bottom: 6px; }
</style>
