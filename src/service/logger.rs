use windows::Win32::UI::Shell::FOLDERID_LocalAppData;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{error::Result, windows_api::WindowsApi, SERVICE_DISPLAY_NAME};

pub struct SluServiceLogger {}

impl SluServiceLogger {
    const REG_BASEKEY: &str = r"SYSTEM\CurrentControlSet\Services\EventLog\Application";

    // remove on v2.3 or v2.4
    pub fn uninstall_old_logging() -> Result<()> {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(Self::REG_BASEKEY)?;
        let path = SERVICE_DISPLAY_NAME.to_string();
        if key.open_subkey(&path).is_ok() {
            key.delete_subkey(&path)?;
        }
        Ok(())
    }

    pub fn init() -> Result<()> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}][{}] {}",
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(std::io::stdout())
            .chain(fern::log_file(
                WindowsApi::known_folder(FOLDERID_LocalAppData)?
                    .join("com.seelen.seelen-ui/logs/SLU Service.log"),
            )?)
            .apply()?;
        Ok(())
    }
}
