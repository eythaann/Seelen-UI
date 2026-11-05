//! Night Light (blue light reduction) settings/state, read from and written to the
//! registry blobs decoded by [`super::cloudstore`] and [`super::bond`].
//!
//! Field layouts adapted from <https://github.com/kvnxiao/win-nightlight-cli>.

use seelen_core::{
    chrono::{NaiveTime, Timelike},
    system_state::{NightlightSettings, ScheduleMode},
};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

use crate::{error::Result, windows_api::WindowsApi};

use super::cloudstore;
use crate::utils::bond::{BondType, CompactBinaryReader, CompactBinaryWriter, FieldHeader};

/// Registry path storing the Night Light on/off (force-enabled) state.
const NIGHT_LIGHT_STATE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\CloudStore\Store\DefaultAccount\Current\default$windows.data.bluelightreduction.bluelightreductionstate\windows.data.bluelightreduction.bluelightreductionstate";
/// Registry path storing the Night Light schedule and color temperature.
const NIGHT_LIGHT_SETTINGS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\CloudStore\Store\DefaultAccount\Current\default$windows.data.bluelightreduction.settings\windows.data.bluelightreduction.settings";
const NIGHT_LIGHT_REG_VALUE_NAME: &str = "Data";

/// Valid Night Light color temperature range, in Kelvin.
const NIGHT_LIGHT_MIN_KELVIN: u16 = 1200;
const NIGHT_LIGHT_MAX_KELVIN: u16 = 6500;

/// Seconds between the Windows FILETIME epoch (1601-01-01) and the Unix epoch (1970-01-01).
const FILETIME_UNIX_EPOCH_OFFSET_SECS: u64 = 11_644_473_600;
/// Number of 100-nanosecond intervals per second.
const FILETIME_TICKS_PER_SEC: u64 = 10_000_000;

/// Converts a Unix timestamp (seconds + sub-second nanoseconds) into a Windows FILETIME:
/// the number of 100-nanosecond intervals since 1601-01-01 UTC.
fn unix_to_filetime(secs: u64, subsec_nanos: u32) -> u64 {
    (secs + FILETIME_UNIX_EPOCH_OFFSET_SECS) * FILETIME_TICKS_PER_SEC
        + u64::from(subsec_nanos / 100)
}

fn now_unix() -> (u64, u32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    (now.as_secs(), now.subsec_nanos())
}

fn read_reg_blob(subkey: &str) -> Result<Vec<u8>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(subkey)?;
    Ok(key.get_raw_value(NIGHT_LIGHT_REG_VALUE_NAME)?.bytes)
}

fn write_reg_blob(subkey: &str, bytes: &[u8]) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags(subkey, winreg::enums::KEY_SET_VALUE)?;
    key.set_raw_value(
        NIGHT_LIGHT_REG_VALUE_NAME,
        &winreg::RegValue {
            bytes: bytes.to_vec(),
            vtype: winreg::enums::REG_BINARY,
        },
    )?;
    Ok(())
}

/// The Night Light *state* blob (force on/off), decoded from its CloudStore/Bond
/// CompactBinary registry representation. Field layout:
/// - Field 0:  int32  — enabled flag (presence = force-enabled)
/// - Field 10: int32  — initialized marker (always 1)
/// - Field 20: uint64 — last transition FILETIME
struct NightLightStateBlob {
    timestamp: u64,
    is_enabled: bool,
    initialized: i32,
    last_transition_filetime: u64,
}

/// Reads a TimeBlock struct: { field 0: int8 = hour, field 1: int8 = minute }.
fn read_night_light_time_block(reader: &mut CompactBinaryReader) -> Result<NaiveTime> {
    let mut hour: u8 = 0;
    let mut minute: u8 = 0;
    loop {
        match reader.read_field_header()? {
            FieldHeader::Stop => break,
            FieldHeader::StopBase => continue,
            FieldHeader::Field {
                id: 0,
                bond_type: BondType::Int8,
            } => {
                hour = reader.read_int8()? as u8;
            }
            FieldHeader::Field {
                id: 1,
                bond_type: BondType::Int8,
            } => {
                minute = reader.read_int8()? as u8;
            }
            FieldHeader::Field { bond_type, .. } => {
                reader.skip_value(bond_type)?;
            }
        }
    }
    let time = NaiveTime::from_hms_opt(hour as u32, minute as u32, 0)
        .ok_or("Invalid Night Light time value in registry")?;
    Ok(time)
}

