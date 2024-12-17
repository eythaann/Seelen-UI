use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

use crate::{error_handler::Result, event_manager, windows_api::WindowsApi};

use super::domain::{KeyboardLayout, Language};

pub struct LanguageManager {}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LanguageEvent {
    LanguageChanged,
    KeyboardLayoutChanged,
}

event_manager!(LanguageManager, LanguageEvent);

impl LanguageManager {
    pub fn enum_langs() -> Result<Vec<Language>> {
        let hklm = RegKey::predef(HKEY_CURRENT_USER);
        let reg_profile = hklm.open_subkey(r"Control Panel\International\User Profile")?;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let reg_layouts = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layouts")?;

        let mut languages = Vec::new();

        for lang_code in reg_profile.enum_keys().flatten() {
            let reg_lang = reg_profile.open_subkey(&lang_code)?;

            let name = if let Ok(path) = reg_lang.get_value::<String, _>("CachedLanguageName") {
                WindowsApi::resolve_indirect_string(&path)?
            } else {
                "Unknown".to_string()
            };

            let mut input_methods = Vec::new();
            for (key, _value) in reg_lang.enum_values().flatten() {
                if !key.contains(":") {
                    continue;
                }

                let layout_id = key.split(":").last().unwrap().to_owned();
                let layout = reg_layouts.open_subkey(&layout_id)?;

                let layout_name =
                    if let Ok(path) = layout.get_value::<String, _>("Layout Display Name") {
                        WindowsApi::resolve_indirect_string(&path)?
                    } else {
                        layout
                            .get_value("Layout Text")
                            .unwrap_or_else(|_| String::from("Unknown"))
                    };

                input_methods.push(KeyboardLayout {
                    id: layout_id,
                    display_name: layout_name,
                });
            }

            languages.push(Language {
                code: lang_code,
                name,
                input_methods,
            });
        }

        Ok(languages)
    }
}
