use anyhow::Result;

/// Options passed to any transcription backend.
#[derive(Clone)]
pub struct TranscribeOpts {
    /// BCP-47 language code ("es", "en") or "auto".
    pub language: String,
    /// Comma-separated custom vocabulary used as Whisper's initial_prompt.
    pub initial_prompt: String,
    /// Jaro-Winkler threshold for vocabulary correction (0.0–1.0).
    pub word_correction_threshold: f32,
}

/// A model backend that transcribes 16kHz mono f32 PCM audio.
///
/// Implementations are expected to cache model weights internally and be
/// thread-safe. `transcribe` is always called from a blocking thread.
pub trait TranscriptionBackend: Send + Sync {
    fn transcribe(&self, samples: &[f32], opts: &TranscribeOpts) -> Result<String>;
}
