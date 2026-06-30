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
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<aside class="sidebar" onmousedown={railDrag}>
  <div class="brand"><Wordmark /></div>
  <nav class="nav">
    {#each items as it}
      <NavItem
        icon={it.icon}
        label={it.label}
        active={view === it.id}
        onclick={() => (view = it.id)}
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
  .nav { display: flex; flex-direction: column; gap: var(--s4); } /* 16px */
</style>
