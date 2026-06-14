<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  type Phase = "recording" | "transcribing" | "done" | "error";
  // speaking: the wave reacts to the voice. processing: calmer, smaller wave
  // while transcribing/pasting. Each mode has its OWN tuned config + palette; the
  // renderer morphs smoothly between them. Done → fades out & the notch hides.
  type Mode = "speaking" | "processing";

  let phase = $state<Phase>("recording");
  let open = $state(false);
  let hasNotch = $state(true);

  const mode = (): Mode => (phase === "recording" ? "speaking" : "processing");

  // Subtle label under the wave (no ellipsis); empty once done/error → hides.
  const labelText = $derived(
    phase === "recording" ? "Listening" : phase === "transcribing" ? "Transcribing" : "",
  );

  // --- per-mode configs (tuned in dev/wave-color-configurator.html) ------
  type Cfg = Record<string, number>;
  const SPK: Cfg = {
    disp: 1.35, dispW: 2.25, pos: 0.56, flow: 0.35, height: 3.1, drift: 0.195,
    idxBase: 0.71, pSat: 0.88, pGamma: 1.7, bright: 0.6, spec: 0.32, fbmScale: 0.5,
    maxAmp: 46, harm: 0.33, freq: 2.5, thick: 3.5, glow: 1.7, rise: 2.1,
    edge: 0.4, widthFrac: 0.84, waveSpeed: 0.022,
  };
  const PRC: Cfg = {
    disp: 2, dispW: 0.4, pos: 0.23, flow: 0.19, height: 0.5, drift: 0.11,
    idxBase: 0.38, pSat: 0.65, pGamma: 1.45, bright: 0.95, spec: 0, fbmScale: 1.1,
    maxAmp: 17, harm: 0, freq: 1, thick: 3.5, glow: 1.7, rise: 2.1,
    edge: 0.4, widthFrac: 0.84, waveSpeed: 0.06,
  };
  const SPK_PAL = ["#ff6524", "#e29746", "#ffffff", "#bfe9ff", "#3171c4"];
  const PRC_PAL = ["#000000", "#ff6f0f", "#ffb3b3", "#bfe9ff", "#9ec8ff"];
  const KEYS = Object.keys(SPK);

  // --- shader wave -------------------------------------------------------
  let canvas: HTMLCanvasElement;
  let raf = 0;
  const W = 340, H = 120;
  const CENTER_FRAC = 0.47; // shared vertical position (lowered a few px)

  let voiceLevel = 0;
  let amp = 0;
  let phaseAcc = 0;
  let morph = 0; // 0 = speaking, 1 = processing (eased)

  const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

  // --- palette LUTs ------------------------------------------------------
  function hex2rgb(h: string): [number, number, number] {
    return [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
  }
  function buildLUT(stops: string[]): Uint8Array {
    const lut = new Uint8Array(256 * 4);
    const segs = stops.length - 1;
    for (let i = 0; i < 256; i++) {
      const t = (i / 255) * segs;
      const a = Math.min(segs - 1, Math.floor(t));
      const f = t - a;
      const c0 = hex2rgb(stops[a]), c1 = hex2rgb(stops[a + 1]);
      lut[i * 4] = c0[0] + (c1[0] - c0[0]) * f;
      lut[i * 4 + 1] = c0[1] + (c1[1] - c0[1]) * f;
      lut[i * 4 + 2] = c0[2] + (c1[2] - c0[2]) * f;
      lut[i * 4 + 3] = 255;
    }
    return lut;
  }
  const LUT_SPK = buildLUT(SPK_PAL);
  const LUT_PRC = buildLUT(PRC_PAL);
  const lutCur = new Uint8Array(256 * 4);

  const VERT = `attribute vec2 p; void main(){ gl_Position = vec4(p, 0.0, 1.0); }`;
  const FRAG = `
    precision highp float;
    uniform vec2 u_res;
    uniform sampler2D u_pal;
    uniform float u_phase,u_time,u_amp,u_thick,u_freq,u_packet,u_wf,u_center,u_maxamp;
    uniform float u_harm,u_glow,u_rise,u_edge;
    uniform float u_disp,u_dispW,u_pos,u_flow,u_height,u_drift,u_idxBase,u_pSat,u_pGamma,u_spec,u_bright,u_fbmScale;
    float hash(vec2 p){ return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453); }
    float noise(vec2 p){ vec2 i=floor(p),f=fract(p); vec2 u=f*f*(3.0-2.0*f);
      return mix(mix(hash(i+vec2(0.0,0.0)),hash(i+vec2(1.0,0.0)),u.x),
                 mix(hash(i+vec2(0.0,1.0)),hash(i+vec2(1.0,1.0)),u.x),u.y); }
    float fbm(vec2 p){ float v=0.0,a=0.5; for(int k=0;k<5;k++){ v+=a*noise(p); p*=2.02; a*=0.5; } return v; }
    float wave(float x){
      float w=sin(x*u_freq*6.28318+u_phase)+u_harm*0.6*sin(x*u_freq*6.28318*1.9+u_phase*1.4+1.3);
      // organic shimmer: the shape keeps morphing even at constant volume
      w+=0.16*sin(x*u_freq*6.28318*0.6 - u_phase*0.6 + u_time*1.1);
      w+=0.24*(fbm(vec2(x*2.2 + u_time*0.5, u_time*0.4))-0.5);
      return w/(1.0+u_harm*0.6+0.28);
    }
    void main(){
      vec2 uv=gl_FragCoord.xy/u_res;
      float xm=(uv.x-0.5)/u_wf+0.5;
      float inside=step(0.0,xm)*step(xm,1.0);
      float ed=max(u_edge,0.001);
      float edge=smoothstep(0.0,ed,xm)*smoothstep(1.0,1.0-ed,xm);
      float pk=exp(-pow(xm-0.5,2.0)/(2.0*0.045));
      float env=mix(1.0,pk,u_packet);
      float w=wave(xm);
      float cy=u_center + w*env*u_amp*u_maxamp;
      float dist=uv.y-cy;
      float tk=u_thick*mix(1.0,u_rise,smoothstep(-0.01,0.06,dist));
      float core=exp(-(dist*dist)/(2.0*tk*tk));
      float ht=tk*u_glow; float halo=exp(-(dist*dist)/(2.0*ht*ht));
      float band=(core+halo*0.5)*edge*inside;
      float flow=fbm(vec2(xm*u_fbmScale - u_time*0.18, uv.y*2.2 + u_time*0.12));
      float n=dist/(tk*max(0.4,u_dispW)); n=clamp(n,-1.0,1.0);
      float idx=u_idxBase + 0.5*n*u_disp + u_pos*(xm-0.5) + u_flow*(flow-0.5) + u_height*(cy-u_center) + u_drift*sin(u_time*0.25);
      idx=clamp(idx,0.0,1.0);
      vec3 col=texture2D(u_pal, vec2(idx,0.5)).rgb;
      col=mix(vec3(dot(col,vec3(0.333))),col,u_pSat);
      col=pow(col,vec3(u_pGamma))*u_bright;
      col+=vec3(1.0)*pow(core,3.0)*u_spec;
      gl_FragColor=vec4(col*band, band);
    }
  `;

  let gl: WebGLRenderingContext | null = null;
  let tex: WebGLTexture | null = null;
  let uni: Record<string, WebGLUniformLocation | null> = {};

  function compile(g: WebGLRenderingContext, type: number, src: string) {
    const sh = g.createShader(type)!;
    g.shaderSource(sh, src);
    g.compileShader(sh);
    if (!g.getShaderParameter(sh, g.COMPILE_STATUS)) console.error("shader:", g.getShaderInfoLog(sh));
    return sh;
  }

  function initGL() {
    const g = canvas.getContext("webgl", { premultipliedAlpha: true, alpha: true, antialias: true });
    if (!g) return;
    gl = g;
    const prog = g.createProgram()!;
    g.attachShader(prog, compile(g, g.VERTEX_SHADER, VERT));
    g.attachShader(prog, compile(g, g.FRAGMENT_SHADER, FRAG));
    g.linkProgram(prog);
    g.useProgram(prog);
    const buf = g.createBuffer();
    g.bindBuffer(g.ARRAY_BUFFER, buf);
    g.bufferData(g.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), g.STATIC_DRAW);
    const loc = g.getAttribLocation(prog, "p");
    g.enableVertexAttribArray(loc);
    g.vertexAttribPointer(loc, 2, g.FLOAT, false, 0, 0);

    tex = g.createTexture();
    g.bindTexture(g.TEXTURE_2D, tex);
    g.texParameteri(g.TEXTURE_2D, g.TEXTURE_MIN_FILTER, g.LINEAR);
    g.texParameteri(g.TEXTURE_2D, g.TEXTURE_MAG_FILTER, g.LINEAR);
    g.texParameteri(g.TEXTURE_2D, g.TEXTURE_WRAP_S, g.CLAMP_TO_EDGE);
    g.texParameteri(g.TEXTURE_2D, g.TEXTURE_WRAP_T, g.CLAMP_TO_EDGE);

    const names = ["u_res","u_pal","u_phase","u_time","u_amp","u_thick","u_freq","u_packet","u_wf","u_center","u_maxamp","u_harm","u_glow","u_rise","u_edge","u_disp","u_dispW","u_pos","u_flow","u_height","u_drift","u_idxBase","u_pSat","u_pGamma","u_spec","u_bright","u_fbmScale"];
    for (const n of names) uni[n] = g.getUniformLocation(prog, n);
    g.uniform1i(uni.u_pal, 0);

    const dpr = window.devicePixelRatio || 1;
    canvas.width = W * dpr;
    canvas.height = H * dpr;
    g.viewport(0, 0, canvas.width, canvas.height);
    g.uniform2f(uni.u_res, canvas.width, canvas.height);
    g.uniform1f(uni.u_center, CENTER_FRAC);
  }

  function render(now: number) {
    const g = gl;
    if (!g) { raf = requestAnimationFrame(render); return; }
    const procTarget = mode() === "processing" ? 1 : 0;
    morph += (procTarget - morph) * 0.06;

    const P: Cfg = {};
    for (const k of KEYS) P[k] = lerp(SPK[k], PRC[k], morph);

    const spkAmp = 0.2 + voiceLevel * 0.62;
    const prcAmp = 0.42 + 0.06 * Math.sin(now / 1100);
    const target = lerp(spkAmp, prcAmp, morph);
    amp += (target - amp) * lerp(0.16, 0.05, morph);
    phaseAcc += P.waveSpeed * lerp(0.6 + voiceLevel * 0.8, 0.4, morph) * 1.4;
    voiceLevel *= 0.88;

    // blend the two palettes by morph, upload as the LUT
    for (let i = 0; i < lutCur.length; i++) lutCur[i] = lerp(LUT_SPK[i], LUT_PRC[i], morph);
    g.activeTexture(g.TEXTURE0);
    g.bindTexture(g.TEXTURE_2D, tex);
    g.texImage2D(g.TEXTURE_2D, 0, g.RGBA, 256, 1, 0, g.RGBA, g.UNSIGNED_BYTE, lutCur);

    g.uniform1f(uni.u_phase, phaseAcc);
    g.uniform1f(uni.u_time, now * 0.001);
    g.uniform1f(uni.u_amp, amp);
    g.uniform1f(uni.u_packet, 1 - morph);
    g.uniform1f(uni.u_maxamp, P.maxAmp / H);
    g.uniform1f(uni.u_thick, P.thick / H);
    g.uniform1f(uni.u_wf, P.widthFrac);
    g.uniform1f(uni.u_edge, P.edge);
    g.uniform1f(uni.u_freq, P.freq);
    g.uniform1f(uni.u_harm, P.harm);
    g.uniform1f(uni.u_glow, P.glow);
    g.uniform1f(uni.u_rise, P.rise);
    g.uniform1f(uni.u_disp, P.disp);
    g.uniform1f(uni.u_dispW, P.dispW);
    g.uniform1f(uni.u_pos, P.pos);
    g.uniform1f(uni.u_flow, P.flow);
    g.uniform1f(uni.u_height, P.height);
    g.uniform1f(uni.u_drift, P.drift);
    g.uniform1f(uni.u_idxBase, P.idxBase);
    g.uniform1f(uni.u_pSat, P.pSat);
    g.uniform1f(uni.u_pGamma, P.pGamma);
    g.uniform1f(uni.u_spec, P.spec);
    g.uniform1f(uni.u_bright, P.bright);
    g.uniform1f(uni.u_fbmScale, P.fbmScale);

    g.clearColor(0, 0, 0, 0);
    g.clear(g.COLOR_BUFFER_BIT);
    g.drawArrays(g.TRIANGLE_STRIP, 0, 4);
    raf = requestAnimationFrame(render);
  }

  // --- timers / collapse -------------------------------------------------
  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let collapseTimer: ReturnType<typeof setTimeout> | null = null;
  const MORPH_MS = 700;
  const unlisten: (() => void)[] = [];

  function clearTimers() {
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
    if (collapseTimer) { clearTimeout(collapseTimer); collapseTimer = null; }
  }
  function scheduleClose(delay: number) {
    clearTimers();
    hideTimer = setTimeout(() => {
      open = false;
      collapseTimer = setTimeout(() => invoke("hide_widget"), MORPH_MS);
    }, delay);
  }

  onMount(async () => {
    // Register listeners FIRST so the notch always reacts to recording-state
    // (expands & shows) even if WebGL init fails for any reason.
    unlisten.push(
      await listen<number>("audio-level", (e) => {
        const v = Math.min(1, e.payload * 8);
        voiceLevel = voiceLevel * 0.55 + v * 0.45;
      }),
      await listen<boolean>("recording-state", (e) => {
        if (e.payload) { clearTimers(); phase = "recording"; open = true; }
        else phase = "transcribing";
      }),
      await listen<boolean>("transcribing", (e) => { if (e.payload) phase = "transcribing"; }),
      await listen<string>("transcription-done", () => { phase = "done"; scheduleClose(450); }),
      await listen<string>("transcribe-error", () => { phase = "error"; scheduleClose(1200); }),
      await listen<boolean>("screen-notch", (e) => { hasNotch = e.payload; }),
    );

    // WebGL is best-effort: a failure must never stop the notch from appearing.
    try {
      initGL();
      raf = requestAnimationFrame(render);
    } catch (e) {
      console.error("[widget] initGL failed", e);
    }
  });

  onDestroy(() => {
    clearTimers();
    if (raf) cancelAnimationFrame(raf);
    unlisten.forEach((fn) => fn());
  });
