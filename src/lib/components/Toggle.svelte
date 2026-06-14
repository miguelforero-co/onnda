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
  .row-label { font-size: 13px; font-weight: 450; color: var(--text); }

  .toggle {
    position: relative; width: 36px; height: 20px;
    background: rgba(0,0,0,0.14); border-radius: 10px;
    flex-shrink: 0; transition: background .18s; cursor: pointer;
  }
  /* On = energized: the film ramp (warm->cool) over coral via soft-light, so the
     "on" track reads like the wave going active. Falls back to solid coral. */
  .toggle.on {
    background: var(--iris-sheen), var(--coral);
    background-size: 160% 100%, auto;
    background-blend-mode: soft-light, normal;
  }
  @media (prefers-contrast: more) { .toggle.on { background: var(--coral); } }
  .toggle input { display: none; }
  .knob {
    position: absolute; top: 2px; left: 2px;
    width: 16px; height: 16px; background: #fff; border-radius: 50%;
    box-shadow: 0 1px 4px rgba(0,0,0,.18), var(--emboss-hi);
    transition: transform .18s cubic-bezier(.4,0,.2,1);
  }
  .toggle.on .knob { transform: translateX(16px); }
</style>
