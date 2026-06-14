<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  type Phase = "recording" | "transcribing" | "done" | "error";

  let phase = $state<Phase>("recording");
  let open = $state(false); // false = collapsed; true = expanded
  let hasNotch = $state(true); // real notch on this screen? changes the collapse target
  let levels = $state<number[]>(Array(22).fill(0.04));

  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let collapseTimer: ReturnType<typeof setTimeout> | null = null;

  const MORPH_MS = 700; // keep in sync with the CSS transition duration

  const unlisten: (() => void)[] = [];

  function clearTimers() {
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
    if (collapseTimer) { clearTimeout(collapseTimer); collapseTimer = null; }
  }

  // Collapse with the morph animation, then hide the window once it finishes.
  function scheduleClose(delay: number) {
    clearTimers();
    hideTimer = setTimeout(() => {
      open = false;
      collapseTimer = setTimeout(() => invoke("hide_widget"), MORPH_MS);
    }, delay);
  }

  onMount(async () => {
    unlisten.push(
      await listen<number>("audio-level", (e) => {
        const v = Math.min(1, e.payload * 9);
        levels = [...levels.slice(1), v];
      }),
      await listen<boolean>("recording-state", (e) => {
        if (e.payload) {
          clearTimers();
          phase = "recording";
          open = true;
        } else {
          phase = "transcribing";
        }
      }),
      await listen<boolean>("transcribing", (e) => {
        if (e.payload) phase = "transcribing";
      }),
      await listen<string>("transcription-done", () => {
        phase = "done";
        scheduleClose(900);
      }),
      await listen<string>("transcribe-error", () => {
        phase = "error";
        scheduleClose(1700);
      }),
      await listen<boolean>("screen-notch", (e) => {
        hasNotch = e.payload;
      }),
    );
  });

  onDestroy(() => {
    clearTimers();
    unlisten.forEach(fn => fn());
  });

  function barH(v: number, i: number): number {
    const n = levels.length;
    const env = 1 - Math.abs(i - (n - 1) / 2) / (n / 2) * 0.25;
    return Math.max(2, Math.min(24, v * 24 * env));
  }
</script>

<!--
  Window is 300×96, transparent. The .notch div is one continuous black shape
  drawn with clip-path: a full-width top that fuses with the screen-top / notch,
  smooth concave shoulders necking down to vertical sides, and convex rounded
  bottom corners — a single silhouette, no seams. It morphs between a collapsed
  (notch-sized) and expanded clip-path. Content sits in the lower band.
-->
<div class="notch" class:collapsed={!open} class:no-notch={!hasNotch}>
  <div class="content">
    {#if phase === "recording"}
      <span class="dot red"></span>
      <div class="bars">
        {#each levels as v, i}
          <div class="bar" style="height:{barH(v,i)}px"></div>
        {/each}
      </div>

    {:else if phase === "transcribing"}
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
</div>

<style>
  :global(*){ box-sizing:border-box; margin:0; padding:0; }
  :global(html), :global(body){
    background: transparent !important;
    overflow: hidden;
    width: 300px; height: 96px;
  }

  /* One continuous black silhouette. Expanded clip-path: full-width top
     (0..300) fusing with the screen, concave shoulders down to the body sides
     (x=14/286), vertical sides, convex bottom corners. */
  .notch{
    position: absolute;
    inset: 0;
    background: #000;
    font-family: -apple-system, "SF Pro Text", sans-serif;
    -webkit-font-smoothing: antialiased;
    clip-path: path('M0,0 L300,0 C293,0 286,7 286,16 L286,72 C286,84 276,92 264,92 L36,92 C24,92 14,84 14,72 L14,16 C14,7 7,0 0,0 Z');
    transition: clip-path 0.7s cubic-bezier(0.32, 1.26, 0.5, 1);
  }

  /* Collapsed on a screen WITH a real notch: shrink to ~the physical notch size
     (≈190×34, centered) so it tucks into/behind the hardware notch. Same command
     structure as the open path → smooth clip-path morph. */
  .notch.collapsed{
    clip-path: path('M55,0 L245,0 C242,0 239,3 239,9 L239,26 C239,31 234,34 227,34 L73,34 C66,34 61,31 61,26 L61,9 C61,3 58,0 55,0 Z');
  }

  /* Collapsed on a screen WITHOUT a notch (external display): shrink to a small
     centered point at the top edge — it contracts in BOTH width and height as it
     retracts up, then vanishes (rather than flattening to a full-width line). */
  .notch.collapsed.no-notch{
    clip-path: path('M118,0 L182,0 C179,0 176,1 176,1.5 L176,2 C176,2.5 171,3 164,3 L136,3 C129,3 124,2.5 124,2 L124,1.5 C124,1 121,0 118,0 Z');
  }

  /* Content lives in the lower band, clear of the camera / notch. */
  .content{
    position: absolute;
    left: 0; right: 0; bottom: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    height: 26px;
    padding: 0 26px;
    opacity: 1;
    transition: opacity 0.18s ease 0.12s;
  }
  .collapsed .content{
    opacity: 0;
    transition: opacity 0.1s ease;
  }

  .dot{
    width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
  }
  .dot.red   { background: #E85535; box-shadow: 0 0 6px rgba(232,85,53,0.9); }
  .dot.blue  { background: #7B9BD2; box-shadow: 0 0 5px rgba(123,155,210,0.7); }
  .dot.dim   { background: rgba(255,255,255,0.3); }

  .bars{
    display: flex; align-items: center; gap: 2.5px;
    height: 24px; flex: 1;
  }
  .bar{
    width: 2.5px; min-height: 2px; flex: 1; max-width: 4px;
    background: linear-gradient(to top, #E85535cc, #F4AA6A88);
    border-radius: 2px;
    transition: height 0.05s ease-out;
  }

  .label{
    font-size: 12px; font-weight: 500; letter-spacing: -.01em;
    color: rgba(255,255,255,0.88);
    white-space: nowrap;
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
