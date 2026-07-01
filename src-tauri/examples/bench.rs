//! Benchmark de inferencia whisper en CPU (Intel) — mide SOLO state.full().
//! Uso: cargo run --release --example bench -- <model.bin> <audio.wav> [n_threads]
//! El wav debe ser PCM 16-bit mono a 16 kHz (formato que graba onnda).

use std::time::Instant;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn read_wav_16k_mono(path: &str) -> Vec<f32> {
    let bytes = std::fs::read(path).expect("read wav");
    // Buscar el chunk "data" y leer i16 LE → f32.
    let pos = bytes
        .windows(4)
        .position(|w| w == b"data")
        .expect("data chunk");
    let data = &bytes[pos + 8..];
    data.chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let model = &args[1];
    let wav = &args[2];
    let n_threads: i32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(6);

    let samples = read_wav_16k_mono(wav);
    let audio_s = samples.len() as f32 / 16000.0;

    let t_load = Instant::now();
    let mut p = WhisperContextParameters::default();
    p.use_gpu(false);
    let ctx = WhisperContext::new_with_params(model, p).expect("load model");
    let load_ms = t_load.elapsed().as_secs_f32() * 1000.0;

    // Warm-up + medición (2 corridas; reportamos la 2ª = caliente).
    let mut last = 0.0;
    for run in 0..2 {
        let mut state = ctx.create_state().expect("state");
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(n_threads);
        params.set_n_max_text_ctx(224);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_no_speech_thold(0.6);
        let lang = std::env::var("BENCH_LANG").unwrap_or_else(|_| "es".to_string());
        params.set_language(Some(Box::leak(lang.into_boxed_str())));

        let t = Instant::now();
        state.full(params, &samples).expect("full");
        let infer = t.elapsed().as_secs_f32();
        last = infer;
        let n = state.full_n_segments().unwrap_or(0);
        let text: String = (0..n)
            .filter_map(|i| state.full_get_segment_text(i).ok())
            .collect::<Vec<_>>()
            .join(" ");
        if run == 1 {
            eprintln!("    text: {}", text.trim());
        }
    }

    let model_name = std::path::Path::new(model)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    println!(
        "{:<28} threads={} audio={:.1}s  load={:.0}ms  infer={:.2}s  RTF={:.2}x",
        model_name, n_threads, audio_s, load_ms, last, last / audio_s
    );
}
