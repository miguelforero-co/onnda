<script lang="ts">
  // Extracted verbatim from +page.svelte .perm-row block (L204-228, 626-646).
  // Granted → --blue dot + "Concedido"; else muted dot + "Abrir ajustes" link-btn.
  let {
    label,
    description = "",
    granted = false,
    onOpen,
  }: {
    label: string;
    description?: string;
    granted?: boolean;
    onOpen?: () => void;
  } = $props();
</script>

<div class="perm-row" class:granted>
  <div class="perm-dot" class:granted></div>
  <div class="perm-info">
    <strong>{label}</strong>
    {#if description}<span>{description}</span>{/if}
  </div>
  {#if granted}
    <span class="perm-status ok">Concedido</span>
  {:else}
    <button class="link-btn" onclick={() => onOpen?.()}>Abrir ajustes</button>
  {/if}
</div>

<style>
  .perm-row {
    display: flex; align-items: center; gap: 13px;
    padding: 14px 0;
    border-bottom: 1px solid var(--line);
  }
  .perm-row:first-child { border-top: 1px solid var(--line); }

  .perm-dot {
    width: 7px; height: 7px; border-radius: 50%;
    background: rgba(0,0,0,.15); flex-shrink: 0;
    transition: background .3s;
  }
  .perm-dot.granted { background: var(--blue); }

  .perm-info { flex: 1; display: flex; flex-direction: column; gap: 1px; }
  .perm-info strong { font-size: 13px; font-weight: 600; color: var(--text); }
  .perm-info span { font-size: 11px; color: var(--faint); }

  .perm-status { font-size: 11px; font-weight: 450; color: var(--faint); }
  .perm-status.ok { color: var(--blue); }

  .link-btn {
    background: none; border: none; padding: 4px 0;
    font-size: 12px; font-weight: 450; color: var(--coral);
    cursor: pointer; text-decoration: none;
  }
  .link-btn:hover { opacity: .75; }
</style>
