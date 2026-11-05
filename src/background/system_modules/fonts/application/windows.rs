use seelen_core::system_state::SeelenFont;
use windows::Win32::{
    Graphics::DirectWrite::{
        DWRITE_FACTORY_TYPE_SHARED, DWriteCreateFactory, IDWriteFactory, IDWriteFontFamily,
    },
    UI::WindowsAndMessaging::WM_FONTCHANGE,
};
use windows_core::{BOOL, HSTRING};

use crate::{
    error::Result,
    system_modules::fonts::domain::{FontsSysEvent, FontsSystem},
    windows_api::{event_window::subscribe_to_background_window, string_utils::WindowsString},
};

/// Stateless: `WM_FONTCHANGE` is broadcast to the background window, whose subscriptions live
/// for the whole app lifetime (nothing to release on `Drop`). The DirectWrite factory is
/// created per query, so no COM object has to be kept across threads.
pub struct WindowsFonts;

impl FontsSystem for WindowsFonts {
    fn create() -> Self {
        Self
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(FontsSysEvent) + Send + Sync + 'static,
    {
        subscribe_to_background_window(move |msg, _w_param, _l_param| {
            if msg == WM_FONTCHANGE {
                on_event(FontsSysEvent::Changed);
            }
            Ok(())
        });
        Ok(())
    }

    fn fonts(&self) -> Result<Vec<SeelenFont>> {
        let mut collection = None;
        let mut fonts = Vec::new();

        unsafe {
            let factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            // `true`: check for installed/removed fonts instead of reusing the cached collection
            factory.GetSystemFontCollection(&mut collection, true)?;
            let collection = collection.ok_or("Failed to get system font collection")?;

            for index in 0..collection.GetFontFamilyCount() {
                let family = collection.GetFontFamily(index)?;
                match font_from_family(&family) {
                    Ok(font) => fonts.push(font),
                    Err(err) => log::warn!("Failed to read font family at index {index}: {err}"),
                }
            }
        }
        Ok(fonts)
    }
}

/// Reads the family name, in English when available (falls back to the first locale).
fn font_from_family(family: &IDWriteFontFamily) -> Result<SeelenFont> {
    unsafe {
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
}
