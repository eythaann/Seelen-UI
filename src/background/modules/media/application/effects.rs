use serde::Serialize;
use windows::Win32::Media::KernelStreaming::*;
use windows_core::GUID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[allow(dead_code)]
pub enum AudioEffectType {
    AcousticEchoCancellation,
    AutomaticGainControl,
    BassBoost,
    BassManagement,
    BeamForming,
    ConstantToneRemoval,
    DeepNoiseSuppression,
    DynamicRangeCompression,
    EnvironmentalEffects,
    Equalizer,
    FarFieldBeamforming,
    LoudnessEqualizer,
    NoiseSuppression,
    RoomCorrection,
    SpeakerCompensation,
    SpeakerFill,
    SpeakerProtection,
    VirtualHeadphones,
    VirtualSurround,
    Unknown,
}

#[allow(dead_code)]
pub fn effect_id_to_type(id: GUID) -> AudioEffectType {
    match id {
        AUDIO_EFFECT_TYPE_ACOUSTIC_ECHO_CANCELLATION => AudioEffectType::AcousticEchoCancellation,
        AUDIO_EFFECT_TYPE_AUTOMATIC_GAIN_CONTROL => AudioEffectType::AutomaticGainControl,
        AUDIO_EFFECT_TYPE_BASS_BOOST => AudioEffectType::BassBoost,
        AUDIO_EFFECT_TYPE_BASS_MANAGEMENT => AudioEffectType::BassManagement,
        AUDIO_EFFECT_TYPE_BEAMFORMING => AudioEffectType::BeamForming,
        AUDIO_EFFECT_TYPE_CONSTANT_TONE_REMOVAL => AudioEffectType::ConstantToneRemoval,
        AUDIO_EFFECT_TYPE_DEEP_NOISE_SUPPRESSION => AudioEffectType::DeepNoiseSuppression,
        AUDIO_EFFECT_TYPE_DYNAMIC_RANGE_COMPRESSION => AudioEffectType::DynamicRangeCompression,
        AUDIO_EFFECT_TYPE_ENVIRONMENTAL_EFFECTS => AudioEffectType::EnvironmentalEffects,
        AUDIO_EFFECT_TYPE_EQUALIZER => AudioEffectType::Equalizer,
        AUDIO_EFFECT_TYPE_FAR_FIELD_BEAMFORMING => AudioEffectType::FarFieldBeamforming,
        AUDIO_EFFECT_TYPE_LOUDNESS_EQUALIZER => AudioEffectType::LoudnessEqualizer,
        AUDIO_EFFECT_TYPE_NOISE_SUPPRESSION => AudioEffectType::NoiseSuppression,
        AUDIO_EFFECT_TYPE_ROOM_CORRECTION => AudioEffectType::RoomCorrection,
        AUDIO_EFFECT_TYPE_SPEAKER_COMPENSATION => AudioEffectType::SpeakerCompensation,
        AUDIO_EFFECT_TYPE_SPEAKER_FILL => AudioEffectType::SpeakerFill,
        AUDIO_EFFECT_TYPE_SPEAKER_PROTECTION => AudioEffectType::SpeakerProtection,
        AUDIO_EFFECT_TYPE_VIRTUAL_HEADPHONES => AudioEffectType::VirtualHeadphones,
        AUDIO_EFFECT_TYPE_VIRTUAL_SURROUND => AudioEffectType::VirtualSurround,
        _ => AudioEffectType::Unknown,
    }
}
