use itertools::Itertools;
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
    fs::DirEntry,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tauri::Manager;
use windows::Win32::{
    Security::Authentication::Identity::NameDisplay,
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
        Ok(match folder_type {
            FolderType::Recent => get_app_handle()
                .path()
                .data_dir()?
                .as_path()
                .join("Microsoft\\Windows\\Recent"),
            FolderType::Desktop => get_app_handle().path().desktop_dir()?,
            FolderType::Downloads => get_app_handle().path().download_dir()?,
            FolderType::Documents => get_app_handle().path().document_dir()?,
            FolderType::Pictures => get_app_handle().path().picture_dir()?,
            FolderType::Videos => get_app_handle().path().video_dir()?,
            FolderType::Music => get_app_handle().path().audio_dir()?,
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

    fn get_recursive_folder(path: PathBuf) -> Box<dyn Iterator<Item = DirEntry>> {
        let read_dir = std::fs::read_dir(path);

        if let Ok(result) = read_dir {
            Box::new(result.flatten().flat_map(|dir| {
                if dir.path().is_dir() {
                    UserManager::get_recursive_folder(dir.path())
                } else {
                    Box::new([dir].into_iter())
                }
            }))
        } else {
            Box::new([].into_iter())
        }
    }

    fn get_folder_content(path: PathBuf, limit: usize, is_recursive: bool) -> Result<Vec<File>> {
        let folders = if is_recursive {
            UserManager::get_recursive_folder(path)
        } else {
            Box::new(std::fs::read_dir(path)?.map(|r| r.unwrap()))
        };

        let files = folders
            .filter(|item| {
                let pathbuf = item.path();
                let path = pathbuf.as_path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "lnk" {
                            if let Ok((result, _)) = WindowsApi::resolve_lnk_target(path) {
                                if result.exists() {
                                    return !result.is_dir();
                                }
                            }
                        } else {
                            return extension != "ini";
                        }
                    }
                }

                false
            })
            .map(|item| {
                let mut current_target = item.path();
                if current_target.extension().unwrap() == "lnk" {
                    if let Ok((target, _)) = WindowsApi::resolve_lnk_target(&current_target) {
                        current_target = target;
                    }
                }

                File {
                    path: std::fs::canonicalize(current_target.clone())
                        .unwrap()
                        .to_str()
                        .unwrap()[4..]
                        .into(),
                    last_access_time: if let Ok(metadata) = item.metadata() {
                        metadata.last_write_time()
                    } else {
                        0
                    },
                }
            })
            .sorted_by_key(|item| {
                if item.last_access_time == 0 {
                    i64::MIN
                } else {
                    -(item.last_access_time as i64)
                }
            })
            .take(limit)
            .collect();

        Ok(files)
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
        let mut user = User {
            name: WindowsApi::get_username(NameDisplay).unwrap_or_default(),
            domain: WindowsApi::get_computer_name(ComputerNameDnsDomain).unwrap_or_default(),
            profile_home_path: PathBuf::new(), // deprecated, remove this is unncessary
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
                model.content =
                    UserManager::get_folder_content(model.path.clone(), model.limit, true)?;
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
                    content: Self::get_folder_content(
                        folder_path,
                        20,
                        *folder != FolderType::Recent,
                    )?,
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
            details.content = Self::get_folder_content(details.path.clone(), limit, false)?;
        }
        log_error!(Self::event_tx().send(UserManagerEvent::FolderChanged(folder_type)));
        Ok(())
    }
}
