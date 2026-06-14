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

  // ── Processing state ──
  // phase: "idle" while nothing runs; "decoding"/"transcribing" while busy;
  // "done" for the brief "Listo ✓" flash; "error" with a message.
  type Phase = "idle" | "decoding" | "transcribing" | "done" | "error";
  let phase = $state<Phase>("idle");
  let errorMsg = $state("");
  const busy = $derived(phase === "decoding" || phase === "transcribing");
  const stageLabel = $derived(
    phase === "decoding" ? "Decodificando audio…"
      : phase === "transcribing" ? "Transcribiendo…"
      : ""
  );

  // ── Elapsed timer (ticks while processing) ──
  let elapsed = $state(0);
  let timer: ReturnType<typeof setInterval> | null = null;
  function startTimer() {
    stopTimer();
    elapsed = 0;
    const t0 = Date.now();
    timer = setInterval(() => { elapsed = Math.floor((Date.now() - t0) / 1000); }, 1000);
  }
  function stopTimer() {
    if (timer) { clearInterval(timer); timer = null; }
  }

  // Auto-clear the "Listo ✓" flash after a moment.
  let doneTimeout: ReturnType<typeof setTimeout> | null = null;

  async function uploadAudio() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Audio", extensions: ["wav", "mp3", "m4a"] }],
    });
    if (!path || Array.isArray(path)) return;
    if (doneTimeout) { clearTimeout(doneTimeout); doneTimeout = null; }
    errorMsg = "";
    phase = "decoding";
    startTimer();
    try {
      await invoke("transcribe_file", { path });
    } catch (e) {
      console.error(e);
      stopTimer();
      phase = "error";
      errorMsg = "No se pudo transcribir el archivo. Formatos admitidos: WAV, MP3, M4A.";
    }
  }

  // ── File-transcribe events ──
  const unlisten: (() => void)[] = [];
  onMount(async () => {
    unlisten.push(
      await listen<string>("file-transcribe-progress", ({ payload }) => {
        phase = payload === "decoding" ? "decoding" : "transcribing";
      }),
      await listen("file-transcribe-done", () => {
        stopTimer();
        phase = "done";
        onRefresh(); // re-pull history so the new file entry shows
        if (doneTimeout) clearTimeout(doneTimeout);
        doneTimeout = setTimeout(() => { if (phase === "done") phase = "idle"; }, 1800);
      }),
      await listen("file-transcribe-error", () => {
        stopTimer();
        phase = "error";
        errorMsg = "No se pudo transcribir el archivo. Formatos admitidos: WAV, MP3, M4A.";
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

  onDestroy(() => {
    unlisten.forEach((fn) => fn());
    audioEl?.pause();
    stopTimer();
    if (doneTimeout) clearTimeout(doneTimeout);
  });
</script>

<div class="head">
  <h1 class="page-title">Importar</h1>
  <button class="btn-primary" onclick={uploadAudio} disabled={busy}>
    Subir audio
  </button>
</div>

{#if busy}
  <div class="processing" role="status" aria-live="polite">
    <div class="loader" aria-hidden="true"><span></span></div>
    <div class="processing-text">
      <span class="stage">{stageLabel}</span>
      <span class="hint">El tiempo depende del tamaño del archivo y del modelo.</span>
    </div>
    <span class="elapsed">{elapsed} s</span>
  </div>
{:else if phase === "done"}
  <div class="toolbar">
    <span class="done-status">Listo ✓</span>
  </div>
{:else if phase === "error"}
  <div class="toolbar">
    <span class="error-status">{errorMsg}</span>
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
    background: var(--accent); color: #fff; border: none;
    border-radius: var(--r); padding: 8px 16px; font-size: 12.5px; font-weight: 600;
    cursor: pointer; letter-spacing: -.01em; white-space: nowrap;
    box-shadow: var(--accent-glow);
    transition: opacity .15s, transform .1s, box-shadow .15s;
  }
  .btn-primary:hover:not(:disabled) { opacity: .92; transform: translateY(-1px); }
  .btn-primary:active:not(:disabled) { transform: scale(.98); }
  .btn-primary:disabled {
    opacity: 1; cursor: default;
    background: rgba(255,255,255,0.06); color: var(--faint);
    box-shadow: inset 0 0 0 1px var(--line);
  }

  .toolbar { display: flex; align-items: center; gap: 12px; margin-top: 18px; }
  .done-status { font-size: 12px; color: var(--blue); animation: fade-out 1.8s var(--ease-soft) forwards; }
  @keyframes fade-out { 0%, 55% { opacity: 1; } 100% { opacity: 0; } }
  .error-status { font-size: 12px; color: var(--coral); line-height: 1.5; }

  /* ── Processing indicator — dark glass panel with iridescent loader ── */
  .processing {
    display: flex; align-items: center; gap: 14px; margin-top: 18px;
    padding: 14px 16px;
    background: var(--glass-fill);
    -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--line); border-radius: var(--r);
    box-shadow: var(--glass-edge), var(--sh-2);
  }
  .processing-text { display: flex; flex-direction: column; gap: 3px; flex: 1; min-width: 0; }
  .processing-text .stage { font-size: 13px; font-weight: 450; color: var(--text); }
  .processing-text .hint { font-size: 11px; color: var(--faint); line-height: 1.4; }
  .elapsed {
    font-size: 11.5px; color: var(--muted); flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  /* Indeterminate sweeping bar — iridescent, GPU-cheap (transform only).
     Static fallback: a faint full-width iridescent track when motion is off. */
  .loader {
    position: relative; flex-shrink: 0;
    width: 56px; height: 4px; border-radius: 2px; overflow: hidden;
    background: rgba(255,255,255,0.08);
  }
  .loader span {
    position: absolute; inset: 0;
    background: var(--iris-ramp);
    border-radius: 2px;
  }
  @media (prefers-reduced-motion: no-preference) {
    .loader span {
      width: 45%;
      animation: loader-sweep 1.15s var(--ease-soft) infinite;
    }
    @keyframes loader-sweep {
      0%   { transform: translateX(-120%); }
      100% { transform: translateX(245%); }
    }
  }

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
    background: rgba(127,200,255,.12); border-radius: 10px; padding: 1px 7px;
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
  .icon-btn:hover { background: rgba(255,255,255,.06); color: var(--muted); }
  .icon-btn.active { color: var(--amber); }
  .icon-btn.del:hover { color: var(--coral); }

  .hist-text {
    font-size: 13px; color: var(--muted); line-height: 1.55;
    word-break: break-word; white-space: pre-wrap;
  }
</style>
