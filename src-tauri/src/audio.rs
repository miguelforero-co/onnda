use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use anyhow::{anyhow, Result};

pub struct AudioCapture {
    pub sample_rate: u32,
    samples: Arc<Mutex<Vec<f32>>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AudioCapture {
    pub fn start(on_level: impl Fn(f32, Vec<f32>) + Send + 'static) -> Result<Self> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow!("No se encontró micrófono"))?;

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let last_emit_ms = Arc::new(AtomicU64::new(0));

        let samples_thread = Arc::clone(&samples);
        let stop_thread = Arc::clone(&stop);

        let mut planner = rustfft::FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        let thread = std::thread::spawn(move || {
            use cpal::traits::StreamTrait;
            let samples_cb = Arc::clone(&samples_thread);
            let last_emit = Arc::clone(&last_emit_ms);

            let stream = device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        {
                            let mut buf = samples_cb.lock().unwrap();
                            for frame in data.chunks(channels) {
                                let mono = frame.iter().sum::<f32>() / channels as f32;
                                buf.push(mono);
                            }
                        }
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        let prev = last_emit.load(Ordering::Relaxed);
                        if now_ms.saturating_sub(prev) >= 50 {
                            last_emit.store(now_ms, Ordering::Relaxed);
                            // Spectrum bands from the most recent window.
                            let window: Vec<f32> = {
                                let buf = samples_cb.lock().unwrap();
                                let start = buf.len().saturating_sub(FFT_SIZE);
                                buf[start..].to_vec()
                            };
                            let bands = compute_bands(&fft, &window, sample_rate);
                            on_level(rms_f32(data), bands);
                        }
                    },
                    |err| eprintln!("cpal error: {err}"),
                    None,
                )
                .expect("failed to build input stream");

            stream.play().expect("failed to start stream");
            while !stop_thread.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        Ok(Self { sample_rate, samples, stop, thread: Some(thread) })
    }

    pub fn samples_arc(&self) -> Arc<Mutex<Vec<f32>>> {
        Arc::clone(&self.samples)
    }

    pub fn stop(mut self) -> (Vec<f32>, u32) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() { let _ = t.join(); }
        let samples = self.samples.lock().unwrap().clone();
        (samples, self.sample_rate)
    }
}

/// Peak-normalize samples to `target` amplitude.
/// Only amplifies if peak is below `min_peak` — avoids boosting digital noise.
pub fn normalize(samples: &mut Vec<f32>, target: f32, min_peak: f32) {
    let peak = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    if peak < min_peak || peak == 0.0 { return; }
    let gain = target / peak;
    for s in samples.iter_mut() {
        *s *= gain;
    }
}

pub fn rms_f32(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}

// ── Spectrum (FFT bands) for the visualizer ─────────────────────────────────
const FFT_SIZE: usize = 1024;
pub const N_BANDS: usize = 12;

/// Compute `N_BANDS` log-spaced (perceptual) magnitude bands (0..1) from the
/// most recent audio window. Hann-windowed FFT over the speech range so each
/// band is a steady "node" of energy at that frequency — what drives the wave.
fn compute_bands(
    fft: &std::sync::Arc<dyn rustfft::Fft<f32>>,
    window: &[f32],
    sample_rate: u32,
) -> Vec<f32> {
    use rustfft::num_complex::Complex;
    let n = FFT_SIZE;
    let mut buf: Vec<Complex<f32>> = (0..n)
        .map(|i| {
            let s = window.get(i).copied().unwrap_or(0.0);
            let w = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (n as f32 - 1.0)).cos();
            Complex { re: s * w, im: 0.0 }
        })
        .collect();
    fft.process(&mut buf);

    let half = n / 2;
    let bin_hz = sample_rate as f32 / n as f32;
    let f_lo = 80.0_f32;
    let f_hi = (sample_rate as f32 / 2.0).min(8000.0);
    let mut bands = vec![0.0_f32; N_BANDS];
    for b in 0..N_BANDS {
        let t0 = b as f32 / N_BANDS as f32;
        let t1 = (b + 1) as f32 / N_BANDS as f32;
        let lo = f_lo * (f_hi / f_lo).powf(t0);
        let hi = f_lo * (f_hi / f_lo).powf(t1);
        let bin_lo = ((lo / bin_hz).floor() as usize).max(1);
        let bin_hi = ((hi / bin_hz).ceil() as usize).clamp(bin_lo + 1, half);
        // PEAK (max bin), not average: a held/tonal note concentrates energy in
        // one bin — averaging over the band's ~20 bins diluted it to nothing.
        let mut peak = 0.0_f32;
        for k in bin_lo..bin_hi {
            let m = buf[k].norm() / n as f32;
            if m > peak { peak = m; }
        }
        peak = (peak - 0.0015).max(0.0); // noise gate → flat when idle/listening
        // Tilt boosts highs vs lows to offset audio's natural bass-heavy 1/f
        // slope, so the resting curve is even (not left-heavy) and peaks balanced.
        let tilt = 0.5 + 1.0 * (b as f32 / (N_BANDS - 1) as f32);
        bands[b] = (peak * 10.0 * tilt).powf(0.7).min(1.0);
    }
    bands
}
