<script lang="ts">
  import type { View } from "$lib/types";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Wordmark from "$lib/components/ui/Wordmark.svelte";
  import NavItem from "$lib/components/ui/NavItem.svelte";
  import type { IconName } from "$lib/components/ui/Icon.svelte";

  // The whole rail is a window drag handle (title bar hidden). Nav clicks pass
  // through (we skip the drag when the target is a .nav-item).
  function railDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest(".nav-item")) return;
    getCurrentWindow().startDragging().catch(() => {});
  }

  let { view = $bindable<View>("home") }: { view?: View } = $props();

  const items: { id: View; label: string; icon: IconName }[] = [
    { id: "home",            label: "Home",            icon: "home" },
    { id: "transcripciones", label: "Transcriptions",  icon: "list" },
    { id: "importar",        label: "Transcribe Files", icon: "page-plus" },
    { id: "diccionario",     label: "Dictionary",      icon: "book" },
    { id: "ajustes",         label: "Settings",        icon: "frame-tool" },
  ];

  // Indicador deslizante: una sola caja negra que se mueve al item activo.
  let els = $state<(HTMLButtonElement | undefined)[]>([]);
  let indY = $state(0);
  let indH = $state(0);
  let ready = $state(false); // evita animar el primer posicionamiento

  $effect(() => {
    const i = items.findIndex((it) => it.id === view);
    const el = els[i];
    if (!el) return;
    indY = el.offsetTop;
    indH = el.offsetHeight;
    if (!ready) requestAnimationFrame(() => (ready = true));
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<aside class="sidebar" onmousedown={railDrag}>
  <div class="brand"><Wordmark /></div>
  <nav class="nav">
    <div
      class="nav-indicator"
      class:ready
      style="transform: translateY({indY}px); height: {indH}px;"
    ></div>
    {#each items as it, i}
      <NavItem
        icon={it.icon}
        label={it.label}
        active={view === it.id}
        onclick={() => (view = it.id)}
        bind:el={els[i]}
      />
    {/each}
  </nav>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-w);
    flex-shrink: 0;
    background: var(--bg);
    border-radius: var(--r-card);
    display: flex;
    flex-direction: column;
    /* top clears the floating macOS traffic lights; left 24 per Figma */
    padding: 51px 24px 24px;
    gap: var(--s6);   /* 24px between brand and nav */
  }
  .brand { display: flex; }
  .nav { position: relative; display: flex; flex-direction: column; gap: var(--s4); } /* 16px */
  /* Caja negra de selección que se desliza entre items */
  .nav-indicator {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    z-index: 0;
    background: var(--nav-active-bg);
    border-radius: var(--r-nav);
    pointer-events: none;
  }
  .nav-indicator.ready {
    transition:
      transform 0.32s cubic-bezier(0.4, 0, 0.2, 1),
      height 0.32s cubic-bezier(0.4, 0, 0.2, 1);
  }
  @media (prefers-reduced-motion: reduce) {
    .nav-indicator.ready { transition: none; }
  }
</style>
