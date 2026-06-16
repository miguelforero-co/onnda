<script lang="ts">
  // Extracted verbatim from +page.svelte (.toggle/.knob, 36×20, knob 16 — do not alter).
  let {
    checked = $bindable(false),
    label = "",
    id,
    onchange,
  }: {
    checked?: boolean;
    label?: string;
    id: string;
    onchange?: () => void;
  } = $props();
</script>

<label class="row" for={id}>
  {#if label}<span class="row-label">{label}</span>{/if}
  <div class="toggle" class:on={checked}>
    <input {id} type="checkbox" bind:checked {onchange} />
    <span class="knob"></span>
  </div>
</label>

<style>
  .row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 14px; gap: 12px; min-height: 42px;
    cursor: pointer;
  }
  .row-label { font-size: 14px; font-weight: 450; color: var(--text); }

  .toggle {
    position: relative; width: 36px; height: 20px;
    background: var(--line); border: 1px solid var(--line);
    border-radius: 10px;
    flex-shrink: 0; transition: background .18s, border-color .18s; cursor: pointer;
  }
  .toggle.on {
    background: var(--nav-active-bg);
    border-color: var(--nav-active-bg);
  }
  @media (prefers-contrast: more) { .toggle.on { background: var(--nav-active-bg); } }
  .toggle input { display: none; }
  .knob {
    position: absolute; top: 2px; left: 2px;
    width: 16px; height: 16px;
    background: var(--text-muted);
    border-radius: 50%;
    transition: transform .18s cubic-bezier(.4,0,.2,1), background .18s;
  }
  .toggle.on .knob {
    transform: translateX(16px);
    background: var(--nav-active-ink);
  }
</style>
