use webrtc_vad::{SampleRate, Vad, VadMode};

/// Returns (start, end) sample indices of the voiced region in 16kHz mono audio.
/// Applies a 200ms keep-margin around detected speech frames.
/// Falls back to (0, len) if the audio is too short to process.
pub fn vad_trim(samples: &[f32]) -> (usize, usize) {
    const FRAME: usize = 320; // 20ms at 16kHz
    const MARGIN: usize = 3200; // 200ms at 16kHz

    let n_frames = samples.len() / FRAME;
    if n_frames == 0 {
        return (0, samples.len());
    }

    // Convert f32 → i16 for webrtc-vad.
    let pcm_i16: Vec<i16> = samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();

    let mut vad = Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Quality);

    let mut first_voiced: Option<usize> = None;
    let mut last_voiced: Option<usize> = None;

    for i in 0..n_frames {
        let frame = &pcm_i16[i * FRAME..(i + 1) * FRAME];
        if vad.is_voice_segment(frame).unwrap_or(false) {
            if first_voiced.is_none() {
                first_voiced = Some(i);
            }
            last_voiced = Some(i);
        }
    }

    let first = first_voiced.unwrap_or(0);
    let last = last_voiced.unwrap_or(n_frames.saturating_sub(1));

    let start = (first * FRAME).saturating_sub(MARGIN);
    let end = ((last + 1) * FRAME + MARGIN).min(samples.len());
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fallback branch: clip shorter than one 20ms frame (< 320 samples) → (0, len).
    #[test]
    fn short_clip_below_one_frame_returns_full_range() {
        let samples = vec![0.0_f32; 100]; // 100 < FRAME=320 → n_frames==0
        assert_eq!(vad_trim(&samples), (0, 100));
    }

    // Edge case: empty slice → (0, 0) via the same fallback branch.
    #[test]
    fn empty_clip_returns_zero_zero() {
        assert_eq!(vad_trim(&[]), (0, 0));
    }

    // Silence (all zeros, 1 second) → no voice detected; the margin saturates/clamps
    // so start==0 and end<=len.  Must not panic.
    #[test]
    fn pure_silence_no_panic_and_valid_range() {
        let n = 16_000_usize;
        let samples = vec![0.0_f32; n];
        let (start, end) = vad_trim(&samples);
        assert_eq!(start, 0, "start should be 0 for silence");
        assert!(end <= n, "end must not exceed input length (got {end})");
    }

    // Exactly one frame (320 samples) of silence → no panic, start==0, end<=320.
    #[test]
    fn exactly_one_frame_of_silence_no_panic() {
        let samples = vec![0.0_f32; 320]; // n_frames==1
        let (start, end) = vad_trim(&samples);
        assert_eq!(start, 0);
        assert!(end <= 320, "end must not exceed 320 (got {end})");
    }

    // Range invariant: for any non-trivial deterministic input the result must satisfy
    // start <= end and end <= len.  Use a sine wave (non-silent) to exercise the
    // voiced-path as well as the margin clamping.
    #[test]
    fn range_invariant_sine_wave() {
        let n = 16_000_usize;
        let samples: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16_000.0).sin())
            .collect();
        let (start, end) = vad_trim(&samples);
        assert!(
            start <= end,
            "start ({start}) must be <= end ({end})"
        );
        assert!(
            end <= n,
            "end ({end}) must not exceed input length ({n})"
        );
    }
}
