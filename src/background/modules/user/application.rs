use notify_debouncer_full::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode, Watcher},
    DebounceEventResult, Debouncer, FileIdMap,
};
use parking_lot::Mutex;
use seelen_core::system_state::{FolderType, User};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tauri::Manager;
use windows::Win32::{
    Security::Authentication::Identity::{NameDisplay, NameSamCompatible},
    System::SystemInformation::ComputerNameDnsDomain,
};
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

use crate::{
    app::get_app_handle, error::Result, event_manager, log_error, trace_lock,
    windows_api::WindowsApi,
};

use super::domain::PictureQuality;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserManagerEvent {
    #[allow(dead_code)]
    UserUpdated,
    FolderChanged(FolderType),
}

#[derive(Debug)]
pub struct FolderDetails {
    pub path: PathBuf,
    pub content: Vec<PathBuf>,
    _watcher: Debouncer<ReadDirectoryChangesWatcher, FileIdMap>,
}

#[derive(Debug)]
pub struct UserManager {
    pub user: User,
    pub folders: HashMap<FolderType, FolderDetails>,
}

unsafe impl Send for UserManager {}
unsafe impl Send for UserManagerEvent {}

event_manager!(UserManager, UserManagerEvent);

impl UserManager {
    pub fn instance() -> &'static Arc<Mutex<Self>> {
        static USER_MANAGER: LazyLock<Arc<Mutex<UserManager>>> = LazyLock::new(|| {
            Arc::new(Mutex::new(
                UserManager::new().expect("Failed to create user manager"),
            ))
        });
        &USER_MANAGER
    }

    fn get_path_from_folder(folder_type: &FolderType) -> Option<PathBuf> {
        let resolver = get_app_handle().path();
        match folder_type {
            FolderType::Recent => {
                Some(resolver.data_dir().ok()?.join("Microsoft\\Windows\\Recent"))
            }
            FolderType::Desktop => resolver.desktop_dir().ok(),
            FolderType::Downloads => resolver.download_dir().ok(),
            FolderType::Documents => resolver.document_dir().ok(),
            FolderType::Pictures => resolver.picture_dir().ok(),
            FolderType::Videos => resolver.video_dir().ok(),
            FolderType::Music => resolver.audio_dir().ok(),
        }
    }

    fn get_logged_on_user_sid() -> Result<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let settings = hklm
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\LogonUI")?;
        Ok(settings.get_value("LastLoggedOnUserSID")?)
    }

    fn get_folder_content(base_path: PathBuf) -> Result<Vec<PathBuf>> {
        let mut list = Vec::new();

        for entry in walkdir::WalkDir::new(base_path)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            list.push(path.to_path_buf());
        }

        Ok(list)
    }

    fn get_user_profile_picture_path(sid: &str, quality: PictureQuality) -> Result<PathBuf> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let settings = hklm.open_subkey(
            format!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AccountPicture\\Users\\{sid}")
                .as_str(),
        )?;
        let path: String = settings.get_value(quality.as_str())?;
        Ok(path.into())
    }

    fn get_one_drive_attributes() -> Result<(String, PathBuf)> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hkcu.open_subkey("SOFTWARE\\Microsoft\\OneDrive\\Accounts\\Personal")?;
        let email: String = settings.get_value("UserEmail")?;
        let path: String = settings.get_value("UserFolder")?;
        Ok((email, PathBuf::from(path)))
    }

    fn get_logged_user() -> User {
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
        };

        if let Ok(sid) = Self::get_logged_on_user_sid() {
            user.profile_picture_path =
                Self::get_user_profile_picture_path(&sid, PictureQuality::Quality1080).ok();
            if let Ok((user_mail, one_drive_path)) = Self::get_one_drive_attributes() {
                user.email = Some(user_mail);
                user.one_drive_path = Some(one_drive_path);
            }
        }

        user
    }

    fn create_folder_watcher(
        folder_type: FolderType,
    ) -> Result<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>> {
        let debouncer = new_debouncer(
            Duration::from_millis(300),
            None,
            move |result: DebounceEventResult| match result {
                Ok(_events) => {
                    log_error!(Self::reload_folder_content(folder_type));
                }
                Err(errors) => {
                    log::error!("Folder Watcher Error for {:?}: {errors:?}", folder_type);
                }
            },
        )?;
        Ok(debouncer)
    }

    fn reload_folder_content(folder_type: FolderType) -> Result<()> {
        let mut manager = trace_lock!(Self::instance());

        if let Some(folder_details) = manager.folders.get_mut(&folder_type) {
            folder_details.content = Self::get_folder_content(folder_details.path.clone())?;
            drop(manager);
            let _ = Self::event_tx().send(UserManagerEvent::FolderChanged(folder_type));
        }

        Ok(())
    }

    pub fn new() -> Result<Self> {
        let mut folders = HashMap::new();

        for &folder_type in FolderType::values() {
            if let Some(path) = Self::get_path_from_folder(&folder_type) {
                let content = Self::get_folder_content(path.clone()).unwrap_or_default();
                let mut watcher = Self::create_folder_watcher(folder_type)?;
                watcher.watcher().watch(&path, RecursiveMode::Recursive)?;

                folders.insert(
                    folder_type,
                    FolderDetails {
                        path,
                        content,
                        _watcher: watcher,
                    },
                );
            }
        }

        Ok(Self {
            user: Self::get_logged_user(),
            folders,
        })
    }
}
