<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
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
  });
  let view = $state<View>("home");
  // onboarding has two steps: "perms" then "models"
  let obStep = $state<"perms" | "models">("perms");
  let models = $state<ModelInfo[]>([]);
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

  const unlisten: (() => void)[] = [];

  onMount(async () => {
    appVersion = await invoke<string>("get_app_version").catch(() => "");
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
      // HARDEN-04: check if the default model is available; show banner if not.
      // Only when onboarding is done (during onboarding the user downloads one).
      try {
        const st = await invoke<{ ready: boolean; model_id: string }>("check_model_status");
        modelReady = st.ready;
      } catch { modelReady = true; } // on invoke error, don't bother the user
    }

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

    initialized = true;
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

{#if view === "onboarding"}
  <!-- ── ONBOARDING (precedes the shell) ── -->
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
        <PermissionRow
          label="Micrófono"
          description="Para capturar tu voz"
          granted={micGranted}
          onOpen={() => invoke("open_microphone_settings")}
        />
        <PermissionRow
          label="Accesibilidad"
          description="Para pegar donde escribes"
          granted={a11yGranted}
          onOpen={() => invoke("open_accessibility_settings")}
        />
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
          <ModelCard
            model={m}
            progress={downloadProgress[m.id]}
            error={downloadErrors[m.id]}
            onDownload={startDownload}
          />
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
{:else}
  <!-- ── SHELL (sidebar + content fill to the top; traffic lights float over the
       top-left, like Wispr Flow). The top header band is a drag handle. ── -->
  <div class="shell">
    <Sidebar bind:view />
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <main class="content" onmousedown={contentDrag} ondblclick={contentDblClick}>
      {#if !modelReady}
        <!-- HARDEN-04: actionable banner when no model is downloaded -->
        <div class="model-banner">
          <span>No hay un modelo de voz descargado. Descarga uno para empezar a dictar.</span>
          <button onclick={() => { view = "ajustes"; }}>Descargar modelo</button>
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

{#if warnMsg}
  <!-- HARDEN-05: transcribe-warning toast (auto-disappears after 4 s) -->
  <div class="warn-toast">{warnMsg}</div>
{/if}

<style>
  /* ── Shell (sidebar + content fill the window to the very top) ── */
  .shell {
    display: flex;
    height: 100vh;
    background: var(--bg);
    border-radius: var(--r-window);
    overflow: hidden;
  }

  .content {
    flex: 1 0 0; min-width: 0;
    height: 100vh; overflow-y: auto; overflow-x: hidden;
    background: var(--bg);
  }

  /* ── Onboarding (preserved) ── */
  .ob {
    height: 100vh; display: flex; flex-direction: column;
    padding: 36px 22px 24px; gap: 28px;
  }
  .ob-intro { display: flex; flex-direction: column; gap: 7px; }
  .ob-intro h1 { font-size: 22px; font-weight: 600; letter-spacing: -.03em; color: var(--text); }
  .ob-intro p { font-size: 13px; color: var(--muted); line-height: 1.7; }

  .perm-list { display: flex; flex-direction: column; }
  .model-list { display: flex; flex-direction: column; gap: 8px; }

  .ob-foot { display: flex; flex-direction: column; gap: 10px; margin-top: auto; }
  .hint { font-size: 11.5px; color: var(--faint); line-height: 1.55; }

  .btn-primary {
    width: 100%; background: var(--accent); color: #fff; border: none;
    border-radius: var(--r); padding: 11px; font-size: 13.5px; font-weight: 600;
    cursor: pointer; letter-spacing: -.01em;
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

  /* Onboarding step progress dots */
  .ob-steps { display: flex; gap: 5px; justify-content: center; }
  .ob-step-dot {
    width: 5px; height: 5px; border-radius: 50%;
    background: rgba(255,255,255,.18);
    transition: background .2s;
  }
  .ob-step-dot.active { background: var(--accent); box-shadow: 0 0 8px -1px rgba(255,106,61,.7); }

  /* HARDEN-04: no-model banner */
  .model-banner {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 14px;
    margin-bottom: 16px;
    border-radius: var(--r);
    background: rgba(255, 106, 61, 0.12);
    border: 1px solid rgba(255, 106, 61, 0.30);
  }
  .model-banner span {
    flex: 1; font-size: 12.5px; color: var(--text); line-height: 1.5;
  }
  .model-banner button {
    flex-shrink: 0;
    background: var(--accent); color: #fff; border: none;
    border-radius: var(--r); padding: 6px 12px;
    font-size: 12px; font-weight: 600; cursor: pointer;
    transition: opacity .15s;
  }
  .model-banner button:hover { opacity: .88; }

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
