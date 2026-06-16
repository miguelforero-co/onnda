<script lang="ts">
  let { value }: { value: string | number } = $props();
</script>

<span class="stat-number">{value}</span>

<style>
  .stat-number {
    font-family: var(--font-sans);
    font-size: 24px;
    font-weight: 700;
    line-height: 1.1;
    letter-spacing: -0.48px;
    color: var(--text);
  }
</style>
