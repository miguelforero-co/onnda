<script lang="ts">
  import type { HistoryEntry } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount, onDestroy } from "svelte";
  import SectionLabel from "$lib/components/ui/SectionLabel.svelte";

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

  // ── Core transcription by path (shared by click-to-pick and drag-drop) ──
  async function transcribeFromPath(path: string) {
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

  async function uploadAudio() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Audio", extensions: ["wav", "mp3", "m4a"] }],
    });
    if (!path || Array.isArray(path)) return;
    await transcribeFromPath(path);
  }

  // ── Drag-over state (for dropzone styling) ──
  let dragOver = $state(false);

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

    // ── Tauri webview drag-drop ──
    try {
      const webview = getCurrentWebview();
      const unlistenDragDrop = await webview.onDragDropEvent((event) => {
        const t = event.payload.type;
        if (t === "enter" || t === "over") {
          dragOver = true;
        } else if (t === "leave") {
          dragOver = false;
        } else if (t === "drop") {
          dragOver = false;
          const audioExts = ["wav", "mp3", "m4a"];
          const audioPath = event.payload.paths.find((p) => {
            const ext = p.split(".").pop()?.toLowerCase() ?? "";
            return audioExts.includes(ext);
          });
          if (audioPath) transcribeFromPath(audioPath);
        }
      });
      unlisten.push(unlistenDragDrop);
    } catch {
      // drag-drop API unavailable; click-to-pick still works
    }
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

  async function copyEntry(text: string) {
    try { await navigator.clipboard.writeText(text); } catch { /* ignore */ }
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

<div class="screen">
  <h1 class="page-title">Importar</h1>

  <!-- ── Dropzone ── -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="dropzone"
    class:drag-over={dragOver}
    class:busy
    onclick={!busy ? uploadAudio : undefined}
    role="button"
    tabindex="0"
    aria-label="Subir archivo de audio"
    onkeydown={(e) => { if (!busy && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); uploadAudio(); } }}
  >
    {#if busy}
      <!-- Processing state inside the dropzone -->
      <div class="dz-processing" role="status" aria-live="polite">
        <div class="loader" aria-hidden="true"><span></span></div>
        <div class="dz-process-text">
          <span class="stage">{stageLabel}</span>
          <span class="sub">El tiempo depende del tamaño del archivo y del modelo. {elapsed}s</span>
        </div>
      </div>
    {:else if phase === "done"}
      <div class="dz-done">
        <svg class="dz-icon" viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="6 17 13 24 26 10"/>
        </svg>
        <span class="dz-label done-label">Transcripción lista</span>
      </div>
    {:else if phase === "error"}
      <div class="dz-idle">
        <svg class="dz-icon" viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="16" cy="16" r="13"/><line x1="16" y1="10" x2="16" y2="18"/><circle cx="16" cy="22" r="1" fill="currentColor" stroke="none"/>
        </svg>
        <span class="dz-label">{errorMsg}</span>
        <span class="dz-sub">Haz clic para intentar con otro archivo</span>
      </div>
    {:else}
      <div class="dz-idle">
        <!-- Upload icon — inline SVG, monochrome currentColor -->
        <svg class="dz-icon" viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M10 22 C6 22 4 19 4 16 C4 13 6 11 9 11 C9 11 9 11 9 10.5 C9 7.5 11.5 5 14.5 5 C17 5 19 6.5 19.5 8.5 C20 8.2 20.5 8 21 8 C23.5 8 25.5 10 25.5 12.5 C25.5 12.7 25.5 12.9 25.5 13 C27 14 28 15.5 28 17 C28 19.5 26 22 23 22"/>
          <line x1="16" y1="14" x2="16" y2="27"/>
          <polyline points="12 18 16 14 20 18"/>
        </svg>
        <span class="dz-label">Arrastra un archivo de audio o haz clic para elegir</span>
        <span class="dz-sub">WAV · MP3 · M4A</span>
      </div>
    {/if}
  </div>

  <!-- ── History section ── -->
  {#if files.length === 0}
    <div class="empty">
      <p class="empty-title">Aún no has transcrito archivos</p>
      <span class="empty-hint">Los archivos importados aparecerán aquí después de transcribirlos.</span>
    </div>
  {:else}
    <div class="section-header">
      <SectionLabel text="Historial" />
    </div>
    <div class="hist-list">
      {#each files as e (e.id)}
        <div class="hist-card">
          <div class="hist-meta">
            <span class="hist-time">{fmtTime(e.timestamp_ms)}</span>
            {#if e.original_filename}<span class="hist-chip hist-file">{e.original_filename}</span>{/if}
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
</div>

<style>
  /* ── Root container: top offset matches Home / Transcripciones ── */
  .screen {
    padding: var(--screen-top) var(--s10) var(--s10);
    display: flex;
    flex-direction: column;
  }

  /* ── Page title: serif, matches system ── */
  .page-title {
    font-family: var(--font-serif);
    font-size: 24px;
    font-weight: 400;
    color: var(--text);
    margin: 0 0 var(--s6) 0;
  }

  /* ── Dropzone ── */
  .dropzone {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--s8);
    border: 1px dashed var(--line);
    border-radius: var(--r-card);
    background: var(--inset);
    cursor: pointer;
    transition: border-color .15s, background .15s;
    min-height: 160px;
    margin-bottom: var(--s6);
    user-select: none;
  }
  .dropzone:hover:not(.busy),
  .dropzone:focus-visible:not(.busy) {
    border-color: var(--text-muted);
    outline: none;
  }
  .dropzone.drag-over:not(.busy) {
    border-color: var(--text-muted);
    background: var(--surface);
  }
  .dropzone.busy {
    cursor: default;
  }

  /* ── Idle / error dropzone content ── */
  .dz-idle,
  .dz-done {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s3);
    text-align: center;
  }
  .dz-icon {
    width: 32px;
    height: 32px;
    color: var(--text-muted);
    flex-shrink: 0;
  }
  .dz-label {
    font-size: 14px;
    font-weight: 400;
    color: var(--text);
    font-family: var(--font-sans);
    max-width: 280px;
    line-height: 1.4;
  }
  .done-label {
    color: var(--dot-on);
  }
  .dz-sub {
    font-size: 12px;
    color: var(--text-muted);
    font-family: var(--font-sans);
  }

  /* ── Processing state inside dropzone ── */
  .dz-processing {
    display: flex;
    align-items: center;
    gap: var(--s3);
    width: 100%;
    max-width: 320px;
  }
  .dz-process-text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
    min-width: 0;
  }
  .stage {
    font-size: 14px;
    font-weight: 400;
    color: var(--text);
    font-family: var(--font-sans);
  }
  .sub {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.4;
    font-family: var(--font-sans);
    font-variant-numeric: tabular-nums;
  }

  /* Indeterminate sweeping bar. Static fallback when motion is off. */
  .loader {
    position: relative;
    flex-shrink: 0;
    width: 56px;
    height: 4px;
    border-radius: 2px;
    overflow: hidden;
    background: var(--line);
  }
  .loader span {
    position: absolute;
    inset: 0;
    background: var(--dot-on);
    border-radius: 2px;
  }
  @media (prefers-reduced-motion: no-preference) {
    .loader span {
      width: 45%;
      animation: loader-sweep 1.15s cubic-bezier(0.32, 0.72, 0, 1) infinite;
    }
    @keyframes loader-sweep {
      0%   { transform: translateX(-120%); }
      100% { transform: translateX(245%); }
    }
  }

  /* ── Section header ── */
  .section-header {
    margin-bottom: var(--s3);
  }

  /* ── Empty state ── */
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px var(--s6);
    gap: 6px;
    text-align: center;
  }
  .empty-title {
    font-size: 14px;
    font-weight: 400;
    color: var(--text);
    font-family: var(--font-sans);
  }
  .empty-hint {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.5;
    font-family: var(--font-sans);
    max-width: 260px;
  }

  /* ── History list: cards ── */
  .hist-list {
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }

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

  /* Chips: bordered, transparent bg */
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
  /* Playing state: muted green accent */
  .icon-btn.active { color: var(--dot-on); }
  /* Delete: monochrome, no red */
  .icon-btn.del { color: var(--text-muted); }
  .icon-btn.del:hover { background: rgba(127,127,127,0.10); color: var(--text); }

  .hist-text {
    font-size: 14px;
    color: var(--text);
    line-height: 1.55;
    word-break: break-word;
    white-space: pre-wrap;
    font-family: var(--font-sans);
  }
</style>
