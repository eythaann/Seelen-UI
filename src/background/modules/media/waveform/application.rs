use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicBool, Ordering},
};

use parking_lot::Mutex;
use rustfft::{FftPlanner, num_complex::Complex};
use seelen_core::{state::PerformanceMode, system_state::AudioWaveform};
use windows::Win32::{
    Media::Audio::{
        AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
        IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator, eMultimedia,
        eRender,
    },
    System::Com::{CLSCTX_ALL, CoTaskMemFree},
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::media::devices::{DevicesEvent, DevicesManager},
    state::application::performance::PERFORMANCE_MODE,
    utils::spawn_named_thread,
    windows_api::{Com, event_window::IS_INTERACTIVE_SESSION},
};

use super::domain::CaptureSession;

/// How often the waveform event is emitted to the UI (responsiveness).
const EVENT_FRAMERATE_MS: f32 = 50.0;
/// Audio history fed into the FFT — larger = better frequency resolution, more low-freq bin uniqueness.
const FFT_CAPTURE_WINDOW_FRAME_MS: f32 = 400.0;
const MIN_RING_SIZE: usize = 1024;
const FFT_BINS: usize = 128;
const SILENCE_DBFS: f32 = -120.0;

#[derive(Debug, Clone)]
pub enum WaveformEvent {
    Tick,
}

unsafe impl Send for WaveformEvent {}

pub struct WaveformManager {
    latest: Arc<Mutex<AudioWaveform>>,
    stop_flag: Arc<AtomicBool>,
}

event_manager!(WaveformManager, WaveformEvent);

unsafe impl Send for WaveformManager {}
unsafe impl Sync for WaveformManager {}

