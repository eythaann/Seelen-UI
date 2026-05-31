use std::sync::LazyLock;

use seelen_core::system_state::{ImeState, KeyboardLayout, SystemLanguage};
use windows::{
    core::GUID,
    Win32::{
        Globalization::{
            GetLocaleInfoEx, LCIDToLocaleName, LOCALE_SLOCALIZEDDISPLAYNAME,
            LOCALE_SNATIVELANGUAGENAME,
        },
        UI::{
            Input::{
                Ime::{
                    ImmGetDefaultIMEWnd, IME_CMODE_FULLSHAPE, IME_CMODE_HANJACONVERT,
                    IME_CMODE_KATAKANA, IME_CMODE_NATIVE, IME_CMODE_ROMAN, IME_CONVERSION_MODE,
                    IME_SENTENCE_MODE,
                },
                KeyboardAndMouse::{GetKeyboardLayout, HKL},
            },
            TextServices::{
                CLSID_TF_InputProcessorProfiles, ITfInputProcessorProfileMgr,
                ITfInputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD, TF_IPPMF_FORPROCESS,
                TF_IPPMF_FORSESSION, TF_IPP_FLAG_ACTIVE, TF_IPP_FLAG_ENABLED,
                TF_PROFILETYPE_INPUTPROCESSOR, TF_PROFILETYPE_KEYBOARDLAYOUT,
            },
            WindowsAndMessaging::WM_IME_CONTROL,
        },
    },
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::{lock_free::SyncVec, spawn_named_thread},
    windows_api::{
        event_window::IS_INTERACTIVE_SESSION, string_utils::WindowsString, Com, WindowsApi,
    },
};

#[allow(clippy::upper_case_acronyms)]
type LANGID = u16;
pub const IMC_GETCONVERSIONMODE: u32 = 0x0001;
// pub const IMC_SETCONVERSIONMODE: u32 = 0x0002;
pub const IMC_GETSENTENCEMODE: u32 = 0x0003;
// pub const IMC_SETSENTENCEMODE: u32 = 0x0004;
pub const IMC_GETOPENSTATUS: u32 = 0x0005;
// pub const IMC_SETOPENSTATUS: u32 = 0x0006;

static LANGUAGE_MANAGER: LazyLock<LanguageManager> = LazyLock::new(|| {
    let mut manager = LanguageManager::new();
    manager.init().log_error();
    manager
});

event_manager!(LanguageManager, LanguageEvent);

#[derive(Debug, Clone)]
pub enum LanguageEvent {
    LayoutChanged,
    ImeChanged,
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

        let eid = Self::subscribe(|_event| {
            if let Ok(langs) = Self::enum_langs() {
                Self::instance().languages.replace(langs);
            }
        });
        Self::set_event_handler_priority(&eid, 1);

