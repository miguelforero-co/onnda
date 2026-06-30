<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { BRAND } from "$lib/brand";

  const FEEDBACK_URL = "https://onnda.app/feedback"; // TODO-LINK: confirm real URL with user before ship
  function open() { openUrl(FEEDBACK_URL).catch(() => {}); }
</script>

<button class="banner" onclick={open}>
  Thanks for using {BRAND}, a lot of love went into building it. If you want to
  leave feedback or request a feature click here
</button>

<style>
  .banner {
    width: 100%;
    text-align: left;
    background: var(--surface-ink);
    color: var(--text-on-ink);
    border: none;
    border-radius: var(--r-card);
    padding: var(--s4);
    font-family: var(--font-sans);
    font-size: 14px;
    line-height: 1.1;
    cursor: pointer;
  }
</style>
