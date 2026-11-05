#[cfg(windows)]
mod windows;

use std::{
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use rustfft::FftPlanner;
use seelen_core::{state::PerformanceMode, system_state::AudioWaveform};

use crate::{
    error::Result,
    event_manager,
    state::application::performance::PERFORMANCE_MODE,
    system_modules::media_devices::{MediaDevicesEvent, MediaDevicesManager},
    utils::{lock_free::TracedMutex, spawn_named_thread},
};

use super::domain::{
    FFT_BINS, LoopbackCapture, SILENCE_DBFS, WaveformEvent, WaveformSystem, build_ordered_samples,
    compute_frequency_bins,
};

#[cfg(windows)]
type Platform = windows::WindowsWaveform;

/// How often the waveform event is emitted to the UI (responsiveness).
const EVENT_FRAMERATE_MS: f32 = 50.0;
/// Audio history fed into the FFT — larger = better frequency resolution, more low-freq bin uniqueness.
const FFT_CAPTURE_WINDOW_FRAME_MS: f32 = 400.0;
const MIN_RING_SIZE: usize = 1024;
/// Time given to the running capture to notice the stop flag before starting the new one.
const RESTART_DELAY: Duration = Duration::from_millis(80);

pub struct WaveformManager {
    platform: Platform,
    latest: TracedMutex<AudioWaveform>,
    stop_flag: AtomicBool,
}

event_manager!(WaveformManager, WaveformEvent);

impl WaveformManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<WaveformManager> = LazyLock::new(|| {
            let m = WaveformManager::create();
            m.init();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            latest: TracedMutex::new(AudioWaveform {
                samples: vec![0.0f32; MIN_RING_SIZE],
                frequencies: vec![SILENCE_DBFS; FFT_BINS],
            }),
            stop_flag: AtomicBool::new(false),
        }
    }

    fn init(&self) {
        Self::start_capture_thread();

        // The capture is bound to the default output, so it is reopened when that changes.
        MediaDevicesManager::subscribe(|event| {
            if let MediaDevicesEvent::DefaultOutputChanged = event {
                Self::restart_capture_async();
            }
        });
    }

    pub fn latest(&self) -> AudioWaveform {
        self.latest.lock().clone()
    }

    fn restart_capture_async() {
        std::thread::spawn(|| {
            let manager = Self::instance();
            manager.stop_flag.store(true, Ordering::SeqCst);
            std::thread::sleep(RESTART_DELAY);
            manager.stop_flag.store(false, Ordering::SeqCst);
            Self::start_capture_thread();
        });
    }

    fn start_capture_thread() {
        spawn_named_thread("Waveform Capture", || {
            let manager = Self::instance();
            let result = manager
                .platform
                .with_loopback(|capture| manager.run_capture_loop(capture));
            if let Err(e) = result {
                log::error!("Waveform capture thread exited: {e:?}");
            }
        });
    }

    fn run_capture_loop(&self, capture: &mut impl LoopbackCapture) -> Result<()> {
        let sample_rate = capture.sample_rate();

        let frame_size =
            ((sample_rate as f32 * EVENT_FRAMERATE_MS / 1000.0) as usize).max(MIN_RING_SIZE);
        let fft_size =
            ((sample_rate as f32 * FFT_CAPTURE_WINDOW_FRAME_MS / 1000.0) as usize).max(frame_size);
        let emit_interval = Duration::from_secs_f32(frame_size as f32 / sample_rate as f32);

        let mut ring = vec![0.0f32; fft_size];
        let mut write_pos: usize = 0;

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(fft_size);
        let mut last_emit = Instant::now();

        loop {
            if self.stop_flag.load(Ordering::Acquire) {
                break;
            }

            // Pause when session is not interactive (screen locked / user switched).
            if !crate::platform::session::is_interactive() {
                capture.drain();
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }

            // In Extreme performance mode, skip capture and FFT to save CPU.
            if PERFORMANCE_MODE.load() == PerformanceMode::Extreme {
                capture.drain();
                std::thread::sleep(emit_interval * 4);
                continue;
            }

            let frames = match capture.read(|sample| {
                ring[write_pos % fft_size] = sample;
                write_pos = write_pos.wrapping_add(1);
            }) {
                Ok(frames) => frames,
                Err(e) => {
                    log::warn!("Waveform: capture read failed: {e:?}");
                    break;
                }
            };

            if last_emit.elapsed() >= emit_interval {
                last_emit = Instant::now();
                let samples = build_ordered_samples(&ring, write_pos);
                let frequencies = compute_frequency_bins(&samples, &fft, sample_rate);
                *self.latest.lock() = AudioWaveform {
                    samples,
                    frequencies,
                };
                Self::send(WaveformEvent::Tick);
            }

            if frames == 0 {
                std::thread::sleep(Duration::from_millis(5));
            }
        }

        Ok(())
    }
}
