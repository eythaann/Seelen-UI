use std::sync::LazyLock;

use seelen_core::system_state::SeelenFont;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFontFamily, DWRITE_FACTORY_TYPE_SHARED,
};
use windows_core::{BOOL, HSTRING};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    windows_api::string_utils::WindowsString,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontManagerEvent {
    FontsChanged,
}

pub struct FontManager {}

unsafe impl Send for FontManager {}
unsafe impl Sync for FontManager {}

event_manager!(FontManager, FontManagerEvent);

impl FontManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<FontManager> = LazyLock::new(|| {
            let mut m = FontManager::new();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn new() -> Self {
        Self {}
    }

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    unsafe fn get_font_family_name(family: &IDWriteFontFamily) -> Result<SeelenFont> {
        let names = family.GetFamilyNames()?;

        let mut locale_index = 0u32;
        let mut exists = BOOL::from(false);
        let _ = names.FindLocaleName(&HSTRING::from("en-us"), &mut locale_index, &mut exists);
        if !exists.as_bool() {
            locale_index = 0;
        }

        let len = names.GetStringLength(locale_index)?;
        let mut buf = WindowsString::new_to_fill(len as usize + 1);
        names.GetString(locale_index, buf.as_mut_slice())?;

        Ok(SeelenFont {
            family: buf.to_string(),
        })
    }

    pub fn get_fonts(&self) -> Result<Vec<SeelenFont>> {
        let mut collection = None;
        let mut fonts = Vec::new();

        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            factory.GetSystemFontCollection(&mut collection, true)?;
            let collection = collection.ok_or("Failed to get system font collection")?;

            let family_count = collection.GetFontFamilyCount();
            for index in 0..family_count {
                let family = collection.GetFontFamily(index)?;
                match Self::get_font_family_name(&family) {
                    Ok(font) => fonts.push(font),
                    Err(err) => log::warn!("Failed to read font family at index {index}: {err}"),
                }
            }
        };

        fonts.sort_by(|a, b| a.family.cmp(&b.family));
        Ok(fonts)
    }
}
