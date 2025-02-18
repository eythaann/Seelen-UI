use lazy_static::lazy_static;
use notify_debouncer_full::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode, Watcher},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use parking_lot::Mutex;
use seelen_core::system_state::{File, FolderType, User};
use std::{
    collections::{HashMap, HashSet},
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::Arc,
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
    error_handler::Result, event_manager, log_error, seelen::get_app_handle, trace_lock,
    windows_api::WindowsApi,
};

use super::domain::PictureQuality;

lazy_static! {
    pub static ref USER_MANAGER: Arc<Mutex<UserManager>> = Arc::new(Mutex::new(
        UserManager::new().expect("Failed to create user manager")
    ));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserManagerEvent {
    #[allow(dead_code)]
    UserUpdated(),
    FolderChanged(FolderType),
}

#[derive(Debug)]
pub struct UserManager {
    pub user: User,
    pub folders: HashMap<FolderType, UserFolderDetails>,
    folder_watcher: Option<Arc<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>>,
}

#[derive(Debug)]
pub struct UserFolderDetails {
    pub path: PathBuf,
    pub limit: usize,
    pub content: Vec<File>,
}

unsafe impl Send for UserManager {}
unsafe impl Send for UserManagerEvent {}

event_manager!(UserManager, UserManagerEvent);

impl UserManager {
    fn get_path_from_folder(folder_type: &FolderType) -> Result<PathBuf> {
        let resolver = get_app_handle().path();
        Ok(match folder_type {
            FolderType::Recent => resolver.data_dir()?.join("Microsoft\\Windows\\Recent"),
            FolderType::Desktop => resolver.desktop_dir()?,
            FolderType::Downloads => resolver.download_dir()?,
            FolderType::Documents => resolver.document_dir()?,
            FolderType::Pictures => resolver.picture_dir()?,
            FolderType::Videos => resolver.video_dir()?,
            FolderType::Music => resolver.audio_dir()?,
            FolderType::Unknown => {
                return Err("There is no such folder could be handled!".into());
            }
        })
    }

    fn get_folder_from_path(path: &Path) -> Result<FolderType> {
        for folder_type in FolderType::values() {
            if path.starts_with(Self::get_path_from_folder(folder_type)?) {
                return Ok(*folder_type);
            }
        }
        Ok(FolderType::Unknown)
    }

    fn get_logged_on_user_sid() -> Result<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let settings = hklm
            .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\LogonUI")?;
        Ok(settings.get_value("LastLoggedOnUserSID")?)
    }

    fn get_folder_content(path: PathBuf, limit: usize) -> Result<Vec<File>> {
        let mut list = Vec::new();

        for entry in std::fs::read_dir(path)?.flatten() {
            let mut path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(extension) = path.extension() {
                if extension == "ini" {
                    continue;
                }
                if extension == "lnk" {
                    path = match WindowsApi::resolve_lnk_target(&path) {
                        Ok((target, _)) if target.is_file() => target,
                        _ => continue,
                    };
                }
            }

            list.push(File {
                path,
                last_access_time: entry.metadata()?.last_write_time(),
            });

            if list.len() >= limit {
                break;
            }
        }

        list.sort_by_key(|item| item.last_access_time);
        list.reverse();
        Ok(list)
    }

    fn get_user_profile_picture_path(sid: &str, quality: PictureQuality) -> Result<PathBuf> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let settings = hklm.open_subkey(
            format!(
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AccountPicture\\Users\\{}",
                sid
            )
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
                let name = name.trim_start_matches(&format!("{}\\", domain));
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

    fn join_debounced_changes(events: Vec<DebouncedEvent>) -> HashSet<PathBuf> {
        let mut result = HashSet::new();
        for event in events {
            for path in event.event.paths {
                result.insert(path);
            }
        }
        result
    }

    fn on_files_changed(changed: HashSet<PathBuf>) -> Result<()> {
        for path in changed {
            let folder_type = Self::get_folder_from_path(&path)?;
            if folder_type == FolderType::Unknown {
                continue;
            }
            let mut guard = trace_lock!(USER_MANAGER);
            if let Some(model) = guard.folders.get_mut(&folder_type) {
                model.content = Self::get_folder_content(model.path.clone(), model.limit)?;
                log_error!(Self::event_tx().send(UserManagerEvent::FolderChanged(folder_type)));
            }
        }
        Ok(())
    }

    fn create_file_watcher(
        &self,
    ) -> Result<Arc<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>> {
        let mut debouncer = new_debouncer(
            Duration::from_millis(100),
            None,
            |result: DebounceEventResult| match result {
                Ok(events) => {
                    let paths = Self::join_debounced_changes(events);
                    log_error!(Self::on_files_changed(paths));
                }
                Err(errors) => {
                    log::error!("RecentFile Watcher Error: {:?}", errors)
                }
            },
        )?;
        let watcher = debouncer.watcher();
        for item in FolderType::values() {
            if let Some(item) = self.folders.get(item) {
                watcher.watch(&item.path, RecursiveMode::Recursive)?;
            }
        }
        Ok(Arc::new(debouncer))
    }

    pub fn new() -> Result<Self> {
        let mut instance = Self {
            user: Self::get_logged_user(),
            folder_watcher: None,
            folders: HashMap::new(),
        };

        for folder in FolderType::values() {
            let folder_path = Self::get_path_from_folder(folder)?;
            instance.folders.insert(
                *folder,
                UserFolderDetails {
                    path: folder_path.clone(),
                    limit: 20,
                    content: Self::get_folder_content(folder_path, 20)?,
                },
            );
        }

        instance.folder_watcher = Some(instance.create_file_watcher()?);
        // TODO add event listeners for account information changes.
        Ok(instance)
    }

    pub fn set_folder_limit(&mut self, folder_type: FolderType, limit: usize) -> Result<()> {
        if let Some(details) = self.folders.get_mut(&folder_type) {
            details.limit = limit;
            details.content = Self::get_folder_content(details.path.clone(), limit)?;
        }
        log_error!(Self::event_tx().send(UserManagerEvent::FolderChanged(folder_type)));
        Ok(())
    }
}
