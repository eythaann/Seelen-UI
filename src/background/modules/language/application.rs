use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::Win32::{
    Globalization::{
        GetLocaleInfoEx, LCIDToLocaleName, LOCALE_SLOCALIZEDDISPLAYNAME, LOCALE_SNATIVELANGUAGENAME,
    },
    UI::Input::KeyboardAndMouse::{
        ActivateKeyboardLayout, GetKeyboardLayout, GetKeyboardLayoutList, LoadKeyboardLayoutW,
        ACTIVATE_KEYBOARD_LAYOUT_FLAGS, HKL, KLF_ACTIVATE, KLF_SETFORPROCESS,
    },
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{
    error_handler::Result,
    event_manager, log_error,
    utils::spawn_named_thread,
    windows_api::{string_utils::WindowsString, WindowsApi},
};

use super::domain::{KeyboardLayout, Language};

lazy_static! {
    pub static ref LANGUAGE_MANAGER: Arc<Mutex<LanguageManager>> =
        Arc::new(Mutex::new(LanguageManager::default()));
}

static LAST_LOADED_HKL: AtomicUsize = AtomicUsize::new(0);

event_manager!(LanguageManager, LanguageEvent);

#[derive(Debug, Clone)]
pub enum LanguageEvent {
    KeyboardLayoutChanged(usize),
}

#[derive(Debug, Default)]
pub struct LanguageManager {
    pub languages: Vec<Language>,
}

impl LanguageManager {
    pub fn init(&mut self) -> Result<()> {
        self.languages = Self::enum_langs()?;

        spawn_named_thread("Keyboard Layout Monitor", || {
            let hkl = unsafe { GetKeyboardLayout(0) };
            LAST_LOADED_HKL.store(hkl.0 as _, Ordering::Relaxed);
            loop {
                let (_, focused_thread) =
                    WindowsApi::window_thread_process_id(WindowsApi::get_foreground_window());
                let current = unsafe { GetKeyboardLayout(focused_thread) }.0 as usize;
                if current != LAST_LOADED_HKL.load(Ordering::Acquire) {
                    LAST_LOADED_HKL.store(current, Ordering::Relaxed);
                    log::info!("Keyboard layout changed to {:08X?}", current);
                    log_error!(Self::event_tx().send(LanguageEvent::KeyboardLayoutChanged(current)));
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        })?;
        Ok(())
    }

    fn get_hkl_list() -> Vec<HKL> {
        unsafe {
            let len = GetKeyboardLayoutList(None) as usize;
            let mut list = vec![HKL::default(); len];
            GetKeyboardLayoutList(Some(&mut list));
            list
        }
    }

    pub fn enum_langs() -> Result<Vec<Language>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let reg_layouts = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layouts")?;
        let available_klids = reg_layouts.enum_keys().flatten().collect_vec();

        let active_hkl = unsafe { GetKeyboardLayout(0) };
        let mut languages: Vec<Language> = Vec::new();

        for hkl in Self::get_hkl_list() {
            // https://learn.microsoft.com/en-us/windows/win32/intl/language-identifiers
            let language_id = hkl.0 as u32 & 0xFFFF; // low word

            let mut device = hkl.0 as u32 >> 16; // high word
            let mut klid = None;

            // `Device Handle` contains `Layout ID`
            if device & 0xF000 == 0xF000 {
                let layout_id_to_search = format!("{:04X}", device & 0x0FFF);
                for current_klid in &available_klids {
                    let reg_layout = reg_layouts.open_subkey(current_klid)?;
                    if let Ok(layout_id) = reg_layout.get_value::<String, _>("Layout Id") {
                        // Layout Id stored using case insensitive
                        if layout_id_to_search == layout_id.to_uppercase() {
                            klid = Some(current_klid.clone());
                            break;
                        }
                    }
                }
                if klid.is_none() {
                    return Err(format!("klid not found for {hkl:?}").into());
                }
            } else {
                // Use language id only if keyboard layout language is not available. This
                // is crucial in cases when keyboard is installed more than once or under
                // different languages. For example when French keyboard is installed under US
                // input language we need to return French keyboard identifier.
                if device == 0 {
                    device = language_id;
                }
                klid = Some(format!("{:08X}", device));
            }

            let klid = match klid {
                Some(klid) => klid,
                None => continue,
            };

            let reg_layout = reg_layouts.open_subkey(&klid)?;
            let layout_display_name =
                if let Ok(path) = reg_layout.get_value::<String, _>("Layout Display Name") {
                    WindowsApi::resolve_indirect_string(&path)?
                } else {
                    reg_layout
                        .get_value("Layout Text")
                        .unwrap_or_else(|_| String::from("Unknown"))
                };

            let mut lang_code = WindowsString::new_to_fill(256);
            unsafe { LCIDToLocaleName(language_id, Some(lang_code.as_mut_slice()), 0) };
            let lang_code_str = lang_code.to_string();

            let language = match languages.iter_mut().find(|l| l.code == lang_code_str) {
                Some(language) => language,
                None => unsafe {
                    let mut display_name = WindowsString::new_to_fill(256);
                    let mut native_name = WindowsString::new_to_fill(256);
                    GetLocaleInfoEx(
                        lang_code.as_pcwstr(),
                        LOCALE_SLOCALIZEDDISPLAYNAME,
                        Some(display_name.as_mut_slice()),
                    );
                    GetLocaleInfoEx(
                        lang_code.as_pcwstr(),
                        LOCALE_SNATIVELANGUAGENAME,
                        Some(native_name.as_mut_slice()),
                    );
                    languages.push(Language {
                        id: format!("{:04X}", language_id),
                        code: lang_code_str.clone(),
                        name: display_name.to_string(),
                        native_name: native_name.to_string(),
                        input_methods: Vec::new(),
                    });
                    languages.last_mut().unwrap()
                },
            };

            language.input_methods.push(KeyboardLayout {
                id: klid,
                handle: format!("{:08X}", hkl.0 as usize),
                display_name: layout_display_name,
                active: hkl == active_hkl,
            });
        }

        Ok(languages)
    }

    pub fn set_keyboard_layout(klid: &str, hkl: &str) -> Result<()> {
        unsafe {
            LoadKeyboardLayoutW(
                WindowsString::from(klid).as_pcwstr(),
                ACTIVATE_KEYBOARD_LAYOUT_FLAGS(KLF_ACTIVATE.0),
            )?;
            let hkl = usize::from_str_radix(hkl, 16)?;
            ActivateKeyboardLayout(HKL(hkl as _), KLF_SETFORPROCESS)?;
            LAST_LOADED_HKL.store(hkl, Ordering::Relaxed);
            Self::event_tx().send(LanguageEvent::KeyboardLayoutChanged(hkl))?;
        }
        Ok(())
    }

    /// Returns true if keyboard layout was changed\
    /// Returns false if keyboard layout wasn't found
    pub fn update_active(&mut self, hkl: usize) -> bool {
        let hkl = format!("{:08X}", hkl);
        let mut found = false;
        for lang in self.languages.iter_mut() {
            for keyboard in lang.input_methods.iter_mut() {
                keyboard.active = keyboard.handle == hkl;
                if keyboard.active {
                    found = true;
                }
            }
        }
        found
    }
}
