use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{atomic::Ordering, LazyLock},
};

use itertools::Itertools;
use seelen_core::system_state::{ImeStatus, KeyboardLayout, SystemLanguage};
use windows::Win32::{
    Globalization::{
        GetLocaleInfoEx, LCIDToLocaleName, LOCALE_SLOCALIZEDDISPLAYNAME, LOCALE_SNATIVELANGUAGENAME,
    },
    UI::{
        Input::{
            Ime::{
                ImmGetContext, ImmGetConversionStatus, ImmGetDefaultIMEWnd, ImmGetDescriptionW,
                IME_CONVERSION_MODE, IME_SENTENCE_MODE,
            },
            KeyboardAndMouse::{
                ActivateKeyboardLayout, GetKeyboardLayout, GetKeyboardLayoutList,
                LoadKeyboardLayoutW, HKL, KLF_SETFORPROCESS,
            },
        },
        WindowsAndMessaging::WM_INPUTLANGCHANGEREQUEST,
    },
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::{lock_free::SyncVec, spawn_named_thread},
    windows_api::{event_window::IS_INTERACTIVE_SESSION, string_utils::WindowsString, WindowsApi},
};

static LANGUAGE_MANAGER: LazyLock<LanguageManager> = LazyLock::new(|| {
    let mut manager = LanguageManager::new();
    manager.init().log_error();
    manager
});

event_manager!(LanguageManager, LanguageEvent);

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LanguageEvent {
    KeyboardLayoutChanged(u32),
    IMEStatusChanged(ImeStatus),
}

#[derive(Debug)]
pub struct LanguageManager {
    languages: SyncVec<SystemLanguage>,
}

