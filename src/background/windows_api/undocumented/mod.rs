mod audio_policy_config;
mod process_image_name;
mod quiet_hours_settings;

pub use audio_policy_config::*;
pub use quiet_hours_settings::*;
use windows::Win32::{
    Media::Audio::{eCommunications, eConsole, eMultimedia},
    Security::SE_DEBUG_NAME,
};

use crate::{error::Result, windows_api::string_utils::WindowsString};

use super::{Com, WindowsApi};

impl WindowsApi {
    /// this function will only work on win32 platform, if using UWP/MSIX
    /// it will be blocked or not translated idk what exactly happens but
    /// as workaround we call this function via cli to set the default device
    pub fn set_default_audio_device(id: &str, role: &str) -> Result<()> {
        let role = match role {
            "multimedia" => eMultimedia,
            "communications" => eCommunications,
            "console" => eConsole,
            _ => return Err("invalid role".into()),
        };

        Com::run_with_context(|| unsafe {
            WindowsApi::enable_privilege(SE_DEBUG_NAME)?;
            let policy: IPolicyConfig = Com::create_instance(&PolicyConfig)?;
            let id = WindowsString::from_str(id);
            policy.SetDefaultEndpoint(id.as_pcwstr(), role)?;
            Ok(())
        })
    }

    /// Returns the id of the active quiet hours profile (`Microsoft.QuietHoursProfile.*`),
    /// the one behind the "Do not disturb" / "Focus assist" toggle.
    pub fn get_quiet_hours_profile() -> Result<String> {
        Com::run_with_context(|| unsafe {
            let settings: IQuietHoursSettings = Com::create_instance(&QuietHoursSettings)?;
            let profile = settings.UserSelectedProfile()?;
            let result = profile.to_string();
            Com::task_mem_free(profile.as_ptr() as _);
            Ok(result?)
        })
    }

    /// Changes the active quiet hours profile, the same way the "Do not disturb" /
    /// "Focus assist" toggle of the system does.
    pub fn set_quiet_hours_profile(profile_id: &str) -> Result<()> {
        Com::run_with_context(|| unsafe {
            let settings: IQuietHoursSettings = Com::create_instance(&QuietHoursSettings)?;
            let profile_id = WindowsString::from_str(profile_id);
            settings.SetUserSelectedProfile(profile_id.as_pcwstr())?;
            Ok(())
        })
    }
}
