use std::sync::Mutex;
use anyhow::Result;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::backend::{TranscribeOpts, TranscriptionBackend};
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
}

impl TranscriptionBackend for WhisperBackend {
    fn transcribe(&self, samples: &[f32], opts: &TranscribeOpts) -> Result<String> {
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

        // Audio is already 16kHz from AudioCapture. WebRTC VAD trims silence.
        let (trim_start, trim_end) = vad_trim(samples);
        let mut trimmed = samples[trim_start..trim_end].to_vec();
        // 500ms of silence so Whisper closes the last spoken token.
        trimmed.extend(vec![0.0f32; 8000]);

        state.full(params, &trimmed)?;

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
