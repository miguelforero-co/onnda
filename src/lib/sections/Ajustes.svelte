<script lang="ts">
  import type { Settings, ModelInfo, DownloadProgress, UpdateStatus } from "$lib/types";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Toggle from "$lib/components/Toggle.svelte";
  import HotkeyRecorder from "$lib/components/HotkeyRecorder.svelte";
  import PermissionRow from "$lib/components/PermissionRow.svelte";
  import SectionLabel from "$lib/components/ui/SectionLabel.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import { userName } from "$lib/stores/userName.svelte";

  // Prop contract from 01-03 (Ajustes stub).
  let {
    settings,
    models,
    downloadProgress,
    downloadErrors,
    micGranted,
    a11yGranted,
    appVersion = "",
    buildHash = "",
    onSave,
    onDownload,
    onCheckPerms,
  }: {
    settings: Settings;
    models: ModelInfo[];
    downloadProgress: Record<string, DownloadProgress>;
    downloadErrors: Record<string, string>;
    micGranted: boolean;
    a11yGranted: boolean;
    appVersion?: string;
    buildHash?: string;
    onSave: (shortcutChanged?: boolean) => void;
    onDownload: (modelId: string) => void;
    onCheckPerms: () => void;
  } = $props();

  // Supported recognition languages.
  const LANGUAGES = [
    { value: "auto", label: "Auto" },
    { value: "es",   label: "Español" },
    { value: "en",   label: "English" },
    { value: "pt",   label: "Português" },
    { value: "fr",   label: "Français" },
    { value: "de",   label: "Deutsch" },
  ];

  // Local model list: mirrors the prop, refreshed after clear_models.
  // svelte-ignore state_referenced_locally
  let modelList = $state<ModelInfo[]>(models);
  $effect(() => { modelList = models; });

  // Derived: the currently selected ModelInfo object.
  const selectedModel = $derived(modelList.find(m => m.id === settings.selected_model));

  // Refresh permissions the moment the section mounts (parent keeps polling every 3s).
  onMount(() => onCheckPerms());

  // ── Actualizaciones (D-14) ──
  let updateMsg = $state("");
  let checkingUpdates = $state(false);

  async function checkUpdates() {
    checkingUpdates = true;
    updateMsg = "";
    try {
      const st = await invoke<UpdateStatus>("check_for_updates");
      updateMsg = st.up_to_date
        ? "You're up to date"
        : `A new version is available (v${st.available_version})`;
    } catch (e) {
      updateMsg = "Could not check. Check your connection and try again.";
      console.error(e);
    } finally {
      checkingUpdates = false;
    }
  }

  // ── Datos (D-15) — destructive actions confirm-gated (T-01-17) ──
  async function revealData() {
    await invoke("reveal_data_dir");
  }

  async function clearHistory() {
    if (!window.confirm("Delete all history and saved audio? This cannot be undone.")) return;
    await invoke("clear_history");
  }

  async function clearModels() {
    if (!window.confirm("Delete downloaded models? You'll need to re-download them to dictate.")) return;
    await invoke("clear_models");
    // Refresh model state after deletion (local copy; parent re-syncs on download).
    modelList = await invoke<ModelInfo[]>("get_models");
  }

  // ── Profile (solo el nombre del saludo; sin cuentas ni contraseñas) ──
  let nameDraft = $state(userName.value);
  let nameSaved = $state(false);
  let nameSavedTimer: ReturnType<typeof setTimeout> | null = null;

  function saveName() {
    userName.set(nameDraft);
    nameDraft = userName.value; // refleja el trim
    nameSaved = true;
    if (nameSavedTimer) clearTimeout(nameSavedTimer);
    nameSavedTimer = setTimeout(() => { nameSaved = false; }, 1800);
  }
</script>

