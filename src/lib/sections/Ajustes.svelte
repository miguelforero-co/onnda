<script lang="ts">
  import type { Settings, ModelInfo, DownloadProgress } from "$lib/types";
  // Stub — filled in 01-07. Prop contract is the source of truth for downstream plans.
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
</script>

<h1 class="page-title">Ajustes</h1>

<style>
  .page-title { font-size: 16px; font-weight: 600; line-height: 1.3; color: var(--text); }
</style>
