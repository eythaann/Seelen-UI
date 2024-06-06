use winreg::{enums::HKEY_CLASSES_ROOT, RegKey};

use crate::error_handler::Result;

pub struct Theme;
impl Theme {
    pub fn create_uri_protocol() -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy().to_string();

        let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

        let (key, _) = hkcr.create_subkey("slu-theme")?;

        key.set_value("", &"URL:Seelen Theme protocol")?;
        key.set_value("URL Protocol", &"")?;

        let (icon_key, _) = key.create_subkey("DefaultIcon")?;
        icon_key.set_value("", &format!("\"{}\",1", exe_path_str))?;

        let (command_key, _) = key.create_subkey("shell\\open\\command")?;
        command_key.set_value("", &format!("\"{exe_path_str}\" uri \"%1\""))?;

        Ok(())
    }

    pub fn create_ext_protocol() -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy().to_string();

        let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

        let (theme_ext_key, _) = hkcr.create_subkey(".theme.slu")?;
        theme_ext_key.set_value("", &"theme.slu")?;

        let (theme_key, _) = hkcr.create_subkey("theme.slu")?;
        theme_key.set_value("", &"Theme Seelen UI File")?;

        let (default_icon_key, _) = theme_key.create_subkey("DefaultIcon")?;
        default_icon_key.set_value("", &format!("\"{}\",1", exe_path_str))?;

        let (command_key, _) = theme_key.create_subkey("shell\\open\\command")?;
        command_key.set_value("", &format!("\"{}\" \"%1\"", exe_path_str))?;
        Ok(())
    }
}
