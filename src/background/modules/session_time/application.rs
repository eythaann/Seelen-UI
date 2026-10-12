use std::{
    fs::{OpenOptions, create_dir_all},
    io::{BufRead, BufReader, Write},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use windows::{
    Win32::{
        Foundation::FILETIME,
        Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC, SND_SYSTEM},
        Media::Multimedia::mciSendStringW,
        System::{
            RemoteDesktop::{
                ProcessIdToSessionId, WTS_CURRENT_SERVER_HANDLE, WTSFreeMemory, WTSLogonTime,
                WTSQuerySessionInformationW,
            },
            SystemInformation::GetTickCount,
            WindowsProgramming::QueryUnbiasedInterruptTime,
        },
        UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    },
    core::{BOOL, PCWSTR, PWSTR},
};

use crate::{
    error::Result, event_manager, modules::media::players::PlayersManager,
    state::application::FULL_STATE, windows_api::event_window::IS_INTERACTIVE_SESSION,
};
use seelen_core::{
    state::{ToolbarItem2, ToolbarState, settings::SessionTimeSound},
    system_state::ActiveSessionTime,
};
use widestring::U16CString;
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const IDLE_THRESHOLD_MILLIS: u32 = 5 * 60 * 1000;
const HUNDRED_NANOSECONDS_PER_SECOND: u64 = 10_000_000;
const MAX_CUSTOM_SOUND_PLAYBACK: Duration = Duration::from_secs(30);
static CUSTOM_SOUND_PLAYING: AtomicBool = AtomicBool::new(false);

pub struct SessionTimeManager {
    state: Arc<Mutex<SessionTimeState>>,
    enabled: Arc<AtomicBool>,
    initialized: AtomicBool,
}

#[derive(Clone)]
struct SessionTimeState {
    logon_time: Option<u64>,
    active_time_100ns: u64,
    longest_active_time_100ns: u64,
    last_unbiased_time: u64,
    is_idle: bool,
}

#[derive(Deserialize, Serialize)]
struct PersistedSessionTime {
    #[serde(default)]
    timestamp_unix_seconds: u64,
    logon_time: Option<u64>,
    active_time_100ns: u64,
    last_unbiased_time: u64,
    #[serde(default)]
    longest_active_time_100ns: u64,
}

#[derive(Debug, Clone)]
pub enum SessionTimeEvent {
    Updated(ActiveSessionTime),
}

event_manager!(SessionTimeManager, SessionTimeEvent);

