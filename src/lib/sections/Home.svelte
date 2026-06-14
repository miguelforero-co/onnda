<script lang="ts">
  import type { Settings, HistoryEntry, View } from "$lib/types";

  // Dictation hub dashboard. Does NOT reproduce the full transcription list
  // (that's Transcripciones); shows hero + stats + quick links instead.
  let {
    settings,
    history,
    onNavigate,
  }: {
    settings: Settings;
    history: HistoryEntry[];
    onNavigate: (v: View) => void;
  } = $props();

  // Lightweight stats for the dashboard.
  const totalCount = $derived(history.length);
  const fileCount = $derived(history.filter((h) => h.source === "file").length);
  // A single most-recent entry for the one-line peek (not a list).
  const latest = $derived(history[0] ?? null);

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
    <span class="dictar-label">Dictar</span>
  </button>
  <p class="hero-hint">
    Presiona <kbd>{shortcutLabel || "Alt + Space"}</kbd> para dictar en cualquier app.
  </p>
</section>

<!-- ── Stats ── -->
<section class="block">
  <h2 class="section-label">Resumen</h2>
  <div class="stats">
    <div class="stat-card">
      <span class="stat-num">{totalCount}</span>
      <span class="stat-label">Transcripciones</span>
    </div>
    <div class="stat-card">
      <span class="stat-num">{fileCount}</span>
      <span class="stat-label">Archivos</span>
    </div>
  </div>
  {#if latest}
    <div class="peek">
      <span class="peek-time">{fmtTime(latest.timestamp_ms)}</span>
      <span class="peek-text">{latest.text}</span>
    </div>
  {/if}
</section>

<!-- ── Acciones rápidas ── -->
<section class="block">
  <h2 class="section-label">Acciones</h2>
  <div class="quick">
    <button class="quick-card" onclick={() => onNavigate("transcripciones")}>Ver transcripciones</button>
    <button class="quick-card" onclick={() => onNavigate("importar")}>Subir audio</button>
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
  /* ── Dictar hero — metallic body + slow specular sheen sweep ──
     The ONE expressive iridescent moment per screen (Raycast rule). A warm-metal
     wash blended (soft-light) over solid coral so the white label stays legible. */
  .btn-primary.dictar {
    position: relative;
    overflow: hidden;                 /* clips the sheen band */
    background:
      var(--iris-sheen) 0 0 / 220% 100%,
      var(--coral);
    background-blend-mode: soft-light, normal;
    color: #fff; border: none;
    border-radius: var(--r); padding: 11px 22px; font-size: 13.5px; font-weight: 600;
    cursor: default; letter-spacing: -.01em;
    display: inline-flex; align-items: center; gap: 8px;
    box-shadow:
      var(--emboss-hi),
      var(--emboss-lo),
      0 2px 8px rgba(232,85,53,0.28);
    transition: transform .14s ease, box-shadow .14s ease;
  }
  .btn-primary.dictar svg { width: 18px; height: 18px; position: relative; z-index: 1; }
  .dictar-label { position: relative; z-index: 1; }
  /* keep the disabled hero readable but premium — no flat dimming on the metal */
  .btn-primary.dictar:disabled { opacity: 1; }

  /* Specular sheen band — skewed translucent highlight that slowly sweeps across,
     the CSS analog of the shader's pow(core,3.0)*spec crest highlight. */
  .btn-primary.dictar::after {
    content: ""; position: absolute; inset: 0; left: -120%;
    background: var(--sheen-band);
    transform: skewX(-18deg);
    pointer-events: none;
  }
  @media (prefers-reduced-motion: no-preference) {
    .btn-primary.dictar::after {
      animation: dictar-sheen var(--sheen-dur) ease-in-out infinite;
    }
    @keyframes dictar-sheen {
      0%, 60% { left: -120%; }   /* long pause, brief sweep — restrained */
      100%    { left: 120%; }
    }
  }
  @media (prefers-contrast: more) {
    .btn-primary.dictar { background: var(--coral); }
  }

  .hero-hint { font-size: 12px; color: var(--muted); line-height: 1.5; text-align: center; }

  /* ── Blocks ── */
  .section-label {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: .06em; color: var(--faint); padding: 0 3px;
  }

  /* ── Stats ── */
  .stats { display: flex; gap: 10px; }
  .stat-card {
    flex: 1; background: var(--panel); border-radius: var(--r);
    padding: 16px; display: flex; flex-direction: column; gap: 4px;
  }
  .stat-num { font-size: 24px; font-weight: 600; color: var(--text); letter-spacing: -.02em; }
  .stat-label {
    font-size: 11px; font-weight: 450; color: var(--faint);
    text-transform: uppercase; letter-spacing: .04em;
  }

  .peek {
    display: flex; align-items: baseline; gap: 8px; padding: 0 3px;
    overflow: hidden;
  }
  .peek-time { font-size: 11px; color: var(--faint); flex-shrink: 0; }
  .peek-text {
    font-size: 12px; color: var(--muted); line-height: 1.4;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }

  /* ── Quick actions ── */
  .quick { display: flex; gap: 10px; }
  .quick-card {
    flex: 1; background: var(--panel); border: none; border-radius: var(--r);
    padding: 14px 16px; font-size: 13px; font-weight: 450; color: var(--text);
    text-align: left; cursor: pointer; transition: background .12s, opacity .12s;
  }
  .quick-card:hover { opacity: .8; }

  kbd {
    display: inline-block; background: rgba(0,0,0,.06);
    border-radius: 4px; padding: 1px 5px;
    font-size: 11px; font-family: inherit; color: var(--muted);
  }
</style>
