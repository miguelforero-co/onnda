<!-- src/lib/components/ui/Select.svelte -->
<!-- Dropdown reutilizable (estilo .sel extraído de Ajustes) -->
<script lang="ts">
  type Option = { label: string; value: string; disabled?: boolean };
  let {
    value = $bindable(),
    options,
    onchange,
    ariaLabel,
  }: {
    value: string;
    options: Option[];
    onchange?: () => void;
    ariaLabel?: string;
  } = $props();
</script>

<select class="sel" bind:value aria-label={ariaLabel} onchange={() => onchange?.()}>
  {#each options as o (o.value)}
    <option value={o.value} disabled={o.disabled}>{o.label}</option>
  {/each}
</select>

<style>
  /* Copiado verbatim de Ajustes.svelte .sel (chevron embebido, paddings, focus) */
  .sel {
    font-size: 14px;
    font-family: var(--font-sans);
    color: var(--text);
    background-color: var(--bg);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888888' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 10px center;
    background-size: 12px;
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 8px 34px 8px 12px;
    outline: none;
    appearance: none;
    -webkit-appearance: none;
    cursor: pointer;
    transition: border-color .15s;
  }
  .sel:hover { border-color: var(--line-strong); }
  .sel:focus { border-color: var(--text-muted); }
</style>