impl SessionTimeManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<SessionTimeManager> = LazyLock::new(|| {
            let manager = SessionTimeManager {
                state: Arc::new(Mutex::new(SessionTimeState {
                    logon_time: None,
                    active_time_100ns: 0,
                    longest_active_time_100ns: 0,
                    last_unbiased_time: 0,
                    is_idle: true,
                })),
                enabled: Arc::new(AtomicBool::new(false)),
                initialized: AtomicBool::new(false),
            };
            manager.start_monitoring();
            manager
        });
        &MANAGER
    }

    pub fn sync_toolbar_items(items: &ToolbarState) -> Result<()> {
        let enabled = items
            .left
            .iter()
            .chain(&items.center)
            .chain(&items.right)
            .any(|item| match item {
                ToolbarItem2::Plugin(id) => id.as_str() == "@default/session-time",
                ToolbarItem2::Inline(item) => item
                    .scopes
                    .iter()
                    .any(|scope| scope.eq_ignore_ascii_case("sessiontime")),
            });
        Self::instance().set_enabled(enabled)
    }

    fn set_enabled(&self, enabled: bool) -> Result<()> {
        if self.enabled.load(Ordering::Acquire) == enabled {
            return Ok(());
        }

        if enabled {
            if !self.initialized.load(Ordering::Acquire) {
                *self.state.lock() = Self::load_state();
                self.initialized.store(true, Ordering::Release);
            } else {
                self.state.lock().last_unbiased_time = unbiased_time_100ns()?;
            }
            self.enabled.store(true, Ordering::Release);
        } else {
            self.enabled.store(false, Ordering::Release);
            let mut timer = self.state.lock();
            if let Ok(now) = unbiased_time_100ns() {
                timer.last_unbiased_time = now;
            }
            persist_state(&timer)?;
        }
        Ok(())
    }

    pub fn snapshot(&self) -> ActiveSessionTime {
        let timer = self.state.lock();
        ActiveSessionTime {
            active_seconds: timer.active_time_100ns / HUNDRED_NANOSECONDS_PER_SECOND,
            longest_active_seconds: timer.longest_active_time_100ns
                / HUNDRED_NANOSECONDS_PER_SECOND,
            is_idle: timer.is_idle,
        }
    }

    fn load_state() -> SessionTimeState {
        let logon_time = match current_logon_time() {
            Ok(time) => Some(time),
            Err(error) => {
                log::error!("Failed to determine Windows logon time: {error}");
                None
            }
        };
        let unbiased_time = match unbiased_time_100ns() {
            Ok(time) => time,
            Err(error) => {
                log::error!("Failed to initialize active session timer: {error}");
                0
            }
        };

        let persisted = match load_persisted_state() {
            Ok(persisted) => persisted,
            Err(error) => {
                log::error!("Failed to load active session time: {error}");
                None
            }
        };
        initial_session_state(logon_time, persisted, unbiased_time)
    }

    fn start_monitoring(&self) {
        let state = Arc::clone(&self.state);
        let enabled = Arc::clone(&self.enabled);
        thread::spawn(move || {
            loop {
                thread::sleep(POLL_INTERVAL);
                if !enabled.load(Ordering::Acquire) {
                    continue;
                }

                let current_unbiased_time = match unbiased_time_100ns() {
                    Ok(time) => time,
                    Err(error) => {
                        log::error!("Failed to read active session timer: {error}");
                        continue;
                    }
                };
                let count_time = match should_count_time() {
                    Ok(count) => count,
                    Err(error) => {
                        log::error!("Failed to check user activity: {error}");
                        false
                    }
                };
                let settings = FULL_STATE.load();
                let cue_enabled = settings.settings.session_time_audio_cue_enabled;
                let cue_interval_seconds = settings
                    .settings
                    .session_time_audio_cue_interval_minutes
                    .max(1) as u64
                    * 60;
                let cue_sound = settings.settings.session_time_audio_cue_sound;
                let cue_custom_path = settings.settings.session_time_audio_cue_custom_path.clone();
                let notification_enabled = settings.settings.session_time_notification_enabled;
                let notification_message =
                    settings.settings.session_time_notification_message.clone();

                let mut timer = state.lock();
                let previous_seconds = timer.active_time_100ns / HUNDRED_NANOSECONDS_PER_SECOND;
                let was_idle = timer.is_idle;
                let delta = if timer.last_unbiased_time == 0 {
                    0
                } else {
                    current_unbiased_time.saturating_sub(timer.last_unbiased_time)
                };
                timer.last_unbiased_time = current_unbiased_time;
                timer.is_idle = !count_time;
                if count_time {
                    timer.active_time_100ns = timer.active_time_100ns.saturating_add(delta);
                    timer.longest_active_time_100ns =
                        timer.longest_active_time_100ns.max(timer.active_time_100ns);
                }
                let current_seconds = timer.active_time_100ns / HUNDRED_NANOSECONDS_PER_SECOND;
                let crossed_cue_interval = count_time
                    && current_seconds / cue_interval_seconds
                        > previous_seconds / cue_interval_seconds;
                if let Err(error) = persist_state(&timer) {
                    log::error!("Failed to append active session time log: {error}");
                }
                let update = (current_seconds != previous_seconds || was_idle != timer.is_idle)
                    .then(|| ActiveSessionTime {
                        active_seconds: current_seconds,
                        longest_active_seconds: timer.longest_active_time_100ns
                            / HUNDRED_NANOSECONDS_PER_SECOND,
                        is_idle: timer.is_idle,
                    });
                drop(timer);
                if crossed_cue_interval {
                    if cue_enabled {
                        if let Err(error) = play_session_time_alert(cue_sound, cue_custom_path) {
                            log::error!("Failed to deliver screen-time alert: {error}");
                        }
                    }
                    if notification_enabled {
                        if let Err(error) = show_session_time_notification(&notification_message) {
                            log::error!("Failed to show screen-time notification: {error}");
                        }
                    }
                }
                if let Some(snapshot) = update {
                    SessionTimeManager::send(SessionTimeEvent::Updated(snapshot));
                }
            }
        });
    }
}

