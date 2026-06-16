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
