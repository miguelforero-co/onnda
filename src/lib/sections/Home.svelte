<script lang="ts">
  import type { HistoryEntry } from "$lib/types";

  // Inicio = lightweight dashboard. Just the "Resumen" stats + a one-line recent
  // peek. No dictation hero (dictation is hotkey-only); navigation is the sidebar.
  let { history }: { history: HistoryEntry[] } = $props();

  const totalCount = $derived(history.length);
  const fileCount = $derived(history.filter((h) => h.source === "file").length);
  const latest = $derived(history[0] ?? null);

  // ── Usage stats (backlog #4) — derived from existing history ──
  function wordCount(t: string): number {
    const s = t.trim();
    return s ? s.split(/\s+/).length : 0;
  }

  const totalWords = $derived(history.reduce((s, h) => s + wordCount(h.text), 0));

  const weekWords = $derived.by(() => {
    const weekAgo = Date.now() - 7 * 24 * 3600 * 1000;
    return history
      .filter((h) => h.timestamp_ms >= weekAgo)
      .reduce((s, h) => s + wordCount(h.text), 0);
  });

  // Time saved: typing the same text at ~40 wpm vs the time actually spoken.
  const savedMinutes = $derived.by(() => {
    let saved = 0;
    for (const h of history) {
      const typeMin = wordCount(h.text) / 40;
      const spokenMin = (h.duration_secs || 0) / 60;
      saved += Math.max(0, typeMin - spokenMin);
    }
    return saved;
  });

  // Consecutive days (ending today or yesterday) with at least one transcription.
  const streak = $derived.by(() => {
    const days = new Set(history.map((h) => new Date(h.timestamp_ms).toDateString()));
    const d = new Date();
    if (!days.has(d.toDateString())) d.setDate(d.getDate() - 1);
    let s = 0;
    while (days.has(d.toDateString())) {
      s++;
      d.setDate(d.getDate() - 1);
    }
    return s;
  });

  function fmtWords(n: number): string {
    return n.toLocaleString("es");
  }
  function fmtSaved(min: number): string {
    if (min < 1) return "—";
    if (min < 60) return `${Math.round(min)} min`;
    const h = Math.floor(min / 60);
    const m = Math.round(min % 60);
    return m > 0 ? `${h} h ${m} min` : `${h} h`;
  }
  function fmtStreak(n: number): string {
    return n === 1 ? "1 día" : `${n} días`;
  }

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

<!-- ── Tu actividad (usage stats, backlog #4) ── -->
{#if totalCount > 0}
  <section class="block">
    <h2 class="section-label">Tu actividad</h2>
    <div class="stats grid">
      <div class="stat-card">
        <span class="stat-num">{fmtWords(totalWords)}</span>
        <span class="stat-label">Palabras dictadas</span>
      </div>
      <div class="stat-card">
        <span class="stat-num">{fmtWords(weekWords)}</span>
        <span class="stat-label">Esta semana</span>
      </div>
      <div class="stat-card">
        <span class="stat-num">{fmtSaved(savedMinutes)}</span>
        <span class="stat-label">Tiempo ahorrado</span>
      </div>
      <div class="stat-card">
        <span class="stat-num">{fmtStreak(streak)}</span>
        <span class="stat-label">Racha</span>
      </div>
    </div>
  </section>
{/if}

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }

  section { margin-top: 24px; display: flex; flex-direction: column; gap: 10px; }

  .section-label {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: .06em; color: var(--faint); padding: 0 3px;
  }

  /* ── Stats — dark glass panels ── */
  .stats { display: flex; gap: 10px; }
  .stats.grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
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
