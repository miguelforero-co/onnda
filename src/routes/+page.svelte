<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { track } from "$lib/analytics";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, onDestroy } from "svelte";
  import "$lib/styles/tokens.css";
  import "@fontsource/goudy-bookletter-1911";
  import type { Settings, HistoryEntry, ModelInfo, DownloadProgress, View } from "$lib/types";

  import Sidebar from "$lib/components/Sidebar.svelte";
  import PermissionRow from "$lib/components/PermissionRow.svelte";
  import ModelCard from "$lib/components/ModelCard.svelte";
  import Home from "$lib/sections/Home.svelte";
  import Transcripciones from "$lib/sections/Transcripciones.svelte";
  import Importar from "$lib/sections/Importar.svelte";
  import Diccionario from "$lib/sections/Diccionario.svelte";
  import Ajustes from "$lib/sections/Ajustes.svelte";
  import Wordmark from "$lib/components/ui/Wordmark.svelte";
  import { userName } from "$lib/stores/userName.svelte";
  import { subscribe } from "$lib/subscribe";
  import { check as checkUpdate, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";

  // Window drag from the content's top header band (the title bar is hidden, so
  // the top ~56px acts as the drag handle — like Wispr Flow). Uses the same
  // explicit startDragging() that already works on the sidebar; skips real
  // controls so clicks/selection still work. Double-click maximizes (macOS).
  const TITLEBAR_BAND = 56;
  function inTopBand(e: MouseEvent) {
    const el = e.currentTarget as HTMLElement;
    return e.clientY - el.getBoundingClientRect().top <= TITLEBAR_BAND;
  }
  function contentDrag(e: MouseEvent) {
    if (e.button !== 0 || !inTopBand(e)) return;
    if ((e.target as HTMLElement).closest("button,a,input,textarea,select,kbd,[contenteditable]")) return;
    getCurrentWindow().startDragging().catch(() => {});
  }
  function contentDblClick(e: MouseEvent) {
    if (!inTopBand(e)) return;
    getCurrentWindow().toggleMaximize().catch(() => {});
  }

  let settings = $state<Settings>({
    shortcut: "Alt+Space", push_to_talk: true, selected_language: "auto",
    selected_model: "large-v3-turbo", autostart: false,
    onboarding_done: false, widget_position: "center", custom_words: "",
    word_correction_threshold: 0.85,
    sounds_enabled: true,
    sound_on_listen: true, sound_on_stop: true, sound_on_cancel: true,
    pause_media: false, dictionary: [], replacements: [],
    auto_learn: true, learned_corrections: [],
    mic_sensitivity: 1.0,
    analytics_enabled: false,
  });
  let view = $state<View>("home");
  // onboarding steps: "welcome" (name + optional email) → "perms" → "models" → "analytics"
  let obStep = $state<"welcome" | "perms" | "models" | "analytics">("welcome");
  let obName = $state("");   // nombre para el saludo (requerido para continuar)
  let obEmail = $state("");  // correo opcional → solo se usa para la lista de lanzamiento
  let models = $state<ModelInfo[]>([]);
  // ¿El modelo seleccionado está disponible (descargado / nativo)? Gate del paso 3.
  const selectedModelReady = $derived(
    models.find((m) => m.id === settings.selected_model)?.downloaded ?? false,
  );
  let history = $state<HistoryEntry[]>([]);
  let micGranted = $state(false);
  let a11yGranted = $state(false);
  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let initialized = false;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  // Download state keyed by model_id
  let downloadProgress = $state<Record<string, DownloadProgress>>({});
  let downloadErrors = $state<Record<string, string>>({});
  let appVersion = $state("");
  let buildHash = $state("");
  let modelReady = $state(true); // assume ready until checked; avoids flash
  let warnMsg = $state(""); // HARDEN-05: transcribe-warning toast (auto-clears)
  let warnTimer: ReturnType<typeof setTimeout> | null = null;
  let ready = $state(false); // flash guard: nothing renders until init resolves

  // ── Auto-update (Tauri updater plugin) ──────────────────────────────────────
  let pendingUpdate: Update | null = null;       // el Update si hay uno disponible
  let updateVersion = $state("");                // versión disponible (muestra banner)
  let updating = $state(false);                  // descarga/instalación en curso
  let updatePct = $state(0);                      // % de descarga
  let updateError = $state("");

  // Comprueba si hay update (silencioso: offline / sin manifest aún = no banner).
  async function checkForUpdate() {
    try {
      const upd = await checkUpdate();
      if (upd?.available) { pendingUpdate = upd; updateVersion = upd.version; }
    } catch { /* sin red / sin release / etc. → no molestar */ }
  }

  // Descarga + instala + reinicia in-app.
  async function installUpdate() {
    if (!pendingUpdate || updating) return;
    updating = true; updateError = ""; updatePct = 0;
    try {
      let total = 0, got = 0;
      await pendingUpdate.downloadAndInstall((e) => {
        if (e.event === "Started") total = e.data.contentLength ?? 0;
        else if (e.event === "Progress") { got += e.data.chunkLength; updatePct = total ? Math.round((got / total) * 100) : 0; }
        else if (e.event === "Finished") updatePct = 100;
      });
      await relaunch();
    } catch (e) {
      updateError = "Update failed. Try again later.";
      updating = false;
      console.error("[update]", e);
    }
  }

  const unlisten: (() => void)[] = [];

  // Onboarding step 1: guarda el nombre para el saludo y, si dieron correo,
  // lo manda (opt-in) a la lista de lanzamiento. El correo NO se guarda local —
  // su único propósito es la suscripción.
  function finishWelcome() {
    userName.value = obName.trim();
    const email = obEmail.trim();
    if (email) void subscribe(email, obName.trim());
    obStep = "perms";
    pollInterval = setInterval(checkPerms, 1500);
  }

  async function init() {
    settings = await invoke("get_settings");
    models = await invoke("get_models");
    await checkPerms();

    // Si el motor seleccionado es Apple, caliéntalo al arrancar para que el primer
    // dictado no sufra el cold-start del modelo on-device de macOS.
    if (settings.selected_model === "apple-speech") {
      invoke("warm_apple_engine").catch(() => {});
    }

    if (!settings.onboarding_done) {
      // Empieza en el paso "welcome" (nombre + correo). El polling de permisos
      // arranca al pasar al paso "perms" (finishWelcome).
      view = "onboarding";
    } else {
      view = "home";
      history = await invoke("get_history");
      // Keep checking permissions so the warning banner updates live
      pollInterval = setInterval(checkPerms, 3000);
      // HARDEN-04: check if the default model is available; show banner if not.
      // Only when onboarding is done (during onboarding the user downloads one).
      try {
        const st = await invoke<{ ready: boolean; model_id: string }>("check_model_status");
        modelReady = st.ready;
      } catch { modelReady = true; } // on invoke error, don't bother the user
    }
  }

  onMount(async () => {
    appVersion = await invoke<string>("get_app_version").catch(() => "");
    buildHash = await invoke<string>("get_build_hash").catch(() => "");

    // Register event listeners immediately, before init resolves.
    unlisten.push(
      // Always re-pull history so the shared store is current regardless of
      // which section is open (real-time refresh, no view-conditional delay).
      await listen("transcription-done", async () => {
        history = await invoke("get_history");
      }),
      // A file transcription can finish while any section is active (the
      // upload lives in Importar but the user may navigate away). Re-pull
      // history so the new file entry is always present in the shared store.
      await listen("file-transcribe-done", async () => {
        history = await invoke("get_history");
      }),
      await listen<DownloadProgress>("download-progress", ({ payload }) => {
        downloadProgress = { ...downloadProgress, [payload.model_id]: payload };
      }),
      // HARDEN-05: show a non-blocking toast when a transcription segment fails
      await listen<string>("transcribe-warning", ({ payload }) => {
        if (warnTimer) clearTimeout(warnTimer);
        warnMsg = payload;
        warnTimer = setTimeout(() => { warnMsg = ""; warnTimer = null; }, 4000);
      }),
      await listen<{ event: string; props: Record<string, unknown> }>("analytics-event", (e) => {
        void track(e.payload.event, e.payload.props);
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
        // HARDEN-04: hide the "no model" banner once a download finishes
        modelReady = true;
      }),
    );

    await init();

    initialized = true;
    ready = true;

    // Comprobar updates en segundo plano (no bloquea el arranque).
    void checkForUpdate();
  });

  onDestroy(() => {
    unlisten.forEach(fn => fn());
    if (pollInterval) clearInterval(pollInterval);
    if (saveTimer) clearTimeout(saveTimer);
    if (warnTimer) clearTimeout(warnTimer);
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

  // Accessibility: the prompt registers the app in the list (so the toggle
  // appears) and shows the system dialog; then we open the pane so the user can
  // flip it. Just opening the pane alone leaves the app absent from the list.
  async function requestA11y() {
    await invoke("request_accessibility").catch(() => {});
    await invoke("open_accessibility_settings").catch(() => {});
    checkPerms();
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
    view = "home";
    // Resume live permission polling for the in-app banner.
    pollInterval = setInterval(checkPerms, 3000);
  }

  async function goHistory() { history = await invoke("get_history"); }
  // Auto-learn (Phase 3) mutates settings on the backend (learned_corrections +
  // replacements). Re-pull so the frontend copy doesn't overwrite them on next save.
  // Cancel any pending debounced save first — it captured the pre-learn object and
  // would clobber the just-learned rules when it fires.
  async function reloadSettings() {
    if (saveTimer) { clearTimeout(saveTimer); saveTimer = null; }
    settings = await invoke("get_settings");
  }
</script>

{#if ready}
{#if view === "onboarding"}
  <!-- ── ONBOARDING (precedes the shell) ── -->
  <div class="ob">
   <div class="ob-inner">
    <div class="ob-brand"><Wordmark /></div>

    {#if obStep === "welcome"}
      <!-- Step 1: name (for the greeting) + optional email (launch list) -->
      <div class="ob-intro">
        <h1>Welcome to onnda</h1>
        <p>Local voice-to-text on your Mac.<br>What should we call you?</p>
      </div>

      <div class="ob-form">
        <label class="ob-field">
          <span class="ob-label">Name</span>
          <input
            class="ob-input"
            type="text"
            placeholder="Your name"
            bind:value={obName}
            onkeydown={(e) => { if (e.key === "Enter" && obName.trim()) finishWelcome(); }}
          />
        </label>
        <label class="ob-field">
          <span class="ob-label">Email <span class="ob-optional">— optional, for launch updates</span></span>
          <input
            class="ob-input"
            type="email"
            placeholder="you@example.com"
            bind:value={obEmail}
            onkeydown={(e) => { if (e.key === "Enter" && obName.trim()) finishWelcome(); }}
          />
        </label>
      </div>

      <div class="ob-foot">
        <p class="hint">No account, no password. Your name stays on this Mac; the email is only used to send launch updates if you enter one.</p>
        <button class="btn-primary" disabled={!obName.trim()} onclick={finishWelcome}>
          Continue
        </button>
      </div>

    {:else if obStep === "perms"}
      <div class="ob-intro">
        <h1>Permissions</h1>
        <p>onnda needs two permissions to get started.</p>
      </div>

      <div class="perm-list">
        <PermissionRow
          label="Microphone"
          description="To capture your voice"
          granted={micGranted}
          onOpen={() => invoke("open_microphone_settings")}
        />
        <PermissionRow
          label="Accessibility"
          description="To paste where you type"
          granted={a11yGranted}
          onOpen={requestA11y}
        />
      </div>

      <div class="ob-foot">
        {#if !a11yGranted && micGranted}
          <p class="hint">Without accessibility, text is copied to the clipboard automatically.</p>
        {/if}
        <button class="btn-primary" disabled={!micGranted}
                onclick={() => { if (pollInterval) { clearInterval(pollInterval); pollInterval = null; } obStep = "models"; }}>
          {micGranted ? "Continue" : "Waiting for microphone permission…"}
        </button>
      </div>

    {:else if obStep === "models"}
      <!-- Step 3: choose a model (Apple needs no download; Whisper downloads). -->
      <div class="ob-intro">
        <h1>Choose a model</h1>
        <p>Apple's engine is instant and needs no download.<br>Whisper models run offline once downloaded.</p>
      </div>

      <div class="model-list">
        {#each models as m}
          <ModelCard
            model={m}
            selected={settings.selected_model === m.id}
            progress={downloadProgress[m.id]}
            error={downloadErrors[m.id]}
            onDownload={startDownload}
            onSelect={(id) => {
              settings.selected_model = id;
              schedSave();
              if (id === "apple-speech") invoke("warm_apple_engine").catch(() => {});
            }}
          />
        {/each}
      </div>

      <div class="ob-foot">
        <p class="hint">Tap a model to select it. You can change it anytime in Settings.</p>
        <button class="btn-primary" disabled={!selectedModelReady} onclick={() => { schedSave(); obStep = "analytics"; }}>
          {selectedModelReady ? "Continue" : "Select an available model to continue"}
        </button>
      </div>

    {:else}
      <!-- Step 3: analytics consent -->
      <div class="ob-intro">
        <h1>Anonymous usage stats</h1>
        <p>Allow onnda to send anonymous usage stats?<br>We never send what you dictate.</p>
      </div>

      <div class="ob-analytics">
        <p class="analytics-detail">We only record events like "transcription completed" without any text. You can change this in Settings at any time.</p>
      </div>

      <div class="ob-foot">
        <button class="btn-primary" onclick={() => { settings.analytics_enabled = true; finishOnboarding(); }}>
          Allow
        </button>
        <button class="btn-secondary" onclick={() => { settings.analytics_enabled = false; finishOnboarding(); }}>
          No thanks
        </button>
      </div>
    {/if}

    <div class="ob-steps">
      <div class="ob-step-dot" class:active={obStep === "welcome"}></div>
      <div class="ob-step-dot" class:active={obStep === "perms"}></div>
      <div class="ob-step-dot" class:active={obStep === "models"}></div>
      <div class="ob-step-dot" class:active={obStep === "analytics"}></div>
    </div>
   </div>
  </div>
{:else}
  <!-- ── SHELL (sidebar + content fill to the top; traffic lights float over the
       top-left, like Wispr Flow). The top header band is a drag handle. ── -->
  <div class="shell">
    <Sidebar bind:view />
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <main class="content" onmousedown={contentDrag} ondblclick={contentDblClick}>
      {#if !a11yGranted}
        <!-- Accessibility missing: dictation copies to clipboard but can't auto-paste
             (Cmd+V via CGEvent needs Accessibility). Prompt the user to grant it. -->
        <div class="a11y-banner">
          <span>onnda needs <strong>Accessibility</strong> permission to paste dictation into other apps. Until then, text is copied to the clipboard but won't paste automatically.</span>
          <button onclick={requestA11y}>Enable</button>
        </div>
      {/if}
      {#if !modelReady}
        <!-- HARDEN-04: actionable banner when no model is downloaded -->
        <div class="model-banner">
          <span>No voice model downloaded. Download one to start dictating.</span>
          <button onclick={() => { view = "ajustes"; }}>Download model</button>
        </div>
      {/if}
      {#if updateVersion}
        <!-- Auto-update: descarga + instala + reinicia sin tocar el DMG. -->
        <div class="update-banner">
          {#if updating}
            <span>Updating onnda… {updatePct}%</span>
          {:else if updateError}
            <span>{updateError}</span>
            <button onclick={installUpdate}>Retry</button>
          {:else}
            <span>onnda <strong>{updateVersion}</strong> is available.</span>
            <button onclick={installUpdate}>Update &amp; restart</button>
          {/if}
        </div>
      {/if}
      {#if view === "home"}
        <Home {history} />
      {/if}
      {#if view === "transcripciones"}
        <Transcripciones {history} onRefresh={goHistory} onSettingsChanged={reloadSettings} />
      {/if}
      {#if view === "importar"}
        <Importar {history} onRefresh={goHistory} />
      {/if}
      {#if view === "diccionario"}
        <Diccionario {settings} onSave={() => schedSave()} />
      {/if}
      {#if view === "ajustes"}
        <Ajustes
          {settings} {models} {downloadProgress} {downloadErrors}
          {micGranted} {a11yGranted} {appVersion} {buildHash}
          onSave={(sc) => schedSave(sc)}
          onDownload={startDownload}
          onCheckPerms={checkPerms}
        />
      {/if}
    </main>
  </div>
{/if}

{/if}

{#if warnMsg}
  <!-- HARDEN-05: transcribe-warning toast (auto-disappears after 4 s) -->
  <div class="warn-toast">{warnMsg}</div>
{/if}

<style>
  /* ── Shell: black base (--shell) shows through as a 2px frame (padding) and a
       2px seam (gap) between two rounded panels — the sidebar and the content.
       Matches Figma: panels float on the black window base. ── */
  .shell {
    display: flex;
    gap: var(--seam);
    height: 100vh;
    background: var(--shell);
    border-radius: var(--r-window);
    overflow: hidden;
  }

  .content {
    flex: 1 0 0; min-width: 0; min-height: 0;
    overflow-y: auto; overflow-x: hidden;
    background-color: var(--bg);
    /* notebook dot texture */
    background-image: radial-gradient(var(--dot-grid) 1px, transparent 1.5px);
    background-size: var(--dot-pitch) var(--dot-pitch);
    border-radius: var(--r-card);
  }

  /* ── Onboarding: columna acotada. .ob es el scroll-container a pantalla
       completa; .ob-inner se centra con margin:auto (centra si cabe, hace scroll
       desde arriba si el contenido es más alto que la ventana — sin recortar el
       botón Continue). ── */
  .ob {
    height: 100vh; overflow-y: auto; display: flex;
  }
  .ob-inner {
    margin: auto; width: 100%; max-width: 380px;
    display: flex; flex-direction: column;
    padding: var(--s8) var(--s8); gap: var(--s6); box-sizing: border-box;
  }
  .ob-brand { display: flex; }
  .ob-brand :global(.wordmark) { height: 32px; }

  .ob-intro { display: flex; flex-direction: column; gap: var(--s2); }
  .ob-intro h1 {
    font-family: var(--font-serif);
    font-size: 30px; font-weight: 400; line-height: 1.1;
    color: var(--text);
  }
  .ob-intro p { font-size: 13.5px; color: var(--text-muted); line-height: 1.65; }

  .perm-list { display: flex; flex-direction: column; }
  .model-list { display: flex; flex-direction: column; gap: var(--s2); }

  /* Welcome step: name + optional email */
  .ob-form { display: flex; flex-direction: column; gap: var(--s4); }
  .ob-field { display: flex; flex-direction: column; gap: var(--s1); }
  .ob-label {
    font-size: 11px; font-weight: 600; letter-spacing: .04em; text-transform: uppercase;
    color: var(--text-section);
  }
  .ob-optional { font-weight: 400; letter-spacing: 0; text-transform: none; color: var(--text-muted); }
  .ob-input {
    width: 100%; box-sizing: border-box;
    background: var(--bg); color: var(--text);
    border: 1px solid var(--line); border-radius: var(--r-nav);
    padding: 11px 13px; font-size: 14px; font-family: var(--font-sans);
    transition: border-color .15s;
  }
  .ob-input::placeholder { color: var(--text-muted); }
  .ob-input:focus { outline: none; border-color: var(--text); }

  .ob-analytics {
    background: var(--surface);
    border-radius: var(--r-card);
    padding: var(--s4);
  }
  .analytics-detail {
    font-size: 12.5px;
    color: var(--text-muted);
    line-height: 1.65;
  }

  .btn-secondary {
    width: 100%; background: transparent; color: var(--text-muted); border: none;
    border-radius: var(--r-nav); padding: var(--s3); font-size: 13px; font-weight: 500;
    cursor: pointer;
    transition: color .15s;
  }
  .btn-secondary:hover { color: var(--text); }

  .ob-foot { display: flex; flex-direction: column; gap: var(--s3); margin-top: var(--s1); }
  .hint { font-size: 12px; color: var(--text-muted); line-height: 1.55; }

  .btn-primary {
    width: 100%; background: var(--nav-active-bg); color: var(--nav-active-ink); border: none;
    border-radius: var(--r-nav); padding: 12px; font-size: 14px; font-weight: 600;
    cursor: pointer;
    transition: opacity .15s, transform .1s;
  }
  .btn-primary:hover:not(:disabled) { opacity: .9; transform: translateY(-1px); }
  .btn-primary:active:not(:disabled) { transform: scale(.98); }
  .btn-primary:disabled {
    cursor: default;
    background: var(--surface); color: var(--text-muted);
    box-shadow: inset 0 0 0 1px var(--line);
  }

  /* Onboarding step progress dots — footer indicator */
  .ob-steps { display: flex; gap: var(--s1); justify-content: flex-start; margin-top: var(--s2); }
  .ob-step-dot {
    width: 5px; height: 5px; border-radius: 50%;
    background: var(--text); opacity: .18;
    transition: opacity .2s;
  }
  .ob-step-dot.active { opacity: 1; }

  /* HARDEN-04: no-model banner */
  /* Accessibility-missing banner (onnda) — shows on any screen until granted. */
  .a11y-banner {
    display: flex; align-items: center; gap: var(--s3);
    margin: var(--s4) var(--s10) 0;
    padding: var(--s3) var(--s4);
    border-radius: var(--r-card);
    background: var(--surface);
    border: 1px solid var(--danger);
  }
  .a11y-banner span { flex: 1; font-size: 14px; color: var(--text); line-height: 1.4; }
  .a11y-banner strong { font-weight: 600; }
  .a11y-banner button {
    flex-shrink: 0;
    background: var(--nav-active-bg); color: var(--nav-active-ink); border: none;
    border-radius: var(--r-nav); padding: 8px 16px;
    font-size: 14px; font-weight: 600; cursor: pointer;
    transition: opacity .15s;
  }
  .a11y-banner button:hover { opacity: .9; }

  .model-banner {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 14px;
    margin-bottom: 16px;
    border-radius: var(--r);
    background: var(--surface);
    border: 1px solid var(--danger);
  }
  .model-banner span {
    flex: 1; font-size: 12.5px; color: var(--text); line-height: 1.5;
  }
  .model-banner button {
    flex-shrink: 0;
    background: var(--nav-active-bg); color: var(--nav-active-ink); border: none;
    border-radius: var(--r); padding: 6px 12px;
    font-size: 12px; font-weight: 600; cursor: pointer;
    transition: opacity .15s;
  }
  .model-banner button:hover { opacity: .88; }

  /* Auto-update banner — tono positivo (no destructivo) */
  .update-banner {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 14px;
    margin-bottom: 16px;
    border-radius: var(--r-nav);
    background: var(--surface);
    border: 1px solid var(--line-strong);
  }
  .update-banner span { flex: 1; font-size: 12.5px; color: var(--text); line-height: 1.5; }
  .update-banner strong { font-weight: 600; }
  .update-banner button {
    flex-shrink: 0;
    background: var(--nav-active-bg); color: var(--nav-active-ink); border: none;
    border-radius: var(--r-nav); padding: 6px 12px;
    font-size: 12px; font-weight: 600; cursor: pointer;
    transition: opacity .15s;
  }
  .update-banner button:hover { opacity: .88; }

  /* HARDEN-05: transcribe-warning toast */
  .warn-toast {
    position: fixed; bottom: 20px; left: 50%; transform: translateX(-50%);
    padding: 8px 16px;
    border-radius: var(--r);
    background: rgba(30, 30, 30, 0.92);
    border: 1px solid var(--line);
    font-size: 12.5px; color: var(--text);
    pointer-events: none;
    z-index: 999;
    animation: toast-in .2s ease;
  }
  @keyframes toast-in {
    from { opacity: 0; transform: translateX(-50%) translateY(6px); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0); }
  }
</style>
