<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import "$lib/styles/tokens.css";
  import type { Settings, HistoryEntry, ModelInfo, DownloadProgress, View } from "$lib/types";

  import Sidebar from "$lib/components/Sidebar.svelte";
  import PermissionRow from "$lib/components/PermissionRow.svelte";
  import ModelCard from "$lib/components/ModelCard.svelte";
  import Home from "$lib/sections/Home.svelte";
  import Transcripciones from "$lib/sections/Transcripciones.svelte";
  import Diccionario from "$lib/sections/Diccionario.svelte";
  import Ajustes from "$lib/sections/Ajustes.svelte";

  let settings = $state<Settings>({
    shortcut: "Alt+Space", push_to_talk: true, selected_language: "auto",
    selected_model: "large-v3-turbo", autostart: false,
    onboarding_done: false, widget_position: "center", custom_words: "",
    word_correction_threshold: 0.85,
    sounds_enabled: true,
    sound_on_listen: true, sound_on_stop: true, sound_on_cancel: true,
    pause_media: false, dictionary: [],
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
  <!-- ── SHELL ── -->
  <div class="shell">
    <Sidebar bind:view {appVersion} {buildHash} />
    <main class="content">
      {#if view === "home"}
        <Home {settings} {history} onNavigate={(v) => (view = v)} />
      {/if}
      {#if view === "transcripciones"}
        <Transcripciones {history} onRefresh={goHistory} />
      {/if}
      {#if view === "diccionario"}
        <Diccionario {settings} onSave={() => schedSave()} />
      {/if}
      {#if view === "ajustes"}
        <Ajustes
          {settings} {models} {downloadProgress} {downloadErrors}
          {micGranted} {a11yGranted} {appVersion}
          onSave={(sc) => schedSave(sc)}
          onDownload={startDownload}
          onCheckPerms={checkPerms}
        />
      {/if}
    </main>
  </div>
{/if}

<style>
  /* ── Shell ── */
  .shell { display: flex; height: 100vh; }
  .content {
    flex: 1; overflow-y: auto; overflow-x: hidden;
    background: var(--bg); padding: 32px;
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
    width: 100%; background: var(--coral); color: #fff; border: none;
    border-radius: var(--r); padding: 11px; font-size: 13.5px; font-weight: 600;
    cursor: pointer; letter-spacing: -.01em;
    transition: opacity .15s, transform .1s;
  }
  .btn-primary:hover:not(:disabled) { opacity: .88; }
  .btn-primary:active:not(:disabled) { transform: scale(.98); }
  .btn-primary:disabled { opacity: .3; cursor: default; }

  /* Onboarding step progress dots */
  .ob-steps { display: flex; gap: 5px; justify-content: center; }
  .ob-step-dot {
    width: 5px; height: 5px; border-radius: 50%;
    background: rgba(0,0,0,.15);
    transition: background .2s;
  }
  .ob-step-dot.active { background: var(--coral); }
</style>
