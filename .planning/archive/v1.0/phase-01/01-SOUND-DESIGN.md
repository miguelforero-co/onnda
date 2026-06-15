# Sound Design — Voice-Dictation UI Earcons (start / done / cancel)

Goal: synthesize three short, **warm and deep** UI cue sounds for a voice-dictation app, in the spirit of Apple's Siri listening pop and ChatGPT Advanced Voice Mode's soft bloom — never the thin, "metallic" single-oscillator beep. Everything here is expressed so it can be hand-written as sample-level DSP in JavaScript (Web Audio / raw float buffers for preview) **and** mirrored in Python `wave` for the shipped WAV: every layer is either (a) a sum of sine partials with per-partial freq / amplitude / glide / ADSR, (b) white-noise through a one-pole low-pass with its own envelope ("air"), or (c) a short transient, all fed through a simple Schroeder/Freeverb-style algorithmic reverb (feedback combs + series allpass).

---

## 1. Why simple beeps sound "metallic" — and how to stay warm

The research is consistent on the cause and the cure.

**What makes a tone metallic / harsh:**
- **Inharmonic partials.** Partials that are *not* near-integer multiples of the fundamental (or that drift sharp as frequency rises) read as bell-like, glassy, metallic. FM with a modulation index above ~2.5, ring modulation, and wavefolding all deliberately generate inharmonic overtones — exactly the timbre we want to avoid. ([North Coast](https://northcoastsynthesis.com/news/maximizing-inharmonicity/), [Unison](https://unison.audio/harmonics-and-overtones/))
- **Too much high-harmonic energy.** A lone oscillator with strong upper harmonics (saw, square, or a sine pushed through distortion) has energy in the 2–8 kHz "presence/edge" band with nothing supporting it underneath, so the ear hears it as thin and piercing.
- **Hard envelopes.** Instant attack and abrupt cutoff create a click + ringing that reads as cheap/harsh. Research on sensory-sensitive earcons found *sudden* earcons annoyed listeners while *gentler, softer, smoother* ones were preferred. ([ACM 3517428.3550365](https://dl.acm.org/doi/fullHtml/10.1145/3517428.3550365))

**What makes a tone warm / deep:**
- **Harmonic (integer-ratio) partials only**, with amplitude **decaying fast as harmonic number rises** — strong fundamental + 2nd, weak 3rd/4th, almost nothing above the 5th. An organ-like recipe such as amplitudes `[1.0, 0.10, 0.20, 0.50]` for partials `1,2,3,4` reads warm because the low partials dominate; a saw (`1/n` for *all* harmonics) reads bright/edgy. The ear *fuses* harmonically-related sines into one perceived note, so adding low partials adds "color/body" without a new pitch. ([teropa additive synthesis](https://teropa.info/blog/2016/09/20/additive-synthesis.html))
- **A low-pass roll-off.** Closing a low-pass removes upper harmonics and makes the sound "darker, rounder, warmer." We bake this into the recipes by simply not writing loud high partials, plus a gentle one-pole LP on noise layers. ([FabFilter filters](https://www.fabfilter.com/learn/synthesis-and-sound-design/basics-filters), [Projet Home Studio](https://www.projethomestudio.fr/en/everything-about-synth-filters/))
- **Slow-ish, smooth envelopes.** Attacks of 8–40 ms (not 0 ms) and releases of 150–600 ms with exponential decay. No hard edges.
- **Layering across frequency roles** so each band is supported (see §2).
- **Subtle detune / chorus width.** Two copies a few cents apart (±5–10 cents, or a fraction of a Hz beat) give a lush, "alive" body without phasiness. ([KVR](https://www.kvraudio.com/forum/viewtopic.php?t=377911), [MDN detune](https://developer.mozilla.org/en-US/docs/Web/API/OscillatorNode/detune))
- **A little reverb space.** Apple's own "Note" tone is literally *"two very short notes" with "muted reverb"* for depth; the philosophy is organic-over-sci-fi, gentle and respectful. ([20k.org — The Sound of Apple](https://www.20k.org/episodes/the-sound-of-apple))

---

## 2. The layering model (sub / body / sparkle / air / transient)

Professional UI sound is built from **complementary frequency roles so layers don't mask each other**: one carries pitch/body, one carries low-end weight, one adds high texture, one adds attack. ([Material Design — Applying sound to UI](https://m2.material.io/design/sound/applying-sound-to-ui.html), [Unison — Layering Sounds 101](https://unison.audio/layering-sounds/))

| Layer | Role | How we build it | Typical band |
|---|---|---|---|
| **Sub** | weight / depth ("deep") | 1 sine, sometimes +1 octave-up sine at low amp | 60–160 Hz |
| **Body** | the note you actually hear | 2–4 summed sine partials (harmonic), optionally detuned pair | 200–900 Hz fundamental |
| **Sparkle** | gentle shine, *not* glass | 1–2 quiet **harmonic** partials high up (4th/6th harmonic) with fast decay | 1.5–4 kHz |
| **Air** | breath / softness | white noise → one-pole low-pass (or band-pass) with slow swell envelope | broadband, LP ~2–6 kHz |
| **Transient** | "it happened" tactility | 3–8 ms noise burst or fast pitch-blip, very low amp | broadband |

Reverb is applied to the **summed** signal (or to a send), not per-layer, to glue it into one space.

The "metallic" failure mode is: only **Body** present, with high partials and a hard envelope. Every recipe below adds at least Sub + Air (and usually a soft Sparkle + Transient) so the result is rounded and three-dimensional.

---

## 3. Envelope & glide conventions (for the DSP)

- **ADSR** in milliseconds. Use **exponential** (or `1 - e^{-t/τ}` style) curves for attack and decay/release, never linear ramps to/from absolute zero — ramp from/to ~0.0005 to avoid clicks. Sustain is a level 0–1.
- **Glide** = linear-in-cents (i.e. exponential-in-Hz) pitch ramp `f0 → f1` over `t_glide` ms, applied by advancing oscillator phase with an interpolated instantaneous frequency. Short glides that **resolve upward** read as positive/opening (good for *start* and *done*); glides that **resolve downward** read as closing/negative (good for *cancel*). Earcon literature: rising/consonant = pleasant/positive, descending/extreme = closing/urgent. ([Brewster ICAD92](https://www.dcs.gla.ac.uk/~stephen/papers/ICAD92.PDF))
- Keep peak amplitudes modest and consistent across the three sounds, then normalize each rendered buffer to about **-14 dBFS peak** (≈ 0.2 linear) and apply a soft tanh limiter (`x → tanh(1.3·x)/tanh(1.3)`) so nothing clips and loudness feels matched. Earcon research stresses keeping intensity in a narrow range. ([Brewster ICAD92](https://www.dcs.gla.ac.uk/~stephen/papers/ICAD92.PDF))
- Sample rate: render at **48 kHz mono, 16-bit** for the shipped WAV (matches the BeepBank earcon-corpus convention). ([BeepBank-500, arXiv:2509.17277](https://arxiv.org/abs/2509.17277))

---

## 4. The reverb (hand-codable Schroeder / Freeverb-style)

A small algorithmic reverb gives the "depth" without samples. Two equivalent options; pick by budget.

**Option A — minimal (good enough, ~30 lines):** 3–4 **parallel feedback comb filters** of mutually-prime delay lengths, summed, then **2 series allpass filters**. This is Schroeder's original topology: parallel combs build the tail, series allpasses thicken echo density. Feedback gain must be **strictly < 1** (=1 is infinite, >1 explodes). ([Schroeder reverberators, JOS](https://www.dsprelated.com/freebooks/pasp/Schroeder_Reverberators.html), [Valhalla DSP](https://valhalladsp.com/2009/05/30/schroeder-reverbs-the-forgotten-algorithm/))

**Option B — Freeverb flavor (warmer):** 8 parallel **low-pass-feedback** comb filters (a damping one-pole inside each comb feedback path rolls off highs each pass → warm, non-metallic tail) → 4 series allpass. ([Freeverb, JOS](https://www.dsprelated.com/freebooks/pasp/Freeverb.html))

Concrete numbers to start from (scale delays by `48000/44100` if you want exact, or just use these — they're already mutually prime-ish):

```
# Comb filters: (delaySamples @48k, feedback)  — small room
combs = [(1153, g), (1297, g), (1453, g), (1601, g)]   # ≈ 24, 27, 30, 33 ms
# Optional extra 4 for Freeverb-style: 1697,1811,1907,2017
# Damping one-pole inside each comb feedback (Freeverb): y = (1-d)*x + d*y_prev, d ≈ 0.25 (warmth)

# Series allpass: (delaySamples, feedback)
allpasses = [(557, 0.5), (441, 0.5), (341, 0.5), (225, 0.5)]   # mutually prime, g≈0.5
```

- **Decay time control:** comb feedback `g` sets RT. Approx `g = 10^(-3 · D_seconds / RT60_seconds)` where `D` is that comb's delay. For our **small room** target use `g ≈ 0.70–0.78` (RT60 ≈ 250–450 ms). For a slightly larger "bloom" use `g ≈ 0.82–0.85` (RT60 ≈ 600–900 ms).
- **Pre-delay:** simply delay the wet signal by `predelay_ms` (write dry into a delay line, read the reverb input from there). 8–25 ms gives separation/depth; longer pre-delay = bigger perceived space.
- **Mix:** `out = (1-mix)*dry + mix*wet`. We want subtle: **mix 0.12–0.25**.
- **Cheapest fallback** if you don't want the comb/allpass loop: sum **3–4 decaying delayed copies** of the dry buffer (taps at ~17, 31, 53, 79 ms, gains 0.5, 0.35, 0.22, 0.13, each low-passed a touch). Not as smooth but reads as "space" and is trivial in Python `wave`.

Apply one shared reverb instance to the final summed earcon; bypass on the dry transient if you want the click to stay tight.

---

## 5. The three shipped earcons

> Relative amplitudes are pre-normalization (the whole buffer is normalized at the end). ADSR in ms. "exp" = exponential segment.

### 5.1 START — "open / I'm listening"

**Feel:** a soft, warm bloom that opens upward — the room takes a gentle breath in. (Default = the "ChatGPT-style soft bloom" variant; see §6 for alternates.)

| Layer | Role | Waveform / partials | Base freq (Hz) | Glide | Amp | ADSR (ms) |
|---|---|---|---|---|---|---|
| L1 | Sub | sine | 110 | — | 0.35 | A40 D120 S0.5 R260 |
| L2 | Body | sines, partials 1·f, 2·f, 3·f @ amps 1.0 / 0.30 / 0.12 | f0 = 392 (G4) | 349→392 (G in over 90 ms) | 0.9 | A25 D180 S0.55 R320 |
| L2b | Body width | duplicate of L2 detuned **+7 cents** | (as L2) | (as L2) | 0.5 | (as L2) |
| L3 | Sparkle | sine on 4th harmonic of body | 1568 | — | 0.12 | A8 D140 S0.0 R160 |
| L4 | Air | white noise → one-pole LP @ 3.5 kHz, slow swell | — | LP 1.5k→4k over 180 ms | 0.10 | A120 D0 S0.8 R300 (swell in, fade with tail) |
| L5 | Transient | 5 ms noise burst → LP @ 6 kHz | — | — | 0.05 | A1 D40 S0 R0 |

**Reverb:** small-room, RT60 ≈ 350 ms (comb g ≈ 0.75), pre-delay 14 ms, **mix 0.18**.
**Total duration:** ~650 ms (tail included). Perceptual onset feels instant because the transient + body attack land in the first ~25 ms.

---

### 5.2 DONE / CONFIRM — "got it, captured"

**Feel:** a warm, resolved two-note lift — like Siri's gentle confirmation, a small upward interval that "lands." (Apple's Note tone = two short notes + muted reverb; we mirror that.) ([20k.org](https://www.20k.org/episodes/the-sound-of-apple))

Built as **two soft notes** in sequence: note A then note B a perfect fifth above, B starting ~90 ms after A.

| Layer | Role | Waveform / partials | Base freq (Hz) | Glide | Amp | ADSR (ms) |
|---|---|---|---|---|---|---|
| Sub | Sub | sine | 130.8 (C3) | — | 0.30 | A30 D150 S0.4 R280 |
| Note A body | Body | partials 1·f,2·f,3·f @ 1.0/0.28/0.10 | 523.25 (C5) | — | 0.85 | A18 D160 S0.0 R220 (starts t=0) |
| Note A width | Body | A body detuned **−6 cents** | (as A) | — | 0.45 | (as A) |
| Note B body | Body | partials 1·f,2·f,3·f @ 1.0/0.25/0.09 | 783.99 (G5) | — | 0.85 | A18 D200 S0.0 R300 (starts t=90 ms) |
| Note B width | Body | B body detuned **+6 cents** | (as B) | — | 0.45 | (as B) |
| Sparkle | Sparkle | sine, 4th harmonic of B | 3136 | — | 0.10 | A8 D120 S0 R140 (with B) |
| Air | Air | noise → LP 4 kHz | — | LP 2k→4k 150 ms | 0.07 | A60 D0 S0.7 R260 |
| Transient | Transient | 4 ms noise burst → LP 6 kHz | — | — | 0.04 | A1 D30 S0 R0 (with A) |

**Reverb:** small-room, RT60 ≈ 400 ms (g ≈ 0.77), pre-delay 16 ms, **mix 0.20** (slightly wetter so the resolve "blooms").
**Total duration:** ~700 ms.

---

### 5.3 CANCEL — "dismissed, nothing kept"

**Feel:** a soft, low, downward settle — gentle and non-punishing (a *closing* gesture, not an error buzzer). Descending pitch = closing; keep it warm and quiet so it never feels like a scold. ([Brewster ICAD92](https://www.dcs.gla.ac.uk/~stephen/papers/ICAD92.PDF))

| Layer | Role | Waveform / partials | Base freq (Hz) | Glide | Amp | ADSR (ms) |
|---|---|---|---|---|---|---|
| Sub | Sub | sine | 98 (G2) | 110→98 over 200 ms | 0.40 | A30 D200 S0.4 R340 |
| Body | Body | partials 1·f,2·f @ 1.0/0.22 (very few partials → dark) | f0 = 392 (G4) | **392→294** (G4→D4, down a fifth) over 220 ms | 0.85 | A22 D220 S0.3 R360 |
| Body width | Body | body detuned **−8 cents** | (as body) | (as body, glide too) | 0.45 | (as body) |
| Air | Air | noise → LP, falling | — | LP 3.5k→1.2k over 240 ms (darkens as it closes) | 0.10 | A40 D0 S0.6 R320 |
| Transient | Transient | 6 ms soft noise → LP 4 kHz (softer/duller than start) | — | — | 0.04 | A2 D40 S0 R0 |

No bright sparkle layer (closing gesture stays dark).
**Reverb:** small-room, RT60 ≈ 300 ms (g ≈ 0.72), pre-delay 12 ms, **mix 0.15** (drier/closer = "shutting").
**Total duration:** ~600 ms.

---

## 6. START-sound VARIANTS to audition (6)

All share the §3 envelope/normalization rules and the §4 reverb. Pick by mood.

**V1 — "ChatGPT-style soft bloom" (= default §5.1).** Body G4 with short upward glide-in, detuned pair, sub at 110 Hz, swelling air, light 4th-harmonic sparkle, mix 0.18. Warm, opening, neutral-premium.

**V2 — "Soft two-layer pop, Siri-like."** Very short, two stacked sines an octave apart that *pop* rather than ring.
- Sub: sine 130 Hz, amp 0.3, A20 D90 S0 R140.
- Body: sine 523 Hz + sine 1046 Hz (amp 1.0 / 0.4), A6 D110 S0 R150, tiny upward blip 494→523 over 25 ms.
- Air: noise→LP 5 kHz, amp 0.06, A8 D60 S0 R120.
- Transient: 4 ms burst, amp 0.06.
- Reverb mix 0.12, RT60 250 ms, pre-delay 8 ms. Duration ~320 ms. Crisp but rounded.

**V3 — "Deep sub + glass sparkle (warm-controlled)."** Depth-forward; the "glass" is a *harmonic* high partial so it shines without going metallic.
- Sub: sine 82 Hz + sine 164 Hz (1.0/0.4), amp 0.45, A35 D200 S0.5 R360.
- Body: partials 1/2/3 of 330 Hz @ 1.0/0.3/0.12, detuned pair ±6 cents, A28 D200 S0.4 R300.
- Sparkle: sine 1320 Hz (4th harmonic) amp 0.10, A10 D200 S0 R220, *exp* decay so it twinkles, not pings.
- Air: noise→LP swell 2k→5k, amp 0.10.
- Reverb mix 0.22, RT60 500 ms, pre-delay 18 ms. Duration ~750 ms.

**V4 — "Warm breath swell."** Air-led, almost a soft inhale; pitch arrives under the breath.
- Air leads: noise→band-pass center 1.5 kHz, amp 0.16, **A180** D0 S0.9 R300 (long swell).
- Body: sine 294 Hz + partial 2 @ 0.25, amp 0.6, A90 (slow) D200 S0.5 R320, glide 262→294 over 160 ms.
- Sub: sine 98 Hz amp 0.3, A60.
- No transient (no click — it breathes in). Reverb mix 0.20, RT60 450 ms, pre-delay 20 ms. Duration ~800 ms. Calm, organic.

**V5 — "Airy rising shimmer."** Upward shimmer that brightens as it rises — still harmonic, no inharmonic glass.
- Body: sine 392 Hz glide **392→523** (G4→C5) over 180 ms, partial 2 @ 0.25, detuned pair ±8 cents, amp 0.8, A20 D180 S0.3 R280.
- Sparkle: sine on 4th harmonic, glides with body (1568→2093), amp 0.10, A12 D200 S0 R200.
- Air: noise→LP rising 2k→6k over 200 ms, amp 0.12, A100.
- Sub: 110 Hz amp 0.3. Reverb mix 0.22, RT60 500 ms, pre-delay 16 ms. Duration ~720 ms. Bright-but-warm, optimistic.

**V6 — "Low warm thud + soft halo."** Minimal, tactile, very gentle — barely-there for power users.
- Sub/body: sine 196 Hz (G3) + partial 2 @ 0.2, amp 0.7, A15 D160 S0 R220, micro-blip 185→196 over 20 ms.
- Air halo: noise→LP 3 kHz, amp 0.08, A40 D0 S0.6 R260.
- Transient: 6 ms soft burst amp 0.05. Reverb mix 0.16, RT60 350 ms, pre-delay 12 ms. Duration ~450 ms. Understated, premium.

---

## 7. DONE-sound VARIANTS to audition (4)

**D1 — "Two-note resolve" (= default §5.2).** C5→G5 lift, detuned pairs, sub, soft air, mix 0.20. Confirms warmly.

**D2 — "Single warm ding-down (settle)."** One note with a gentle downward resolve — "set down gently."
- Body: partials 1/2/3 of 587 Hz (D5) @ 1.0/0.25/0.1, glide **587→523** over 120 ms, detuned pair ±6 cents, amp 0.85, A16 D220 S0 R300.
- Sub: 131 Hz amp 0.3. Sparkle: 4th harmonic amp 0.08 fast decay. Air: LP 4 kHz amp 0.07.
- Reverb mix 0.18, RT60 400 ms, pre-delay 14 ms. Duration ~620 ms.

**D3 — "Soft major third bloom."** Two notes a major third apart played near-together (gentle, consonant, friendly).
- Note A: 523 Hz (C5), Note B: 659 Hz (E5) starting +60 ms; each partials 1/2 @ 1.0/0.25, detuned pairs ±5 cents, amp 0.8, A16 D200 S0 R280.
- Sub 131 Hz amp 0.28. Air LP 4 kHz amp 0.07. Sparkle on B's 3rd harmonic amp 0.08.
- Reverb mix 0.22, RT60 450 ms, pre-delay 16 ms. Duration ~700 ms. Warm, slightly richer than D1.

**D4 — "Quick confirm tap."** Short and unobtrusive for high-frequency use.
- Body: sine 660 Hz + partial 2 @ 0.2, amp 0.8, A8 D120 S0 R140, blip 622→660 over 20 ms.
- Sub: 165 Hz amp 0.25, A12 D100. Transient 4 ms amp 0.05. Air LP 5 kHz amp 0.05.
- Reverb mix 0.12, RT60 280 ms, pre-delay 8 ms. Duration ~320 ms. Crisp, light, warm.

---

## 8. Implementation checklist (JS preview ↔ Python `wave` parity)

1. **Oscillator core** (shared spec): `sineGlide(f0, f1, tGlide, durMs, sr)` returns a float buffer; instantaneous freq interpolates exponentially in Hz (linear in cents). Body = sum of `partial_n = sineGlide(n·f0, n·f1, ...) · ampScale_n`.
2. **ADSR** as a multiplier buffer with exp segments; floor at 0.0005 to avoid clicks. Multiply each layer by its own ADSR, place at its start-offset into the mix buffer (additive).
3. **Air** = white noise (seeded PRNG so JS and Python match if you want bit-identical previews) → one-pole LP `y += a·(x − y)` with `a = 1 − e^{−2π·fc/sr}`; sweep `fc` per the glide column → its own ADSR.
4. **Transient** = short noise slice → one-pole LP → tiny linear fade.
5. **Sum** all layers → **reverb** (§4) → **tanh soft-limit** → **normalize to −14 dBFS** → write 16-bit/48 kHz mono WAV (`wave` module: `setnchannels(1)`, `setsampwidth(2)`, `setframerate(48000)`, clamp to int16).
6. Keep one shared `params` dict per earcon so the JS preview and the Python renderer read the *same* recipe (single source of truth → no drift between audition and ship).

---

## Sources

1. [Twenty Thousand Hertz — "The Sound of Apple"](https://www.20k.org/episodes/the-sound-of-apple) — Apple philosophy: organic over sci-fi, "two short notes + muted reverb," gentleness/respect.
2. [Apple WWDC17 — Designing Sound](https://developer.apple.com/videos/play/wwdc2017/803/) — sound from the start of design; 10 ms timing matters.
3. [Material Design — Applying sound to UI](https://m2.material.io/design/sound/applying-sound-to-ui.html) — layering by complementary frequency role.
4. [Unison — Layering Sounds 101](https://unison.audio/layering-sounds/) — sub/body/sparkle/air role split.
5. [Unison — Harmonics & Overtones](https://unison.audio/harmonics-and-overtones/) — inharmonicity → metallic/brilliant.
6. [North Coast Synthesis — Maximizing inharmonicity](https://northcoastsynthesis.com/news/maximizing-inharmonicity/) — what generates metallic/bell timbres (to avoid).
7. [teropa — Additive Synthesis & the Harmonic Series (Web Audio)](https://teropa.info/blog/2016/09/20/additive-synthesis.html) — summed sine partials, warm vs bright amplitude profiles, perceptual fusion.
8. [FabFilter — Basics: Filters](https://www.fabfilter.com/learn/synthesis-and-sound-design/basics-filters) and [Projet Home Studio — Synth filters](https://www.projethomestudio.fr/en/everything-about-synth-filters/) — low-pass → warm/round/dark.
9. [KVR — detuned oscillators](https://www.kvraudio.com/forum/viewtopic.php?t=377911) and [MDN — OscillatorNode.detune](https://developer.mozilla.org/en-US/docs/Web/API/OscillatorNode/detune) — ±5–10 cent detune for width/warmth.
10. [Schroeder Reverberators — JOS / DSPRelated](https://www.dsprelated.com/freebooks/pasp/Schroeder_Reverberators.html) and [Valhalla DSP — Schroeder reverbs](https://valhalladsp.com/2009/05/30/schroeder-reverbs-the-forgotten-algorithm/) — parallel combs + series allpass, feedback < 1, mutually-prime delays.
11. [Freeverb — JOS / DSPRelated](https://www.dsprelated.com/freebooks/pasp/Freeverb.html) — 8 LP-feedback combs + 4 allpass, damping = warm tail; concrete delay/feedback values.
12. [Brewster et al., ICAD '92 — Effectiveness of Earcons](https://www.dcs.gla.ac.uk/~stephen/papers/ICAD92.PDF) — pitch contour, intensity in narrow range, rising = positive.
13. [ACM 3517428.3550365 — Sensory-Sensitive Earcons](https://dl.acm.org/doi/fullHtml/10.1145/3517428.3550365) — softer/smoother = pleasant; sudden = annoying.
14. [Filtered White Noise — Spectral Audio Signal Processing (JOS)](https://www.dsprelated.com/freebooks/sasp/Filtered_White_Noise.html) and [iZotope — White noise, filters & effects](https://www.izotope.com/en/learn/white-noise-filters-and-effects-how-to-use-them-together.html) — building "air"/shimmer layers (HP/LP + reverb, swell envelopes).
15. [BeepBank-500 — A Synthetic Earcon Mini-Corpus (arXiv:2509.17277)](https://arxiv.org/abs/2509.17277) — earcon synthesis pipeline: oscillator family + AM + ADSR + Schroeder reverb, 48 kHz/16-bit mono convention.

## SOUND DESIGN COMPLETE
