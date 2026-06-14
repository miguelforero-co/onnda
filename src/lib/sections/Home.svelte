<script lang="ts">
  import type { Settings, HistoryEntry, View } from "$lib/types";
  // Stub — filled in 01-07. Prop contract is the source of truth for downstream plans.
  let {
    settings,
    history,
    onNavigate,
  }: {
    settings: Settings;
    history: HistoryEntry[];
    onNavigate: (v: View) => void;
  } = $props();
</script>

<h1 class="page-title">Home</h1>

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }
</style>
