//! Incremental ("streaming") transcription support.
//!
//! Whisper is not a native streaming model, so instead of re-transcribing the
//! whole recording at stop, we commit completed speech segments *while the user
//! speaks*. A segment is considered complete when it is followed by a real pause
//! (a silence run), which is a clean cut point — no word is split. Everything up
//! to that pause is transcribed once and never touched again; at stop only the
//! short un-committed tail remains. This makes the perceived latency at release
//! nearly constant regardless of how long the user talks, and it is agnostic to
//! which model the user picks in the UI.

use crate::audio::rms_f32;

/// Window size for silence analysis, in milliseconds.
const WINDOW_MS: usize = 30;
/// A silence run at least this long closes a speech segment (end-of-utterance).
const MIN_SILENCE_MS: usize = 500;
/// Require at least this much speech before a commit, so we never commit pure noise.
const MIN_SPEECH_MS: usize = 200;
/// Below this peak the buffer is essentially silent — nothing to commit.
const MIN_PEAK: f32 = 0.02;

/// Find the sample index up to which `audio` (mono, `sample_rate`) can be
/// committed as a finished utterance. Returns the start of the last sufficiently
/// long silence run that is preceded by real speech, or `None` if no stable
/// boundary exists yet (in which case the caller keeps accumulating and the tail
/// is handled at stop — i.e. it degrades gracefully to a single final pass).
pub fn find_commit_point(audio: &[f32], sample_rate: u32) -> Option<usize> {
    let win = (sample_rate as usize * WINDOW_MS) / 1000;
    if win == 0 {
        return None;
    }
    let n = audio.len() / win;
    if n < 8 {
        return None;
    }

    // Adaptive threshold: silence is relative to how loud this buffer's speech is,
    // so it works whether the user speaks loudly or quietly.
    let peak = audio.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    if peak < MIN_PEAK {
        return None;
    }
    let speech_thresh = (peak * 0.12).max(0.01);

    let min_silence_wins = (MIN_SILENCE_MS / WINDOW_MS).max(1);
    let min_speech_wins = (MIN_SPEECH_MS / WINDOW_MS).max(1);

    let is_speech: Vec<bool> = (0..n)
        .map(|w| {
            let s = w * win;
            rms_f32(&audio[s..s + win]) >= speech_thresh
        })
        .collect();

    // Walk windows; remember the start of the last long silence run that had
    // enough speech before it. Commit up to that point.
    let mut best_cut: Option<usize> = None;
    let mut speech_count = 0usize;
    let mut w = 0;
    while w < n {
        if is_speech[w] {
            speech_count += 1;
            w += 1;
        } else {
            let run_start = w;
            while w < n && !is_speech[w] {
                w += 1;
            }
            let run_len = w - run_start;
            if speech_count >= min_speech_wins && run_len >= min_silence_wins {
                best_cut = Some(run_start * win);
            }
        }
    }
    best_cut
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(samples: &mut Vec<f32>, secs: f32, rate: u32, amp: f32) {
        let n = (secs * rate as f32) as usize;
        for i in 0..n {
            // arbitrary non-zero waveform; sign alternates so rms is ~amp
            samples.push(if i % 2 == 0 { amp } else { -amp });
        }
    }
    fn silence(samples: &mut Vec<f32>, secs: f32, rate: u32) {
        let n = (secs * rate as f32) as usize;
        samples.extend(std::iter::repeat(0.0).take(n));
    }

    #[test]
    fn commits_after_a_pause() {
        let rate = 16000;
        let mut a = Vec::new();
        tone(&mut a, 1.0, rate, 0.3); // 1s speech
        silence(&mut a, 0.6, rate); // 600ms pause -> closes segment
        tone(&mut a, 0.5, rate, 0.3); // ongoing speech
        let cut = find_commit_point(&a, rate).expect("should find a cut");
        // cut should land around 1.0s (start of the pause), not at the end
        let cut_secs = cut as f32 / rate as f32;
        assert!(cut_secs > 0.8 && cut_secs < 1.2, "cut at {cut_secs}s");
    }

    #[test]
    fn no_cut_without_a_pause() {
        let rate = 16000;
        let mut a = Vec::new();
        tone(&mut a, 2.0, rate, 0.3); // continuous speech, no pause
        assert_eq!(find_commit_point(&a, rate), None);
    }

    #[test]
    fn no_cut_on_pure_silence() {
        let rate = 16000;
        let mut a = Vec::new();
        silence(&mut a, 2.0, rate);
        assert_eq!(find_commit_point(&a, rate), None);
    }
}
