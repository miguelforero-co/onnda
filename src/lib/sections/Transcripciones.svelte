<script lang="ts">
  import type { HistoryEntry } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  // Pure unified list: source filter + play + delete + copy.
  // File upload lives in the Importar section now.
  let {
    history,
    onRefresh,
  }: {
    history: HistoryEntry[];
    onRefresh: () => void;
  } = $props();

  // ── Source filter ──
  let filter = $state<"all" | "dictation" | "file">("all");
  const filtered = $derived(
    filter === "all" ? history : history.filter((e) => e.source === filter)
  );

  // Re-pull history on mount so the list is current when navigated to.
  onMount(() => onRefresh());

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
  }

  async function copyEntry(text: string) {
    try { await navigator.clipboard.writeText(text); } catch { /* ignore */ }
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

<div class="head">
  <h1 class="page-title">Transcripciones</h1>
</div>

<div class="toolbar">
  <div class="seg" role="tablist">
    <button class="seg-btn" class:on={filter === "all"} onclick={() => (filter = "all")}>Todas</button>
    <button class="seg-btn" class:on={filter === "dictation"} onclick={() => (filter = "dictation")}>Dictado</button>
    <button class="seg-btn" class:on={filter === "file"} onclick={() => (filter = "file")}>Archivo</button>
  </div>
</div>

{#if filtered.length === 0}
  <div class="empty">
    <p>Sin transcripciones aún</p>
    <span>Presiona tu atajo para dictar, o sube un archivo de audio.</span>
  </div>
{:else}
  <div class="hist-list">
    {#each filtered as e (e.id)}
      <div class="hist-item">
        <div class="hist-meta">
          <span class="hist-time">{fmtTime(e.timestamp_ms)}</span>
          {#if e.duration_secs >= 1}<span class="hist-dur">{fmtDur(e.duration_secs)}</span>{/if}
          {#if e.source === "file" && e.original_filename}<span class="hist-file">{e.original_filename}</span>{/if}
          <div class="hist-actions">
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
        <p class="hist-text">{e.text}</p>
      </div>
    {/each}
  </div>
{/if}

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }

  .head { display: flex; align-items: center; justify-content: space-between; gap: 12px; }

  /* ── Toolbar / filter ── */
  .toolbar { display: flex; align-items: center; gap: 12px; margin-top: 18px; }
  .seg { display: inline-flex; background: var(--panel); border: 1px solid var(--line); border-radius: 7px; padding: 2px; }
  .seg-btn {
    background: none; border: none; cursor: pointer;
    font-size: 12px; font-weight: 450; color: var(--muted);
    padding: 4px 12px; border-radius: 5px; transition: background .12s, color .12s;
  }
  .seg-btn:hover { color: var(--text); }
  .seg-btn.on { background: var(--bg); color: var(--text); font-weight: 600; }

  /* ── History list (reused from pre-refactor +page.svelte) ── */
  .empty {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; padding: 60px 20px; gap: 6px; text-align: center;
  }
  .empty p { font-size: 14px; font-weight: 450; color: var(--muted); }
  .empty span { font-size: 12px; color: var(--faint); line-height: 1.5; }

  .hist-list { display: flex; flex-direction: column; margin-top: 16px; }
  .hist-item {
    padding: 12px 0;
    border-bottom: 1px solid var(--line);
    display: flex; flex-direction: column; gap: 5px;
  }
  .hist-item:first-child { border-top: 1px solid var(--line); }
  .hist-meta { display: flex; align-items: center; gap: 6px; }
  .hist-time { font-size: 11px; color: var(--faint); }
  .hist-dur {
    font-size: 10.5px; color: var(--faint);
    background: rgba(0,0,0,.05); border-radius: 10px; padding: 1px 6px;
  }
  .hist-file {
    font-size: 10.5px; color: var(--muted);
    background: rgba(123,155,210,.12); border-radius: 10px; padding: 1px 7px;
    max-width: 180px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .hist-actions { display: flex; gap: 2px; margin-left: auto; }
  .icon-btn {
    width: 22px; height: 22px; background: none; border: none;
    border-radius: 5px; color: var(--faint); cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: background .12s, color .12s;
  }
  .icon-btn svg { width: 9px; height: 9px; }
  .icon-btn:hover { background: rgba(0,0,0,.06); color: var(--muted); }
  .icon-btn.active { color: var(--amber); }
  .icon-btn.del:hover { color: var(--coral); }

  .hist-text {
    font-size: 13px; color: var(--muted); line-height: 1.55;
    word-break: break-word; white-space: pre-wrap;
  }
</style>