fn show_session_time_notification(message: &str) -> Result<()> {
    use windows::{
        Data::Xml::Dom::XmlDocument,
        UI::Notifications::{ToastNotification, ToastNotificationManager},
        core::HSTRING,
    };

    // Windows PowerShell's AppUserModelId is always registered, so unpackaged apps can raise toasts.
    const NOTIFIER_ID: &str =
        "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";
    let text = if message.trim().is_empty() {
        "Screen time reminder: time to take a break."
    } else {
        message.trim()
    };
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let xml = XmlDocument::new()?;
    xml.LoadXml(&HSTRING::from(format!(
        "<toast><visual><binding template=\"ToastGeneric\"><text>Seelen UI</text><text>{escaped}</text></binding></visual></toast>"
    )))?;
    let toast = ToastNotification::CreateToastNotification(&xml)?;
    ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(NOTIFIER_ID))?
        .Show(&toast)?;
    Ok(())
}

fn play_session_time_alert(sound: SessionTimeSound, custom_path: Option<String>) -> Result<()> {
    match play_sound(sound, custom_path) {
        Ok(()) => Ok(()),
        Err(error) => {
            log::warn!("Screen-time cue failed; trying the Windows alert sound: {error}");
            if let Err(fallback_error) = play_sound(SessionTimeSound::SystemDefault, None) {
                return Err(format!(
                    "Configured screen-time cue failed ({error}); Windows alert sound fallback also failed ({fallback_error})"
                )
                .into());
            }
            Ok(())
        }
    }
}

fn unbiased_time_100ns() -> Result<u64> {
    let mut time = 0;
    if unsafe { QueryUnbiasedInterruptTime(&mut time as *mut u64) }.as_bool() {
        Ok(time)
    } else {
        Err(std::io::Error::last_os_error().into())
    }
}

fn should_count_time() -> Result<bool> {
    let is_interactive = IS_INTERACTIVE_SESSION.load(Ordering::Acquire);
    let media_playing = PlayersManager::is_any_session_playing();
    if media_playing {
        return Ok(true);
    }
    if !is_interactive {
        return Ok(false);
    }

    let mut last_input = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if unsafe { GetLastInputInfo(&mut last_input as *mut LASTINPUTINFO) } == BOOL(0) {
        return Err(std::io::Error::last_os_error().into());
    }

    let idle_millis = unsafe { GetTickCount() }.wrapping_sub(last_input.dwTime);
    Ok(should_count_active_time(
        is_interactive,
        idle_millis,
        media_playing,
    ))
}

fn should_count_active_time(is_interactive: bool, idle_millis: u32, media_playing: bool) -> bool {
    media_playing || (is_interactive && idle_millis <= IDLE_THRESHOLD_MILLIS)
}

fn initial_session_state(
    logon_time: Option<u64>,
    persisted: Option<PersistedSessionTime>,
    current_unbiased_time: u64,
) -> SessionTimeState {
    let longest_active_time_100ns = persisted
        .as_ref()
        .map_or(0, |state| state.longest_active_time_100ns);
    if let Some(saved) = persisted.as_ref() {
        let same_boot =
            current_unbiased_time > 0 && saved.last_unbiased_time <= current_unbiased_time;
        let logon_identity_is_compatible =
            logon_time.is_none() || saved.logon_time.is_none() || logon_time == saved.logon_time;
        if same_boot && logon_identity_is_compatible {
            return SessionTimeState {
                logon_time,
                active_time_100ns: saved.active_time_100ns,
                longest_active_time_100ns,
                last_unbiased_time: current_unbiased_time,
                is_idle: false,
            };
        }
    }

    SessionTimeState {
        logon_time,
        active_time_100ns: 0,
        longest_active_time_100ns,
        last_unbiased_time: current_unbiased_time,
        is_idle: false,
    }
}

