<script lang="ts">
  import type { Settings, HistoryEntry, View } from "$lib/types";

  // Prop contract from 01-03 (Home stub). D-03: dictation hub dashboard.
  let {
    settings,
    history,
    onNavigate,
  }: {
    settings: Settings;
    history: HistoryEntry[];
    onNavigate: (v: View) => void;
  } = $props();

  // The five most recent entries for the "Recientes" block.
  const recent = $derived(history.slice(0, 5));

  // Pretty-print the shortcut combo for the kbd badge ("Alt+Space" → "Alt + Space").
  const shortcutLabel = $derived(
    (settings.shortcut || "").split("+").map((p) => p.trim()).filter(Boolean).join(" + ")
  );

  function fmtTime(ms: number) {
    const d = new Date(ms), now = new Date();
    return now.toDateString() === d.toDateString()
      ? d.toLocaleTimeString("es", { hour: "2-digit", minute: "2-digit" })
      : d.toLocaleDateString("es", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
  }
</script>

<h1 class="page-title">Inicio</h1>

<!-- ── Hero: dictation affordance (informational — dictation fires via global hotkey) ── -->
<section class="hero">
  <button class="btn-primary dictar" disabled title="Usa el atajo global para dictar">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <rect x="9" y="2" width="6" height="12" rx="3"/>
      <path d="M5 10a7 7 0 0 0 14 0"/>
      <line x1="12" y1="19" x2="12" y2="22"/>
    </svg>
    Dictar
  </button>
  <p class="hero-hint">
    Presiona <kbd>{shortcutLabel || "Alt + Space"}</kbd> para dictar en cualquier app.
  </p>
</section>

<!-- ── Recientes ── -->
<section class="block">
  <div class="block-head">
    <h2 class="section-label">Recientes</h2>
    <button class="link-btn" onclick={() => onNavigate("transcripciones")}>Ver todas</button>
  </div>

  {#if recent.length === 0}
    <div class="empty">
      <p>Sin transcripciones aún</p>
      <span>Presiona <kbd>{shortcutLabel || "Alt + Space"}</kbd> para dictar, o sube un archivo de audio.</span>
    </div>
  {:else}
    <div class="hist-list">
      {#each recent as e (e.id)}
        <div class="hist-item">
          <div class="hist-meta">
            <span class="hist-time">{fmtTime(e.timestamp_ms)}</span>
            {#if e.source === "file"}<span class="hist-dur">Archivo</span>{/if}
          </div>
          <p class="hist-text">{e.text}</p>
        </div>
      {/each}
    </div>
  {/if}
</section>

<!-- ── Acción rápida ── -->
<section class="block">
  <h2 class="section-label">Acciones</h2>
  <div class="quick">
    <button class="link-btn" onclick={() => onNavigate("transcripciones")}>Subir audio</button>
  </div>
</section>

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }

  section { margin-top: 24px; display: flex; flex-direction: column; gap: 10px; }

  /* ── Hero ── */
  .hero {
    background: var(--panel);
    border-radius: var(--r);
    padding: 24px;
    align-items: center;
    gap: 12px;
  }
  .btn-primary {
    background: var(--coral); color: #fff; border: none;
    border-radius: var(--r); padding: 11px 22px; font-size: 13.5px; font-weight: 600;
    cursor: pointer; letter-spacing: -.01em;
    display: inline-flex; align-items: center; gap: 8px;
    transition: opacity .15s, transform .1s;
  }
  .btn-primary svg { width: 18px; height: 18px; }
  .btn-primary:disabled { opacity: .55; cursor: default; }
  .hero-hint { font-size: 12px; color: var(--muted); line-height: 1.5; text-align: center; }

  /* ── Blocks ── */
  .block-head { display: flex; align-items: center; justify-content: space-between; }

  .section-label {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: .06em; color: var(--faint); padding: 0 3px;
  }

  .link-btn {
    background: none; border: none; padding: 4px 0;
    font-size: 12px; font-weight: 450; color: var(--coral);
    cursor: pointer; text-decoration: none;
  }
  .link-btn:hover { opacity: .75; }

  .quick { padding: 0 3px; }

  /* ── History list (reused from pre-refactor +page.svelte) ── */
  .empty {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; padding: 48px 20px; gap: 6px; text-align: center;
  }
  .empty p { font-size: 14px; font-weight: 450; color: var(--muted); }
  .empty span { font-size: 12px; color: var(--faint); line-height: 1.5; }

  .hist-list { display: flex; flex-direction: column; }
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
  .hist-text {
    font-size: 13px; color: var(--muted); line-height: 1.55;
    word-break: break-word; white-space: pre-wrap;
    display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2;
    -webkit-box-orient: vertical; overflow: hidden;
  }

  kbd {
    display: inline-block; background: rgba(0,0,0,.06);
    border-radius: 4px; padding: 1px 5px;
    font-size: 11px; font-family: inherit; color: var(--muted);
  }
</style>
