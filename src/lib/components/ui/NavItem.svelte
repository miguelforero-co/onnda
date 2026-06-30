<script lang="ts">
  import Icon, { type IconName } from "./Icon.svelte";
  let {
    icon,
    label,
    active = false,
    onclick,
    el = $bindable<HTMLButtonElement | undefined>(),
  }: {
    icon: IconName;
    label: string;
    active?: boolean;
    onclick: () => void;
    el?: HTMLButtonElement;
  } = $props();
</script>

<button class="nav-item" class:active {onclick} bind:this={el}>
  <span class="ic"><Icon name={icon} size={24} /></span>
  <span class="label">{label}</span>
</button>

<style>
  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--s3);            /* 12px */
    width: 100%;
    padding: var(--s2) var(--s3); /* 8 / 12 */
    border: none;
    border-radius: var(--r-nav);
    background: transparent;
    color: var(--nav-ink);
    cursor: pointer;
    text-align: left;
    /* el fondo activo lo dibuja el indicador deslizante del Sidebar (detrás);
       aquí solo animamos el color del texto/ícono */
    position: relative;
    z-index: 1;
    transition: background .12s, color .18s;
  }
  .nav-item:hover { background: rgba(127,127,127,0.10); }
  .nav-item.active { color: var(--nav-active-ink); }
  /* sin tinte de hover sobre el activo: el indicador negro va detrás */
  .nav-item.active:hover { background: transparent; }
  .ic { display: flex; flex-shrink: 0; }
  .label {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 400;
  }
</style>
