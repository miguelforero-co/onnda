<script lang="ts">
  import Icon, { type IconName } from "./Icon.svelte";
  let {
    icon,
    label,
    active = false,
    onclick,
  }: { icon: IconName; label: string; active?: boolean; onclick: () => void } = $props();
</script>

<button class="nav-item" class:active {onclick}>
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
    transition: background .12s, color .12s;
  }
  .nav-item:hover { background: rgba(127,127,127,0.10); }
  .nav-item.active { background: var(--nav-active-bg); color: var(--nav-active-ink); }
  .nav-item.active:hover { background: var(--nav-active-bg); }
  .ic { display: flex; flex-shrink: 0; }
  .label {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 400;
  }
</style>
