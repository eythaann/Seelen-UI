mod audio_policy_config;

pub use audio_policy_config::*;
use windows::Win32::{
    Media::Audio::{eCommunications, eConsole, eMultimedia},
    Security::SE_DEBUG_NAME,
};

use crate::{error_handler::Result, windows_api::string_utils::WindowsString};

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
}
