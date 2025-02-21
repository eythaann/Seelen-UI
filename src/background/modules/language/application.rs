use itertools::Itertools;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardLayoutNameW, LoadKeyboardLayoutW, ACTIVATE_KEYBOARD_LAYOUT_FLAGS,
    KLF_REPLACELANG, KLF_SUBSTITUTE_OK,
};
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

use crate::{
    error_handler::Result,
    event_manager,
    windows_api::{string_utils::WindowsString, WindowsApi},
};

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

        let (active_lang_id, active_layout_id) = Self::get_active_keyboard_layout()?;

        println!(
            "lang_id: {}, layout_name: {}",
            active_lang_id, active_layout_id
        );

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

                let parts = key.split(":").collect_vec();
                let (lang_id, layout_id) = match parts.len() {
                    2 => (parts[0].to_owned(), parts[1].to_owned()),
                    _ => continue,
                };

                let active = lang_id == active_lang_id && layout_id == active_layout_id;

                let reg_layout = reg_layouts.open_subkey(&layout_id)?;
                let display_name =
                    if let Ok(path) = reg_layout.get_value::<String, _>("Layout Display Name") {
                        WindowsApi::resolve_indirect_string(&path)?
                    } else {
                        reg_layout
                            .get_value("Layout Text")
                            .unwrap_or_else(|_| String::from("Unknown"))
                    };

                input_methods.push(KeyboardLayout {
                    id: layout_id,
                    display_name,
                    active,
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

    pub fn get_active_keyboard_layout() -> Result<(String, String)> {
        unsafe {
            let hkl = GetKeyboardLayout(0).0 as usize;
            println!("hkl: {:x}", hkl);

            let lang_id = format!("{:04x}", hkl & 0xFFFF);

            let mut klid = [0; 9];
            GetKeyboardLayoutNameW(&mut klid)?;
            let klid = WindowsString::from_slice(&klid);

            Ok((lang_id, klid.to_string()))
        }
    }

    pub fn set_keyboard_layout(layout_name: &str) -> Result<()> {
        let code = WindowsString::from(layout_name);
        unsafe {
            LoadKeyboardLayoutW(
                code.as_pcwstr(),
                ACTIVATE_KEYBOARD_LAYOUT_FLAGS(KLF_REPLACELANG.0 | KLF_SUBSTITUTE_OK.0),
            )?;
        }
        Ok(())
    }
}
