<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount, onDestroy } from "svelte";

  interface Settings {
    shortcut: string;
    push_to_talk: boolean;
    selected_language: string;
    selected_model: string;
    autostart: boolean;
    onboarding_done: boolean;
    widget_position: string;
    custom_words: string;
    word_correction_threshold: number;
  }
  interface HistoryEntry {
    id: string; timestamp_ms: number; text: string;
    audio_filename: string | null; duration_secs: number;
  }
  interface ModelInfo { id: string; name: string; size_mb: number; downloaded: boolean; }
  interface DownloadProgress { model_id: string; downloaded_mb: number; total_mb: number; percent: number; }

  let settings = $state<Settings>({
    shortcut: "Alt+Space", push_to_talk: true, selected_language: "auto",
    selected_model: "large-v3-turbo", autostart: false,
    onboarding_done: false, widget_position: "center", custom_words: "",
    word_correction_threshold: 0.85,
  });
  let view = $state<"onboarding" | "settings" | "history">("settings");
  // onboarding has two steps: "perms" then "models"
  let obStep = $state<"perms" | "models">("perms");
  let models = $state<ModelInfo[]>([]);
  let history = $state<HistoryEntry[]>([]);
  let playingId = $state<string | null>(null);
  let audioEl: HTMLAudioElement | null = null;
  let micGranted = $state(false);
  let a11yGranted = $state(false);
  let pasteStatus = $state("");
  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let initialized = false;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  // Download state keyed by model_id
  let downloadProgress = $state<Record<string, DownloadProgress>>({});
  let downloadErrors = $state<Record<string, string>>({});
  let appVersion = $state("");
  let buildHash = $state("");

  const LANGUAGES = [
    { value: "auto", label: "Automático" },
    { value: "es",   label: "Español" },
    { value: "en",   label: "English" },
    { value: "pt",   label: "Português" },
    { value: "fr",   label: "Français" },
    { value: "de",   label: "Deutsch" },
  ];
  const POSITIONS = [
    { value: "center", label: "Centro" },
    { value: "left",   label: "Izquierda" },
    { value: "right",  label: "Derecha" },
  ];

  const unlisten: (() => void)[] = [];

  onMount(async () => {
    appVersion = await getVersion();
    buildHash = await invoke<string>("get_build_hash").catch(() => "");
    settings = await invoke("get_settings");
    models = await invoke("get_models");
    await checkPerms();

    if (!settings.onboarding_done) {
      view = "onboarding";
      pollInterval = setInterval(checkPerms, 1500);
    } else {
      history = await invoke("get_history");
      // Keep checking permissions so the warning banner updates live
      pollInterval = setInterval(checkPerms, 3000);
    }

    unlisten.push(
      await listen("transcription-done", async () => {
        if (view === "history") history = await invoke("get_history");
      }),
      await listen<DownloadProgress>("download-progress", ({ payload }) => {
        downloadProgress = { ...downloadProgress, [payload.model_id]: payload };
      }),
      await listen<string>("download-complete", async ({ payload: modelId }) => {
        delete downloadProgress[modelId];
        downloadProgress = { ...downloadProgress };
        delete downloadErrors[modelId];
        downloadErrors = { ...downloadErrors };
        models = await invoke("get_models");
        // Auto-select the downloaded model if none selected yet
        if (!models.some(m => m.downloaded && m.id === settings.selected_model)) {
          settings.selected_model = modelId;
          schedSave();
        }
      }),
    );

    initialized = true;
  });

  onDestroy(() => {
    unlisten.forEach(fn => fn());
    if (pollInterval) clearInterval(pollInterval);
    if (saveTimer) clearTimeout(saveTimer);
    audioEl?.pause();
  });

  function schedSave(scChanged = false) {
    if (!initialized) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      await invoke("save_settings", { newSettings: settings, shortcutChanged: scChanged });
    }, 600);
  }

  async function checkPerms() {
    micGranted  = await invoke<boolean>("check_mic_permission");
    a11yGranted = await invoke<boolean>("check_accessibility_permission");
  }

  async function startDownload(modelId: string) {
    delete downloadErrors[modelId];
    downloadErrors = { ...downloadErrors };
    try {
      await invoke("download_model", { modelId });
    } catch (e: unknown) {
      downloadErrors = { ...downloadErrors, [modelId]: String(e) };
      delete downloadProgress[modelId];
      downloadProgress = { ...downloadProgress };
    }
  }

  async function finishOnboarding() {
    if (pollInterval) { clearInterval(pollInterval); pollInterval = null; }
    settings.onboarding_done = true;
    await invoke("save_settings", { newSettings: settings, shortcutChanged: false });
    history = await invoke("get_history");
    view = "settings";
  }

  async function goHistory() { history = await invoke("get_history"); view = "history"; }

  async function deleteEntry(id: string) {
    await invoke("delete_history_entry", { id });
    history = history.filter(e => e.id !== id);
  }

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

  function fmtTime(ms: number) {
    const d = new Date(ms), now = new Date();
    return now.toDateString() === d.toDateString()
      ? d.toLocaleTimeString("es", { hour: "2-digit", minute: "2-digit" })
      : d.toLocaleDateString("es", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
  }
  function fmtDur(s: number) { return s >= 1 ? `${s < 60 ? Math.round(s) + "s" : Math.round(s/60) + "m"}` : ""; }
</script>

<div class="app">

  <!-- ── Header ── -->
  <header>
    <span class="wordmark">Voz Local{#if appVersion}<span class="version">v{appVersion}{#if buildHash} · {buildHash}{/if}</span>{/if}</span>
    {#if settings.onboarding_done}
      <nav class="tabs">
        <button class="tab" class:on={view==="settings"} onclick={() => view="settings"}>Ajustes</button>
        <button class="tab" class:on={view==="history"} onclick={goHistory}>Historial</button>
      </nav>
    {/if}
  </header>

  <!-- ── ONBOARDING ── -->
  {#if view === "onboarding"}
  <div class="ob">
    <div class="ob-steps">
      <div class="ob-step-dot" class:active={obStep === "perms"}></div>
      <div class="ob-step-dot" class:active={obStep === "models"}></div>
    </div>

    {#if obStep === "perms"}
      <div class="ob-intro">
        <h1>Bienvenido</h1>
        <p>Voz a texto local en tu Mac.<br>Necesitamos dos permisos para funcionar.</p>
      </div>

      <div class="perm-list">
        <div class="perm-row" class:granted={micGranted}>
          <div class="perm-dot" class:granted={micGranted}></div>
          <div class="perm-info">
            <strong>Micrófono</strong>
            <span>Para capturar tu voz</span>
          </div>
          {#if micGranted}
            <span class="perm-status ok">Concedido</span>
          {:else}
            <button class="link-btn" onclick={() => invoke("open_microphone_settings")}>Abrir ajustes</button>
          {/if}
        </div>

        <div class="perm-row" class:granted={a11yGranted}>
          <div class="perm-dot" class:granted={a11yGranted}></div>
          <div class="perm-info">
            <strong>Accesibilidad</strong>
            <span>Para pegar donde escribes</span>
          </div>
          {#if a11yGranted}
            <span class="perm-status ok">Concedido</span>
          {:else}
            <button class="link-btn" onclick={() => invoke("open_accessibility_settings")}>Abrir ajustes</button>
          {/if}
        </div>
      </div>

      <div class="ob-foot">
        {#if !a11yGranted && micGranted}
          <p class="hint">Sin accesibilidad, el texto se copia al portapapeles automáticamente.</p>
        {/if}
        <button class="btn-primary" disabled={!micGranted}
                onclick={() => { if (pollInterval) { clearInterval(pollInterval); pollInterval = null; } obStep = "models"; }}>
          {micGranted ? "Continuar" : "Esperando permiso de micrófono…"}
        </button>
      </div>

    {:else}
      <!-- Step 2: download a model -->
      <div class="ob-intro">
        <h1>Descargar modelo</h1>
        <p>Elige el modelo de reconocimiento.<br>Se guarda en tu Mac y funciona sin internet.</p>
      </div>

      <div class="model-list">
        {#each models as m}
          {@const prog = downloadProgress[m.id]}
          {@const err  = downloadErrors[m.id]}
          <div class="model-row">
            <div class="model-info">
              <strong>{m.name}</strong>
              <span>{m.size_mb} MB</span>
            </div>
            <div class="model-action">
              {#if m.downloaded}
                <span class="perm-status ok">Instalado</span>
              {:else if prog}
                <div class="dl-progress">
                  <div class="dl-bar" style="width:{prog.percent}%"></div>
                </div>
                <span class="dl-pct">{Math.round(prog.percent)}%</span>
              {:else}
                <button class="link-btn" onclick={() => startDownload(m.id)}>
                  Descargar
                </button>
              {/if}
            </div>
            {#if err}
              <p class="dl-error">{err}</p>
            {/if}
          </div>
        {/each}
      </div>

      <div class="ob-foot">
        <p class="hint">Puedes descargar más modelos desde Ajustes en cualquier momento.</p>
        <button class="btn-primary" disabled={!models.some(m => m.downloaded)} onclick={finishOnboarding}>
          {models.some(m => m.downloaded) ? "Empezar" : "Descarga al menos un modelo…"}
        </button>
      </div>
    {/if}

  </div>
  {/if}

  <!-- ── SETTINGS ── -->
  {#if view === "settings"}
  <div class="scroll">

    {#if !a11yGranted}
    <button class="banner" onclick={() => invoke("open_accessibility_settings")}>
      <span class="banner-dot"></span>
      <span>El pegado automático requiere permiso de Accesibilidad.</span>
      <span class="banner-link">Abrir ajustes →</span>
    </button>
    {:else}
    <button class="banner ok" onclick={async () => { pasteStatus = await invoke("test_paste"); }}>
      <span class="banner-dot ok"></span>
      <span>Accesibilidad concedida.</span>
      <span class="banner-link">{pasteStatus ? `Estado: ${pasteStatus}` : "Probar pegado →"}</span>
    </button>
    {/if}

    <section>
      <h2 class="section-label">Grabación</h2>
      <div class="rows">
        <div class="row">
          <span class="row-label">Atajo de teclado</span>
          <input
            class="ipt"
            type="text"
            bind:value={settings.shortcut}
            oninput={() => schedSave(true)}
            placeholder="Alt+Space"
          />
        </div>
        <div class="sep"></div>
        <label class="row" for="ptt">
          <span class="row-label">Push to talk</span>
          <div class="toggle" class:on={settings.push_to_talk}>
            <input id="ptt" type="checkbox" bind:checked={settings.push_to_talk} onchange={() => schedSave()} />
            <span class="knob"></span>
          </div>
        </label>
      </div>
      <p class="section-hint">{settings.push_to_talk ? "Mantén presionado para grabar, suelta para transcribir." : "Presiona una vez para iniciar, otra para detener."}</p>
    </section>

    <section>
      <h2 class="section-label">Reconocimiento</h2>
      <div class="rows">
        <div class="row">
          <span class="row-label">Idioma</span>
          <select class="sel" bind:value={settings.selected_language} onchange={() => schedSave()}>
            {#each LANGUAGES as l}<option value={l.value}>{l.label}</option>{/each}
          </select>
        </div>
        <div class="sep"></div>
        <div class="row">
          <span class="row-label">Modelo</span>
          <select class="sel" bind:value={settings.selected_model} onchange={() => schedSave()}>
            {#each models.filter(m => m.downloaded) as m}
              <option value={m.id}>{m.name} · {m.size_mb} MB</option>
            {/each}
          </select>
        </div>
      </div>
      <div class="words-block">
        <span class="row-label">Vocabulario personalizado</span>
        <textarea
          class="words-ta"
          bind:value={settings.custom_words}
          oninput={() => schedSave()}
          placeholder="GitHub, Claude Code, Node.js, TypeScript, npm, API..."
          rows="3"
        ></textarea>
      </div>
      <p class="section-hint">Palabras o frases que Whisper debe reconocer con precisión, separadas por comas.</p>
      <div class="setting-row" style="margin-top:8px;">
        <span class="row-label">Umbral de corrección</span>
        <div class="threshold-ctrl">
          <input
            type="range" min="0.7" max="1.0" step="0.01"
            bind:value={settings.word_correction_threshold}
            oninput={() => schedSave()}
          />
          <span class="threshold-val">{(settings.word_correction_threshold ?? 0.85).toFixed(2)}</span>
        </div>
      </div>
      <p class="section-hint">Qué tan parecida debe ser una palabra para corregirla. 0.85 recomendado — más alto = más estricto.</p>
    </section>

    <section>
      <h2 class="section-label">Interfaz</h2>
      <div class="rows">
        <div class="row">
          <span class="row-label">Posición del widget</span>
          <select class="sel" bind:value={settings.widget_position} onchange={() => schedSave()}>
            {#each POSITIONS as p}<option value={p.value}>{p.label}</option>{/each}
          </select>
        </div>
      </div>
    </section>

    <section>
      <h2 class="section-label">Sistema</h2>
      <div class="rows">
        <label class="row" for="autostart">
          <span class="row-label">Iniciar con el sistema</span>
          <div class="toggle" class:on={settings.autostart}>
            <input id="autostart" type="checkbox" bind:checked={settings.autostart} onchange={() => schedSave()} />
            <span class="knob"></span>
          </div>
        </label>
      </div>
    </section>

    <section>
      <h2 class="section-label">Modelos</h2>
      <div class="rows">
        {#each models as m, i}
          {@const prog = downloadProgress[m.id]}
          {@const err  = downloadErrors[m.id]}
          <div class="row model-settings-row" style="flex-wrap:wrap; gap:8px;">
            <div style="flex:1; min-width:0;">
              <span class="row-label">{m.name}</span>
              <span class="meta">{m.size_mb} MB</span>
            </div>
            <div class="model-action" style="flex-shrink:0;">
              {#if m.downloaded}
                <span class="badge installed">Instalado</span>
              {:else if prog}
                <div class="dl-inline">
                  <div class="dl-bar-wrap">
                    <div class="dl-bar" style="width:{prog.percent}%"></div>
                  </div>
                  <span class="dl-pct">{Math.round(prog.percent)}%</span>
                </div>
              {:else}
                <button class="link-btn" onclick={() => startDownload(m.id)}>Descargar</button>
              {/if}
            </div>
            {#if err}
              <p class="dl-error" style="width:100%; padding:0 0 6px;">{err}</p>
            {/if}
          </div>
          {#if i < models.length - 1}<div class="sep"></div>{/if}
        {/each}
      </div>
    </section>

    <div class="pad"></div>
  </div>
  {/if}

  <!-- ── HISTORY ── -->
  {#if view === "history"}
  <div class="scroll">
    {#if history.length === 0}
      <div class="empty">
        <p>Sin transcripciones aún</p>
        <span>Presiona <kbd>{settings.shortcut}</kbd> para empezar</span>
      </div>
    {:else}
      <div class="hist-list">
        {#each history as e (e.id)}
          <div class="hist-item">
            <div class="hist-meta">
              <span class="hist-time">{fmtTime(e.timestamp_ms)}</span>
              {#if e.duration_secs >= 1}<span class="hist-dur">{fmtDur(e.duration_secs)}</span>{/if}
              <div class="hist-actions">
                {#if e.audio_filename}
                  <button
                    class="icon-btn"
                    class:active={playingId===e.id}
                    onclick={() => playAudio(e)}
                    title="Reproducir"
                  >
                    {#if playingId===e.id}
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
    <div class="pad"></div>
  </div>
  {/if}

</div>

<style>
  :root {
    --bg:     #F4F0EB;
    --panel:  #FDFCFA;
    --text:   #1C1917;
    --muted:  #78716C;
    --faint:  #A8A29E;
    --line:   rgba(0,0,0,0.07);
    --coral:  #E85535;
    --amber:  #F4AA6A;
    --blue:   #7B9BD2;
    --r:      9px;
  }

  :global(*){ box-sizing:border-box; margin:0; padding:0; }
  :global(body){
    font-family: -apple-system, "SF Pro Text", system-ui, sans-serif;
    background: var(--bg);
    color: var(--text);
    font-size: 13px;
    -webkit-font-smoothing: antialiased;
    overflow: hidden;
    height: 100vh;
  }

  .app{ display:flex; flex-direction:column; height:100vh; background:var(--bg); }

  /* ── Header ── */
  header{
    display:flex; align-items:center; justify-content:space-between;
    padding:0 20px;
    height:48px;
    background:var(--panel);
    border-bottom:1px solid var(--line);
    flex-shrink:0;
  }
  .wordmark{
    font-size:13.5px; font-weight:650; color:var(--text);
    letter-spacing:-.02em;
  }
  .version{
    margin-left:6px; font-size:10.5px; font-weight:500;
    color:var(--faint); letter-spacing:0; vertical-align:middle;
  }

  .tabs{ display:flex; gap:2px; }
  .tab{
    padding:4px 12px; background:none; border:none;
    font-size:12.5px; font-weight:500; color:var(--faint);
    cursor:pointer; border-radius:6px;
    transition:color .12s, background .12s;
  }
  .tab:hover{ color:var(--muted); }
  .tab.on{ color:var(--text); background:var(--bg); }

  /* ── Scroll area ── */
  .scroll{
    flex:1; overflow-y:auto; overflow-x:hidden;
    padding:20px 18px 0;
    display:flex; flex-direction:column; gap:22px;
  }
  .pad{ height:20px; flex-shrink:0; }

  /* ── Sections ── */
  section{ display:flex; flex-direction:column; gap:5px; }

  .section-label{
    font-size:10.5px; font-weight:600; text-transform:uppercase;
    letter-spacing:.06em; color:var(--faint); padding:0 3px;
  }
  .section-hint{
    font-size:11px; color:var(--faint); padding:0 3px; line-height:1.5;
  }

  /* ── Rows block (NO border) ── */
  .rows{
    background:var(--panel);
    border-radius:var(--r);
    overflow:hidden;
  }
  .sep{ height:1px; background:var(--line); margin:0 12px; }

  .row{
    display:flex; align-items:center; justify-content:space-between;
    padding:10px 14px; gap:12px; min-height:42px;
    cursor:default;
  }
  label.row{ cursor:pointer; }

  .row-label{ font-size:13px; font-weight:450; color:var(--text); }
  .meta{ font-size:11px; color:var(--faint); margin-left:7px; }

  /* ── Controls ── */
  .ipt{
    font-size:12.5px; color:var(--text); background:var(--bg);
    border:1px solid var(--line); border-radius:6px;
    padding:4px 9px; outline:none; text-align:right; width:140px;
    transition:border-color .15s;
  }
  .ipt:focus{ border-color:rgba(232,85,53,.4); }

  .sel{
    font-size:12.5px; color:var(--text); background:var(--bg);
    border:1px solid var(--line); border-radius:6px;
    padding:4px 9px; outline:none; -webkit-appearance:none;
    text-align:right; width:auto; max-width:190px; cursor:pointer;
  }
  .sel:focus{ border-color:rgba(232,85,53,.4); }

  .toggle{
    position:relative; width:36px; height:20px;
    background:rgba(0,0,0,0.14); border-radius:10px;
    flex-shrink:0; transition:background .18s; cursor:pointer;
  }
  .toggle.on{ background:var(--coral); }
  .toggle input{ display:none; }
  .knob{
    position:absolute; top:2px; left:2px;
    width:16px; height:16px; background:#fff; border-radius:50%;
    box-shadow:0 1px 4px rgba(0,0,0,.18);
    transition:transform .18s cubic-bezier(.4,0,.2,1);
  }
  .toggle.on .knob{ transform:translateX(16px); }

  .badge{
    font-size:11px; font-weight:500; border-radius:20px;
    padding:2px 9px; color:var(--faint); background:rgba(0,0,0,.05);
  }
  .badge.installed{ color:var(--blue); background:rgba(123,155,210,.12); }

  /* ── Onboarding ── */
  .ob{
    flex:1; display:flex; flex-direction:column;
    padding:36px 22px 24px; gap:28px;
  }
  .ob-intro{ display:flex; flex-direction:column; gap:7px; }
  .ob-intro h1{ font-size:22px; font-weight:700; letter-spacing:-.03em; color:var(--text); }
  .ob-intro p{ font-size:13px; color:var(--muted); line-height:1.7; }

  .perm-list{ display:flex; flex-direction:column; }
  .perm-row{
    display:flex; align-items:center; gap:13px;
    padding:14px 0;
    border-bottom:1px solid var(--line);
  }
  .perm-row:first-child{ border-top:1px solid var(--line); }

  .perm-dot{
    width:7px; height:7px; border-radius:50%;
    background:rgba(0,0,0,.15); flex-shrink:0;
    transition:background .3s;
  }
  .perm-dot.granted{ background:var(--blue); }

  .perm-info{ flex:1; display:flex; flex-direction:column; gap:1px; }
  .perm-info strong{ font-size:13px; font-weight:550; color:var(--text); }
  .perm-info span{ font-size:11px; color:var(--faint); }

  .perm-status{ font-size:11px; font-weight:500; color:var(--faint); }
  .perm-status.ok{ color:var(--blue); }

  .link-btn{
    background:none; border:none; padding:4px 0;
    font-size:12px; font-weight:500; color:var(--coral);
    cursor:pointer; text-decoration:none;
  }
  .link-btn:hover{ opacity:.75; }

  .ob-foot{ display:flex; flex-direction:column; gap:10px; margin-top:auto; }
  .hint{ font-size:11.5px; color:var(--faint); line-height:1.55; }

  /* ── History ── */
  .empty{
    flex:1; display:flex; flex-direction:column; align-items:center;
    justify-content:center; padding:60px 20px; gap:6px; text-align:center;
  }
  .empty p{ font-size:14px; font-weight:500; color:var(--muted); }
  .empty span{ font-size:12px; color:var(--faint); }

  .hist-list{ display:flex; flex-direction:column; }

  .hist-item{
    padding:12px 0;
    border-bottom:1px solid var(--line);
    display:flex; flex-direction:column; gap:5px;
  }
  .hist-item:first-child{ border-top:1px solid var(--line); }

  .hist-meta{ display:flex; align-items:center; gap:6px; }
  .hist-time{ font-size:11px; color:var(--faint); }
  .hist-dur{
    font-size:10.5px; color:var(--faint);
    background:rgba(0,0,0,.05); border-radius:10px; padding:1px 6px;
  }

  .hist-actions{ display:flex; gap:2px; margin-left:auto; }
  .icon-btn{
    width:22px; height:22px; background:none; border:none;
    border-radius:5px; color:var(--faint); cursor:pointer;
    display:flex; align-items:center; justify-content:center;
    transition:background .12s, color .12s;
  }
  .icon-btn svg{ width:9px; height:9px; }
  .icon-btn:hover{ background:rgba(0,0,0,.06); color:var(--muted); }
  .icon-btn.active{ color:var(--amber); }
  .icon-btn.del:hover{ color:var(--coral); }

  .hist-text{
    font-size:13px; color:var(--muted); line-height:1.55;
    word-break:break-word; white-space:pre-wrap;
  }

  /* ── Permission banner ── */
  .banner{
    display:flex; align-items:center; gap:8px;
    background:rgba(232,85,53,0.07); border-radius:8px;
    padding:10px 13px; cursor:pointer;
    font-size:12px; color:var(--muted);
    transition:background .15s;
    border:none; width:100%; text-align:left;
  }
  .banner:hover{ background:rgba(232,85,53,0.12); }
  .banner.ok{ background:rgba(123,155,210,0.07); }
  .banner.ok:hover{ background:rgba(123,155,210,0.12); }
  .banner-dot{
    width:6px; height:6px; border-radius:50%;
    background:var(--coral); flex-shrink:0;
  }
  .banner-dot.ok{ background:var(--blue); }
  .banner span:nth-child(2){ flex:1; }
  .banner-link{ color:var(--coral); font-weight:500; white-space:nowrap; }

  /* ── Primary button ── */
  .btn-primary{
    width:100%; background:var(--coral); color:#fff; border:none;
    border-radius:var(--r); padding:11px; font-size:13.5px; font-weight:600;
    cursor:pointer; letter-spacing:-.01em;
    transition:opacity .15s, transform .1s;
  }
  .btn-primary:hover:not(:disabled){ opacity:.88; }
  .btn-primary:active:not(:disabled){ transform:scale(.98); }
  .btn-primary:disabled{ opacity:.3; cursor:default; }

  kbd{
    display:inline-block; background:rgba(0,0,0,.06);
    border-radius:4px; padding:1px 5px;
    font-size:11px; font-family:inherit; color:var(--muted);
  }

  /* ── Model list (onboarding step 2) ── */
  .model-list{ display:flex; flex-direction:column; }
  .model-row{
    display:flex; align-items:center; gap:12px;
    padding:14px 0; border-bottom:1px solid var(--line);
    flex-wrap:wrap;
  }
  .model-row:first-child{ border-top:1px solid var(--line); }
  .model-info{ flex:1; display:flex; flex-direction:column; gap:2px; }
  .model-info strong{ font-size:13px; font-weight:550; color:var(--text); }
  .model-info span{ font-size:11px; color:var(--faint); }
  .model-action{ display:flex; align-items:center; gap:8px; flex-shrink:0; }

  /* ── Download progress bar ── */
  .dl-inline{ display:flex; align-items:center; gap:7px; }
  .dl-bar-wrap{
    width:80px; height:4px; background:rgba(0,0,0,.08);
    border-radius:2px; overflow:hidden;
  }
  .dl-bar{
    height:100%; background:var(--coral);
    border-radius:2px; transition:width .3s linear;
  }
  .dl-pct{ font-size:11px; color:var(--faint); width:30px; text-align:right; }
  .dl-error{ font-size:11px; color:var(--coral); width:100%; padding-bottom:6px; }

  /* ── Custom words textarea ── */
  .words-block{
    background:var(--panel);
    border-radius:var(--r);
    padding:10px 14px;
    display:flex; flex-direction:column; gap:6px;
    margin-top:1px;
  }
  .words-ta{
    width:100%; resize:none;
    font-size:12.5px; color:var(--muted);
    background:var(--bg); border:1px solid var(--line);
    border-radius:6px; padding:7px 9px;
    outline:none; font-family:inherit; line-height:1.5;
    transition:border-color .15s;
  }
  .words-ta:focus{ border-color:rgba(232,85,53,.4); }
  .words-ta::placeholder{ color:var(--faint); }
  .threshold-ctrl{ display:flex; align-items:center; gap:8px; flex:1; }
  .threshold-ctrl input[type=range]{ flex:1; accent-color:var(--coral); }
  .threshold-val{ font-size:11.5px; color:var(--muted); min-width:28px; text-align:right; }

  /* Onboarding step progress dots */
  .ob-steps{ display:flex; gap:5px; justify-content:center; }
  .ob-step-dot{
    width:5px; height:5px; border-radius:50%;
    background:rgba(0,0,0,.15);
    transition:background .2s;
  }
  .ob-step-dot.active{ background:var(--coral); }
</style>
