<script lang="ts">
  import type { Settings, Replacement } from "$lib/types";
  import SectionLabel from "$lib/components/ui/SectionLabel.svelte";

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

  // ── Replacements / snippets (deterministic post-transcription) ──
  let rFrom = $state("");
  let rTo = $state("");
  let rRegex = $state(false);

  function addReplacement() {
    const from = rFrom.trim();
    if (!from) return;
    const next: Replacement = { from, to: rTo, regex: rRegex };
    settings.replacements = [...(settings.replacements ?? []), next];
    rFrom = ""; rTo = ""; rRegex = false;
    onSave();
  }

  function removeReplacement(idx: number) {
    settings.replacements = (settings.replacements ?? []).filter((_, i) => i !== idx);
    onSave();
  }

  function onReplKey(e: KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); addReplacement(); }
  }

  function onAddKey(e: KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); addWord(); }
  }
  function onEditKey(e: KeyboardEvent, idx: number) {
    if (e.key === "Enter") { e.preventDefault(); commitEdit(idx); }
    if (e.key === "Escape") { editingIdx = null; }
  }
</script>

<div class="screen">
  <h1 class="page-title">Diccionario</h1>

  <!-- ── Words section ── -->
  <section class="section">
    <SectionLabel text="Palabras" />
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
      <button class="btn-primary" onclick={addWord} disabled={!draft.trim()}>Añadir</button>
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

  <!-- ── Replacements section ── -->
  <section class="section">
    <SectionLabel text="Reemplazos" />
    <p class="section-hint">
      Corrige términos o expande atajos en cada transcripción (ambos motores).
      Ej: «air table» → «Airtable», o «mi correo» → tu email. No distingue mayúsculas.
    </p>

    <div class="repl-add">
      <input class="ipt" type="text" placeholder="Buscar (lo que se dice)" bind:value={rFrom} onkeydown={onReplKey} />
      <span class="arrow">→</span>
      <input class="ipt" type="text" placeholder="Reemplazar por" bind:value={rTo} onkeydown={onReplKey} />
      <label class="rx-toggle" title="Tratar «Buscar» como expresión regular">
        <input type="checkbox" bind:checked={rRegex} /> regex
      </label>
      <button class="btn-primary" onclick={addReplacement} disabled={!rFrom.trim()}>Añadir</button>
    </div>

    {#if (settings.replacements ?? []).length === 0}
      <div class="empty">
        <p>Sin reemplazos</p>
        <span>Agrega reglas para corregir nombres o expandir atajos al dictar.</span>
      </div>
    {:else}
      <ul class="repl-list">
        {#each settings.replacements as r, idx (idx)}
          <li class="repl-card">
            <code class="repl-from">{r.from}</code>
            <span class="arrow">→</span>
            <code class="repl-to">{r.to || "(vacío)"}</code>
            {#if r.regex}<span class="rx-badge">regex</span>{/if}
            <button class="icon-btn del" onclick={() => removeReplacement(idx)} title="Eliminar">
              <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
              </svg>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  /* ── Root container: 81px top offset matches Home / Transcripciones ── */
  .screen {
    padding: 81px var(--s10) var(--s10);
    display: flex;
    flex-direction: column;
    gap: var(--s8);
  }

  /* ── Page title: serif, matches system ── */
  .page-title {
    font-family: var(--font-serif);
    font-size: 24px;
    font-weight: 400;
    color: var(--text);
  }

  /* ── Section grouping ── */
  .section {
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }
  .section-hint {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  /* ── Add row ── */
  .add-row {
    display: flex;
    align-items: center;
    gap: var(--s3);
  }

  /* ── Inputs ── */
  .ipt {
    font-size: 14px;
    font-family: var(--font-sans);
    color: var(--text);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 8px 12px;
    outline: none;
    flex: 1;
    transition: border-color .15s;
  }
  .ipt::placeholder { color: var(--text-muted); }
  .ipt:focus { border-color: var(--text-muted); }

  /* ── Primary button ── */
  .btn-primary {
    background: var(--nav-active-bg);
    color: var(--nav-active-ink);
    border: none;
    border-radius: var(--r-nav);
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 600;
    font-family: var(--font-sans);
    cursor: pointer;
    white-space: nowrap;
    transition: opacity .15s;
  }
  .btn-primary:hover:not(:disabled) { opacity: .9; }
  .btn-primary:disabled { opacity: .35; cursor: default; }

  /* ── Chips (dictionary words) ── */
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s2);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 1px 4px 1px 10px;
  }
  .chip-word {
    background: none;
    border: none;
    padding: 0;
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
    font-family: var(--font-sans);
  }
  .chip-word:hover { color: var(--text); }
  .chip-edit {
    border-radius: var(--r-nav);
    padding: 4px 12px;
    flex: 0 1 auto;
    min-width: 120px;
  }

  /* ── Icon buttons (delete inside chips / replacement rows) ── */
  .icon-btn {
    width: 18px; height: 18px; background: none; border: none;
    border-radius: 50%; color: var(--text-muted); cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: background .12s, color .12s;
  }
  .icon-btn svg { width: 8px; height: 8px; }
  /* Destructive: monochrome, no red */
  .icon-btn.del:hover { background: rgba(127,127,127,0.10); color: var(--text); }

  /* ── Empty state ── */
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px var(--s6);
    gap: 6px;
    text-align: center;
    background: var(--surface);
    border-radius: var(--r-card);
  }
  .empty p { font-size: 14px; font-weight: 400; color: var(--text); }
  .empty span { font-size: 12px; color: var(--text-muted); line-height: 1.5; }

  /* ── Replacements ── */
  .repl-add {
    display: flex;
    align-items: center;
    gap: var(--s2);
    flex-wrap: wrap;
  }
  .arrow { color: var(--text-muted); font-size: 14px; }
  .rx-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
    white-space: nowrap;
    font-family: var(--font-sans);
  }
  .repl-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }
  .repl-card {
    display: flex;
    align-items: center;
    gap: var(--s2);
    background: var(--surface);
    border-radius: var(--r-card);
    padding: var(--s4);
  }
  .repl-from, .repl-to {
    font-size: 12px;
    color: var(--text);
    background: var(--bg);
    border-radius: var(--r-nav);
    padding: 2px 7px;
    max-width: 38%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-sans);
  }
  .repl-to { color: var(--text-muted); }
  .rx-badge {
    font-size: 11px;
    color: var(--text-muted);
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 1px 6px;
    font-family: var(--font-sans);
  }
  .repl-card .icon-btn { margin-left: auto; }
</style>
