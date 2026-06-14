<script lang="ts">
  import type { HistoryEntry } from "$lib/types";
  // Stub — filled in 01-08. Prop contract is the source of truth for downstream plans.
  let {
    history,
    onRefresh,
  }: {
    history: HistoryEntry[];
    onRefresh: () => void;
  } = $props();
</script>

<h1 class="page-title">Transcripciones</h1>

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }
</style>