/// Writes a TimeBlock struct. Omits fields with value 0 (Bond default omission).
fn write_night_light_time_block(writer: &mut CompactBinaryWriter, field_id: u16, time: NaiveTime) {
    writer.write_field_header(field_id, BondType::Struct);
    let hour = time.hour() as u8;
    let minute = time.minute() as u8;
    if hour > 0 {
        writer.write_field_header(0, BondType::Int8);
        writer.write_int8(hour as i8);
    }
    if minute > 0 {
        writer.write_field_header(1, BondType::Int8);
        writer.write_int8(minute as i8);
    }
    writer.write_stop();
}

/// Decodes the Night Light *settings* blob (schedule + color temperature) from its
/// CloudStore/Bond CompactBinary registry representation. Field layout:
/// - Field 0:  bool   — schedule_enabled
/// - Field 10: bool   — set_hours_mode (presence = set hours mode)
/// - Field 20: struct — schedule start time
/// - Field 30: struct — schedule end time
/// - Field 40: int16  — color temperature (Kelvin)
/// - Field 50: struct — sunset time
/// - Field 60: struct — sunrise time
fn deserialize_night_light_settings(data: &[u8]) -> Result<NightlightSettings> {
    let (timestamp, inner_payload) = cloudstore::cloudstore_unwrap(data)?;

    let mut reader = CompactBinaryReader::new(inner_payload);
    reader.read_marshaled_header()?;

    let mut schedule_enabled = false;
    let mut set_hours_mode = false;
    let mut start_time = NaiveTime::MIN;
    let mut end_time = NaiveTime::MIN;
    let mut color_temperature: i16 = 0;
    let mut sunset_time = NaiveTime::MIN;
    let mut sunrise_time = NaiveTime::MIN;

    loop {
        match reader.read_field_header()? {
            FieldHeader::Stop => break,
            FieldHeader::StopBase => continue,
            FieldHeader::Field {
                id: 0,
                bond_type: BondType::Bool,
            } => {
                schedule_enabled = reader.read_bool()?;
            }
            FieldHeader::Field {
                id: 10,
                bond_type: BondType::Bool,
            } => {
                let _ = reader.read_bool()?;
                set_hours_mode = true; // presence is the signal
            }
            FieldHeader::Field {
                id: 20,
                bond_type: BondType::Struct,
            } => {
                start_time = read_night_light_time_block(&mut reader)?;
            }
            FieldHeader::Field {
                id: 30,
                bond_type: BondType::Struct,
            } => {
                end_time = read_night_light_time_block(&mut reader)?;
            }
            FieldHeader::Field {
                id: 40,
                bond_type: BondType::Int16,
            } => {
                color_temperature = reader.read_int16()?;
            }
            FieldHeader::Field {
                id: 50,
                bond_type: BondType::Struct,
            } => {
                sunset_time = read_night_light_time_block(&mut reader)?;
            }
            FieldHeader::Field {
                id: 60,
                bond_type: BondType::Struct,
            } => {
                sunrise_time = read_night_light_time_block(&mut reader)?;
            }
            FieldHeader::Field { bond_type, .. } => {
                reader.skip_value(bond_type)?;
            }
        }
    }

    let schedule_mode = if schedule_enabled {
        if set_hours_mode {
            ScheduleMode::SetHours
        } else {
            ScheduleMode::SunsetToSunrise
        }
    } else {
        ScheduleMode::Off
    };

    Ok(NightlightSettings {
        timestamp,
        schedule_mode,
        color_temperature: color_temperature as u16,
        start_time,
        end_time,
        sunset_time,
        sunrise_time,
    })
}

fn serialize_night_light_settings(settings: &NightlightSettings) -> Vec<u8> {
    let mut inner = CompactBinaryWriter::new();
    inner.write_marshaled_header();

    if settings.schedule_mode != ScheduleMode::Off {
        inner.write_field_header(0, BondType::Bool);
        inner.write_bool(true);
    }
    if settings.schedule_mode == ScheduleMode::SetHours {
        inner.write_field_header(10, BondType::Bool);
        inner.write_bool(false);
    }

    write_night_light_time_block(&mut inner, 20, settings.start_time);
    write_night_light_time_block(&mut inner, 30, settings.end_time);

    inner.write_field_header(40, BondType::Int16);
    inner.write_int16(settings.color_temperature as i16);

    write_night_light_time_block(&mut inner, 50, settings.sunset_time);
    write_night_light_time_block(&mut inner, 60, settings.sunrise_time);

    inner.write_stop();
    cloudstore::cloudstore_wrap(settings.timestamp, &inner.into_bytes())
}

