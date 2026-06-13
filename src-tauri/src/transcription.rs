use std::sync::Mutex;
use anyhow::Result;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub use crate::audio::rms_f32;

// Cache the loaded model so we don't re-read from disk on every recording.
static MODEL_CACHE: Mutex<Option<(String, WhisperContext)>> = Mutex::new(None);

pub fn transcribe(
    model_path: &str,
    samples: &[f32],
    sample_rate: u32,
    language: &str,
    initial_prompt: &str,
    word_correction_threshold: f32,
) -> Result<String> {
    let mut cache = MODEL_CACHE.lock().unwrap();

    let needs_load = cache.as_ref().map(|(p, _)| p.as_str() != model_path).unwrap_or(true);
    if needs_load {
        eprintln!("[voz-local] loading model: {model_path}");
        let mut ctx_params = WhisperContextParameters::default();
        // Metal GPU only available on Apple Silicon; x86_64 falls back to CPU.
        #[cfg(target_arch = "aarch64")]
        {
            ctx_params.use_gpu(true);
            ctx_params.flash_attn(true);
        }
        let ctx = WhisperContext::new_with_params(model_path, ctx_params)?;
        *cache = Some((model_path.to_string(), ctx));
    }

    let ctx = &cache.as_ref().unwrap().1;
    let mut state = ctx.create_state()?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(6);
    params.set_n_max_text_ctx(224);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_no_speech_thold(0.6);

    if !initial_prompt.is_empty() {
        params.set_initial_prompt(initial_prompt);
    }

    let lang = if language == "auto" { None } else { Some(language) };
    params.set_language(lang);

    // Trim leading/trailing silence before resampling — reduces the audio Whisper
    // needs to process, cutting inference time by 20-40% on typical short recordings.
    let (trim_start, trim_end) = trim_silence_range(samples, sample_rate);
    let mut trimmed = samples[trim_start..trim_end].to_vec();
    // Append 500ms of silence at the native sample rate so Whisper sees a clean
    // pause after the last word — without this, the final token is often cut off.
    let tail = (sample_rate as usize * 500) / 1000;
    trimmed.extend(vec![0.0f32; tail]);
    let audio = resample(&trimmed, sample_rate as usize, 16000);
    state.full(params, &audio)?;

    let n = state.full_n_segments()?;
    let text = (0..n)
        .filter_map(|i| state.full_get_segment_text(i).ok())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    let corrected = correct_words(&text, initial_prompt, word_correction_threshold);
    Ok(corrected)
}

/// Returns (start, end) sample indices spanning the voiced region plus a small margin.
/// Skips leading/trailing silence to reduce the audio Whisper processes.
pub fn trim_silence_range(samples: &[f32], sample_rate: u32) -> (usize, usize) {
    let sr = sample_rate as usize;
    let window = sr * 30 / 1000;       // 30 ms analysis window
    let margin = sr * 150 / 1000;      // 150 ms keep-margin around detected speech
    const THRESHOLD: f32 = 0.015;      // RMS energy threshold

    if samples.len() < window * 2 {
        return (0, samples.len());
    }

    let first = samples
        .chunks(window)
        .position(|w| rms_f32(w) > THRESHOLD)
        .unwrap_or(0);

    let last = samples
        .chunks(window)
        .enumerate()
        .rev()
        .find(|(_, w)| rms_f32(w) > THRESHOLD)
        .map(|(i, _)| i + 1)
        .unwrap_or(samples.len() / window);

    let start = (first * window).saturating_sub(margin);
    let end = ((last * window) + margin).min(samples.len());
    (start, end)
}


/// Replace transcribed words with custom vocabulary entries when they are close matches.
/// Scans the text word-by-word; replaces a run of 1–3 consecutive words if their
/// Jaro-Winkler similarity to a vocab entry meets the threshold.
/// Handles multi-word entries like "Claude Code" or "Node.js".
pub fn correct_words(text: &str, vocab_csv: &str, threshold: f32) -> String {
    if vocab_csv.is_empty() || threshold >= 1.0 { return text.to_string(); }

    let vocab: Vec<String> = vocab_csv
        .split([',', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if vocab.is_empty() { return text.to_string(); }

    // Tokenize preserving surrounding punctuation for re-attachment.
    let tokens: Vec<(String, String, String)> = text
        .split_whitespace()
        .map(|w| {
            let lead: String = w.chars().take_while(|c| !c.is_alphanumeric()).collect();
            let trail: String = w.chars().rev().take_while(|c| !c.is_alphanumeric()).collect::<String>().chars().rev().collect();
            let core = w[lead.len()..w.len() - trail.len()].to_string();
            (lead, core, trail)
        })
        .collect();

    let n = tokens.len();
    let mut out: Vec<String> = Vec::with_capacity(n);
    let mut i = 0;

    while i < n {
        let mut matched = false;
        'outer: for window in [3usize, 2, 1] {
            if i + window > n { continue; }
            let candidate: String = tokens[i..i + window]
                .iter()
                .map(|(_, core, _)| core.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            for entry in &vocab {
                if jaro_winkler_match(&candidate, entry, threshold) {
                    let lead = &tokens[i].0;
                    let trail = &tokens[i + window - 1].2;
                    out.push(format!("{}{}{}", lead, entry, trail));
                    i += window;
                    matched = true;
                    break 'outer;
                }
            }
        }
        if !matched {
            let (lead, core, trail) = &tokens[i];
            out.push(format!("{}{}{}", lead, core, trail));
            i += 1;
        }
    }

    out.join(" ")
}

/// Returns true when the Jaro-Winkler similarity of `candidate` and `entry` (case-insensitive)
/// meets or exceeds `threshold`. Short words (≤3 chars) require exact match to avoid
/// false positives on common words like "or", "if", "npm".
fn jaro_winkler_match(candidate: &str, entry: &str, threshold: f32) -> bool {
    let a = candidate.to_lowercase();
    let b = entry.to_lowercase();
    if a == b { return true; }
    // Guard: don't correct short common words
    if a.len() <= 3 || b.len() <= 3 { return false; }
    strsim::jaro_winkler(&a, &b) >= threshold as f64
}

pub fn resample(samples: &[f32], from_hz: usize, to_hz: usize) -> Vec<f32> {
    if from_hz == to_hz || samples.is_empty() { return samples.to_vec(); }
    let ratio = from_hz as f64 / to_hz as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    (0..out_len).map(|i| {
        let pos = i as f64 * ratio;
        let lo = pos as usize;
        let hi = (lo + 1).min(samples.len() - 1);
        let frac = (pos - lo as f64) as f32;
        samples[lo] * (1.0 - frac) + samples[hi] * frac
    }).collect()
}