fn current_logon_time() -> Result<u64> {
    let mut session_id = 0;
    unsafe { ProcessIdToSessionId(std::process::id(), &mut session_id)? };
    let mut buffer = PWSTR::null();
    let mut bytes_returned = 0;
    unsafe {
        WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            session_id,
            WTSLogonTime,
            &mut buffer,
            &mut bytes_returned,
        )?;
    }

    if buffer.0.is_null() || bytes_returned < std::mem::size_of::<FILETIME>() as u32 {
        if !buffer.0.is_null() {
            unsafe { WTSFreeMemory(buffer.0.cast()) };
        }
        return Err("Windows returned an invalid session logon time".into());
    }

    let filetime = unsafe { *buffer.0.cast::<FILETIME>() };
    unsafe { WTSFreeMemory(buffer.0.cast()) };
    Ok(((filetime.dwHighDateTime as u64) << 32) | filetime.dwLowDateTime as u64)
}

fn session_time_path() -> Result<std::path::PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| "LOCALAPPDATA environment variable is not set")?;
    Ok(std::path::PathBuf::from(local_app_data)
        .join("com.seelen.seelen-ui")
        .join("active-session-time.log"))
}

fn load_persisted_state() -> Result<Option<PersistedSessionTime>> {
    let path = session_time_path()?;
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut latest = None;
            let mut highest_score = 0;
            for (line_number, line) in BufReader::new(file).lines().enumerate() {
                let line = line?;
                match serde_json::from_str::<PersistedSessionTime>(&line) {
                    Ok(mut record) => {
                        highest_score = highest_score.max(record.longest_active_time_100ns);
                        record.longest_active_time_100ns = highest_score;
                        latest = Some(record);
                    }
                    Err(_) if line.trim().is_empty() => {}
                    Err(error) => {
                        log::warn!(
                            "Ignoring invalid session-time log line {}: {error}",
                            line_number + 1
                        );
                    }
                }
            }
            if latest.is_some() {
                return Ok(latest);
            }
            load_legacy_persisted_state()
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => load_legacy_persisted_state(),
        Err(error) => Err(error.into()),
    }
}

