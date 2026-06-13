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
    pub fn start(on_level: impl Fn(f32) + Send + 'static) -> Result<Self> {
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
                            on_level(rms_f32(data));
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
