<script lang="ts">
  import type { Settings } from "$lib/types";
  // Stub — filled in 01-08. Prop contract is the source of truth for downstream plans.
  let {
    settings,
    onSave,
  }: {
    settings: Settings;
    onSave: () => void;
  } = $props();
</script>

<h1 class="page-title">Diccionario</h1>

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }
</style>
