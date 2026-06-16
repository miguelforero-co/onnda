<script lang="ts">
  import type { Settings, ModelInfo, DownloadProgress, UpdateStatus } from "$lib/types";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Toggle from "$lib/components/Toggle.svelte";
  import HotkeyRecorder from "$lib/components/HotkeyRecorder.svelte";
  import PermissionRow from "$lib/components/PermissionRow.svelte";
  import ModelCard from "$lib/components/ModelCard.svelte";
  import SectionLabel from "$lib/components/ui/SectionLabel.svelte";
  import { theme, type ThemeMode } from "$lib/stores/theme.svelte";
  const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
    { value: "light", label: "Claro" },
    { value: "dark",  label: "Oscuro" },
    { value: "auto",  label: "Automático" },
  ];

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

  // Supported recognition languages (was +page.svelte LANGUAGES before the shell refactor).
  const LANGUAGES = [
    { value: "auto", label: "Automático" },
    { value: "es",   label: "Español" },
    { value: "en",   label: "English" },
    { value: "pt",   label: "Português" },
    { value: "fr",   label: "Français" },
    { value: "de",   label: "Deutsch" },
  ];

  // Local model list: mirrors the prop, but can be refreshed locally after a
  // destructive clear_models (the parent only re-fetches on download events).
  // The $effect below keeps it in sync with the prop on every change.
  // svelte-ignore state_referenced_locally
  let modelList = $state<ModelInfo[]>(models);
  $effect(() => { modelList = models; });

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
        ? "Estás al día"
        : `Hay una versión nueva disponible (v${st.available_version})`;
    } catch (e) {
      updateMsg = "No se pudo comprobar. Revisa tu conexión e inténtalo de nuevo.";
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
    if (!window.confirm("¿Borrar todo el historial y los audios guardados? Esta acción no se puede deshacer.")) return;
    await invoke("clear_history");
  }

  async function clearModels() {
    if (!window.confirm("¿Borrar los modelos descargados? Tendrás que volver a descargarlos para dictar.")) return;
    await invoke("clear_models");
    // Refresh model state after deletion (local copy; parent re-syncs on download).
    modelList = await invoke<ModelInfo[]>("get_models");
  }
</script>

