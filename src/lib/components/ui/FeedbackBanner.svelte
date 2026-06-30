<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  const EMAIL = "hello@miguelforero.co";
  // Abrir el cliente de correo vía el plugin opener (el webview no navega mailto: solo).
  function mail(e: MouseEvent) {
    e.preventDefault();
    openUrl(`mailto:${EMAIL}`).catch(() => {});
  }
</script>

<p class="feedback">
  Built with love. Got feedback or a feature request? Email me at
  <a href="mailto:{EMAIL}" onclick={mail}>{EMAIL}</a>.
</p>

<style>
  .feedback {
    margin: 0;
    width: 100%;
    text-align: center;
    color: var(--text-muted);
    font-family: var(--font-sans);
    font-size: 12.5px;
    line-height: 1.5;
  }
  .feedback a {
    color: var(--text);
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
  }
  .feedback a:hover { color: var(--text-muted); }
</style>
