use std::path::PathBuf;

use seelen_core::system_state::User;
use windows::Win32::{
    Security::Authentication::Identity::{NameDisplay, NameSamCompatible},
    System::SystemInformation::ComputerNameDnsDomain,
};
use winreg::{
    RegKey,
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
};

use crate::{error::Result, system_modules::user::domain::UserSystem, windows_api::WindowsApi};

/// Registry values of the account picture, from highest to lowest quality.
const PICTURE_QUALITIES: [&str; 11] = [
    "Image1080",
    "Image448",
    "Image424",
    "Image240",
    "Image208",
    "Image192",
    "Image96",
    "Image64",
    "Image48",
    "Image40",
    "Image32",
];

/// Stateless: everything is read from the registry and the Win32 account APIs on demand.
pub struct WindowsUser;

impl UserSystem for WindowsUser {
    fn create() -> Self {
        Self
    }

    fn logged_user(&self) -> User {
        let domain = WindowsApi::get_computer_name(ComputerNameDnsDomain).unwrap_or_default();
        let name = WindowsApi::get_username(NameDisplay)
            .or_else(|_| -> Result<String> {
                // A legacy account name (for example, Engineering\JSmith).
                // The domain-only version includes trailing backslashes (\).
                let name = WindowsApi::get_username(NameSamCompatible)?;
                let name = name.split("\\").last().unwrap_or_default();
                match name.is_empty() {
                    true => Err("Empty username".into()),
                    false => Ok(name.to_string()),
                }
            })
            .unwrap_or_else(|_| "???".to_string()); // no username

        let mut user = User {
            name,
            domain,
            profile_home_path: PathBuf::new(), // deprecated, remove this is unnecessary
            email: None,
            one_drive_path: None,
            profile_picture_path: None,
            xbox_gamertag: None,
        };

        if let Ok(sid) = logged_on_user_sid() {
            user.profile_picture_path = profile_picture_path(&sid).ok();
            if let Ok((user_mail, one_drive_path)) = one_drive_attributes() {
                user.email = Some(user_mail);
                user.one_drive_path = Some(one_drive_path);
            }
        }

        user.xbox_gamertag = xbox_gamertag().ok();
        user
    }
}

fn logged_on_user_sid() -> Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let settings =
        hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\LogonUI")?;
    Ok(settings.get_value("LastLoggedOnUserSID")?)
}

/// Tries every known picture quality from highest to lowest, since Windows only
/// populates the registry values it actually generated: Windows 11 writes `Image1080`,
/// but Windows 10 stops at `Image448`, so requesting `Image1080` there always fails.
fn profile_picture_path(sid: &str) -> Result<PathBuf> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let settings = hklm.open_subkey(
        format!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AccountPicture\\Users\\{sid}")
            .as_str(),
    )?;

    for quality in PICTURE_QUALITIES {
        if let Ok(path) = settings.get_value::<String, _>(quality) {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    Err("No profile picture found for any known quality".into())
}

fn one_drive_attributes() -> Result<(String, PathBuf)> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu.open_subkey("SOFTWARE\\Microsoft\\OneDrive\\Accounts\\Personal")?;
    let email: String = settings.get_value("UserEmail")?;
    let path: String = settings.get_value("UserFolder")?;
    Ok((email, PathBuf::from(path)))
}

fn xbox_gamertag() -> Result<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu.open_subkey("SOFTWARE\\Microsoft\\XboxLive")?;
    Ok(settings.get_value("Gamertag")?)
}
