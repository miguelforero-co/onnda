<script lang="ts">
  import type { Settings, ModelInfo, DownloadProgress, UpdateStatus } from "$lib/types";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Toggle from "$lib/components/Toggle.svelte";
  import HotkeyRecorder from "$lib/components/HotkeyRecorder.svelte";
  import PermissionRow from "$lib/components/PermissionRow.svelte";
  import SectionLabel from "$lib/components/ui/SectionLabel.svelte";
  import { theme, type ThemeMode } from "$lib/stores/theme.svelte";
  import { auth } from "$lib/auth.svelte";

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
    onLogout,
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
    onLogout: () => void;
  } = $props();

  // Supported recognition languages.
  const LANGUAGES = [
    { value: "auto", label: "Automático" },
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

  // ── Cuenta ──
  let loggingOut = $state(false);
  let newPassword = $state("");
  let pwSaving = $state(false);
  let pwMsg = $state("");
  let pwError = $state(false);

  async function handleLogout() {
    loggingOut = true;
    try {
      await auth.logout();
      onLogout();
    } finally {
      loggingOut = false;
    }
  }

  async function handleChangePassword() {
    if (!newPassword.trim()) return;
    pwSaving = true;
    pwMsg = "";
    pwError = false;
    try {
      await auth.resetPassword(auth.account!.email, newPassword.trim());
      pwMsg = "Contraseña actualizada.";
      newPassword = "";
    } catch (e) {
      pwMsg = "No se pudo actualizar. Intenta de nuevo.";
      pwError = true;
      console.error(e);
    } finally {
      pwSaving = false;
    }
  }
</script>

<div class="screen">
  <h1 class="page-title">Ajustes</h1>

  <!-- ── Cuenta (Task 8 — account info, logout, change password) ── -->
  <section class="section">
    <SectionLabel text="Cuenta" />
    <div class="card">
      <div class="row">
        <span class="row-label">Nombre</span>
        <span class="account-value">{auth.account?.name ?? ""}</span>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Correo</span>
        <span class="account-value">{auth.account?.email ?? ""}</span>
      </div>
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Cambiar contraseña</span>
        <div class="pw-row">
          <input
            class="pw-input"
            type="password"
            placeholder="Nueva contraseña"
            bind:value={newPassword}
            onkeydown={(e) => { if (e.key === "Enter") handleChangePassword(); }}
          />
          <button class="btn-primary" onclick={handleChangePassword} disabled={pwSaving || !newPassword.trim()}>
            {pwSaving ? "Guardando…" : "Guardar"}
          </button>
        </div>
      </div>
      {#if pwMsg}
        <p class="pw-msg" class:pw-msg-error={pwError}>{pwMsg}</p>
      {/if}
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Sesión</span>
        <button class="destruct-btn" onclick={handleLogout} disabled={loggingOut}>
          {loggingOut ? "Cerrando sesión…" : "Cerrar sesión"}
        </button>
      </div>
    </div>
  </section>

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

  <!-- ── Grabación (D-11 hotkey, D-12 push-to-talk, idioma) ── -->
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
      <div class="sep"></div>
      <div class="row">
        <span class="row-label">Sensibilidad de la animación</span>
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
        <span class="row-label">Idioma</span>
        <select class="sel" bind:value={settings.selected_language} onchange={() => onSave()}>
          {#each LANGUAGES as l}<option value={l.value}>{l.label}</option>{/each}
        </select>
      </div>
    </div>
    <p class="section-hint">
      {settings.push_to_talk
        ? "Mantén presionado para grabar, suelta para transcribir."
        : "Presiona una vez para iniciar, otra para detener."}
    </p>
  </section>

  <!-- ── Modelos (D-13) — compact dropdown + single download control ── -->
  <section class="section">
    <SectionLabel text="Modelos" />
    <div class="card">
      <div class="row">
        <span class="row-label">Modelo activo</span>
        <select
          class="sel"
          bind:value={settings.selected_model}
          onchange={() => onSave()}
        >
          {#each modelList as m (m.id)}
            <option value={m.id} disabled={m.coming_soon || !!m.disabled_reason}>
              {m.name}{m.size_mb > 0 ? ` · ${(m.size_mb / 1024).toFixed(1)} GB` : " · Nativo"}
            </option>
          {/each}
        </select>
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
            <span class="dl-status">Descargado</span>
          {:else}
            <!-- Not downloaded: primary download button -->
            <button class="btn-primary" onclick={() => onDownload(settings.selected_model)}>
              Descargar
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
    <p class="section-hint">Cuando corriges una transcripción y repites la misma corrección, onnda crea una regla para aplicarla sola. Edita una transcripción en "Transcripciones".</p>
  </section>

  <!-- ── Privacidad (Task 6 analytics opt-in) ── -->
  <section class="section">
    <SectionLabel text="Privacidad" />
    <div class="card">
      <Toggle
        id="analytics-enabled"
        label="Estadísticas anónimas de uso"
        bind:checked={settings.analytics_enabled}
        onchange={() => onSave()}
      />
    </div>
    <p class="section-hint">Nunca enviamos lo que dictas. Solo eventos anónimos como "transcripción completada".</p>
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

  /* ── Select (language + model) ── */
  .sel {
    font-size: 14px;
    font-family: var(--font-sans);
    color: var(--text);
    background-color: var(--bg);
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

  /* ── Cuenta ── */
  .account-value {
    font-size: 14px;
    color: var(--text-muted);
  }
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
  .pw-msg {
    font-size: 12px;
    color: var(--text-muted);
    margin: 0;
  }
  .pw-msg-error { color: var(--danger); opacity: 1; }

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
