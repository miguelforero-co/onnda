<script lang="ts">
  import type { HistoryEntry } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  // Pure unified list: source filter + play + delete + copy.
  // File upload lives in the Importar section now.
  let {
    history,
    onRefresh,
    onSettingsChanged,
  }: {
    history: HistoryEntry[];
    onRefresh: () => void;
    onSettingsChanged?: () => void;
  } = $props();

  // ── Source filter ──
  let filter = $state<"all" | "dictation" | "file">("all");
  const filtered = $derived(
    filter === "all" ? history : history.filter((e) => e.source === filter)
  );

  // ── Storage usage (bytes from disk: history.json + recordings) ──
  let storageBytes = $state(0);
  async function refreshStorage() {
    storageBytes = await invoke<number>("get_storage_usage").catch(() => 0);
  }
  // Format: < 1 MB → "734 KB en uso", else "1,2 MB en uso" (one decimal, es locale).
  function formatBytes(b: number): string {
    if (b < 1024 * 1024) {
      const kb = Math.round(b / 1024);
      return `${kb.toLocaleString("es")} KB en uso`;
    }
    const mb = (b / (1024 * 1024)).toLocaleString("es", { minimumFractionDigits: 1, maximumFractionDigits: 1 });
    return `${mb} MB en uso`;
  }

  // Re-pull history + storage on mount so both are current when navigated to.
  onMount(() => { onRefresh(); refreshStorage(); });

  async function clearAll() {
    if (!window.confirm("¿Borrar todas las transcripciones y sus audios? Se eliminan del computador y no se puede deshacer.")) return;
    await invoke("clear_history");
    onRefresh();
    refreshStorage();
  }

  // ── Playback (lifted from pre-refactor +page.svelte) ──
  let playingId = $state<string | null>(null);
  let audioEl: HTMLAudioElement | null = null;

  async function playAudio(entry: HistoryEntry) {
    if (!entry.audio_filename) return;
    if (playingId === entry.id) { audioEl?.pause(); playingId = null; return; }
    audioEl?.pause(); playingId = entry.id;
    try {
      const b64: string = await invoke("get_recording_audio", { filename: entry.audio_filename });
      if (!b64) { playingId = null; return; }
      if (!audioEl) audioEl = new Audio();
      audioEl.src = `data:audio/wav;base64,${b64}`;
      audioEl.onended = () => { playingId = null; };
      await audioEl.play();
    } catch { playingId = null; }
  }

  async function deleteEntry(id: string) {
    await invoke("delete_history_entry", { id });
    onRefresh();
    refreshStorage();
  }

  async function copyEntry(text: string) {
    try { await navigator.clipboard.writeText(text); } catch { /* ignore */ }
  }

  // ── Edit + auto-learn from corrections (Phase 3) ──
  let editingId = $state<string | null>(null);
  let editText = $state("");
  let learnNote = $state<string | null>(null);
  let noteTimer: ReturnType<typeof setTimeout> | null = null;

  function startEdit(e: HistoryEntry) {
    editingId = e.id;
    editText = e.text;
  }
  function cancelEdit() { editingId = null; }

  function showNote(msg: string) {
    learnNote = msg;
    if (noteTimer) clearTimeout(noteTimer);
    noteTimer = setTimeout(() => { learnNote = null; }, 6000);
  }

  async function saveEdit(id: string) {
    const newText = editText.trim();
    editingId = null;
    if (!newText) return;
    const outcome = await invoke<{ learned: [string, string][]; promoted: [string, string][] }>(
      "correct_history_entry", { id, newText }
    ).catch(() => null);
    onRefresh();
    onSettingsChanged?.();
    if (outcome && outcome.promoted.length > 0) {
      const [from, to] = outcome.promoted[0];
      showNote(`Aprendido: «${from}» → «${to}». Se corregirá solo de ahora en adelante.`);
    }
  }

  function onEditKey(e: KeyboardEvent, id: string) {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) { e.preventDefault(); saveEdit(id); }
    if (e.key === "Escape") { e.preventDefault(); cancelEdit(); }
  }

  function fmtTime(ms: number) {
    const d = new Date(ms), now = new Date();
    return now.toDateString() === d.toDateString()
      ? d.toLocaleTimeString("es", { hour: "2-digit", minute: "2-digit" })
      : d.toLocaleDateString("es", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
  }
  function fmtDur(s: number) { return s >= 1 ? `${s < 60 ? Math.round(s) + "s" : Math.round(s / 60) + "m"}` : ""; }

  onDestroy(() => audioEl?.pause());
</script>

<div class="screen">
  <div class="head">
    <h1 class="page-title">Transcripciones</h1>
  </div>

  <div class="toolbar">
    <div class="seg" role="tablist">
      <button class="seg-btn" class:on={filter === "all"} onclick={() => (filter = "all")}>Todas</button>
      <button class="seg-btn" class:on={filter === "dictation"} onclick={() => (filter = "dictation")}>Dictado</button>
      <button class="seg-btn" class:on={filter === "file"} onclick={() => (filter = "file")}>Archivo</button>
    </div>
    <div class="storage-row">
      <span class="storage-metric">{formatBytes(storageBytes)}</span>
      {#if history.length > 0}
        <button class="clear-btn" onclick={clearAll}>Borrar todo</button>
      {/if}
    </div>
  </div>

  {#if learnNote}
    <div class="learn-note">✓ {learnNote}</div>
  {/if}

  {#if filtered.length === 0}
    <div class="empty">
      <p>Sin transcripciones aún</p>
      <span>Presiona tu atajo para dictar, o sube un archivo de audio.</span>
    </div>
  {:else}
    <div class="hist-list">
      {#each filtered as e (e.id)}
        <div class="hist-card">
          <div class="hist-meta">
            <span class="hist-time">{fmtTime(e.timestamp_ms)}</span>
            {#if e.duration_secs >= 1}<span class="hist-chip">{fmtDur(e.duration_secs)}</span>{/if}
            {#if e.source === "file" && e.original_filename}<span class="hist-chip hist-file">{e.original_filename}</span>{/if}
            <div class="hist-actions">
              <button class="icon-btn" onclick={() => startEdit(e)} title="Editar / corregir">
                <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M6.8 1.4l1.8 1.8-5 5L1.4 8.6 1.8 6.4z"/>
                </svg>
              </button>
              <button class="icon-btn" onclick={() => copyEntry(e.text)} title="Copiar">
                <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="2.5" y="2.5" width="5.5" height="5.5" rx="1"/><path d="M5.5 2.5V1.5a1 1 0 0 0-1-1H1.5a1 1 0 0 0-1 1v3a1 1 0 0 0 1 1h1"/>
                </svg>
              </button>
              {#if e.audio_filename}
                <button class="icon-btn" class:active={playingId === e.id} onclick={() => playAudio(e)} title="Reproducir">
                  {#if playingId === e.id}
                    <svg viewBox="0 0 10 10" fill="currentColor"><rect x="0" y="0" width="3.5" height="10" rx="1"/><rect x="5.5" y="0" width="3.5" height="10" rx="1"/></svg>
                  {:else}
                    <svg viewBox="0 0 10 10" fill="currentColor"><path d="M1 0.5l8 4.5-8 4.5z"/></svg>
                  {/if}
                </button>
              {/if}
              <button class="icon-btn del" onclick={() => deleteEntry(e.id)} title="Eliminar">
                <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                  <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
                </svg>
              </button>
            </div>
          </div>
          {#if editingId === e.id}
            <div class="edit-box">
              <!-- svelte-ignore a11y_autofocus -->
              <textarea
                class="edit-area"
                bind:value={editText}
                onkeydown={(ev) => onEditKey(ev, e.id)}
                autofocus
              ></textarea>
              <div class="edit-actions">
                <button class="link-btn" onclick={() => saveEdit(e.id)}>Guardar</button>
                <button class="link-btn text-muted" onclick={cancelEdit}>Cancelar</button>
                <span class="edit-hint">⌘↵ guardar · Esc cancelar</span>
              </div>
            </div>
          {:else}
            <p class="hist-text">{e.text}</p>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* ── Root container: top offset matches Home (81px) ── */
  .screen {
    padding: 81px var(--s10) var(--s10);
    display: flex;
    flex-direction: column;
  }

  /* ── Page title: serif, matches Home greeting ── */
  .head { display: flex; align-items: center; justify-content: space-between; gap: var(--s3); }
  .page-title {
    font-family: var(--font-serif);
    font-size: 24px;
    font-weight: 400;
    color: var(--text);
  }

  /* ── Toolbar / filter ── */
  .toolbar { display: flex; align-items: center; gap: var(--s3); margin-top: var(--s6); }

  /* Storage metric + subtle "Borrar todo" tag, pushed right */
  .storage-row { display: flex; align-items: center; gap: var(--s3); margin-left: auto; }
  .storage-metric { font-size: 12px; color: var(--text-muted); }

  /* "Borrar todo" — subtle monochrome tag, no red */
  .clear-btn {
    background: transparent;
    border: 1px solid var(--line);
    color: var(--text-muted);
    border-radius: var(--r-nav);
    padding: 4px 10px;
    font-size: 13px;
    font-family: var(--font-sans);
    line-height: 1;
    cursor: pointer;
    transition: background .12s, color .12s;
  }
  .clear-btn:hover {
    background: var(--surface);
    color: var(--text);
  }

  /* ── Link-style buttons (Guardar / Cancelar inside edit) ── */
  .link-btn {
    background: none; border: none; padding: 4px 0;
    font-size: 14px; font-weight: 400; color: var(--text-muted);
    cursor: pointer; text-decoration: none;
    font-family: var(--font-sans);
  }
  .link-btn:hover { opacity: .75; }
  .link-btn.text-muted { color: var(--text-muted); }

  /* ── Segmented control (onnda theme-selector style) ── */
  .seg {
    display: inline-flex;
    padding: 2px;
    border-radius: var(--r-nav);
    background: var(--surface);
  }
  .seg-btn {
    background: none; border: none; cursor: pointer;
    font-size: 13px; font-weight: 400; color: var(--text-muted);
    padding: 6px 12px; border-radius: 6px;
    transition: background .12s, color .12s;
    font-family: var(--font-sans);
  }
  .seg-btn:hover { color: var(--text); }
  .seg-btn.on {
    background: var(--nav-active-bg);
    color: var(--nav-active-ink);
  }

  /* ── History list: cards stacked vertically ── */
  .empty {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; padding: 60px var(--s6); gap: 6px; text-align: center;
  }
  .empty p { font-size: 14px; font-weight: 400; color: var(--text); }
  .empty span { font-size: 12px; color: var(--text-muted); line-height: 1.5; }

  .hist-list {
    display: flex;
    flex-direction: column;
    gap: var(--s3);
    margin-top: var(--s4);
  }

  /* Each history entry is a card */
  .hist-card {
    background: var(--surface);
    border-radius: var(--r-card);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }

  .hist-meta { display: flex; align-items: center; gap: 6px; }
  .hist-time { font-size: 12px; color: var(--text-muted); }

  /* Chips: bordered, no background fill */
  .hist-chip {
    font-size: 12px;
    color: var(--text-muted);
    background: transparent;
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 1px 8px;
  }
  .hist-file {
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hist-actions { display: flex; gap: 2px; margin-left: auto; }

  .icon-btn {
    width: 22px; height: 22px; background: none; border: none;
    border-radius: 5px; color: var(--text-muted); cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: background .12s, color .12s;
  }
  .icon-btn svg { width: 9px; height: 9px; }
  .icon-btn:hover { background: rgba(127,127,127,0.10); color: var(--text); }
  /* Playing state uses muted green accent */
  .icon-btn.active { color: var(--dot-on); }
  /* Delete icon: muted, no red on hover */
  .icon-btn.del { color: var(--text-muted); }
  .icon-btn.del:hover { background: rgba(127,127,127,0.10); color: var(--text); }

  .hist-text {
    font-size: 14px; color: var(--text); line-height: 1.55;
    word-break: break-word; white-space: pre-wrap;
    font-family: var(--font-sans);
  }

  /* ── Inline edit (auto-learn) ── */
  .edit-box { display: flex; flex-direction: column; gap: 6px; }
  .edit-area {
    font-size: 14px; color: var(--text); line-height: 1.55;
    background: var(--surface); border: 1px solid var(--line);
    border-radius: var(--r-nav); padding: var(--s2) 10px; resize: vertical;
    min-height: 56px; outline: none; width: 100%; box-sizing: border-box;
    font-family: var(--font-sans);
  }
  .edit-area:focus {
    border-color: var(--text-muted);
  }
  .edit-actions { display: flex; align-items: center; gap: var(--s4); }
  .edit-hint { font-size: 12px; color: var(--text-muted); margin-left: auto; }

  /* ── Learn note: subtle card, no blue tint ── */
  .learn-note {
    margin-top: var(--s3); font-size: 12px; color: var(--text);
    background: var(--surface); border: 1px solid var(--line);
    border-radius: var(--r-card); padding: var(--s3) var(--s4);
  }
</style>
