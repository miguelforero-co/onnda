//! Decode arbitrary audio containers (wav/mp3/m4a) to mono f32 PCM via symphonia.
//!
//! The output (mono f32 + the source sample rate) feeds directly into
//! `transcription::resample` and then the Whisper backend. Everything here runs
//! on a blocking thread (the `transcribe_file` command wraps it in
//! `spawn_blocking`) so the UI never stalls on a large file.
//!
//! Threat model: a user-selected file is untrusted. We never `unwrap`/`panic` on
//! decode failures — every fallible call maps to a `String` error, per-packet
//! decode errors are skipped (not fatal), and symphonia streams packets so a
//! huge file does not get buffered whole in memory.

/// Decode an audio file to mono f32 PCM.
///
/// Returns `(samples, sample_rate)` where `samples` is the downmixed-to-mono
/// signal at the file's *native* sample rate (the caller resamples to 16 kHz).
/// Returns `Err` (never panics) on a missing, unrecognized, or corrupt file.
pub fn decode_file(path: &str) -> Result<(Vec<f32>, u32), String> {
    use symphonia::core::audio::Channels;
    use symphonia::core::codecs::audio::AudioDecoderOptions;
    use symphonia::core::codecs::CodecParameters;
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::{FormatOptions, TrackType};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;

    let file = std::fs::File::open(path).map_err(|e| format!("no se pudo abrir el archivo: {e}"))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
    {
        hint.with_extension(ext);
    }

    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| format!("formato no reconocido: {e}"))?;

    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| "sin pista de audio".to_string())?;
    let track_id = track.id;

    let audio_params = match track.codec_params.as_ref() {
        Some(CodecParameters::Audio(p)) => p,
        _ => return Err("la pista no es de audio".to_string()),
    };

    let sample_rate = audio_params
        .sample_rate
        .ok_or_else(|| "sin sample rate".to_string())?;
    let channels = audio_params
        .channels
        .as_ref()
        .map(Channels::count)
        .unwrap_or(1)
        .max(1);

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
        .map_err(|e| format!("no se pudo crear el decodificador: {e}"))?;

    let mut out: Vec<f32> = Vec::new();
    let mut interleaved: Vec<f32> = Vec::new();

    loop {
        // next_packet -> Result<Option<Packet>>: Ok(None) is clean EOF,
        // Err is a read error we treat as end-of-stream (we keep what decoded).
        let packet = match format.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(_) => break,
        };
        if packet.track_id != track_id {
            continue;
        }
        // Per-packet decode errors are skipped, not fatal (crafted-file safety).
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };
        interleaved.clear();
        decoded.copy_to_vec_interleaved(&mut interleaved);
        // Downmix to mono by averaging each frame's channels.
        for frame in interleaved.chunks(channels) {
            let sum: f32 = frame.iter().copied().sum();
            out.push(sum / channels as f32);
        }
    }

    if out.is_empty() {
        return Err("no audio decoded".to_string());
    }
    Ok((out, sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Write a minimal RIFF/WAVE mono 16-bit PCM file (shape mirrors
    /// history.rs::write_wav) so the test owns a known, decodable fixture.
    fn write_wav_fixture(path: &std::path::Path, samples: &[i16], sample_rate: u32) {
        let data_size = (samples.len() * 2) as u32;
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(b"RIFF").unwrap();
        f.write_all(&(36 + data_size).to_le_bytes()).unwrap();
        f.write_all(b"WAVE").unwrap();
        f.write_all(b"fmt ").unwrap();
        f.write_all(&16u32.to_le_bytes()).unwrap();
        f.write_all(&1u16.to_le_bytes()).unwrap(); // PCM
        f.write_all(&1u16.to_le_bytes()).unwrap(); // mono
        f.write_all(&sample_rate.to_le_bytes()).unwrap();
        f.write_all(&(sample_rate * 2).to_le_bytes()).unwrap(); // byte rate
        f.write_all(&2u16.to_le_bytes()).unwrap(); // block align
        f.write_all(&16u16.to_le_bytes()).unwrap(); // bits per sample
        f.write_all(b"data").unwrap();
        f.write_all(&data_size.to_le_bytes()).unwrap();
        for &s in samples {
            f.write_all(&s.to_le_bytes()).unwrap();
        }
        f.flush().unwrap();
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "voz-local-decode-{}-{}",
            std::process::id(),
            name
        ));
        p
    }

    #[test]
    fn decode_wav_fixture() {
        let path = temp_path("fixture.wav");
        let rate = 16_000u32;
        // 0.25s of a 440Hz sine at 16kHz mono.
        let n = rate as usize / 4;
        let samples: Vec<i16> = (0..n)
            .map(|i| {
                let t = i as f32 / rate as f32;
                ((t * 440.0 * std::f32::consts::TAU).sin() * 16_000.0) as i16
            })
            .collect();
        write_wav_fixture(&path, &samples, rate);

        let result = decode_file(path.to_str().unwrap());
        std::fs::remove_file(&path).ok();

        let (decoded, decoded_rate) = result.expect("wav fixture must decode");
        assert!(!decoded.is_empty(), "decoded samples must be non-empty");
        assert_eq!(decoded_rate, rate, "reported rate must match fixture");
        // Roughly the same number of frames we wrote.
        assert!(
            (decoded.len() as i64 - n as i64).abs() < (n as i64 / 10).max(8),
            "expected ~{} frames, got {}",
            n,
            decoded.len()
        );
    }

    #[test]
    fn decode_missing_file_errs() {
        let path = temp_path("does-not-exist.wav");
        std::fs::remove_file(&path).ok();
        let result = decode_file(path.to_str().unwrap());
        assert!(result.is_err(), "missing file must return Err, not panic");
    }

    #[test]
    fn decode_garbage_errs() {
        let path = temp_path("garbage.bin");
        let mut f = std::fs::File::create(&path).unwrap();
        // Deterministic non-audio bytes (no valid container header).
        let junk: Vec<u8> = (0..4096).map(|i| (i * 37 + 11) as u8).collect();
        f.write_all(&junk).unwrap();
        f.flush().unwrap();
        drop(f);

        let result = decode_file(path.to_str().unwrap());
        std::fs::remove_file(&path).ok();
        assert!(result.is_err(), "garbage file must return Err, not panic");
    }
}