fn load_legacy_persisted_state() -> Result<Option<PersistedSessionTime>> {
    let path = session_time_path()?.with_file_name("active-session-time.json");
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(Some(serde_json::from_str(&contents)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn persist_state(state: &SessionTimeState) -> Result<()> {
    let path = session_time_path()?;
    let persisted = PersistedSessionTime {
        timestamp_unix_seconds: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_secs(),
        logon_time: state.logon_time,
        active_time_100ns: state.active_time_100ns,
        last_unbiased_time: state.last_unbiased_time,
        longest_active_time_100ns: state.longest_active_time_100ns,
    };
    create_dir_all(
        path.parent()
            .ok_or_else(|| "Active session time path has no parent directory")?,
    )?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, &persisted)?;
    file.write_all(b"\n")?;
    file.flush()?;
    file.sync_data()?;
    Ok(())
}

pub(super) fn play_sound(sound: SessionTimeSound, custom_path: Option<String>) -> Result<()> {
    let alias = match sound {
        SessionTimeSound::SystemAlarm1 => "Notification.Looping.Alarm",
        SessionTimeSound::SystemAlarm2 => "Notification.Looping.Alarm2",
        SessionTimeSound::SystemAlarm3 => "Notification.Looping.Alarm3",
        SessionTimeSound::SystemAlarm4 => "Notification.Looping.Alarm4",
        SessionTimeSound::SystemAlarm5 => "Notification.Looping.Alarm5",
        SessionTimeSound::SystemAlarm6 => "Notification.Looping.Alarm6",
        SessionTimeSound::SystemAlarm7 => "Notification.Looping.Alarm7",
        SessionTimeSound::SystemAlarm8 => "Notification.Looping.Alarm8",
        SessionTimeSound::SystemAlarm9 => "Notification.Looping.Alarm9",
        SessionTimeSound::SystemAlarm10 => "Notification.Looping.Alarm10",
        SessionTimeSound::SystemAsterisk => "SystemAsterisk",
        SessionTimeSound::SystemCall1 => "Notification.Looping.Call",
        SessionTimeSound::SystemCall2 => "Notification.Looping.Call2",
        SessionTimeSound::SystemCall3 => "Notification.Looping.Call3",
        SessionTimeSound::SystemCall4 => "Notification.Looping.Call4",
        SessionTimeSound::SystemCall5 => "Notification.Looping.Call5",
        SessionTimeSound::SystemCall6 => "Notification.Looping.Call6",
        SessionTimeSound::SystemCall7 => "Notification.Looping.Call7",
        SessionTimeSound::SystemCall8 => "Notification.Looping.Call8",
        SessionTimeSound::SystemCall9 => "Notification.Looping.Call9",
        SessionTimeSound::SystemCall10 => "Notification.Looping.Call10",
        SessionTimeSound::SystemCriticalBattery => "CriticalBatteryAlarm",
        SessionTimeSound::SystemDeviceConnect => "DeviceConnect",
        SessionTimeSound::SystemDeviceDisconnect => "DeviceDisconnect",
        SessionTimeSound::SystemDeviceFail => "DeviceFail",
        SessionTimeSound::SystemDefault => "SystemDefault",
        SessionTimeSound::SystemExclamation => "SystemExclamation",
        SessionTimeSound::SystemExit => "SystemExit",
        SessionTimeSound::SystemFaxBeep => "FaxBeep",
        SessionTimeSound::SystemHand => "SystemHand",
        SessionTimeSound::SystemIm => "Notification.IM",
        SessionTimeSound::SystemLogon => "WindowsLogon",
        SessionTimeSound::SystemLowBattery => "LowBatteryAlarm",
        SessionTimeSound::SystemMail => "Notification.Mail",
        SessionTimeSound::SystemMessageNudge => "MessageNudge",
        SessionTimeSound::SystemNotification => "Notification.Default",
        SessionTimeSound::SystemProximity => "Notification.Proximity",
        SessionTimeSound::SystemProximityConnection => "ProximityConnection",
        SessionTimeSound::SystemQuestion => "SystemQuestion",
        SessionTimeSound::SystemReminder => "Notification.Reminder",
        SessionTimeSound::SystemStart => "SystemStart",
        SessionTimeSound::SystemUac => "WindowsUAC",
        SessionTimeSound::SystemUnlock => "WindowsUnlock",
        SessionTimeSound::SystemWelcome => "SystemWelcome",
        SessionTimeSound::CustomMp3 => {
            let Some(path) = custom_path else {
                return Err("No MP3 file is configured for the custom screen-time sound".into());
            };
            return play_mp3(path);
        }
    };
    if sound != SessionTimeSound::SystemDefault {
        let Some(path) = system_sound_path(sound) else {
            return Err(format!(
                "The Windows sound {sound:?} is not configured in the current sound scheme"
            )
            .into());
        };
        let path = U16CString::from_str(path.to_string_lossy().as_ref())
            .map_err(|_| "Windows system sound path contains an invalid null character")?;
        return if unsafe {
            PlaySoundW(
                PCWSTR(path.as_ptr()),
                None,
                SND_ASYNC
                    | windows::Win32::Media::Audio::SND_FILENAME
                    | windows::Win32::Media::Audio::SND_NODEFAULT,
            )
        }
        .as_bool()
        {
            Ok(())
        } else {
            Err(format!("Windows failed to play system sound: {sound:?}").into())
        };
    }
    let alias =
        U16CString::from_str(alias).map_err(|_| "Windows system sound alias contains a null")?;
    if unsafe {
        PlaySoundW(
            PCWSTR(alias.as_ptr()),
            None,
            SND_ALIAS | SND_ASYNC | SND_SYSTEM,
        )
    }
    .as_bool()
    {
        Ok(())
    } else {
        Err(format!("Windows failed to play system sound: {sound:?}").into())
    }
}

pub(super) fn available_system_sounds() -> Vec<SessionTimeSound> {
    let candidates = [
        SessionTimeSound::SystemAsterisk,
        SessionTimeSound::SystemExclamation,
        SessionTimeSound::SystemHand,
        SessionTimeSound::SystemQuestion,
        SessionTimeSound::SystemStart,
        SessionTimeSound::SystemExit,
        SessionTimeSound::SystemWelcome,
        SessionTimeSound::SystemDefault,
        SessionTimeSound::SystemAlarm1,
        SessionTimeSound::SystemAlarm2,
        SessionTimeSound::SystemAlarm3,
        SessionTimeSound::SystemAlarm4,
        SessionTimeSound::SystemAlarm5,
        SessionTimeSound::SystemAlarm6,
        SessionTimeSound::SystemAlarm7,
        SessionTimeSound::SystemAlarm8,
        SessionTimeSound::SystemAlarm9,
        SessionTimeSound::SystemAlarm10,
        SessionTimeSound::SystemCall1,
        SessionTimeSound::SystemCall2,
        SessionTimeSound::SystemCall3,
        SessionTimeSound::SystemCall4,
        SessionTimeSound::SystemCall5,
        SessionTimeSound::SystemCall6,
        SessionTimeSound::SystemCall7,
        SessionTimeSound::SystemCall8,
        SessionTimeSound::SystemCall9,
        SessionTimeSound::SystemCall10,
        SessionTimeSound::SystemCriticalBattery,
        SessionTimeSound::SystemDeviceConnect,
        SessionTimeSound::SystemDeviceDisconnect,
        SessionTimeSound::SystemDeviceFail,
        SessionTimeSound::SystemFaxBeep,
        SessionTimeSound::SystemIm,
        SessionTimeSound::SystemLogon,
        SessionTimeSound::SystemLowBattery,
        SessionTimeSound::SystemMail,
        SessionTimeSound::SystemMessageNudge,
        SessionTimeSound::SystemNotification,
        SessionTimeSound::SystemProximity,
        SessionTimeSound::SystemProximityConnection,
        SessionTimeSound::SystemReminder,
        SessionTimeSound::SystemUac,
        SessionTimeSound::SystemUnlock,
    ];
    let mut available = vec![SessionTimeSound::SystemDefault];
    let mut seen_paths = HashSet::new();
    for sound in candidates {
        if sound == SessionTimeSound::SystemDefault {
            continue;
        }
        let Some(path) = system_sound_path(sound) else {
            continue;
        };
        let key = path
            .canonicalize()
            .unwrap_or(path)
            .to_string_lossy()
            .to_lowercase();
        if seen_paths.insert(key) {
            available.push(sound);
        }
    }
    available
}

fn system_sound_path(sound: SessionTimeSound) -> Option<std::path::PathBuf> {
    let alias = match sound {
        SessionTimeSound::SystemAlarm1 => "Notification.Looping.Alarm",
        SessionTimeSound::SystemAlarm2 => "Notification.Looping.Alarm2",
        SessionTimeSound::SystemAlarm3 => "Notification.Looping.Alarm3",
        SessionTimeSound::SystemAlarm4 => "Notification.Looping.Alarm4",
        SessionTimeSound::SystemAlarm5 => "Notification.Looping.Alarm5",
        SessionTimeSound::SystemAlarm6 => "Notification.Looping.Alarm6",
        SessionTimeSound::SystemAlarm7 => "Notification.Looping.Alarm7",
        SessionTimeSound::SystemAlarm8 => "Notification.Looping.Alarm8",
        SessionTimeSound::SystemAlarm9 => "Notification.Looping.Alarm9",
        SessionTimeSound::SystemAlarm10 => "Notification.Looping.Alarm10",
        SessionTimeSound::SystemAsterisk => "SystemAsterisk",
        SessionTimeSound::SystemCall1 => "Notification.Looping.Call",
        SessionTimeSound::SystemCall2 => "Notification.Looping.Call2",
        SessionTimeSound::SystemCall3 => "Notification.Looping.Call3",
        SessionTimeSound::SystemCall4 => "Notification.Looping.Call4",
        SessionTimeSound::SystemCall5 => "Notification.Looping.Call5",
        SessionTimeSound::SystemCall6 => "Notification.Looping.Call6",
        SessionTimeSound::SystemCall7 => "Notification.Looping.Call7",
        SessionTimeSound::SystemCall8 => "Notification.Looping.Call8",
        SessionTimeSound::SystemCall9 => "Notification.Looping.Call9",
        SessionTimeSound::SystemCall10 => "Notification.Looping.Call10",
        SessionTimeSound::SystemCriticalBattery => "CriticalBatteryAlarm",
        SessionTimeSound::SystemDeviceConnect => "DeviceConnect",
        SessionTimeSound::SystemDeviceDisconnect => "DeviceDisconnect",
        SessionTimeSound::SystemDeviceFail => "DeviceFail",
        SessionTimeSound::SystemExclamation => "SystemExclamation",
        SessionTimeSound::SystemExit => "SystemExit",
        SessionTimeSound::SystemFaxBeep => "FaxBeep",
        SessionTimeSound::SystemHand => "SystemHand",
        SessionTimeSound::SystemIm => "Notification.IM",
        SessionTimeSound::SystemLogon => "WindowsLogon",
        SessionTimeSound::SystemLowBattery => "LowBatteryAlarm",
        SessionTimeSound::SystemMail => "Notification.Mail",
        SessionTimeSound::SystemMessageNudge => "MessageNudge",
        SessionTimeSound::SystemNotification => "Notification.Default",
        SessionTimeSound::SystemProximity => "Notification.Proximity",
        SessionTimeSound::SystemProximityConnection => "ProximityConnection",
        SessionTimeSound::SystemQuestion => "SystemQuestion",
        SessionTimeSound::SystemReminder => "Notification.Reminder",
        SessionTimeSound::SystemStart => "SystemStart",
        SessionTimeSound::SystemUac => "WindowsUAC",
        SessionTimeSound::SystemUnlock => "WindowsUnlock",
        SessionTimeSound::SystemWelcome => "SystemWelcome",
        SessionTimeSound::SystemDefault | SessionTimeSound::CustomMp3 => return None,
    };
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(format!(r"AppEvents\Schemes\Apps\.Default\{alias}\.Current"))
        .ok()?;
    let value: String = key.get_value("").ok()?;
    let path = std::path::PathBuf::from(value);
    path.is_file().then_some(path)
}

fn play_mp3(path: String) -> Result<()> {
    if !std::path::Path::new(&path).is_file() {
        return Err("The selected screen-time MP3 file no longer exists".into());
    }
    if CUSTOM_SOUND_PLAYING
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        return Err("The previous screen-time MP3 cue is still playing".into());
    }
    let (started_sender, started_receiver) = mpsc::channel();
    if let Err(error) = thread::Builder::new()
        .name("screen-time-audio-cue".to_string())
        .spawn(move || {
            let result = (|| {
                let open_command =
                    format!("open \"{path}\" type mpegvideo alias slu_session_time_cue");
                send_mci_command(&open_command)
                    .map_err(|error| format!("Could not open the selected MP3: {error}"))?;
                send_mci_command("setaudio slu_session_time_cue volume to 1000")
                    .map_err(|error| format!("Could not configure MP3 playback: {error}"))?;
                send_mci_command("play slu_session_time_cue")
                    .map_err(|error| format!("Could not play the selected MP3: {error}"))?;
                Ok::<(), String>(())
            })();
            let started = result.is_ok();
            let _ = started_sender.send(result);
            if started {
                let started_at = Instant::now();
                loop {
                    thread::sleep(Duration::from_millis(250));
                    if started_at.elapsed() >= MAX_CUSTOM_SOUND_PLAYBACK {
                        if let Err(error) = send_mci_command("stop slu_session_time_cue") {
                            log::error!(
                                "Failed to stop screen-time MP3 at its 30-second limit: {error}"
                            );
                        }
                        break;
                    }
                    match query_mci_command("status slu_session_time_cue mode") {
                        Ok(mode) if mode.eq_ignore_ascii_case("playing") => {}
                        Ok(_) => break,
                        Err(error) => {
                            log::error!("Failed to monitor screen-time MP3 playback: {error}");
                            break;
                        }
                    }
                }
            }
            if let Err(error) = send_mci_command("close slu_session_time_cue") {
                log::debug!("Could not close screen-time MP3 device: {error}");
            }
            CUSTOM_SOUND_PLAYING.store(false, Ordering::Release);
        })
    {
        CUSTOM_SOUND_PLAYING.store(false, Ordering::Release);
        return Err(error.into());
    }
    match started_receiver.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(error.into()),
        Err(error) => Err(format!("MP3 playback worker exited unexpectedly: {error}").into()),
    }
}