fn deserialize_night_light_state(data: &[u8]) -> Result<NightLightStateBlob> {
    let (timestamp, inner_payload) = cloudstore::cloudstore_unwrap(data)?;

    let mut reader = CompactBinaryReader::new(inner_payload);
    reader.read_marshaled_header()?;

    let mut is_enabled = false;
    let mut initialized: i32 = 0;
    let mut last_transition_filetime: u64 = 0;

    loop {
        match reader.read_field_header()? {
            FieldHeader::Stop => break,
            FieldHeader::StopBase => continue,
            FieldHeader::Field {
                id: 0,
                bond_type: BondType::Int32,
            } => {
                let _ = reader.read_int32()?;
                is_enabled = true; // presence is the signal
            }
            FieldHeader::Field {
                id: 10,
                bond_type: BondType::Int32,
            } => {
                initialized = reader.read_int32()?;
            }
            FieldHeader::Field {
                id: 20,
                bond_type: BondType::UInt64,
            } => {
                last_transition_filetime = reader.read_uint64()?;
            }
            FieldHeader::Field { bond_type, .. } => {
                reader.skip_value(bond_type)?;
            }
        }
    }

    Ok(NightLightStateBlob {
        timestamp,
        is_enabled,
        initialized,
        last_transition_filetime,
    })
}

fn serialize_night_light_state(state: &NightLightStateBlob) -> Vec<u8> {
    let mut inner = CompactBinaryWriter::new();
    inner.write_marshaled_header();

    if state.is_enabled {
        inner.write_field_header(0, BondType::Int32);
        inner.write_int32(0);
    }
    inner.write_field_header(10, BondType::Int32);
    inner.write_int32(state.initialized);
    inner.write_field_header(20, BondType::UInt64);
    inner.write_uint64(state.last_transition_filetime);
    inner.write_stop();

    cloudstore::cloudstore_wrap(state.timestamp, &inner.into_bytes())
}

pub fn get_settings() -> Result<NightlightSettings> {
    deserialize_night_light_settings(&read_reg_blob(NIGHT_LIGHT_SETTINGS_KEY)?)
}

pub fn get_enabled() -> Result<bool> {
    let state = deserialize_night_light_state(&read_reg_blob(NIGHT_LIGHT_STATE_KEY)?)?;
    Ok(state.is_enabled)
}

/// Enables/disables Night Light. Enabling force-activates it regardless of the schedule;
/// disabling also turns the schedule off, matching Windows' own "turn off until tomorrow" behavior.
pub fn set_enabled(enabled: bool) -> Result<()> {
    log::info!("Setting night light enabled to {enabled}");

    if !enabled {
        let mut settings =
            deserialize_night_light_settings(&read_reg_blob(NIGHT_LIGHT_SETTINGS_KEY)?)?;
        if settings.schedule_mode != ScheduleMode::Off {
            settings.schedule_mode = ScheduleMode::Off;
            settings.timestamp = now_unix().0;
            write_reg_blob(
                NIGHT_LIGHT_SETTINGS_KEY,
                &serialize_night_light_settings(&settings),
            )?;
        }
    }

    let mut state = deserialize_night_light_state(&read_reg_blob(NIGHT_LIGHT_STATE_KEY)?)?;
    if state.is_enabled != enabled {
        state.is_enabled = enabled;
        let (secs, nanos) = now_unix();
        state.timestamp = secs;
        state.last_transition_filetime = unix_to_filetime(secs, nanos);
        write_reg_blob(NIGHT_LIGHT_STATE_KEY, &serialize_night_light_state(&state))?;
        WindowsApi::broadcast_setting_change("ImmersiveColorSet");
    }

    Ok(())
}

/// Sets the Night Light color temperature, in a range between 1200 and 6500 Kelvin.
pub fn set_color_temperature(temperature: u16) -> Result<()> {
    log::info!("Setting night light color temperature to {temperature}K");

    let mut settings = deserialize_night_light_settings(&read_reg_blob(NIGHT_LIGHT_SETTINGS_KEY)?)?;
    if settings.color_temperature == temperature {
        return Ok(());
    }

    if !(NIGHT_LIGHT_MIN_KELVIN..=NIGHT_LIGHT_MAX_KELVIN).contains(&temperature) {
        return Err(format!("Invalid Night Light color temperature {temperature}").into());
    }

    settings.color_temperature = temperature;
    settings.timestamp = now_unix().0;
    write_reg_blob(
        NIGHT_LIGHT_SETTINGS_KEY,
        &serialize_night_light_settings(&settings),
    )?;
    WindowsApi::broadcast_setting_change("ImmersiveColorSet");

    Ok(())
}
