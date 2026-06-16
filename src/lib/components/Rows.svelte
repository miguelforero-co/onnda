<script lang="ts">
  import type { Snippet } from "svelte";
  // Panel surface that groups rows. Extracted from +page.svelte .rows (L561-567).
  // Rows render as a card container with internal spacing; no separator dividers
  // by default. If a divider is truly needed between children, use
  // <div class="sep"></div> — rendered as a 1px var(--line) hairline (margin 0 12px).
  let { children }: { children?: Snippet } = $props();
</script>

<div class="rows">
  {@render children?.()}
</div>

<style>
  .rows {
    background: var(--surface);
    border-radius: var(--r-card);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s3);
    overflow: hidden;
  }
  .rows :global(.sep) { height: 1px; background: var(--line); margin: 0 12px; }
</style>
