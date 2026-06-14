<script lang="ts">
  import type { HistoryEntry } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount, onDestroy } from "svelte";

  // Dedicated screen to upload + transcribe + review audio files.
  // Accepts the shared history store + a refresh callback from +page.
  let {
    history,
    onRefresh,
  }: {
    history: HistoryEntry[];
    onRefresh: () => void;
  } = $props();

  // Only the file-sourced transcriptions belong here.
  const files = $derived(history.filter((e) => e.source === "file"));

  // ── Upload state ── "" | "Decodificando…" | "Transcribiendo…" | error text
  let uploadState = $state("");
  const busy = $derived(uploadState === "Decodificando…" || uploadState === "Transcribiendo…");

  async function uploadAudio() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Audio", extensions: ["wav", "mp3", "m4a"] }],
    });
    if (!path || Array.isArray(path)) return;
    uploadState = "Decodificando…";
    try {
      await invoke("transcribe_file", { path });
    } catch (e) {
      console.error(e);
      uploadState = "No se pudo transcribir el archivo. Formatos admitidos: WAV, MP3, M4A.";
    }
  }

  // ── File-transcribe events ──
  const unlisten: (() => void)[] = [];
  onMount(async () => {
    unlisten.push(
      await listen<string>("file-transcribe-progress", ({ payload }) => {
        uploadState = payload === "decoding" ? "Decodificando…" : "Transcribiendo…";
      }),
      await listen("file-transcribe-done", () => {
        uploadState = "";
        onRefresh(); // re-pull history so the new file entry shows
      }),
      await listen("file-transcribe-error", () => {
        uploadState = "No se pudo transcribir el archivo. Formatos admitidos: WAV, MP3, M4A.";
      }),
    );
  });

  // ── Playback (lifted from Transcripciones) ──
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

  function fmtTime(ms: number) {
    const d = new Date(ms), now = new Date();
    return now.toDateString() === d.toDateString()
      ? d.toLocaleTimeString("es", { hour: "2-digit", minute: "2-digit" })
      : d.toLocaleDateString("es", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
  }

  onDestroy(() => { unlisten.forEach((fn) => fn()); audioEl?.pause(); });
</script>

<div class="head">
  <h1 class="page-title">Importar</h1>
  <button class="btn-primary" onclick={uploadAudio} disabled={busy}>
    Subir audio
  </button>
</div>

{#if uploadState}
  <div class="toolbar">
    <span class="upload-status">{uploadState}</span>
  </div>
{/if}

{#if files.length === 0}
  <div class="empty">
    <p>Sube un archivo de audio para transcribirlo.</p>
    <button class="btn-primary" onclick={uploadAudio} disabled={busy}>Subir audio</button>
  </div>
{:else}
  <div class="hist-list">
    {#each files as e (e.id)}
      <div class="hist-item">
        <div class="hist-meta">
          <span class="hist-time">{fmtTime(e.timestamp_ms)}</span>
          {#if e.original_filename}<span class="hist-file">{e.original_filename}</span>{/if}
          <div class="hist-actions">
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
  .btn-primary {
    background: var(--coral); color: #fff; border: none;
    border-radius: var(--r); padding: 8px 16px; font-size: 12.5px; font-weight: 600;
    cursor: pointer; letter-spacing: -.01em; white-space: nowrap;
    transition: opacity .15s, transform .1s;
  }
  .btn-primary:hover:not(:disabled) { opacity: .88; }
  .btn-primary:active:not(:disabled) { transform: scale(.98); }
  .btn-primary:disabled { opacity: .4; cursor: default; }

  .toolbar { display: flex; align-items: center; gap: 12px; margin-top: 18px; }
  .upload-status { font-size: 12px; color: var(--muted); }

  /* ── History list (reused from Transcripciones) ── */
  .empty {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; padding: 60px 20px; gap: 12px; text-align: center;
  }
  .empty p { font-size: 14px; font-weight: 450; color: var(--muted); }

  .hist-list { display: flex; flex-direction: column; margin-top: 16px; }
  .hist-item {
    padding: 12px 0;
    border-bottom: 1px solid var(--line);
    display: flex; flex-direction: column; gap: 5px;
  }
  .hist-item:first-child { border-top: 1px solid var(--line); }
  .hist-meta { display: flex; align-items: center; gap: 6px; }
  .hist-time { font-size: 11px; color: var(--faint); }
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
