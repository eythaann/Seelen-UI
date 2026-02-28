use windows::Win32::UI::Shell::FOLDERID_LocalAppData;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{error::Result, windows_api::WindowsApi, SERVICE_DISPLAY_NAME};

pub struct SluServiceLogger {}

impl SluServiceLogger {
    const REG_BASEKEY: &str = r"SYSTEM\CurrentControlSet\Services\EventLog\Application";
    const MAX_LOG_SIZE: u64 = 1024 * 1024; // 1MB

    // remove on v2.3 or v2.4
    pub fn uninstall_old_logging() -> Result<()> {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(Self::REG_BASEKEY)?;
        let path = SERVICE_DISPLAY_NAME.to_string();
        if key.open_subkey(&path).is_ok() {
            key.delete_subkey(&path)?;
        }
        Ok(())
    }

    pub fn register_panic_hook() {
        let base_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let cause = info
                .payload()
                .downcast_ref::<String>()
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    info.payload()
                        .downcast_ref::<&str>()
                        .unwrap_or(&"<cause unknown>")
                        .to_string()
                });

            let mut string_location = String::from("<location unknown>");
            if let Some(location) = info.location() {
                string_location = format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                );
            }

            log::error!("A panic occurred:\n  Cause: {cause}\n  Location: {string_location}");
            base_hook(info);
        }));
    }

    pub fn init() -> Result<()> {
        let logs_folder =
            WindowsApi::known_folder(FOLDERID_LocalAppData)?.join("com.seelen.seelen-ui/logs");
        std::fs::create_dir_all(&logs_folder)?;

        let log_path = logs_folder.join("SLU Service.log");
        if log_path.exists() {
            let metadata = std::fs::metadata(&log_path)?;
            if metadata.len() > Self::MAX_LOG_SIZE {
                std::fs::remove_file(&log_path)?;
            }
        }

        let format =
            time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")?;
        let local_now = time::OffsetDateTime::now_local()?;
        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    local_now.format(&format).expect("Failed to format time"),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(std::io::stdout())
            .chain(fern::log_file(log_path)?)
            .apply()?;
        Self::register_panic_hook();
        Ok(())
    }
}
