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

  // Refocus the word input after adding so fast entry works
  let wordInputEl: HTMLInputElement;
  function addWordAndRefocus() {
    addWord();
    wordInputEl?.focus();
  }
  function onAddKeyRefocus(e: KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); addWordAndRefocus(); }
  }

  // Refocus rFrom after adding a replacement
  let rFromEl: HTMLInputElement;
  function addReplacementAndRefocus() {
    addReplacement();
    rFromEl?.focus();
  }
  function onReplKeyRefocus(e: KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); addReplacementAndRefocus(); }
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

    <div class="card">
      <!-- Inline add row (always visible at the top of the card) -->
      <div class="add-row">
        <input
          bind:this={wordInputEl}
          class="ipt add-ipt"
          type="text"
          placeholder="Agregar palabra…"
          bind:value={draft}
          onkeydown={onAddKeyRefocus}
        />
        <button class="btn-primary" onclick={addWordAndRefocus} disabled={!draft.trim()}>
          Agregar
        </button>
      </div>

      <!-- Table or empty state -->
      {#if settings.dictionary.length === 0}
        <div class="empty-row">Sin palabras aún</div>
      {:else}
        <table class="words-table">
          <tbody>
            {#each settings.dictionary as word, idx (idx)}
              <tr class="word-row">
                <td class="word-cell">
                  {#if editingIdx === idx}
                    <input
                      class="ipt inline-edit"
                      type="text"
                      bind:value={editValue}
                      onkeydown={(e) => onEditKey(e, idx)}
                      onblur={() => commitEdit(idx)}
                    />
                  {:else}
                    <span class="word-text">{word}</span>
                  {/if}
                </td>
                <td class="action-cell">
                  <div class="row-actions">
                    <button
                      class="icon-btn"
                      onclick={() => startEdit(idx)}
                      title="Editar"
                      tabindex="0"
                    >
                      <!-- Pencil icon -->
                      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M9.5 2.5l2 2L4 12H2v-2L9.5 2.5z"/>
                      </svg>
                    </button>
                    <button
                      class="icon-btn"
                      onclick={() => removeWord(idx)}
                      title="Eliminar"
                      tabindex="0"
                    >
                      <!-- X icon -->
                      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                        <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </section>

  <!-- ── Replacements section ── -->
  <section class="section">
    <SectionLabel text="Reemplazos" />
    <p class="section-hint">
      Corrige términos o expande atajos en cada transcripción (ambos motores).
      Ej: «air table» → «Airtable», o «mi correo» → tu email. No distingue mayúsculas.
    </p>

    <div class="card">
      <!-- Inline add row -->
      <div class="repl-add-row">
        <input
          bind:this={rFromEl}
          class="ipt repl-ipt"
          type="text"
          placeholder="De…"
          bind:value={rFrom}
          onkeydown={onReplKeyRefocus}
        />
        <span class="arrow" aria-hidden="true">→</span>
        <input
          class="ipt repl-ipt"
          type="text"
          placeholder="A…"
          bind:value={rTo}
          onkeydown={onReplKeyRefocus}
        />
        <label class="rx-toggle" title="Tratar «De» como expresión regular">
          <input type="checkbox" bind:checked={rRegex} />
          <span>regex</span>
        </label>
        <button class="btn-primary" onclick={addReplacementAndRefocus} disabled={!rFrom.trim()}>
          Agregar
        </button>
      </div>

      <!-- Replacements table or empty state -->
      {#if (settings.replacements ?? []).length === 0}
        <div class="empty-row">Sin reemplazos aún</div>
      {:else}
        <table class="repl-table">
          <thead>
            <tr class="repl-head-row">
              <th class="col-from">De</th>
              <th class="col-arrow"></th>
              <th class="col-to">A</th>
              <th class="col-badge"></th>
              <th class="col-actions"></th>
            </tr>
          </thead>
          <tbody>
            {#each settings.replacements as r, idx (idx)}
              <tr class="repl-row">
                <td class="col-from">
                  <span class="code-cell">{r.from}</span>
                </td>
                <td class="col-arrow">
                  <span class="arrow" aria-hidden="true">→</span>
                </td>
                <td class="col-to">
                  <span class="code-cell muted">{r.to || "(vacío)"}</span>
                </td>
                <td class="col-badge">
                  {#if r.regex}<span class="rx-badge">regex</span>{/if}
                </td>
                <td class="col-actions">
                  <div class="row-actions">
                    <button
                      class="icon-btn"
                      onclick={() => removeReplacement(idx)}
                      title="Eliminar"
                      tabindex="0"
                    >
                      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                        <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </section>
</div>

<style>
  /* ── Root container ── */
  .screen {
    padding: var(--screen-top) var(--s10) var(--s10);
    display: flex;
    flex-direction: column;
    gap: var(--s8);
  }

  /* ── Page title ── */
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

  /* ── Card wrapper ── */
  .card {
    background: var(--surface);
    border-radius: var(--r-card);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
  }

  /* ── Inline add rows ── */
  .add-row {
    display: flex;
    align-items: center;
    gap: var(--s2);
    padding-bottom: var(--s3);
    border-bottom: 1px solid var(--line);
    margin-bottom: var(--s1);
  }

  .repl-add-row {
    display: flex;
    align-items: center;
    gap: var(--s2);
    padding-bottom: var(--s3);
    border-bottom: 1px solid var(--line);
    margin-bottom: var(--s1);
    flex-wrap: wrap;
  }

  /* ── Inputs ── */
  .ipt {
    font-size: 14px;
    font-family: var(--font-sans);
    color: var(--text);
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 8px 12px;
    outline: none;
    transition: border-color 0.15s;
  }
  .ipt::placeholder { color: var(--text-muted); }
  .ipt:focus { border-color: var(--text-muted); }

  .add-ipt { flex: 1; }
  .repl-ipt { flex: 1; min-width: 80px; }

  /* Inline edit inside a table row */
  .inline-edit {
    width: 100%;
    min-width: 100px;
    padding: 4px 8px;
    font-size: 14px;
  }

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
    transition: opacity 0.15s;
    flex-shrink: 0;
  }
  .btn-primary:hover:not(:disabled) { opacity: 0.85; }
  .btn-primary:disabled { opacity: 0.35; cursor: default; }

  /* ── Empty state row ── */
  .empty-row {
    font-size: 13px;
    color: var(--text-muted);
    padding: var(--s4) 0 var(--s2);
    text-align: center;
  }

  /* ── Words table ── */
  .words-table {
    width: 100%;
    border-collapse: collapse;
  }

  /* No per-row lines — hover background separates rows instead */
  .word-row {
    border-radius: var(--r-nav);
    transition: background 0.12s;
  }
  .word-row:hover {
    background: var(--inset);
  }
  /* Show actions on hover or keyboard focus-within */
  .word-row .row-actions {
    opacity: 0;
    transition: opacity 0.12s;
  }
  .word-row:hover .row-actions,
  .word-row:focus-within .row-actions {
    opacity: 1;
  }

  .word-cell {
    padding: 10px 8px 10px 8px;
    width: 100%;
    height: 42px;
  }

  .word-text {
    font-size: 14px;
    font-family: var(--font-sans);
    color: var(--text);
    line-height: 1.4;
  }

  .action-cell {
    padding: 10px 4px 10px 0;
    text-align: right;
    white-space: nowrap;
  }

  /* ── Replacements table ── */
  .repl-table {
    width: 100%;
    border-collapse: collapse;
  }

  /* Faint header underline only — no per-row lines */
  .repl-head-row {
    border-bottom: 1px solid var(--line);
  }

  .repl-table th {
    font-size: 12px;
    font-weight: 400;
    color: var(--text-muted);
    text-align: left;
    padding: 6px 8px 6px 0;
    white-space: nowrap;
  }

  .repl-row {
    border-radius: var(--r-nav);
    transition: background 0.12s;
  }
  .repl-row:hover {
    background: var(--inset);
  }
  /* Show actions on hover or keyboard focus-within */
  .repl-row .row-actions {
    opacity: 0;
    transition: opacity 0.12s;
  }
  .repl-row:hover .row-actions,
  .repl-row:focus-within .row-actions {
    opacity: 1;
  }

  .repl-row td {
    padding: 10px 8px 10px 0;
    vertical-align: middle;
    height: 42px;
  }

  .col-from   { width: 34%; }
  .col-arrow  { width: 24px; text-align: center; padding-left: 0; padding-right: 0; }
  .col-to     { width: 34%; }
  .col-badge  { width: 52px; }
  .col-actions { width: 36px; text-align: right; padding-right: 4px; }

  .code-cell {
    font-size: 13px;
    font-family: var(--font-sans);
    color: var(--text);
    background: var(--bg);
    border-radius: var(--r-nav);
    padding: 2px 8px;
    display: inline-block;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .code-cell.muted { color: var(--text-muted); }

  /* ── Arrow separator ── */
  .arrow {
    color: var(--text-muted);
    font-size: 14px;
    flex-shrink: 0;
  }

  /* ── Regex badge ── */
  .rx-badge {
    font-size: 11px;
    color: var(--text-muted);
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 1px 6px;
    font-family: var(--font-sans);
    white-space: nowrap;
  }

  /* ── Regex toggle (in add row) ── */
  .rx-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-muted);
    cursor: pointer;
    white-space: nowrap;
    font-family: var(--font-sans);
    flex-shrink: 0;
  }
  .rx-toggle input[type="checkbox"] {
    accent-color: var(--nav-active-bg);
    cursor: pointer;
  }

  /* ── Row actions container ── */
  .row-actions {
    display: inline-flex;
    align-items: center;
    gap: var(--s1);
  }

  /* ── Icon buttons (edit + delete) ── */
  .icon-btn {
    width: 28px;
    height: 28px;
    background: none;
    border: none;
    border-radius: var(--r-nav);
    color: var(--text-muted);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: background 0.12s, color 0.12s;
    flex-shrink: 0;
  }
  .icon-btn:hover {
    background: var(--line);
    color: var(--text);
  }
  .icon-btn:focus-visible {
    outline: 2px solid var(--text-muted);
    outline-offset: 1px;
  }
</style>
