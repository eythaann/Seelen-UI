use std::sync::Arc;

use rustfft::{Fft, num_complex::Complex};

use crate::error::Result;

pub const FFT_BINS: usize = 128;
pub const SILENCE_DBFS: f32 = -120.0;

/// OS contract of the waveform module: a loopback capture of what the default output plays.
pub trait WaveformSystem: Sized + Send + Sync {
    type Capture: LoopbackCapture;

    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Opens a loopback capture on the current default output and runs `f` with it, on the
    /// calling thread. The capture is closed when `f` returns.
    fn with_loopback<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self::Capture) -> Result<()>;

    /// False while the screen is locked or another user is active.
    fn is_session_interactive(&self) -> bool;
}

/// An open loopback capture.
pub trait LoopbackCapture {
    fn sample_rate(&self) -> u32;

    /// Reads every pending frame, mixed down to mono, into `push`.
    /// Returns the number of frames read (0 when nothing was pending).
    fn read<F: FnMut(f32)>(&mut self, push: F) -> Result<usize>;

    /// Discards every pending frame, so the OS buffer doesn't overflow while paused.
    fn drain(&mut self);
}

#[derive(Debug, Clone)]
pub enum WaveformEvent {
    Tick,
}

/// Re-orders the ring buffer into chronological order (oldest sample first).
pub fn build_ordered_samples(ring: &[f32], write_pos: usize) -> Vec<f32> {
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
pub fn compute_frequency_bins(
    samples: &[f32],
    fft: &Arc<dyn Fft<f32>>,
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
