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

<!-- ── Hero: dictation is hotkey-only — decorative orb + shortcut, no button ── -->
<section class="hero">
  <div class="dictar-orb" aria-hidden="true">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
      <rect x="9" y="2" width="6" height="12" rx="3"/>
      <path d="M5 10a7 7 0 0 0 14 0"/>
      <line x1="12" y1="19" x2="12" y2="22"/>
    </svg>
  </div>
  <p class="hero-title">Dicta con tu voz</p>
  <p class="hero-hint">
    Presiona <kbd>{shortcutLabel || "Alt + Space"}</kbd> en cualquier app para empezar a dictar.
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

  /* ── Hero — the signature dark glass surface ── */
  .hero {
    position: relative;
    background: var(--glass-fill);
    -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--line); border-radius: var(--r-lg);
    box-shadow: var(--glass-edge), var(--glass-glow), var(--glass-shadow);
    padding: 28px;
    align-items: center;
    gap: 12px;
  }
  /* ── Dictar orb — decorative (NOT a button); dictation is hotkey-only.
     Dark glass core, iridescent edge ring, slow holographic sheen sweep
     (Vercel beam), warm inner glow. The ONE expressive moment. */
  .dictar-orb {
    position: relative;
    overflow: hidden;                 /* clips the sheen band */
    width: 64px; height: 64px; border-radius: 50%;
    display: inline-flex; align-items: center; justify-content: center;
    color: var(--text);
    /* dark glass core + iridescent ring via padding-box/border-box (respects radius) */
    background:
      linear-gradient(var(--elev-2), var(--elev-2)) padding-box,
      var(--iris-ramp) border-box;
    border: 1.5px solid transparent;
    box-shadow:
      inset 0 1px 0 var(--edge-hi),
      0 0 26px -6px rgba(180,140,252,0.45),
      var(--accent-glow);
  }
  .dictar-orb svg { width: 26px; height: 26px; position: relative; z-index: 2; }

  /* Holographic sheen sweep — Vercel's prism beam, skewed, swept across.
     mix-blend screen so it GLOWS on dark instead of glaring. */
  .dictar-orb::after {
    content: ""; position: absolute; inset: 0; left: -130%; z-index: 1;
    background: var(--iris-sheen); transform: skewX(-16deg);
    pointer-events: none; mix-blend-mode: screen;
  }
  @media (prefers-reduced-motion: no-preference) {
    .dictar-orb::after {
      animation: dictar-sheen var(--sheen-dur) var(--ease-soft) infinite;
    }
    @keyframes dictar-sheen {
      0%, 55% { left: -130%; }   /* long pause, brief sweep — restrained */
      100%    { left: 130%; }
    }
  }
  @media (prefers-contrast: more) {
    .dictar-orb { background: var(--elev-2); border-color: var(--accent); box-shadow: none; }
    .dictar-orb::after { display: none; }
  }

  .hero-title { font-size: 15px; font-weight: 600; color: var(--text); letter-spacing: -.01em; }
  .hero-hint { font-size: 12px; color: var(--muted); line-height: 1.5; text-align: center; }

  /* ── Blocks ── */
  .section-label {
    font-size: 11px; font-weight: 600; text-transform: uppercase;
    letter-spacing: .06em; color: var(--faint); padding: 0 3px;
  }

  /* ── Stats — dark glass panels ── */
  .stats { display: flex; gap: 10px; }
  .stat-card {
    flex: 1; position: relative;
    background: var(--glass-fill);
    -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--line); border-radius: var(--r);
    box-shadow: var(--glass-edge), var(--sh-2);
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

  /* ── Quick actions — dark glass cards ── */
  .quick { display: flex; gap: 10px; }
  .quick-card {
    flex: 1; position: relative;
    background: var(--glass-fill);
    -webkit-backdrop-filter: var(--glass-blur); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--line); border-radius: var(--r);
    box-shadow: var(--glass-edge), var(--sh-2);
    padding: 14px 16px; font-size: 13px; font-weight: 450; color: var(--text);
    text-align: left; cursor: pointer;
    transition: transform .16s var(--ease-soft), background .16s, border-color .16s, box-shadow .16s;
  }
  .quick-card:hover {
    transform: translateY(-1px);
    background: var(--glass-fill-hi);
    border-color: var(--line-strong);
    box-shadow: var(--glass-edge), var(--sh-3);
  }

  kbd {
    display: inline-block; background: rgba(255,255,255,0.08);
    border: 1px solid var(--line); border-radius: 5px; padding: 1px 6px;
    font-size: 11px; font-family: inherit; color: var(--muted);
  }
</style>