</script>

<!--
  Window is 340×120, transparent. The .notch div is one continuous black shape
  (clip-path). Inside it, a WebGL canvas renders the wave: a flowing band whose
  colour comes from a per-mode palette LUT driven by dispersion across the
  wave's thickness. Per-mode configs + palettes morph smoothly. No bars/dot/text.
-->
<div class="notch" class:collapsed={!open} class:no-notch={!hasNotch}>
  <div class="wave-wrap">
    <canvas bind:this={canvas} style="width:{W}px; height:{H}px;"></canvas>
  </div>
  <div class="label" class:visible={!!labelText}>{labelText}</div>
</div>

<style>
  :global(*){ box-sizing:border-box; margin:0; padding:0; }
  :global(html), :global(body){
    background: transparent !important;
    overflow: hidden;
    width: 340px; height: 120px;
  }
  .notch{
    position: absolute;
    inset: 0;
    background: #000;
    clip-path: path('M0,0 L340,0 C332,0 324,8 324,18 L324,98 C324,111 313,120 300,120 L40,120 C27,120 16,111 16,98 L16,18 C16,8 8,0 0,0 Z');
    transition: clip-path 0.7s cubic-bezier(0.32, 1.26, 0.5, 1);
  }
  .notch.collapsed{
    clip-path: path('M74,0 L266,0 C263,0 260,3 260,9 L260,26 C260,31 255,34 248,34 L92,34 C85,34 80,31 80,26 L80,9 C80,3 77,0 74,0 Z');
  }
  .notch.collapsed.no-notch{
    clip-path: path('M138,0 L202,0 C199,0 196,1 196,1.5 L196,2 C196,2.5 191,3 184,3 L156,3 C149,3 144,2.5 144,2 L144,1.5 C144,1 141,0 138,0 Z');
  }
  .wave-wrap{
    position: absolute; inset: 0;
    opacity: 1;
    transition: opacity 0.18s ease 0.12s;
  }
  .collapsed .wave-wrap{
    opacity: 0;
    transition: opacity 0.1s ease;
  }
  canvas{ display: block; }

  /* Super-subtle label centred under the wave, with breathing room. */
  .label{
    position: absolute;
    left: 24px; bottom: 16px;
    text-align: left;
    /* Native macOS system font (San Francisco) + Apple HIG small-text rules:
       subheadline-ish 11px, Regular, system optical tracking, secondary label. */
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
    font-size: 11px; font-weight: 400; letter-spacing: normal;
    color: rgba(255, 255, 255, 0.55);
    -webkit-font-smoothing: antialiased;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.3s ease;
  }
  .label.visible{ opacity: 1; }
  .collapsed .label{ opacity: 0; transition: opacity 0.1s ease; }
</style>
