<script lang="ts">
  // Extracted verbatim from +page.svelte .perm-row block (L204-228, 626-646).
  // Granted → --dot-on dot + "Concedido"; else muted dot + "Abrir ajustes" link.
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
    <span class="perm-status ok">Granted</span>
  {:else}
    <button class="link-btn" onclick={() => onOpen?.()}>Open settings</button>
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
    background: var(--text-muted); flex-shrink: 0;
    transition: background .3s;
  }
  .perm-dot.granted { background: var(--dot-on); }

  .perm-info { flex: 1; display: flex; flex-direction: column; gap: 1px; }
  .perm-info strong { font-size: 14px; font-weight: 600; color: var(--text); }
  .perm-info span { font-size: 12px; color: var(--text-muted); }

  .perm-status { font-size: 12px; font-weight: 450; color: var(--text-muted); }
  .perm-status.ok { color: var(--dot-on); }

  .link-btn {
    background: none; border: none; padding: 4px 0;
    font-size: 12px; font-weight: 450; color: var(--text-muted);
    cursor: pointer; text-decoration: none; font-family: inherit;
    transition: color .15s;
  }
  .link-btn:hover { color: var(--text); }
</style>
