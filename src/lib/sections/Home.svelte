<script lang="ts">
  import type { HistoryEntry } from "$lib/types";

  // Inicio = lightweight dashboard. Just the "Resumen" stats + a one-line recent
  // peek. No dictation hero (dictation is hotkey-only); navigation is the sidebar.
  let { history }: { history: HistoryEntry[] } = $props();

  const totalCount = $derived(history.length);
  const fileCount = $derived(history.filter((h) => h.source === "file").length);
  const latest = $derived(history[0] ?? null);

  function fmtTime(ms: number) {
    const d = new Date(ms), now = new Date();
    return now.toDateString() === d.toDateString()
      ? d.toLocaleTimeString("es", { hour: "2-digit", minute: "2-digit" })
      : d.toLocaleDateString("es", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
  }
</script>

<h1 class="page-title">Inicio</h1>

<!-- ── Resumen ── -->
<section class="block">
  <h2 class="section-label">Resumen</h2>
  <div class="stats">
    <div class="stat-card">
      <span class="stat-num">{totalCount}</span>
      <span class="stat-label">Transcripciones</span>
    </div>
    <div class="stat-card">
      <span class="stat-num">{fileCount}</span>
      <span class="stat-label">Archivos</span>
    </div>
  </div>
  {#if latest}
    <div class="peek">
      <span class="peek-time">{fmtTime(latest.timestamp_ms)}</span>
      <span class="peek-text">{latest.text}</span>
    </div>
  {/if}
</section>

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }

  section { margin-top: 24px; display: flex; flex-direction: column; gap: 10px; }

  .section-label {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: .06em; color: var(--faint); padding: 0 3px;
  }

  /* ── Stats — dark glass panels ── */
  .stats { display: flex; gap: 10px; }
  .stat-card {
    flex: 1; position: relative;
    background: var(--glass-fill);
    -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--line); border-radius: var(--r);
    box-shadow: var(--glass-edge), var(--sh-2);
    padding: 16px; display: flex; flex-direction: column; gap: 4px;
  }
  .stat-num { font-size: 24px; font-weight: 600; color: var(--text); letter-spacing: -.02em; }
  .stat-label {
    font-size: 11px; font-weight: 450; color: var(--faint);
    text-transform: uppercase; letter-spacing: .04em;
  }

  .peek {
    display: flex; align-items: baseline; gap: 8px; padding: 0 3px;
    overflow: hidden;
  }
  .peek-time { font-size: 11px; color: var(--faint); flex-shrink: 0; }
  .peek-text {
    font-size: 12px; color: var(--muted); line-height: 1.4;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
</style>
