use getset::Getters;
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
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

use crate::{
    error_handler::Result, event_manager, log_error, seelen::get_app_handle, trace_lock,
    utils::spawn_named_thread, windows_api::WindowsApi,
};

use super::domain::PictureQuality;

const USER_PROFILE_REG_PATH_PATTERN: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AccountPicture\\Users\\";
const USER_PROFILE_ONEDRIVE_PATH: &str = "SOFTWARE\\Microsoft\\OneDrive\\Accounts\\Personal";
const USER_PROFILE_ONEDRIVE_EMAIL_KEY: &str = "UserEmail";
const USER_PROFILE_ONEDRIVE_FOLDER_KEY: &str = "UserFolder";
const USER_SID_AUTH_PATH: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\LogonUI";
const USER_SID_AUTH_PATH_KEY: &str = "LastLoggedOnUserSID";
const USER_USERNAME_AUTH_PATH_KEY: &str = "LastLoggedOnUser";

lazy_static! {
    pub static ref USER_MANAGER: Arc<Mutex<UserManager>> = Arc::new(Mutex::new(
        UserManager::new().expect("Failed to create user manager")
    ));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserManagerEvent {
    UserUpdated(),
    FolderChanged(FolderType),
}

#[derive(Debug, Getters)]
pub struct UserManager {
    user_sid: String,
    #[getset(get = "pub")]
    user_details: Option<User>,
    folder_watcher: Option<Arc<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>>,
    #[getset(get = "pub")]
    folders: HashMap<FolderType, UserFolderDetails>,
}

#[derive(Debug, Getters)]
pub struct UserFolderDetails {
    #[getset(get = "pub")]
    path: PathBuf,
    #[getset(get = "pub")]
    limit: usize,
    #[getset(get = "pub")]
    content: Option<Vec<File>>,
}

unsafe impl Send for UserManager {}
unsafe impl Send for UserManagerEvent {}

event_manager!(UserManager, UserManagerEvent);

// Static
impl UserManager {
    pub fn new() -> Result<Self> {
        let mut instance = Self {
            user_sid: Self::get_user_sid().ok().unwrap(),
            user_details: None,
            folder_watcher: None,
            folders: HashMap::new(),
        };
        instance.user_details = instance.create_user().ok();

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
                    )
                    .ok(),
                },
            );
        }

        instance.folder_watcher = Some(instance.create_file_watcher()?);

        spawn_named_thread("User Manager", move || loop {
            let mut changed = false;
            {
                let mut manager = trace_lock!(USER_MANAGER);
                let changeable_attributes = (
                    manager.get_one_drive_attributes(),
                    manager.get_user_profile_picture_path(PictureQuality::Quality1080),
                );
                if let Some(ref mut current_user) = manager.user_details {
                    if let (Ok((mail, drive_path)), Ok(picture_path)) = changeable_attributes {
                        let mail_option = Some(mail);
                        let drive_option = Some(drive_path);
                        let picture_option = Some(picture_path);

                        if current_user.email != mail_option
                            || current_user.one_drive_path != drive_option
                            || current_user.profile_picture_path != picture_option
                        {
                            current_user.email = mail_option;
                            current_user.one_drive_path = drive_option;
                            current_user.profile_picture_path = picture_option;
                            changed = true;
                        }
                    } else if let (Err(_), Ok(picture_path)) = changeable_attributes {
                        let picture_option = Some(picture_path);
                        if current_user.email.is_some()
                            || current_user.one_drive_path.is_some()
                            || current_user.profile_picture_path != picture_option
                        {
                            current_user.email = None;
                            current_user.one_drive_path = None;
                            current_user.profile_picture_path = picture_option;
                            changed = true;
                        }
                    }
                } else if let Ok(user) = manager.create_user() {
                    manager.user_details = Some(user);
                    changed = true;
                }
            }

            if changed {
                let sender = Self::event_tx();
                log_error!(sender.send(UserManagerEvent::UserUpdated()))
            }

            std::thread::sleep(Duration::from_millis(5000));
        })?;

        Ok(instance)
    }

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

    fn get_user_sid() -> Result<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let settings = hklm.open_subkey(USER_SID_AUTH_PATH)?;
        let sid: String = settings.get_value(USER_SID_AUTH_PATH_KEY)?;
        Ok(sid)
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

        Ok(folders
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
            .collect())
    }
}

// Instance
impl UserManager {
    pub fn create_user(&self) -> Result<User> {
        let elevated = WindowsApi::is_elevated()?;

        let (username, domain, profile_home) = if !elevated {
            (
                std::env!("USERNAME").to_string(),
                std::env!("USERDOMAIN").to_string(),
                PathBuf::from(std::env!("USERPROFILE")),
            )
        } else {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let settings = hklm.open_subkey(USER_SID_AUTH_PATH)?;
            let username: String = settings.get_value(USER_USERNAME_AUTH_PATH_KEY)?;
            let splitted = username.split('\\').collect::<Vec<&str>>();
            (
                splitted[1].to_string(),
                splitted[0].to_string(),
                get_app_handle().path().home_dir()?,
            )
        };

        let mut user = User {
            name: username,
            domain,
            profile_home_path: profile_home,
            email: None,
            one_drive_path: None,
            profile_picture_path: None,
        };
        log::trace!("User {:?}", user);
        user.profile_picture_path = self
            .get_user_profile_picture_path(PictureQuality::Quality1080)
            .ok();
        if let Ok((user_mail, one_drive_path)) = self.get_one_drive_attributes() {
            user.email = Some(user_mail);
            user.one_drive_path = Some(one_drive_path);
        }

        Ok(user)
    }

    pub fn set_folder_limit(&mut self, folder_type: FolderType, limit: usize) -> Result<()> {
        let folder = self.folders.get_mut(&folder_type);
        if let Some(model) = folder {
            model.limit = limit;
            model.content = Self::get_folder_content(model.path.clone(), limit, false).ok();
        }

        let sender = Self::event_tx();
        log_error!(sender.send(UserManagerEvent::FolderChanged(folder_type)));

        Ok(())
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

            let folders = &mut trace_lock!(USER_MANAGER).folders;
            let folder = folders.get_mut(&folder_type);
            if let Some(model) = folder {
                model.content =
                    UserManager::get_folder_content(model.path.clone(), model.limit, true).ok();
                let sender = Self::event_tx();
                log_error!(sender.send(UserManagerEvent::FolderChanged(folder_type)));
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

    fn get_user_profile_picture_path(&self, quality: PictureQuality) -> Result<PathBuf> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let mut key = String::default();
        key.push_str(USER_PROFILE_REG_PATH_PATTERN);
        key.push_str(self.user_sid.as_str());
        let settings = hklm.open_subkey(key)?;
        let path: String = settings.get_value(quality.as_str())?;

        Ok(path.into())
    }

    fn get_one_drive_attributes(&self) -> Result<(String, PathBuf)> {
        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hklm.open_subkey(USER_PROFILE_ONEDRIVE_PATH)?;
        let email: String = settings.get_value(USER_PROFILE_ONEDRIVE_EMAIL_KEY)?;
        let path: String = settings.get_value(USER_PROFILE_ONEDRIVE_FOLDER_KEY)?;

        Ok((email, PathBuf::from(path)))
    }
}
