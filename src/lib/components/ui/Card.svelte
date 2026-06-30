<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    children,
    width,
    grow = false,
  }: { children: Snippet; width?: string; grow?: boolean } = $props();
</script>

<div class="card" class:grow style={width ? `width:${width}` : ""}>
  {@render children()}
</div>

<style>
  .card {
    background: var(--surface);
    border-radius: var(--r-card);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s2);
  }
  .card.grow { flex: 1 0 0; min-width: 0; }
</style>
