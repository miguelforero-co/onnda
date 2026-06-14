<script lang="ts">
  import type { Settings, ModelInfo, DownloadProgress, UpdateStatus } from "$lib/types";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Toggle from "$lib/components/Toggle.svelte";
  import HotkeyRecorder from "$lib/components/HotkeyRecorder.svelte";
  import PermissionRow from "$lib/components/PermissionRow.svelte";
  import ModelCard from "$lib/components/ModelCard.svelte";

  // Prop contract from 01-03 (Ajustes stub).
  let {
    settings,
    models,
    downloadProgress,
    downloadErrors,
    micGranted,
    a11yGranted,
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

  // Models that can be the active dictation model (downloaded, not coming-soon).
  const downloadedModels = $derived(modelList.filter((m) => m.downloaded && !m.coming_soon));

  // Refresh permissions the moment the section mounts (parent keeps polling every 3s).
  onMount(() => onCheckPerms());

  // ── Actualizaciones (D-14) ──
  let updateMsg = $state("");
  let updateVersion = $state("");
  let checkingUpdates = $state(false);

  async function checkUpdates() {
    checkingUpdates = true;
    updateMsg = "";
    try {
      const st = await invoke<UpdateStatus>("check_for_updates");
      updateVersion = st.current_version;
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

<h1 class="page-title">Ajustes</h1>

<!-- ── Grabación (D-11 hotkey, D-12 push-to-talk) ── -->
<section>
  <h2 class="section-label">Grabación</h2>
  <div class="rows">
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

<!-- ── Reconocimiento (idioma + modelo activo) ── -->
<section>
  <h2 class="section-label">Reconocimiento</h2>
  <div class="rows">
    <div class="row">
      <span class="row-label">Idioma</span>
      <select class="sel" bind:value={settings.selected_language} onchange={() => onSave()}>
        {#each LANGUAGES as l}<option value={l.value}>{l.label}</option>{/each}
      </select>
    </div>
    <div class="sep"></div>
    <div class="row">
      <span class="row-label">Modelo activo</span>
      <select class="sel" bind:value={settings.selected_model} onchange={() => onSave()}>
        {#each downloadedModels as m}
          <option value={m.id}>{m.name} · {m.size_mb} MB</option>
        {/each}
      </select>
    </div>
  </div>
  <p class="section-hint">Elige entre los modelos descargados. Gestiona descargas en "Modelos".</p>
</section>

<!-- ── Sonidos (D-07) + Pausar multimedia (D-08) ── -->
<section>
  <h2 class="section-label">Sonidos</h2>
  <div class="rows">
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

<!-- ── Sistema (D-10 launch-at-login) ── -->
<section>
  <h2 class="section-label">Sistema</h2>
  <div class="rows">
    <Toggle
      id="autostart"
      label="Iniciar con el sistema"
      bind:checked={settings.autostart}
      onchange={() => onSave()}
    />
  </div>
</section>

<!-- ── Permisos (D-09) — live, reuses parent polling ── -->
<section>
  <h2 class="section-label">Permisos</h2>
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
<section>
  <h2 class="section-label">Modelos</h2>
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
<section>
  <h2 class="section-label">Actualizaciones</h2>
  <div class="rows">
    <div class="row">
      <span class="row-label">Versión {updateVersion || "actual"}</span>
      <div class="update-action">
        {#if updateMsg}<span class="update-msg">{updateMsg}</span>{/if}
        <button class="link-btn" onclick={checkUpdates} disabled={checkingUpdates}>
          {checkingUpdates ? "Comprobando…" : "Buscar actualizaciones"}
        </button>
      </div>
    </div>
  </div>
</section>

<!-- ── Datos (D-15) — destructive actions confirm-gated ── -->
<section>
  <h2 class="section-label">Datos</h2>
  <div class="rows">
    <div class="row">
      <span class="row-label">Carpeta de datos</span>
      <button class="link-btn" onclick={revealData}>Abrir carpeta de datos</button>
    </div>
    <div class="sep"></div>
    <div class="row">
      <span class="row-label">Historial y audios</span>
      <button class="link-btn danger" onclick={clearHistory}>Borrar historial y audios</button>
    </div>
    <div class="sep"></div>
    <div class="row">
      <span class="row-label">Modelos descargados</span>
      <button class="link-btn danger" onclick={clearModels}>Borrar modelos descargados</button>
    </div>
  </div>
  <p class="section-hint">Las acciones de borrado son permanentes y piden confirmación.</p>
</section>

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }

  section { margin-top: 22px; display: flex; flex-direction: column; gap: 8px; }

  .section-label {
    font-size: 10.5px; font-weight: 600; text-transform: uppercase;
    letter-spacing: .06em; color: var(--faint); padding: 0 3px;
  }
  .section-hint {
    font-size: 11px; color: var(--faint); padding: 0 3px; line-height: 1.5;
  }

  .rows {
    background: var(--panel);
    border-radius: var(--r);
    overflow: hidden;
  }
  .sep { height: 1px; background: var(--line); margin: 0 12px; }

  .row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 14px; gap: 12px; min-height: 42px;
    cursor: default;
  }
  .row-label { font-size: 13px; font-weight: 450; color: var(--text); }

  .sel {
    font-size: 12.5px; color: var(--text); background: var(--bg);
    border: 1px solid var(--line); border-radius: 6px;
    padding: 4px 9px; outline: none; appearance: none; -webkit-appearance: none;
    text-align: right; width: auto; max-width: 190px; cursor: pointer;
    transition: border-color .15s;
  }
  .sel:focus { border-color: rgba(232,85,53,.4); }

  /* ── Permisos ── */
  .perm-list { display: flex; flex-direction: column; }

  /* ── Modelos ── */
  .model-list { display: flex; flex-direction: column; gap: 8px; }

  /* ── Actualizaciones / Datos ── */
  .update-action { display: flex; align-items: center; gap: 12px; }
  .update-msg { font-size: 12px; color: var(--muted); }

  .link-btn {
    background: none; border: none; padding: 4px 0;
    font-size: 12px; font-weight: 450; color: var(--coral);
    cursor: pointer; text-decoration: none;
  }
  .link-btn:hover { opacity: .75; }
  .link-btn:disabled { color: var(--faint); cursor: default; opacity: .7; }
  /* Destructive actions reuse coral per UI-SPEC (no separate red); confirm gates intent. */
  .link-btn.danger { color: var(--coral); }
</style>
