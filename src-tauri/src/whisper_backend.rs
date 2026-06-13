use std::sync::Mutex;
use anyhow::Result;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::backend::{TranscribeOpts, TranscriptionBackend};
use crate::audio::normalize;
use crate::transcription::correct_words;
use crate::vad::vad_trim;

static MODEL_CACHE: Mutex<Option<(String, WhisperContext)>> = Mutex::new(None);

pub struct WhisperBackend {
    pub model_path: String,
}

impl WhisperBackend {
    pub fn new(model_path: String) -> Self {
        Self { model_path }
    }

    /// Load the model into the shared cache if it isn't already loaded for this
    /// path. Called at recording start to warm the model so the final pass is
    /// pure inference (no cold load) — this is what the old chunking did for free.
    pub fn ensure_loaded(&self) -> Result<()> {
        let mut cache = MODEL_CACHE.lock().unwrap();

        let needs_load = cache
            .as_ref()
            .map(|(p, _)| p.as_str() != self.model_path)
            .unwrap_or(true);

        if needs_load {
            eprintln!("[voz-local] loading model: {}", self.model_path);
            let mut ctx_params = WhisperContextParameters::default();
            #[cfg(target_arch = "aarch64")]
            {
                ctx_params.use_gpu(true);
                ctx_params.flash_attn(true);
            }
            let ctx = WhisperContext::new_with_params(&self.model_path, ctx_params)?;
            *cache = Some((self.model_path.clone(), ctx));
        }
        Ok(())
    }
}

impl TranscriptionBackend for WhisperBackend {
    fn transcribe(&self, samples: &[f32], opts: &TranscribeOpts) -> Result<String> {
        self.ensure_loaded()?;
        let cache = MODEL_CACHE.lock().unwrap();
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

        if !opts.initial_prompt.is_empty() {
            params.set_initial_prompt(&opts.initial_prompt);
        }

        let lang = if opts.language == "auto" { None } else { Some(opts.language.as_str()) };
        params.set_language(lang);

        // Resample to 16kHz then VAD trim.
        let t_pre = std::time::Instant::now();
        let audio_16k = crate::transcription::resample(samples, opts.sample_rate as usize, 16000);
        let t_resample = t_pre.elapsed();
        let t_v = std::time::Instant::now();
        let (trim_start, trim_end) = vad_trim(&audio_16k);
        let mut trimmed = audio_16k[trim_start..trim_end].to_vec();
        let t_vad = t_v.elapsed();
        // Amplify quiet recordings to 90% peak (only if peak < 30%).
        normalize(&mut trimmed, 0.9, 0.3);
        // 500ms of silence so Whisper closes the last spoken token.
        trimmed.extend(vec![0.0f32; 8000]);

        let t_inf = std::time::Instant::now();
        state.full(params, &trimmed)?;
        eprintln!(
            "[voz-local][timing] resample={:?} vad={:?} infer={:?} | in_samples={} trimmed_16k={} ({:.1}s audio)",
            t_resample, t_vad, t_inf.elapsed(),
            samples.len(), trimmed.len(), trimmed.len() as f32 / 16000.0
        );

        let n = state.full_n_segments()?;
        let text = (0..n)
            .filter_map(|i| state.full_get_segment_text(i).ok())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        Ok(correct_words(&text, &opts.initial_prompt, opts.word_correction_threshold))
    }
}
