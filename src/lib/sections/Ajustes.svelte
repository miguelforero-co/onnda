<script lang="ts">
  import type { Settings, ModelInfo, DownloadProgress } from "$lib/types";
  import { onMount } from "svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import HotkeyRecorder from "$lib/components/HotkeyRecorder.svelte";

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

  // Models that can be the active dictation model (downloaded, not coming-soon).
  const downloadedModels = $derived(models.filter((m) => m.downloaded && !m.coming_soon));

  // Refresh permissions the moment the section mounts (parent keeps polling every 3s).
  onMount(() => onCheckPerms());
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
      id="snd-listen"
      label="Sonido al escuchar"
      bind:checked={settings.sound_on_listen}
      onchange={() => onSave()}
    />
    <div class="sep"></div>
    <Toggle
      id="snd-stop"
      label="Sonido al transcribir"
      bind:checked={settings.sound_on_stop}
      onchange={() => onSave()}
    />
    <div class="sep"></div>
    <Toggle
      id="snd-cancel"
      label="Sonido al cancelar"
      bind:checked={settings.sound_on_cancel}
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
    padding: 4px 9px; outline: none; -webkit-appearance: none;
    text-align: right; width: auto; max-width: 190px; cursor: pointer;
    transition: border-color .15s;
  }
  .sel:focus { border-color: rgba(232,85,53,.4); }
</style>
