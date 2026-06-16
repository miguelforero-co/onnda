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
    padding: 51px var(--s10) var(--s10);  /* top aligns with the sidebar wordmark (51px) */
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
