use windows::Win32::{
    Media::Audio::{
        AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
        IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator, eMultimedia,
        eRender,
    },
    System::Com::{CLSCTX_ALL, CoTaskMemFree},
};

use crate::{
    error::Result,
    system_modules::media_waveform::domain::{LoopbackCapture, WaveformSystem},
    windows_api::Com,
};

/// Stateless: each capture opens its own WASAPI session.
pub struct WindowsWaveform;

impl WaveformSystem for WindowsWaveform {
    type Capture = WasapiLoopback;

    fn create() -> Self {
        Self
    }

    fn with_loopback<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self::Capture) -> Result<()>,
    {
        Com::run_with_context(|| {
            let mut capture = WasapiLoopback::open()?;
            f(&mut capture)
        })
    }
}

/// WASAPI loopback session, owned by the capture thread (never shared).
pub struct WasapiLoopback {
    audio_client: IAudioClient,
    capture_client: IAudioCaptureClient,
    /// Number of channels in the device's native mix format.
    channels: u16,
    /// Sample rate reported by GetMixFormat, needed for Hz-accurate FFT bins.
    sample_rate: u32,
}

impl WasapiLoopback {
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
    fn open() -> Result<Self> {
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

            Ok(Self {
                audio_client,
                capture_client,
                channels,
                sample_rate,
            })
        }
    }

    /// Takes the next packet, if any: `(data, frames, flags)`. Must be released after use.
    fn next_packet(&self) -> Option<(*mut u8, u32, u32)> {
        let pkt_size = unsafe { self.capture_client.GetNextPacketSize() }.ok()?;
        if pkt_size == 0 {
            return None;
        }
        let mut data_ptr: *mut u8 = std::ptr::null_mut();
        let mut num_frames: u32 = 0;
        let mut flags: u32 = 0;
        unsafe {
            self.capture_client
                .GetBuffer(&mut data_ptr, &mut num_frames, &mut flags, None, None)
        }
        .ok()?;
        Some((data_ptr, num_frames, flags))
    }
}

impl LoopbackCapture for WasapiLoopback {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn read<F: FnMut(f32)>(&mut self, mut push: F) -> Result<usize> {
        // a failure here means the device is gone, the capture has to be reopened
        if unsafe { self.capture_client.GetNextPacketSize()? } == 0 {
            return Ok(0);
        }

        let mut total_frames = 0;
        while let Some((data_ptr, num_frames, flags)) = self.next_packet() {
            let silent = (flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32) != 0;
            if silent || data_ptr.is_null() {
                for _ in 0..num_frames {
                    push(0.0);
                }
            } else {
                let ch = self.channels as usize;
                let total_samples = num_frames as usize * ch;
                // Shared-mode loopback on modern Windows always provides IEEE float PCM.
                let pcm =
                    unsafe { std::slice::from_raw_parts(data_ptr as *const f32, total_samples) };
                for frame in pcm.chunks_exact(ch) {
                    push(frame.iter().sum::<f32>() / ch as f32);
                }
            }
            total_frames += num_frames as usize;
            let _ = unsafe { self.capture_client.ReleaseBuffer(num_frames) };
        }
        Ok(total_frames)
    }

    fn drain(&mut self) {
        while let Some((_, num_frames, _)) = self.next_packet() {
            let _ = unsafe { self.capture_client.ReleaseBuffer(num_frames) };
        }
    }
}

impl Drop for WasapiLoopback {
    fn drop(&mut self) {
        let _ = unsafe { self.audio_client.Stop() };
    }
}
