<script lang="ts">
  import type { ModelInfo, DownloadProgress } from "$lib/types";
  // Extracted from +page.svelte model rows (L404-431) + dl-bar CSS (L749-760).
  // States: Installed (muted badge), Downloading (dl-bar + %), Descargar (link-btn),
  // Selected (nav-active-bg border), comingSoon (muted/disabled, "Próximamente").
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

  const hardwareDisabled = $derived(!!model.disabled_reason);
  const selectable = $derived(model.downloaded && !comingSoon && !model.disabled_reason);
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="model-card"
  class:selected={selected && !comingSoon}
  class:coming-soon={comingSoon || hardwareDisabled}
  role={selectable ? "button" : "presentation"}
  tabindex={selectable ? 0 : -1}
  onclick={() => { if (selectable) onSelect?.(model.id); }}
  onkeydown={(e) => { if (selectable && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); onSelect?.(model.id); } }}
>
  <div class="model-info">
    <strong>{model.name}</strong>
    <span>{model.size_mb > 0 ? `${model.size_mb} MB` : "On device · no download · fastest"}</span>
    {#if comingSoon}<span class="sub">Available in a future version.</span>{/if}
    {#if hardwareDisabled}<span class="sub" title={model.disabled_reason}>{model.disabled_reason}</span>{/if}
  </div>

  <div class="model-action">
    {#if hardwareDisabled}
      <span class="badge soon">Not available</span>
    {:else if comingSoon}
      <span class="badge soon">Coming soon</span>
    {:else if model.downloaded && selected}
      <span class="badge active">Active</span>
    {:else if model.downloaded}
      <span class="badge installed">{model.size_mb > 0 ? "Installed" : "Native"}</span>
    {:else if progress}
      <div class="dl-bar-wrap"><div class="dl-bar" style="width:{progress.percent}%"></div></div>
      <span class="dl-pct">{Math.round(progress.percent)}%</span>
    {:else}
      <button class="download-btn" onclick={(e) => { e.stopPropagation(); onDownload?.(model.id); }}>Download</button>
    {/if}
  </div>

  {#if error}<p class="dl-error">{error}</p>{/if}
</div>

<style>
  .model-card {
    position: relative;
    display: flex; align-items: center; gap: 12px; flex-wrap: wrap;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--r-card);
    padding: var(--s4);
    transition: background .16s, border-color .16s;
  }
  .model-card[role="button"]:hover {
    border-color: var(--line-strong);
  }
  .model-card.selected {
    border-color: var(--nav-active-bg);
  }
  .model-card.coming-soon { opacity: .55; cursor: default; }
  .model-card[role="button"] { cursor: pointer; }

  .model-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .model-info strong { font-size: 14px; font-weight: 450; color: var(--text); }
  .model-info span { font-size: 12px; color: var(--text-muted); }
  .model-info .sub { font-size: 12px; color: var(--text-muted); }

  .model-action { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }

  .badge {
    font-size: 12px; font-weight: 450; border-radius: var(--r-nav);
    padding: 2px 9px; color: var(--text-muted); background: transparent;
    border: 1px solid var(--line);
  }
  .badge.installed { color: var(--text-muted); }
  .badge.active { color: var(--dot-on); border-color: var(--dot-on); font-weight: 600; }
  .badge.soon { color: var(--text-muted); }

  .download-btn {
    background: var(--nav-active-bg);
    color: var(--nav-active-ink);
    border: none;
    border-radius: var(--r-nav);
    padding: 8px 16px;
    font-size: 14px; font-weight: 600;
    cursor: pointer;
    transition: opacity .15s;
    font-family: inherit;
  }
  .download-btn:hover { opacity: .85; }

  .dl-bar-wrap {
    width: 80px; height: 4px; background: var(--line);
    border-radius: 2px; overflow: hidden;
  }
  .dl-bar {
    height: 100%; background: var(--dot-on);
    border-radius: 2px; transition: width .3s linear;
  }
  .dl-pct { font-size: 12px; color: var(--text-muted); width: 30px; text-align: right; }
  .dl-error { font-size: 12px; color: var(--text-muted); width: 100%; padding-bottom: 6px; }
</style>
