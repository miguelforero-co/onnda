<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  type Phase = "recording" | "transcribing" | "done" | "error";

  let phase = $state<Phase>("recording");
  let levels = $state<number[]>(Array(18).fill(0.04));
  let hideTimer: ReturnType<typeof setTimeout> | null = null;

  const unlisten: (() => void)[] = [];

  onMount(async () => {
    unlisten.push(
      await listen<number>("audio-level", (e) => {
        const v = Math.min(1, e.payload * 9);
        levels = [...levels.slice(1), v];
      }),
      await listen<boolean>("recording-state", (e) => {
        phase = e.payload ? "recording" : "transcribing";
      }),
      await listen<boolean>("transcribing", (e) => {
        if (e.payload) phase = "transcribing";
      }),
      await listen<string>("transcription-done", () => {
        phase = "done";
        hideTimer = setTimeout(() => invoke("hide_widget"), 1000);
      }),
      await listen<string>("transcribe-error", () => {
        phase = "error";
        hideTimer = setTimeout(() => invoke("hide_widget"), 2000);
      }),
    );
  });

  onDestroy(() => {
    if (hideTimer) clearTimeout(hideTimer);
    unlisten.forEach(fn => fn());
  });

  function barH(v: number, i: number): number {
    const n = levels.length;
    const env = 1 - Math.abs(i - (n - 1) / 2) / (n / 2) * 0.25;
    return Math.max(2, Math.min(28, v * 28 * env));
  }
</script>

<!-- Window is 300×52. Vibrancy fills the whole window = frosted pill shape. CSS is transparent. -->
<div class="root">

  {#if phase === "recording"}
    <span class="dot red"></span>
    <div class="bars">
      {#each levels as v, i}
        <div class="bar" style="height:{barH(v,i)}px"></div>
      {/each}
    </div>

  {:else if phase === "transcribing"}
    <span class="dot amber"></span>
    <div class="spin"></div>
    <span class="label">Transcribiendo</span>

  {:else if phase === "done"}
    <span class="dot blue"></span>
    <span class="label">Listo</span>

  {:else}
    <span class="dot dim"></span>
    <span class="label err">Sin voz</span>
  {/if}

</div>

<style>
  :global(*){ box-sizing:border-box; margin:0; padding:0; }
  :global(html), :global(body){
    background: transparent !important;
    overflow: hidden;
    width: 300px; height: 52px;
  }

  /* Fill the whole window — vibrancy (applied in Rust) IS the background */
  .root{
    width: 300px;
    height: 52px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 18px;
    background: transparent;
    font-family: -apple-system, "SF Pro Text", sans-serif;
    -webkit-font-smoothing: antialiased;
  }

  .dot{
    width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
  }
  .dot.red   { background: #E85535; box-shadow: 0 0 6px rgba(232,85,53,0.9); }
  .dot.amber { background: #F4AA6A; box-shadow: 0 0 5px rgba(244,170,106,0.7); }
  .dot.blue  { background: #7B9BD2; box-shadow: 0 0 5px rgba(123,155,210,0.7); }
  .dot.dim   { background: rgba(255,255,255,0.3); }

  .bars{
    display: flex; align-items: center; gap: 2.5px;
    height: 28px; flex: 1;
  }
  .bar{
    width: 2.5px; min-height: 2px; flex: 1; max-width: 3.5px;
    background: linear-gradient(to top, #E85535cc, #F4AA6A88);
    border-radius: 2px;
    transition: height 0.05s ease-out;
  }

  .label{
    font-size: 12px; font-weight: 500; letter-spacing: -.01em;
    color: rgba(255,255,255,0.88); flex: 1;
  }
  .label.err { color: rgba(255,255,255,0.45); }

  .spin{
    width: 13px; height: 13px; flex-shrink: 0;
    border: 1.5px solid rgba(244,170,106,0.2);
    border-top-color: #F4AA6A;
    border-radius: 50%;
    animation: sp 0.7s linear infinite;
  }
  @keyframes sp { to { transform: rotate(360deg) } }
</style>