fn send_mci_command(command: &str) -> Result<()> {
    let command_string =
        U16CString::from_str(command).map_err(|_| "Windows multimedia command contains a null")?;
    let result = unsafe { mciSendStringW(PCWSTR(command_string.as_ptr()), None, None) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!("Windows multimedia error {result} while running `{command}`").into())
    }
}

fn query_mci_command(command: &str) -> Result<String> {
    let command_string =
        U16CString::from_str(command).map_err(|_| "Windows multimedia command contains a null")?;
    let mut response = [0u16; 64];
    let result =
        unsafe { mciSendStringW(PCWSTR(command_string.as_ptr()), Some(&mut response), None) };
    if result != 0 {
        return Err(format!("Windows multimedia error {result} while running `{command}`").into());
    }
    Ok(String::from_utf16_lossy(&response)
        .trim_matches(char::from(0))
        .trim()
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        IDLE_THRESHOLD_MILLIS, PersistedSessionTime, initial_session_state,
        should_count_active_time,
    };

    #[test]
    fn counts_interactive_time_until_idle_threshold() {
        assert!(should_count_active_time(true, IDLE_THRESHOLD_MILLIS, false));
        assert!(!should_count_active_time(
            true,
            IDLE_THRESHOLD_MILLIS + 1,
            false
        ));
    }

    #[test]
    fn playing_media_counts_time_while_idle_or_locked() {
        assert!(should_count_active_time(
            true,
            IDLE_THRESHOLD_MILLIS + 1,
            true
        ));
        assert!(should_count_active_time(false, u32::MAX, true));
    }

    #[test]
    fn locked_session_without_media_is_not_counted() {
        assert!(!should_count_active_time(false, 0, false));
    }

    #[test]
    fn restores_time_only_for_the_same_windows_logon() {
        let persisted = Some(PersistedSessionTime {
            timestamp_unix_seconds: 0,
            logon_time: Some(12),
            active_time_100ns: 100,
            last_unbiased_time: 50,
            longest_active_time_100ns: 200,
        });

        let restored = initial_session_state(Some(12), persisted, 80);
        assert_eq!(restored.active_time_100ns, 100);
        assert_eq!(restored.longest_active_time_100ns, 200);
        assert_eq!(restored.last_unbiased_time, 80);
        assert_eq!(restored.logon_time, Some(12));
    }

    #[test]
    fn crash_recovery_does_not_count_time_since_the_last_checkpoint() {
        let restored = initial_session_state(
            None,
            Some(PersistedSessionTime {
                timestamp_unix_seconds: 0,
                logon_time: None,
                active_time_100ns: 100,
                last_unbiased_time: 50,
                longest_active_time_100ns: 200,
            }),
            500,
        );
        assert_eq!(restored.active_time_100ns, 100);
        assert_eq!(restored.last_unbiased_time, 500);
    }

    #[test]
    fn a_new_logon_resets_current_time_but_keeps_the_high_score() {
        let state = initial_session_state(
            Some(13),
            Some(PersistedSessionTime {
                timestamp_unix_seconds: 0,
                logon_time: Some(12),
                active_time_100ns: 100,
                last_unbiased_time: 50,
                longest_active_time_100ns: 200,
            }),
            80,
        );
        assert_eq!(state.active_time_100ns, 0);
        assert_eq!(state.longest_active_time_100ns, 200);
        assert_eq!(state.logon_time, Some(13));
    }

    #[test]
    fn unknown_logon_restores_same_boot_time_and_score() {
        let state = initial_session_state(
            None,
            Some(PersistedSessionTime {
                timestamp_unix_seconds: 0,
                logon_time: Some(20),
                active_time_100ns: 100,
                last_unbiased_time: 50,
                longest_active_time_100ns: 200,
            }),
            80,
        );
        assert_eq!(state.active_time_100ns, 100);
        assert_eq!(state.longest_active_time_100ns, 200);
    }

    #[test]
    fn unknown_logon_after_reboot_resets_current_time_but_keeps_the_high_score() {
        let state = initial_session_state(
            None,
            Some(PersistedSessionTime {
                timestamp_unix_seconds: 0,
                logon_time: None,
                active_time_100ns: 100,
                last_unbiased_time: 90,
                longest_active_time_100ns: 200,
            }),
            10,
        );
        assert_eq!(state.active_time_100ns, 0);
        assert_eq!(state.longest_active_time_100ns, 200);
    }
}
