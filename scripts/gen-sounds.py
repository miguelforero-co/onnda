#!/usr/bin/env python3
"""Generate the three Voz Local feedback cues as mono 44.1kHz 16-bit PCM WAVs.

Design goals (per product feedback): cooler and more dynamic than the macOS
system cues, but still subtle/premium — never loud or harsh. Every tone gets a
short fade-in/out so there are NO clicks/pops, plus a touch of 2nd harmonic for
richness.

Outputs:
  src-tauri/sounds/listen.wav  — quick bright RISING blip   (~520→780 Hz, 120ms)
  src-tauri/sounds/stop.wav    — two-note ascending chime   (660 then 990 Hz, ~180ms)
  src-tauri/sounds/cancel.wav  — soft DESCENDING tone        (~480→300 Hz, 140ms)

Run:  python3 scripts/gen-sounds.py
"""

import math
import os
import struct
import wave

SR = 44100
OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "sounds")


def _fade(samples, fade_ms=6):
    """Apply a short linear fade-in and fade-out to kill clicks."""
    n = len(samples)
    f = max(1, int(SR * fade_ms / 1000))
    f = min(f, n // 2)
    for i in range(f):
        g = i / f
        samples[i] *= g
        samples[n - 1 - i] *= g
    return samples


def _tone(freq_start, freq_end, dur_ms, amp=0.32, harmonic=0.18, decay=0.0):
    """A sine glide from freq_start→freq_end with optional 2nd harmonic and a
    gentle exponential amplitude decay (decay>0 damps the tail)."""
    n = int(SR * dur_ms / 1000)
    out = [0.0] * n
    phase = 0.0
    for i in range(n):
        t = i / n
        f = freq_start + (freq_end - freq_start) * t
        phase += 2 * math.pi * f / SR
        env = math.exp(-decay * t) if decay else 1.0
        s = math.sin(phase) + harmonic * math.sin(2 * phase)
        out[i] = amp * env * s / (1.0 + harmonic)
    return out


def _mix(base, overlay, offset):
    """Overlay `overlay` onto `base` starting at sample `offset`, extending base."""
    end = offset + len(overlay)
    if end > len(base):
        base.extend([0.0] * (end - len(base)))
    for i, v in enumerate(overlay):
        base[offset + i] += v
    return base


def _write(name, samples):
    samples = _fade(samples)
    path = os.path.abspath(os.path.join(OUT_DIR, name))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    frames = bytearray()
    for s in samples:
        v = max(-1.0, min(1.0, s))
        frames += struct.pack("<h", int(v * 32767))
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(bytes(frames))
    print(f"wrote {path} ({len(samples)} samples, {len(samples)*1000//SR} ms)")


def main():
    # listen: quick bright rising blip
    listen = _tone(520, 780, 120, amp=0.30, harmonic=0.16, decay=1.2)
    _write("listen.wav", listen)

    # stop: two-note ascending chime (perfect fifth), slight overlap
    n1 = _tone(660, 660, 110, amp=0.30, harmonic=0.20, decay=2.0)
    n2 = _tone(990, 990, 120, amp=0.30, harmonic=0.20, decay=2.2)
    chime = list(n1)
    overlap = int(SR * 0.05)  # ~50ms overlap → ~180ms total
    chime = _mix(chime, n2, len(n1) - overlap)
    _write("stop.wav", chime)

    # cancel: soft descending tone, a touch damped
    cancel = _tone(480, 300, 140, amp=0.26, harmonic=0.12, decay=2.6)
    _write("cancel.wav", cancel)


if __name__ == "__main__":
    main()