<div class="screen">
  <h1 class="page-title">Ajustes</h1>

  <!-- ── Apariencia (onnda theme selector) ── -->
  <section class="section">
    <SectionLabel text="Apariencia" />
    <div class="card">
      <div class="theme-row">
        <span class="row-label">Apariencia</span>
        <div class="seg">
          {#each THEME_OPTIONS as opt}
            <button
              class="seg-btn"
              class:on={theme.mode === opt.value}
              onclick={() => theme.set(opt.value)}
            >{opt.label}</button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <!-- ── Grabación (D-11 hotkey, D-12 push-to-talk) ── -->
  <section class="section">
    <SectionLabel text="Grabación" />
    <div class="card">
      <div class="row">
        <span class="row-label">Atajo de teclado</span>
        <HotkeyRecorder bind:shortcut={settings.shortcut} onCommitted={() => onSave(true)} />
      </div>
      <div class="sep"></div>
      <Toggle
        id="ptt"
        label="Push to talk"
        bind:checked={settings.push_to_talk}
        onchange={() => onSave()}
      />
    </div>
    <p class="section-hint">
      {settings.push_to_talk
        ? "Mantén presionado para grabar, suelta para transcribir."
        : "Presiona una vez para iniciar, otra para detener."}
    </p>
  </section>

  <!-- ── Reconocimiento (idioma) ── -->
  <section class="section">
    <SectionLabel text="Reconocimiento" />
    <div class="card">
      <div class="row">
        <span class="row-label">Idioma</span>
        <select class="sel" bind:value={settings.selected_language} onchange={() => onSave()}>
          {#each LANGUAGES as l}<option value={l.value}>{l.label}</option>{/each}
        </select>
      </div>
    </div>
    <p class="section-hint">Elige el modelo activo en "Modelos".</p>
  </section>

  <!-- ── Sonidos (D-07) + Pausar multimedia (D-08) ── -->
  <section class="section">
    <SectionLabel text="Sonidos" />
    <div class="card">
      <Toggle
        id="sounds-enabled"
        label="Sonidos"
        bind:checked={settings.sounds_enabled}
        onchange={() => onSave()}
      />
      <div class="sep"></div>
      <Toggle
        id="pause-media"
        label="Pausar multimedia al grabar"
        bind:checked={settings.pause_media}
        onchange={() => onSave()}
      />
    </div>
    <p class="section-hint">Reproduce un sonido al iniciar, terminar y cancelar el dictado.</p>
  </section>

  <!-- ── Aprendizaje (Phase 3 — auto-learn from corrections) ── -->
  <section class="section">
    <SectionLabel text="Aprendizaje" />
    <div class="card">
      <Toggle
        id="auto-learn"
        label="Aprender de mis correcciones"
        bind:checked={settings.auto_learn}
        onchange={() => onSave()}
      />
    </div>
    <p class="section-hint">Cuando corriges una transcripción y repites la misma corrección, Voz Local crea una regla para aplicarla sola. Edita una transcripción en "Transcripciones".</p>
  </section>

  <!-- ── Sistema (D-10 launch-at-login) ── -->
  <section class="section">
    <SectionLabel text="Sistema" />
    <div class="card">
      <Toggle
        id="autostart"
        label="Iniciar con el sistema"
        bind:checked={settings.autostart}
        onchange={() => onSave()}
      />
    </div>
  </section>

  <!-- ── Permisos (D-09) — live, reuses parent polling ── -->
  <section class="section">
    <SectionLabel text="Permisos" />
    <div class="perm-list">
      <PermissionRow
        label="Micrófono"
        description="Necesario para grabar tu voz."
        granted={micGranted}
        onOpen={() => invoke("open_microphone_settings")}
      />
      <PermissionRow
        label="Accesibilidad"
        description="Necesario para pegar el texto dictado."
        granted={a11yGranted}
        onOpen={() => invoke("open_accessibility_settings")}
      />
    </div>
  </section>

  <!-- ── Modelos (D-13) — cards incl. Parakeet "Próximamente" ── -->
  <section class="section">
    <SectionLabel text="Modelos" />
    <div class="model-list">
      {#each modelList as m (m.id)}
        <ModelCard
          model={m}
          comingSoon={m.coming_soon}
          selected={settings.selected_model === m.id}
          progress={downloadProgress[m.id]}
          error={downloadErrors[m.id]}
          onDownload={() => onDownload(m.id)}
          onSelect={() => { settings.selected_model = m.id; onSave(); }}
        />
      {/each}
    </div>
  </section>

  <!-- ── Actualizaciones (D-14) ── -->
  <section class="section">
    <SectionLabel text="Actualizaciones" />
    <div class="card">
      <div class="row">
        <span class="row-label">Versión</span>
        <span class="version-value">v{appVersion}{#if buildHash} · {buildHash}{/if}</span>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Buscar actualizaciones</span>
        <div class="update-action">
          {#if updateMsg}<span class="update-msg">{updateMsg}</span>{/if}
          <button class="btn-primary" onclick={checkUpdates} disabled={checkingUpdates}>
            {checkingUpdates ? "Comprobando…" : "Buscar actualizaciones"}
          </button>
        </div>
      </div>
    </div>
  </section>

  <!-- ── Datos (D-15) — destructive actions confirm-gated ── -->
  <section class="section">
    <SectionLabel text="Datos" />
    <div class="card">
      <div class="row">
        <span class="row-label">Carpeta de datos</span>
        <button class="link-btn" onclick={revealData}>Abrir carpeta de datos</button>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Historial y audios</span>
        <button class="destruct-btn" onclick={clearHistory}>Borrar historial y audios</button>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Modelos descargados</span>
        <button class="destruct-btn" onclick={clearModels}>Borrar modelos descargados</button>
      </div>
    </div>
    <p class="section-hint">Las acciones de borrado son permanentes y piden confirmación.</p>
  </section>
</div>

<style>
  /* ── Root container: 81px top offset matches Home / Diccionario / Importar ── */
  .screen {
    padding: 81px var(--s10) var(--s10);
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

  /* ── Language select ── */
  .sel {
    font-size: 14px;
    font-family: var(--font-sans);
    color: var(--text);
    background-color: var(--bg);
    /* chevron so it reads as a dropdown, not an input */
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888888' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 10px center;
    background-size: 12px;
    border: 1px solid var(--line);
    border-radius: var(--r-nav);
    padding: 8px 34px 8px 12px;
    outline: none;
    appearance: none;
    -webkit-appearance: none;
    cursor: pointer;
    transition: border-color .15s;
  }
  .sel:hover { border-color: var(--line-strong); }
  .sel:focus { border-color: var(--text-muted); }

  /* ── Permisos ── */
  .perm-list { display: flex; flex-direction: column; }

  /* ── Modelos ── */
  .model-list { display: flex; flex-direction: column; gap: var(--s3); }

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

  /* ── Primary button (check updates) ── */
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

  /* ── Apariencia (onnda theme selector) ── */
  .theme-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s4);
    min-height: 42px;
  }
  .seg {
    display: inline-flex;
    background: var(--bg);
    border-radius: var(--r-nav);
    padding: 2px;
    gap: 2px;
  }
  .seg-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--text-muted);
    padding: 6px 12px;
    border-radius: 6px;
    transition: background .12s, color .12s;
  }
  .seg-btn.on { background: var(--nav-active-bg); color: var(--nav-active-ink); }
</style>
