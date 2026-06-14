<script lang="ts">
  import type { Snippet } from "svelte";
  // Panel surface that groups rows. Extracted from +page.svelte .rows (L561-567).
  // Place <div class="sep"></div> between rows for the hairline separator
  // (1px var(--line), margin 0 12px). The .sep style is global so children rendered
  // into the slot pick it up.
  let { children }: { children?: Snippet } = $props();
</script>

<div class="rows">
  {@render children?.()}
</div>

<style>
  .rows {
    background: var(--panel);
    border-radius: var(--r);
    overflow: hidden;
  }
  .rows :global(.sep) { height: 1px; background: var(--line); margin: 0 12px; }
</style>
