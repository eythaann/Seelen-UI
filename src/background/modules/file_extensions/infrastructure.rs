use winreg::{enums::HKEY_CLASSES_ROOT, RegKey};

use crate::error_handler::Result;

pub struct Theme;
impl Theme {
    pub fn create_uri_protocol() -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy().to_string();

        let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

        let (key, _) = hkcr.create_subkey("Seelen.UI.URI")?;

        key.set_value("", &"URL:Seelen Theme protocol")?;
        key.set_value("URL Protocol", &"")?;

        let (icon_key, _) = key.create_subkey("DefaultIcon")?;
        icon_key.set_value("", &format!("\"{}\",1", exe_path_str))?;

        let (command_key, _) = key.create_subkey("shell\\open\\command")?;
        command_key.set_value("", &format!("\"{exe_path_str}\" load uri \"%1\""))?;

        Ok(())
    }

    pub fn create_ext_protocol() -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy().to_string();

        let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

        // register the extension
        let (ext_key, _) = hkcr.create_subkey(".slu")?;
        ext_key.set_value("", &"Seelen.UI")?;

        // register the app
        let (app_key, _) = hkcr.create_subkey("Seelen.UI")?;

        let (default_icon_key, _) = app_key.create_subkey("DefaultIcon")?;
        default_icon_key.set_value("", &format!("\"{}\",1", exe_path_str))?;

        let command = format!("\"{}\" load file \"%1\"", exe_path_str);

        let (command_key, _) = app_key.create_subkey("shell\\open\\command")?;
        command_key.set_value("", &command)?;

        let (command_key, _) = app_key.create_subkey("shell\\edit\\command")?;
        command_key.set_value("", &command)?;

        Ok(())
    }
}