impl WaveformManager {
    fn new() -> Self {
        Self {
            latest: Arc::new(Mutex::new(AudioWaveform {
                samples: vec![0.0f32; MIN_RING_SIZE],
                frequencies: vec![SILENCE_DBFS; FFT_BINS],
            })),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<WaveformManager> = LazyLock::new(|| {
            let mut m = WaveformManager::new();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    pub fn get_latest(&self) -> AudioWaveform {
        self.latest.lock().clone()
    }

    fn init(&mut self) -> Result<()> {
        self.start_capture_thread()?;

        // Reinitialize when the default render device changes.
        DevicesManager::subscribe(|event| {
            if let DevicesEvent::DefaultDeviceChanged { flow, role, .. } = event
                && flow == eRender
                && role == eMultimedia
            {
                WaveformManager::instance().restart_capture_async();
            }
        });

        Ok(())
    }

    fn restart_capture_async(&self) {
        let stop_flag = Arc::clone(&self.stop_flag);
        std::thread::spawn(move || {
            stop_flag.store(true, Ordering::SeqCst);
            std::thread::sleep(std::time::Duration::from_millis(80));
            stop_flag.store(false, Ordering::SeqCst);
            WaveformManager::instance()
                .start_capture_thread()
                .log_error();
        });
    }

    fn start_capture_thread(&self) -> Result<()> {
        let latest = Arc::clone(&self.latest);
        let stop_flag = Arc::clone(&self.stop_flag);
        spawn_named_thread("Waveform Capture", move || {
            let result = Com::run_with_context(|| run_capture_loop(latest, stop_flag));
            if let Err(e) = result {
                log::error!("Waveform capture thread exited: {e:?}");
            }
        });
        Ok(())
    }
}

/// Opens a WASAPI loopback session on the current default render device.
///
/// Initialization sequence:
///   1. IMMDeviceEnumerator::GetDefaultAudioEndpoint(eRender, eMultimedia)
///   2. IMMDevice::Activate → IAudioClient
///   3. IAudioClient::GetMixFormat (CoTaskMem-allocated WAVEFORMATEX*)
///   4. IAudioClient::Initialize(SHARED | LOOPBACK, 200ms buffer)
///   5. CoTaskMemFree(mix_format_ptr)
///   6. IAudioClient::GetService::<IAudioCaptureClient>()
///   7. IAudioClient::Start()
fn open_loopback_session() -> Result<CaptureSession> {
    unsafe {
        let enumerator: IMMDeviceEnumerator = Com::create_instance(&MMDeviceEnumerator)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
        let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;

        let fmt_ptr = audio_client.GetMixFormat()?;
        let channels = (*fmt_ptr).nChannels;
        let sample_rate = (*fmt_ptr).nSamplesPerSec;

        audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK,
            2_000_000i64,
            0i64,
            fmt_ptr,
            None,
        )?;
        CoTaskMemFree(Some(fmt_ptr as *const _));

        let capture_client: IAudioCaptureClient = audio_client.GetService()?;
        audio_client.Start()?;

        Ok(CaptureSession {
            audio_client,
            capture_client,
            channels,
            sample_rate,
        })
    }
}

/// Drains all pending WASAPI packets without processing them.
/// Called during paused states to prevent the capture buffer from overflowing.
fn drain_silently(session: &CaptureSession) {
    loop {
        let pkt_size = unsafe {
            match session.capture_client.GetNextPacketSize() {
                Ok(n) => n,
                Err(_) => return,
            }
        };
        if pkt_size == 0 {
            return;
        }
        let mut data_ptr: *mut u8 = std::ptr::null_mut();
        let mut num_frames: u32 = 0;
        let mut flags: u32 = 0;
        if unsafe {
            session
                .capture_client
                .GetBuffer(&mut data_ptr, &mut num_frames, &mut flags, None, None)
        }
        .is_err()
        {
            return;
        }
        let _ = unsafe { session.capture_client.ReleaseBuffer(num_frames) };
    }
}

fn run_capture_loop(latest: Arc<Mutex<AudioWaveform>>, stop_flag: Arc<AtomicBool>) -> Result<()> {
    let session = open_loopback_session()?;
    let sample_rate = session.sample_rate;

    let frame_size =
        ((sample_rate as f32 * EVENT_FRAMERATE_MS / 1000.0) as usize).max(MIN_RING_SIZE);
    let fft_size =
        ((sample_rate as f32 * FFT_CAPTURE_WINDOW_FRAME_MS / 1000.0) as usize).max(frame_size);
    let emit_interval = std::time::Duration::from_secs_f32(frame_size as f32 / sample_rate as f32);

    let mut ring = vec![0.0f32; fft_size];
    let mut write_pos: usize = 0;

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    let mut last_emit = std::time::Instant::now();

    loop {
        if stop_flag.load(Ordering::Acquire) {
            break;
        }

        // Pause when session is not interactive (screen locked / user switched).
        if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
            drain_silently(&session);
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        // In Extreme performance mode, skip capture and FFT to save CPU.
        if PERFORMANCE_MODE.load() == PerformanceMode::Extreme {
            drain_silently(&session);
            std::thread::sleep(emit_interval * 4);
            continue;
        }

        let next_packet_size = unsafe {
            match session.capture_client.GetNextPacketSize() {
                Ok(n) => n,
                Err(e) => {
                    log::warn!("Waveform: GetNextPacketSize failed: {e:?}");
                    break;
                }
            }
        };

        if next_packet_size == 0 {
            if last_emit.elapsed() >= emit_interval {
                last_emit = std::time::Instant::now();
                let ordered = build_ordered_samples(&ring, write_pos);
                let freqs = compute_frequency_bins(&ordered, &fft, sample_rate);
                *latest.lock() = AudioWaveform {
                    samples: ordered,
                    frequencies: freqs,
                };
                WaveformManager::send(WaveformEvent::Tick);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
            continue;
        }

        loop {
            let pkt_size = unsafe {
                match session.capture_client.GetNextPacketSize() {
                    Ok(n) => n,
                    Err(_) => break,
                }
            };
            if pkt_size == 0 {
                break;
            }

            let mut data_ptr: *mut u8 = std::ptr::null_mut();
            let mut num_frames: u32 = 0;
            let mut flags: u32 = 0;

            if unsafe {
                session.capture_client.GetBuffer(
                    &mut data_ptr,
                    &mut num_frames,
                    &mut flags,
                    None,
                    None,
                )
            }
            .is_err()
            {
                break;
            }

            let silent = (flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32) != 0;

            if num_frames > 0 {
                if silent || data_ptr.is_null() {
                    for _ in 0..num_frames as usize {
                        ring[write_pos % fft_size] = 0.0;
                        write_pos = write_pos.wrapping_add(1);
                    }
                } else {
                    let ch = session.channels as usize;
                    let total_samples = num_frames as usize * ch;
                    // Shared-mode loopback on modern Windows always provides IEEE float PCM.
                    let pcm = unsafe {
                        std::slice::from_raw_parts(data_ptr as *const f32, total_samples)
                    };
                    for frame in 0..num_frames as usize {
                        let mono = (0..ch).map(|c| pcm[frame * ch + c]).sum::<f32>() / ch as f32;
                        ring[write_pos % fft_size] = mono;
                        write_pos = write_pos.wrapping_add(1);
                    }
                }
            }

            let _ = unsafe { session.capture_client.ReleaseBuffer(num_frames) };
        }

        if last_emit.elapsed() >= emit_interval {
            last_emit = std::time::Instant::now();
            let ordered = build_ordered_samples(&ring, write_pos);
            let freqs = compute_frequency_bins(&ordered, &fft, sample_rate);
            *latest.lock() = AudioWaveform {
                samples: ordered,
                frequencies: freqs,
            };
            WaveformManager::send(WaveformEvent::Tick);
        }
    }

    Ok(())
}

/// Re-orders the ring buffer into chronological order (oldest sample first).
fn build_ordered_samples(ring: &[f32], write_pos: usize) -> Vec<f32> {
    let len = ring.len();
    let start = write_pos % len;
    let mut out = Vec::with_capacity(len);
    out.extend_from_slice(&ring[start..]);
    out.extend_from_slice(&ring[..start]);
    out
}

/// Applies a Hann window, runs forward FFT, and returns FFT_BINS magnitudes in dBFS.
///
/// Pipeline:
///   1. Hann window: w[i] = 0.5 × (1 − cos(2π·i / (N−1)))
///   2. rustfft forward FFT on windowed complex buffer
///   3. Use only bins 0..N/2 (positive-frequency half; Hermitian symmetry)
///   4. Group into FFT_BINS buckets with logarithmic frequency spacing (20 Hz → Nyquist)
///   5. Normalize by N, convert to dBFS: 20·log10(mag), clamped at SILENCE_DBFS
fn compute_frequency_bins(
    samples: &[f32],
    fft: &Arc<dyn rustfft::Fft<f32>>,
    sample_rate: u32,
) -> Vec<f32> {
    let n = samples.len();

    let mut buf: Vec<Complex<f32>> = samples
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos());
            Complex::new(s * w, 0.0)
        })
        .collect();

    fft.process(&mut buf);

    let half = n / 2;
    let nyquist = sample_rate as f32 / 2.0;
    let hz_per_fft_bin = nyquist / half as f32;

    // Logarithmic frequency axis: 20 Hz → Nyquist, matching human hearing perception.
    let min_hz: f32 = 20.0;
    let max_hz: f32 = nyquist.min(20_000.0);
    let log_ratio = (max_hz / min_hz).ln();

    (0..FFT_BINS)
        .map(|b| {
            let t0 = b as f32 / FFT_BINS as f32;
            let t1 = (b + 1) as f32 / FFT_BINS as f32;
            let f_start = min_hz * (t0 * log_ratio).exp();
            let f_end = min_hz * (t1 * log_ratio).exp();

            let bin_start = ((f_start / hz_per_fft_bin) as usize).min(half - 1);
            let bin_end = ((f_end / hz_per_fft_bin) as usize).clamp(bin_start + 1, half);
            let count = (bin_end - bin_start) as f32;

            let avg_mag = buf[bin_start..bin_end]
                .iter()
                .map(|c| c.norm() / n as f32)
                .sum::<f32>()
                / count;

            if avg_mag > 1e-10 {
                (20.0 * avg_mag.log10()).max(SILENCE_DBFS)
            } else {
                SILENCE_DBFS
            }
        })
        .collect()
}
