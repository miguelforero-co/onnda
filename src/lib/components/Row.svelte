<script lang="ts">
  import type { Snippet } from "svelte";
  // Generic settings row — label on the left, control (children) on the right.
  // Extracted from +page.svelte .row (L569-577). Wrap groups of rows in a
  // <div class="rows"> with <div class="sep"> separators (see Rows.svelte).
  let {
    label = "",
    children,
  }: {
    label?: string;
    children?: Snippet;
  } = $props();
</script>

<div class="row">
  {#if label}<span class="row-label">{label}</span>{/if}
  {@render children?.()}
</div>

<style>
  .row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 14px; gap: 12px; min-height: 42px;
    cursor: default;
  }
  .row-label { font-size: 13px; font-weight: 450; color: var(--text); }
</style>