        spawn_named_thread("Input Profile Monitor", || {
            let mut hkl = Self::get_active_hkl();
            let mut active_tip = Self::get_active_tip().unwrap_or_default();
            let mut ime = Self::get_ime_state().unwrap_or_default();

            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                if !IS_INTERACTIVE_SESSION.load(std::sync::atomic::Ordering::Acquire) {
                    continue;
                }

                let current_tip = Self::get_active_tip().unwrap_or_default();
                let current_hkl = Self::get_active_hkl();
                if hkl != current_hkl || active_tip != current_tip {
                    log::debug!("Language changed: {hkl:?} -> {current_hkl:?} | {active_tip} -> {current_tip}");
                    active_tip = current_tip;
                    hkl = current_hkl;
                    Self::send(LanguageEvent::LayoutChanged);
                }

                let current_ime = Self::get_ime_state().unwrap_or_default();
                if ime != current_ime {
                    log::debug!("IME status changed: {ime:?} -> {current_ime:?}");
                    ime = current_ime;
                    Self::send(LanguageEvent::ImeChanged);
                }
            }
        });

        Ok(())
    }

    // ── TSF helpers ──────────────────────────────────────────────────────────

    fn tsf_profiles() -> Result<ITfInputProcessorProfiles> {
        Com::create_instance(&CLSID_TF_InputProcessorProfiles)
    }

    fn tsf_profile_mgr() -> Result<ITfInputProcessorProfileMgr> {
        Com::create_instance(&CLSID_TF_InputProcessorProfiles)
    }

    // active keyboard layout is set per thread, so we show the active layout for the foregrounded one.
    fn get_active_hkl() -> HKL {
        let (_, focused_thread) =
            WindowsApi::window_thread_process_id(WindowsApi::get_foreground_window());
        unsafe { GetKeyboardLayout(focused_thread) }
    }

    fn _get_active_tip() -> Result<String> {
        Com::run_with_context(|| {
            let manager = Self::tsf_profile_mgr()?;

            let mut profile = Default::default();
            unsafe { manager.GetActiveProfile(&GUID_TFCAT_TIP_KEYBOARD, &mut profile)? };

            let tip = if profile.dwProfileType == TF_PROFILETYPE_KEYBOARDLAYOUT {
                let klid = Self::hkl_to_klid(profile.hkl.0 as _)?;
                format!("{:04X}:{klid}", profile.langid)
            } else {
                format!(
                    "{:04X}:{{{:?}}}{{{:?}}}",
                    profile.langid, profile.clsid, profile.guidProfile
                )
            };
            Ok(tip)
        })
    }

    fn get_active_tip() -> Result<String> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        std::thread::spawn(move || {
            let tip = Self::_get_active_tip().unwrap_or_default();
            let _ = tx.send(tip);
        })
        .join()
        .ok();
        Ok(rx.recv()?)
    }

    // ── Language enumeration ─────────────────────────────────────────────────

    fn enum_langs() -> Result<Vec<SystemLanguage>> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        std::thread::spawn(move || {
            let _ = tx.send(Self::_enum_langs());
        })
        .join()
        .ok();

        rx.recv()?
    }

    pub fn get_ime_state() -> Result<ImeState> {
        unsafe {
            let ime_hwnd = ImmGetDefaultIMEWnd(WindowsApi::get_foreground_window());
            if ime_hwnd.is_invalid() {
                return Err("failed to get default IME window".into());
            }

            let open =
                WindowsApi::send_message(ime_hwnd, WM_IME_CONTROL, IMC_GETOPENSTATUS as _, 0)?;
            let conversion =
                WindowsApi::send_message(ime_hwnd, WM_IME_CONTROL, IMC_GETCONVERSIONMODE as _, 0)?;
            let sentence =
                WindowsApi::send_message(ime_hwnd, WM_IME_CONTROL, IMC_GETSENTENCEMODE as _, 0)?;

            let open = open != 0;
            let conversion = IME_CONVERSION_MODE(conversion as _);
            let _sentence = IME_SENTENCE_MODE(sentence as _);

            let ime_status = ImeState {
                native: conversion.contains(IME_CMODE_NATIVE),
                full_shape: conversion.contains(IME_CMODE_FULLSHAPE),
                katakana: conversion.contains(IME_CMODE_KATAKANA),
                roman: conversion.contains(IME_CMODE_ROMAN),
                hanja: conversion.contains(IME_CMODE_HANJACONVERT),
                open,
            };
            Ok(ime_status)
        }
    }

    /// Enumerates installed input profiles via TSF's `ITfInputProcessorProfileMgr::EnumProfiles`.
    ///
    /// The user language list still comes from the registry — it is the authoritative
    /// source written by Windows language settings. Per-language profile details
    /// (display names, types, active state) are resolved through TSF directly.
    fn _enum_langs() -> Result<Vec<SystemLanguage>> {
        Com::run_with_context(|| unsafe {
            let mut languages = Vec::new();

            let profiles = Self::tsf_profiles()?;
            let manager = Self::tsf_profile_mgr()?;

            let mut lang_ids = std::ptr::null_mut::<LANGID>();
            let mut count = 0u32;
            profiles.GetLanguageList(&mut lang_ids, &mut count)?;

            let slice = std::slice::from_raw_parts(lang_ids, count as usize);

            for lang_id in slice {
                let mut language = Self::get_language(*lang_id as u32)?;
                let list = manager.EnumProfiles(*lang_id)?;

                loop {
                    let mut items = [Default::default(); 1];
                    let mut fetched = 0u32;

                    list.Next(&mut items, &mut fetched)?;
                    if fetched == 0 {
                        break;
                    }

                    let item = &items[0];
                    if item.dwFlags & TF_IPP_FLAG_ENABLED == 0 {
                        continue;
                    }

                    let is_active_profile = item.dwFlags & TF_IPP_FLAG_ACTIVE != 0;

                    let input_method = if item.dwProfileType == TF_PROFILETYPE_KEYBOARDLAYOUT {
                        let klid = Self::hkl_to_klid(item.hkl.0 as _)?;
                        let display_name = Self::get_klid_display_name(&klid)
                            .unwrap_or_else(|_| format!("Keyboard ({klid})"));

                        KeyboardLayout {
                            id: format!("{:08X}", item.hkl.0 as usize),
                            handle: format!("{lang_id:04X}:{klid}"),
                            display_name,
                            active: is_active_profile,
                        }
                    } else if item.catid == GUID_TFCAT_TIP_KEYBOARD {
                        let tip = format!(
                            "{:04X}:{{{:?}}}{{{:?}}}",
                            item.langid, item.clsid, item.guidProfile
                        );

                        let display_name = profiles
                            .GetLanguageProfileDescription(
                                &item.clsid,
                                *lang_id,
                                &item.guidProfile,
                            )?
                            .to_string();

                        KeyboardLayout {
                            id: tip.clone(),
                            handle: tip,
                            display_name,
                            active: is_active_profile,
                        }
                    } else {
                        continue;
                    };

                    language.keyboard_layouts.push(input_method);
                }

                languages.push(language);
            }

            Com::task_mem_free(lang_ids as _);
            // log::debug!("{:#?}", languages);
            Ok(languages)
        })
    }

    fn get_klid_display_name(klid: &str) -> Result<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let reg_layouts = hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layouts")?;
        let reg_layout = reg_layouts.open_subkey(klid)?;
        let display_name =
            if let Ok(path) = reg_layout.get_value::<String, _>("Layout Display Name") {
                WindowsApi::resolve_indirect_string(&path)?
            } else {
                reg_layout
                    .get_value("Layout Text")
                    .unwrap_or_else(|_| String::from("Unknown"))
            };
        Ok(display_name)
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

    // ── Layout activation ────────────────────────────────────────────────────

    pub fn set_keyboard_layout(id: &str, handle: &str) -> Result<()> {
        Com::run_with_context(|| unsafe {
            let colon = handle.find(':').ok_or("invalid TSF tip: missing colon")?;
            let lang_id = u16::from_str_radix(&handle[..colon], 16)?;
            let rest = &handle[colon + 1..];

            let (profile_type, hkl, clsid, profile_guid) = if handle.contains('{') {
                if rest.len() < 76 {
                    return Err(format!("TSF tip too short: {handle:?}").into());
                }

                let clsid = parse_guid(&rest[..38]).ok_or("invalid CLSID in TSF tip")?;
                let profile_guid =
                    parse_guid(&rest[38..76]).ok_or("invalid ProfileGUID in TSF tip")?;

                (
                    TF_PROFILETYPE_INPUTPROCESSOR,
                    Default::default(),
                    clsid,
                    profile_guid,
                )
            } else {
                let hkl = usize::from_str_radix(id, 16)?;
                (
                    TF_PROFILETYPE_KEYBOARDLAYOUT,
                    HKL(hkl as _),
                    Default::default(),
                    Default::default(),
                )
            };

            /* if profile_type == TF_PROFILETYPE_KEYBOARDLAYOUT {
                let klid = WindowsString::from_str(rest);
                let hkl = usize::from_str_radix(id, 16)?;

                LoadKeyboardLayoutW(klid.as_pcwstr(), Default::default())?;

                let foreground = WindowsApi::get_foreground_window();
                WindowsApi::post_message(foreground, WM_INPUTLANGCHANGEREQUEST, 0, hkl as _)?;

                ActivateKeyboardLayout(HKL(hkl as _), KLF_SETFORPROCESS)?;
            } else {
                Self::tsf_profiles()?.ChangeCurrentLanguage(lang_id)?;
                Self::tsf_profile_mgr()?.ActivateProfile(
                    profile_type,
                    lang_id,
                    &clsid,
                    &profile_guid,
                    hkl,
                    TF_IPPMF_FORPROCESS | TF_IPPMF_FORSESSION,
                )?;
            } */

            Self::tsf_profiles()?.ChangeCurrentLanguage(lang_id)?;
            Self::tsf_profile_mgr()?.ActivateProfile(
                profile_type,
                lang_id,
                &clsid,
                &profile_guid,
                hkl,
                TF_IPPMF_FORPROCESS | TF_IPPMF_FORSESSION,
            )?;

            Ok(())
        })?;

        Self::send(LanguageEvent::LayoutChanged);
        Ok(())
    }

    // ── Legacy HKL → KLID (active-profile detection for non-TSF layouts) ────

    fn hkl_to_klid(hkl: u32) -> Result<String> {
        let language_id = hkl & 0xFFFF;
        let mut device_id = hkl >> 16;

        if device_id & 0xF000 == 0xF000 {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let reg_layouts =
                hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Keyboard Layouts")?;
            let layout_id_to_search = format!("{:04X}", device_id & 0x0FFF);

            for klid in reg_layouts.enum_keys().flatten() {
                let entry = reg_layouts.open_subkey(&klid)?;
                if let Ok(id) = entry.get_value::<String, _>("Layout Id") {
                    if layout_id_to_search == id.to_uppercase() {
                        return Ok(klid);
                    }
                }
            }
            Err(format!("KLID not found for HKL {hkl:08X}").into())
        } else {
            if device_id == 0 {
                device_id = language_id;
            }
            Ok(format!("{device_id:08X}"))
        }
    }

    // ── Public API ───────────────────────────────────────────────────────────

    pub fn get_languages(&self) -> Vec<SystemLanguage> {
        self.languages.to_vec()
    }
}

// ── GUID utilities ───────────────────────────────────────────────────────────

fn parse_guid(s: &str) -> Option<GUID> {
    let s = s.trim_matches(|c: char| c == '{' || c == '}');
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return None;
    }
    let data1 = u32::from_str_radix(parts[0], 16).ok()?;
    let data2 = u16::from_str_radix(parts[1], 16).ok()?;
    let data3 = u16::from_str_radix(parts[2], 16).ok()?;
    if parts[3].len() != 4 || parts[4].len() != 12 {
        return None;
    }
    let tail = format!("{}{}", parts[3], parts[4]);
    let mut data4 = [0u8; 8];
    for (i, byte) in data4.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&tail[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(GUID {
        data1,
        data2,
        data3,
        data4,
    })
}