<div class="screen">
  <h1 class="page-title">Settings</h1>

  <!-- ── Profile (solo el nombre del saludo; sin cuentas) ── -->
  <section class="section">
    <SectionLabel text="Profile" />
    <div class="card">
      <div class="row">
        <span class="row-label">Name</span>
        <div class="pw-row">
          <input
            class="pw-input"
            type="text"
            placeholder="Your name"
            bind:value={nameDraft}
            onkeydown={(e) => { if (e.key === "Enter") saveName(); }}
          />
          <button class="btn-primary" onclick={saveName} disabled={nameDraft.trim() === userName.value}>
            {nameSaved ? "Saved" : "Save"}
          </button>
        </div>
      </div>
    </div>
  </section>

  <!-- ── Grabación (D-11 hotkey, D-12 push-to-talk, idioma) ── -->
  <section class="section">
    <SectionLabel text="Recording" />
    <div class="card">
      <div class="row">
        <span class="row-label">Keyboard shortcut</span>
        <HotkeyRecorder bind:shortcut={settings.shortcut} onCommitted={() => onSave(true)} />
      </div>
      <div class="sep"></div>
      <Toggle
        id="ptt"
        label="Push to talk"
        bind:checked={settings.push_to_talk}
        onchange={() => onSave()}
      />
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Animation sensitivity</span>
        <div class="slider-wrap">
          <input
            class="slider"
            type="range" min="0.3" max="2.5" step="0.1"
            bind:value={settings.mic_sensitivity}
            onchange={() => onSave()}
          />
          <span class="slider-val">{settings.mic_sensitivity.toFixed(1)}×</span>
        </div>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Language</span>
        <Select
          bind:value={settings.selected_language}
          options={LANGUAGES.map((l) => ({ label: l.label, value: l.value }))}
          onchange={onSave}
          ariaLabel="Language"
        />
      </div>
    </div>
    <p class="section-hint">
      {settings.push_to_talk
        ? "Hold to record, release to transcribe."
        : "Press once to start, again to stop."}
    </p>
  </section>

  <!-- ── Modelos (D-13) — compact dropdown + single download control ── -->
  <section class="section">
    <SectionLabel text="Models" />
    <div class="card">
      <div class="row">
        <span class="row-label">Active model</span>
        <Select
          bind:value={settings.selected_model}
          options={modelList.map((m) => ({
            label: m.coming_soon
              ? `${m.name} · coming soon`
              : m.size_mb > 0
              ? `${m.name} · ${(m.size_mb / 1024).toFixed(1)} GB`
              : `${m.name} · Native`,
            value: m.id,
            disabled: m.coming_soon || !!m.disabled_reason,
          }))}
          onchange={onSave}
          ariaLabel="Model"
        />
      </div>

      {#if selectedModel}
        <div class="sep"></div>
        <div class="model-dl-row">
          {#if downloadProgress[settings.selected_model]}
            <!-- Downloading: slim progress bar + percent -->
            <div class="dl-bar-wrap">
              <div class="dl-bar" style="width:{downloadProgress[settings.selected_model].percent}%"></div>
            </div>
            <span class="dl-pct">{Math.round(downloadProgress[settings.selected_model].percent)}%</span>
          {:else if selectedModel.downloaded}
            <!-- Already downloaded: subtle dot + label -->
            <span class="dot-on-indicator"></span>
            <span class="dl-status">Downloaded</span>
          {:else}
            <!-- Not downloaded: primary download button -->
            <button class="btn-primary" onclick={() => onDownload(settings.selected_model)}>
              Download
            </button>
          {/if}
          {#if downloadErrors[settings.selected_model]}
            <span class="dl-error">{downloadErrors[settings.selected_model]}</span>
          {/if}
        </div>
      {/if}
    </div>
  </section>

  <!-- ── Sonidos (D-07) + Pausar multimedia (D-08) ── -->
  <section class="section">
    <SectionLabel text="Sounds" />
    <div class="card">
      <Toggle
        id="sounds-enabled"
        label="Sounds"
        bind:checked={settings.sounds_enabled}
        onchange={() => onSave()}
      />
      <div class="sep"></div>
      <Toggle
        id="pause-media"
        label="Pause media while recording"
        bind:checked={settings.pause_media}
        onchange={() => onSave()}
      />
    </div>
    <p class="section-hint">Plays a sound when dictation starts, stops, or is cancelled.</p>
  </section>

  <!-- ── Aprendizaje (Phase 3 — auto-learn from corrections) ── -->
  <section class="section">
    <SectionLabel text="Learning" />
    <div class="card">
      <Toggle
        id="auto-learn"
        label="Learn from my corrections"
        bind:checked={settings.auto_learn}
        onchange={() => onSave()}
      />
    </div>
    <p class="section-hint">When you correct a transcription and repeat the same fix, onnda creates a rule to apply it automatically. Edit a transcription in Transcriptions.</p>
  </section>

  <!-- ── Privacidad (Task 6 analytics opt-in) ── -->
  <section class="section">
    <SectionLabel text="Privacy" />
    <div class="card">
      <Toggle
        id="analytics-enabled"
        label="Anonymous usage stats"
        bind:checked={settings.analytics_enabled}
        onchange={() => onSave()}
      />
    </div>
    <p class="section-hint">We never send what you dictate. Only anonymous events like "transcription completed".</p>
  </section>

  <!-- ── Sistema (D-10 launch-at-login) ── -->
  <section class="section">
    <SectionLabel text="System" />
    <div class="card">
      <Toggle
        id="autostart"
        label="Launch at login"
        bind:checked={settings.autostart}
        onchange={() => onSave()}
      />
    </div>
  </section>

  <!-- ── Permisos (D-09) — live, reuses parent polling ── -->
  <section class="section">
    <SectionLabel text="Permissions" />
    <div class="perm-list">
      <PermissionRow
        label="Microphone"
        description="Required to record your voice."
        granted={micGranted}
        onOpen={() => invoke("open_microphone_settings")}
      />
      <PermissionRow
        label="Accessibility"
        description="Required to paste dictated text."
        granted={a11yGranted}
        onOpen={() => invoke("open_accessibility_settings")}
      />
    </div>
  </section>

  <!-- ── Actualizaciones (D-14) ── -->
  <section class="section">
    <SectionLabel text="Updates" />
    <div class="card">
      <div class="row">
        <span class="row-label">Version</span>
        <span class="version-value">v{appVersion}{#if buildHash} · {buildHash}{/if}</span>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Check for updates</span>
        <div class="update-action">
          {#if updateMsg}<span class="update-msg">{updateMsg}</span>{/if}
          <button class="btn-primary" onclick={checkUpdates} disabled={checkingUpdates}>
            {checkingUpdates ? "Checking…" : "Check for updates"}
          </button>
        </div>
      </div>
    </div>
  </section>

  <!-- ── Datos (D-15) — destructive actions confirm-gated ── -->
  <section class="section">
    <SectionLabel text="Data" />
    <div class="card">
      <div class="row">
        <span class="row-label">Data folder</span>
        <button class="link-btn" onclick={revealData}>Open data folder</button>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">History and audio</span>
        <button class="destruct-btn" onclick={clearHistory}>Clear history and audio</button>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Downloaded models</span>
        <button class="destruct-btn" onclick={clearModels}>Delete downloaded models</button>
      </div>
    </div>
    <p class="section-hint">Delete actions are permanent and will ask for confirmation.</p>
  </section>
</div>

<style>
  /* ── Root container ── */
  .screen {
    padding: var(--screen-top) var(--s10) var(--s10);
    display: flex;
    flex-direction: column;
    gap: var(--s8);
  }

  /* ── Page title: serif, matches system ── */
  .page-title {
    font-family: var(--font-serif);
    font-size: 24px;
    font-weight: 400;
    color: var(--text);
  }

  /* ── Section grouping ── */
  .section {
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }
  .section-hint {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  /* ── Settings card ── */
  .card {
    background: var(--surface);
    border-radius: var(--r-card);
    padding: var(--s4);
    display: flex;
    flex-direction: column;
    gap: var(--s3);
  }

  .sep { height: 1px; background: var(--line); }

  /* ── Row (label + control) ── */
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s3);
    min-height: 42px;
    cursor: default;
  }
  .row-label {
    font-size: 14px;
    font-weight: 450;
    color: var(--text);
  }

  /* ── Model download row ── */
  .model-dl-row {
    display: flex;
    align-items: center;
    gap: var(--s3);
    min-height: 32px;
  }

  /* Downloaded state: subtle dot + label */
  .dot-on-indicator {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--dot-on);
    flex-shrink: 0;
  }
  .dl-status {
    font-size: 13px;
    color: var(--text-muted);
  }

  /* Progress bar */
  .dl-bar-wrap {
    width: 100px;
    height: 4px;
    background: var(--line);
    border-radius: 2px;
    overflow: hidden;
  }
  .dl-bar {
    height: 100%;
    background: var(--dot-on);
    border-radius: 2px;
    transition: width .3s linear;
  }
  .dl-pct {
    font-size: 12px;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    min-width: 32px;
  }

  /* Download error — subtle, no red */
  .dl-error {
    font-size: 12px;
    color: var(--text-muted);
  }

  /* ── Mic-animation sensitivity slider (monochrome) ── */
  .slider-wrap { display: flex; align-items: center; gap: var(--s3); }
  .slider-val {
    font-size: 13px; color: var(--text-muted);
    font-variant-numeric: tabular-nums; min-width: 30px; text-align: right;
  }
  .slider {
    -webkit-appearance: none; appearance: none;
    width: 140px; height: 4px; border-radius: 2px;
    background: var(--line); cursor: pointer; outline: none;
  }
  .slider::-webkit-slider-thumb {
    -webkit-appearance: none; appearance: none;
    width: 16px; height: 16px; border-radius: 50%;
    background: var(--nav-active-bg); cursor: pointer;
  }
  .slider::-moz-range-thumb {
    width: 16px; height: 16px; border: none; border-radius: 50%;
    background: var(--nav-active-bg); cursor: pointer;
  }

  /* ── Permisos ── */
  .perm-list { display: flex; flex-direction: column; }

  /* ── Actualizaciones ── */
  .update-action { display: flex; align-items: center; gap: var(--s3); }
  .update-msg {
    font-size: 12px;
    color: var(--text-muted);
  }
  .version-value {
    font-size: 12px;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  /* ── Primary button (download, check updates) ── */
  .btn-primary {
    background: var(--nav-active-bg);
    color: var(--nav-active-ink);
    border: none;
    border-radius: var(--r-nav);
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 600;
    font-family: var(--font-sans);
    cursor: pointer;
    white-space: nowrap;
    transition: opacity .15s;
  }
  .btn-primary:hover:not(:disabled) { opacity: .9; }
  .btn-primary:disabled { opacity: .35; cursor: default; }

  /* ── Secondary / link button (reveal data folder) ── */
  .link-btn {
    background: transparent;
    border: none;
    padding: 0;
    font-size: 14px;
    font-family: var(--font-sans);
    color: var(--text-muted);
    cursor: pointer;
    transition: color .15s;
  }
  .link-btn:hover { color: var(--text); }

  /* ── Destructive buttons — subtle monochrome, no red ── */
  .destruct-btn {
    background: transparent;
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 4px 10px;
    font-size: 13px;
    font-family: var(--font-sans);
    color: var(--text-muted);
    cursor: pointer;
    transition: background .15s, color .15s;
  }
  .destruct-btn:hover {
    background: var(--surface);
    color: var(--text);
  }

  /* ── Profile (nombre del saludo) ── */
  .pw-row {
    display: flex;
    align-items: center;
    gap: var(--s3);
  }
  .pw-input {
    font-size: 14px;
    font-family: var(--font-sans);
    color: var(--text);
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 6px 10px;
    outline: none;
    width: 160px;
    transition: border-color .15s;
  }
  .pw-input:focus { border-color: var(--text-muted); }

</style>
