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
