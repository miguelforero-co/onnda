<script lang="ts">
  import Card from "./Card.svelte";
  import StatNumber from "./StatNumber.svelte";
  let {
    value,
    label,
    grow = true,
    width,
  }: { value: string | number; label: string; grow?: boolean; width?: string } = $props();
</script>

<Card {grow} {width}>
  <StatNumber {value} />
  <span class="label">{label}</span>
</Card>

<style>
  .label {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 400;
    line-height: 1.1;
    color: var(--text-muted);
  }
</style>
