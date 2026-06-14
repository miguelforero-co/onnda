<script lang="ts">
  import type { Settings } from "$lib/types";

  // Prop contract from 01-03 (Diccionario stub). D-19/D-20/D-21: item-list editor.
  let {
    settings,
    onSave,
  }: {
    settings: Settings;
    onSave: () => void;
  } = $props();

  let draft = $state("");
  // Inline-edit state: index being edited + its working value.
  let editingIdx = $state<number | null>(null);
  let editValue = $state("");

  // D-20 CRITICAL: keep custom_words synced so Whisper's initial_prompt / correct_words
  // keep working. The Rust side still reads custom_words. Then persist via onSave().
  function syncAndSave() {
    settings.custom_words = settings.dictionary.join(", ");
    onSave();
  }

  function addWord() {
    const w = draft.trim();
    if (!w) return;
    // No duplicates (case-insensitive).
    if (settings.dictionary.some((d) => d.toLowerCase() === w.toLowerCase())) {
      draft = "";
      return;
    }
    settings.dictionary = [...settings.dictionary, w];
    draft = "";
    syncAndSave();
  }

  function removeWord(idx: number) {
    settings.dictionary = settings.dictionary.filter((_, i) => i !== idx);
    if (editingIdx === idx) editingIdx = null;
    syncAndSave();
  }

  function startEdit(idx: number) {
    editingIdx = idx;
    editValue = settings.dictionary[idx];
  }

  function commitEdit(idx: number) {
    const w = editValue.trim();
    if (!w) {
      // Empty edit removes the item.
      removeWord(idx);
      editingIdx = null;
      return;
    }
    // Reject a duplicate of another entry; otherwise commit.
    const dup = settings.dictionary.some((d, i) => i !== idx && d.toLowerCase() === w.toLowerCase());
    if (!dup && w !== settings.dictionary[idx]) {
      const next = [...settings.dictionary];
      next[idx] = w;
      settings.dictionary = next;
      syncAndSave();
    }
    editingIdx = null;
  }

  function onAddKey(e: KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); addWord(); }
  }
  function onEditKey(e: KeyboardEvent, idx: number) {
    if (e.key === "Enter") { e.preventDefault(); commitEdit(idx); }
    if (e.key === "Escape") { editingIdx = null; }
  }
</script>

<h1 class="page-title">Diccionario</h1>

<section>
  <p class="section-hint">
    Agrega palabras o nombres propios para que Whisper los reconozca mejor.
  </p>

  <!-- ── Add input ── -->
  <div class="add-row">
    <input
      class="ipt"
      type="text"
      placeholder="Escribe una palabra y pulsa Enter"
      bind:value={draft}
      onkeydown={onAddKey}
    />
    <button class="link-btn" onclick={addWord} disabled={!draft.trim()}>+ Agregar palabra</button>
  </div>

  <!-- ── Item list / empty state ── -->
  {#if settings.dictionary.length === 0}
    <div class="empty">
      <p>Tu diccionario está vacío</p>
      <span>Agrega palabras o nombres propios para que Whisper los reconozca mejor.</span>
    </div>
  {:else}
    <div class="chips">
      {#each settings.dictionary as word, idx (idx)}
        {#if editingIdx === idx}
          <input
            class="ipt chip-edit"
            type="text"
            bind:value={editValue}
            onkeydown={(e) => onEditKey(e, idx)}
            onblur={() => commitEdit(idx)}
          />
        {:else}
          <span class="chip">
            <button class="chip-word" onclick={() => startEdit(idx)} title="Editar">{word}</button>
            <button class="icon-btn del" onclick={() => removeWord(idx)} title="Eliminar">
              <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
              </svg>
            </button>
          </span>
        {/if}
      {/each}
    </div>
  {/if}
</section>

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }

  section { margin-top: 22px; display: flex; flex-direction: column; gap: 12px; }
  .section-hint { font-size: 11px; color: var(--faint); padding: 0 3px; line-height: 1.5; }

  .add-row { display: flex; align-items: center; gap: 12px; }

  .ipt {
    font-size: 12.5px; color: var(--text); background: var(--bg-2);
    border: 1px solid var(--line); border-radius: var(--r-sm);
    padding: 7px 10px; outline: none; flex: 1;
    transition: border-color .15s, box-shadow .15s, background .15s;
  }
  .ipt::placeholder { color: var(--faint); }
  .ipt:focus {
    background: var(--elev-1); border-color: transparent;
    box-shadow: 0 0 0 1px var(--iris-4), 0 0 0 4px rgba(127,200,255,0.16);
  }

  .link-btn {
    background: none; border: none; padding: 4px 0;
    font-size: 12px; font-weight: 450; color: var(--coral);
    cursor: pointer; text-decoration: none; white-space: nowrap;
  }
  .link-btn:hover { opacity: .75; }
  .link-btn:disabled { color: var(--faint); cursor: default; opacity: .7; }

  /* ── Chips ── */
  .chips { display: flex; flex-wrap: wrap; gap: 8px; }
  .chip {
    display: inline-flex; align-items: center; gap: 4px;
    background: var(--glass-fill); border: 1px solid var(--line);
    box-shadow: var(--glass-edge);
    border-radius: 16px; padding: 4px 6px 4px 12px;
  }
  .chip-word {
    background: none; border: none; padding: 0;
    font-size: 12.5px; color: var(--text); cursor: pointer;
  }
  .chip-edit { border-radius: 16px; padding: 4px 12px; flex: 0 1 auto; min-width: 120px; }

  .icon-btn {
    width: 18px; height: 18px; background: none; border: none;
    border-radius: 50%; color: var(--faint); cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: background .12s, color .12s;
  }
  .icon-btn svg { width: 8px; height: 8px; }
  .icon-btn.del:hover { background: rgba(255,106,61,.14); color: var(--coral); }

  /* ── Empty state ── */
  .empty {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; padding: 48px 20px; gap: 6px; text-align: center;
  }
  .empty p { font-size: 14px; font-weight: 450; color: var(--muted); }
  .empty span { font-size: 12px; color: var(--faint); line-height: 1.5; }
</style>