impl LanguageManager {
    pub fn instance() -> &'static Self {
        &LANGUAGE_MANAGER
    }

    fn new() -> Self {
        Self {
            languages: SyncVec::new(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.languages.replace(Self::enum_langs()?);

        let eid = Self::subscribe(|event| {
            match event {
                LanguageEvent::IMEStatusChanged(status) => {
                    log::info!("IME status changed: {status:?}");
                }
                LanguageEvent::KeyboardLayoutChanged(hkl) => {
                    log::info!("Keyboard layout changed: {hkl:08X?}");
                }
            }

            if let Ok(langs) = Self::enum_langs() {
                Self::instance().languages.replace(langs);
            }
        });
        Self::set_event_handler_priority(&eid, 1);

        spawn_named_thread("Keyboard Layout Monitor", || {
            let mut hkl = Self::get_active_hkl();
            // let mut ime = Self::get_active_ime();

            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));

                // Pause when session is not interactive to reduce CPU usage
                if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
                    continue;
                }

                let current = Self::get_active_hkl();
                if hkl != current {
                    hkl = current;
                    Self::send(LanguageEvent::KeyboardLayoutChanged(current.0 as _));
                }

                /* let current = Self::get_active_ime();
                if ime != current {
                    ime = current;
                    Self::send(LanguageEvent::IMEStatusChanged(current));
                } */
            }
        });
        Ok(())
    }

    fn get_input_locale_list() -> Vec<HKL> {
        unsafe {
            let len = GetKeyboardLayoutList(None) as usize;
            let mut list = vec![HKL::default(); len];
            GetKeyboardLayoutList(Some(&mut list));
            list
        }
    }

    // active keyboard layout is set per thread, so we show the active layout for the foregrounded one.
    fn get_active_hkl() -> HKL {
        let (_, focused_thread) =
            WindowsApi::window_thread_process_id(WindowsApi::get_foreground_window());
        unsafe { GetKeyboardLayout(focused_thread) }
    }

    // same as keyboard layout, IME status is set per process.
    #[allow(dead_code)]
    fn get_active_ime() -> ImeStatus {
        unsafe {
            let hime = ImmGetDefaultIMEWnd(WindowsApi::get_foreground_window());
            let himc = ImmGetContext(hime);

            let mut conversion_mode = IME_CONVERSION_MODE::default();
            let mut sentence_mode = IME_SENTENCE_MODE::default();
            let _ = ImmGetConversionStatus(
                himc,
                Some(&mut conversion_mode as _),
                Some(&mut sentence_mode as _),
            );

            ImeStatus {
                conversion_mode: conversion_mode.0,
                sentence_mode: sentence_mode.0,
            }
        }
    }

    fn enum_langs() -> Result<Vec<SystemLanguage>> {
        let mut languages: HashMap<u32, SystemLanguage> = HashMap::new();
        let active_hkl = Self::get_active_hkl();

        for hkl in Self::get_input_locale_list() {
            let lang_id = hkl.0 as u32 & 0xFFFF; // low word

            let lang = match languages.entry(lang_id) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let lang = Self::get_language(lang_id)?;
                    entry.insert(lang)
                }
            };

            let layout: KeyboardLayout = Self::get_keyboard_layout(hkl, hkl == active_hkl)?;
            lang.keyboard_layouts.push(layout);
        }

        Ok(languages.into_values().collect())
    }

    fn get_language(lang_id: u32) -> Result<SystemLanguage> {
        let mut lang_code = WindowsString::new_to_fill(256);
        let mut display_name = WindowsString::new_to_fill(256);
        let mut native_name = WindowsString::new_to_fill(256);

        unsafe {
            LCIDToLocaleName(lang_id, Some(lang_code.as_mut_slice()), 0);
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
        };

        Ok(SystemLanguage {
            id: format!("{lang_id:04X}"),
            code: lang_code.to_string(),
            name: display_name.to_string(),
            native_name: native_name.to_string(),
            keyboard_layouts: Vec::new(),
        })
    }

    fn get_keyboard_layout(hkl: HKL, active: bool) -> Result<KeyboardLayout> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let reg_layouts = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layouts")?;

        let layout_id = Self::get_reg_layout_id(hkl.0 as _)?;

        let reg_layout = reg_layouts.open_subkey(&layout_id)?;
        let display_name =
            if let Ok(path) = reg_layout.get_value::<String, _>("Layout Display Name") {
                WindowsApi::resolve_indirect_string(&path)?
            } else {
                reg_layout
                    .get_value("Layout Text")
                    .unwrap_or_else(|_| String::from("Unknown"))
            };

        Self::get_ime_family(hkl).log_error();

        Ok(KeyboardLayout {
            id: layout_id,
            handle: format!("{:08X}", hkl.0 as usize),
            display_name,
            active,
        })
    }

    fn get_ime_family(hkl: HKL) -> Result<()> {
        unsafe {
            let len = ImmGetDescriptionW(hkl, None);
            let mut description = WindowsString::new_to_fill((len + 1) as usize);
            ImmGetDescriptionW(hkl, Some(description.as_mut_slice()));

            // log::debug!("({:?}) IME description: {}", hkl, description.to_string());
        }
        Ok(())
    }

    /// this function is used to get the real layout id.
    /// Examples:
    /// - 0409080A = spanish lang english keyboard => layout id is 0409
    /// - F0020409 = english lang dvorak keyboard => layout id is 0002 but mapped to 00010409
    fn get_reg_layout_id(hkl: u32) -> Result<String> {
        // https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/windows-language-pack-default-values
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let reg_layouts = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layouts")?;
        let available_klids = reg_layouts.enum_keys().flatten().collect_vec();

        // https://learn.microsoft.com/en-us/windows/win32/intl/locale-identifiers
        // https://learn.microsoft.com/en-us/windows/win32/intl/language-identifiers
        let language_id = hkl & 0xFFFF; // low word
        let mut device_id = hkl >> 16; // high word

        // `Device Handle` contains `Layout ID`
        if device_id & 0xF000 == 0xF000 {
            let layout_id_to_search = format!("{:04X}", device_id & 0x0FFF);

            for current_klid in &available_klids {
                let reg_layout = reg_layouts.open_subkey(current_klid)?;
                if let Ok(layout_id) = reg_layout.get_value::<String, _>("Layout Id") {
                    // Layout Id stored using case insensitive
                    if layout_id_to_search == layout_id.to_uppercase() {
                        return Ok(current_klid.clone());
                    }
                }
            }
        } else {
            // Use language id only if keyboard layout language is not available. This
            // is crucial in cases when keyboard is installed more than once or under
            // different languages. For example when French keyboard is installed under US
            // input language we need to return French keyboard identifier.
            if device_id == 0 {
                device_id = language_id;
            }
            return Ok(format!("{device_id:08X}"));
        }

        Err(format!("klid not found for {hkl:?}").into())
    }

    pub fn set_keyboard_layout(klid: &str, hkl: &str) -> Result<()> {
        let klid = WindowsString::from_str(klid);
        let hkl = usize::from_str_radix(hkl, 16)?;

        unsafe { LoadKeyboardLayoutW(klid.as_pcwstr(), Default::default())? };

        let foreground = WindowsApi::get_foreground_window();
        WindowsApi::post_message(foreground, WM_INPUTLANGCHANGEREQUEST, 0, hkl as _)?;

        unsafe { ActivateKeyboardLayout(HKL(hkl as _), KLF_SETFORPROCESS)? };

        Self::send(LanguageEvent::KeyboardLayoutChanged(hkl as u32));
        Ok(())
    }

    pub fn get_languages(&self) -> Vec<SystemLanguage> {
        self.languages.to_vec()
    }
}
