use getset::Getters;
use itertools::Itertools;
use lazy_static::lazy_static;
use notify_debouncer_full::{
    new_debouncer,
    notify::{EventKind, ReadDirectoryChangesWatcher, RecursiveMode, Watcher},
    DebounceEventResult, Debouncer, FileIdMap,
};
use parking_lot::Mutex;
use std::{os::windows::fs::MetadataExt, path::PathBuf, sync::Arc, time::Duration};
use tauri::async_runtime::block_on;
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

use crate::{
    error_handler::AppError,
    event_manager, log_error, trace_lock,
    utils::{pwsh::PwshScript, spawn_named_thread},
    windows_api::WindowsApi,
};

use super::domain::{PictureQuality, RecentFile, User};

const USER_PROFILE_REG_PATH_PATTERN: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AccountPicture\\Users\\";
const USER_PROFILE_ONEDRIVE_PATH: &str = "SOFTWARE\\Microsoft\\OneDrive\\Accounts\\Personal";
const USER_PROFILE_ONEDRIVE_EMAIL_KEY: &str = "UserEmail";
const USER_PROFILE_ONEDRIVE_FOLDER_KEY: &str = "UserFolder";
const USER_SID_EXTRACTION_SCRIPT: &str =  "(New-Object -ComObject Microsoft.DiskQuota).TranslateLogonNameToSID((Get-WmiObject -Class Win32_ComputerSystem).Username)";

lazy_static! {
    pub static ref USER_MANAGER: Arc<Mutex<UserManager>> = Arc::new(Mutex::new(
        UserManager::new().expect("Failed to create user manager")
    ));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserManagerEvent {
    UserUpdated(),
    RecentFolderChanged(),
}

#[derive(Getters)]
pub struct UserManager {
    user_sid: String,
    #[getset(get = "pub")]
    recent_folder_path: PathBuf,
    #[getset(get = "pub")]
    user_details: Option<User>,
    #[getset(get = "pub")]
    recent_folder: Option<Vec<RecentFile>>,
    recent_folder_wathcer: Option<Arc<Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>>>,
    #[getset(get = "pub")]
    recent_folder_limit: usize,
}

unsafe impl Send for UserManager {}
unsafe impl Send for UserManagerEvent {}

event_manager!(UserManager, UserManagerEvent);

// Static
impl UserManager {
    pub fn new() -> Result<Self, AppError> {
        let mut instance = Self {
            user_sid: block_on(Self::get_user_sid()).ok().unwrap(),
            recent_folder_path: Self::get_recent_folder_path().ok().unwrap(),
            user_details: None,
            recent_folder: None,
            recent_folder_wathcer: None,
            recent_folder_limit: 20,
        };
        instance.user_details = instance.create_user().ok();
        instance.recent_folder = instance.get_recent_files().ok();
        instance.recent_folder_wathcer = instance.create_file_wathcer().ok();

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

    async fn get_user_sid() -> Result<String, AppError> {
        let sid = PwshScript::new(USER_SID_EXTRACTION_SCRIPT)
            .execute()
            .await?;

        Ok(sid)
    }
    fn get_recent_folder_path() -> Result<PathBuf, AppError> {
        let path = std::env::temp_dir().join("..\\..\\Roaming\\Microsoft\\Windows\\Recent");

        Ok(path)
    }
}

// Instance
impl UserManager {
    pub fn create_user(&self) -> Result<User, AppError> {
        let mut user = User {
            name: std::env!("USERNAME").to_string(),
            domain: std::env!("USERDOMAIN").to_string(),
            profile_home_path: PathBuf::from(std::env!("USERPROFILE")),
            email: None,
            one_drive_path: None,
            profile_picture_path: None,
        };
        user.profile_picture_path = self
            .get_user_profile_picture_path(PictureQuality::Quality1080)
            .ok();
        if let Ok((user_mail, one_drive_path)) = self.get_one_drive_attributes() {
            user.email = Some(user_mail);
            user.one_drive_path = Some(one_drive_path);
        }

        Ok(user)
    }

    pub fn set_recent_folder_limit(&mut self, limit: usize) -> Result<(), AppError> {
        self.recent_folder_limit = limit;

        let sender = Self::event_tx();
        log_error!(sender.send(UserManagerEvent::RecentFolderChanged()));

        Ok(())
    }

    fn create_file_wathcer(
        &self,
    ) -> Result<Arc<Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>>, AppError> {
        let mut debouncer = new_debouncer(
            Duration::from_millis(100),
            None,
            |result: DebounceEventResult| match result {
                Ok(events) => {
                    // log::info!("RecentFile watcher events: {:?}", events);
                    for event in events {
                        if let EventKind::Create(_) = event.kind {
                            for pathbuf in &event.paths {
                                let path = pathbuf.as_path();
                                if let Ok((result, _)) = WindowsApi::resolve_lnk_target(path) {
                                    if result.exists() {
                                        let file = RecentFile {
                                            path: std::fs::canonicalize(result.clone())
                                                .unwrap()
                                                .to_str()
                                                .unwrap()[4..]
                                                .into(),
                                            last_access_time: if let Ok(metadata) =
                                                pathbuf.metadata()
                                            {
                                                metadata.last_write_time()
                                            } else {
                                                0
                                            },
                                        };

                                        if let Some(ref mut folder) =
                                            trace_lock!(USER_MANAGER).recent_folder
                                        {
                                            if folder[0] != file {
                                                folder.insert(0, file.clone());

                                                let sender = Self::event_tx();
                                                log_error!(sender
                                                    .send(UserManagerEvent::RecentFolderChanged()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Err(errors) => errors
                    .iter()
                    .for_each(|e| log::error!("RecentFile Watcher Error: {:?}", e)),
            },
        )?;

        debouncer
            .watcher()
            .watch(&self.recent_folder_path, RecursiveMode::Recursive)?;

        Ok(Arc::new(Some(debouncer)))
    }

    fn get_user_profile_picture_path(&self, quality: PictureQuality) -> Result<PathBuf, AppError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let mut key = String::default();
        key.push_str(USER_PROFILE_REG_PATH_PATTERN);
        key.push_str(self.user_sid.as_str());
        let settings = hklm.open_subkey(key)?;
        let path: String = settings.get_value(quality.as_str())?;

        Ok(path.into())
    }

    fn get_one_drive_attributes(&self) -> Result<(String, PathBuf), AppError> {
        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hklm.open_subkey(USER_PROFILE_ONEDRIVE_PATH)?;
        let email: String = settings.get_value(USER_PROFILE_ONEDRIVE_EMAIL_KEY)?;
        let path: String = settings.get_value(USER_PROFILE_ONEDRIVE_FOLDER_KEY)?;

        Ok((email, PathBuf::from(path)))
    }

    fn get_recent_files(&self) -> Result<Vec<RecentFile>, AppError> {
        let result = std::fs::read_dir(self.recent_folder_path.clone())?
            .map(|r| r.unwrap())
            .filter(|item| {
                let pathbuf = item.path();
                let path = pathbuf.as_path();
                if let Some(extension) = path.extension() {
                    if extension == "lnk" {
                        if let Ok((result, _)) = WindowsApi::resolve_lnk_target(path) {
                            if result.exists() {
                                return !result.is_dir();
                            }
                        }
                    }
                }

                false
            })
            .map(|item| {
                let mut current_target = item.path();
                if let Ok((target, _)) = WindowsApi::resolve_lnk_target(&current_target) {
                    current_target = target;
                }

                RecentFile {
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
            .collect();

        Ok(result)
    }
}
