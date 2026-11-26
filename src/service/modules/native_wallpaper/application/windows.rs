use slu_utils::windows::{Com, WindowsString};
use windows::{
    Win32::{
        Foundation::COLORREF,
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree},
        UI::Shell::{
            Common::ITEMIDLIST, DESKTOP_SLIDESHOW_OPTIONS, DESKTOP_WALLPAPER_POSITION,
            DSO_SHUFFLEIMAGES, DSS_SLIDESHOW, DWPOS_FILL, DesktopWallpaper, IDesktopWallpaper,
            ILFree, SHCreateShellItemArrayFromIDLists, SHParseDisplayName, SIGDN_FILESYSPATH,
        },
    },
    core::{PCWSTR, PWSTR},
};

use crate::{
    error::Result,
    modules::native_wallpaper::domain::{NativeWallpaperSystem, OriginalWallpaper, Slideshow},
};

/// `IDesktopWallpaper` lives in explorer, so a new instance is created on each call: a cached one
/// would be disconnected by an explorer restart.
pub struct WindowsNativeWallpaper;

impl WindowsNativeWallpaper {
    fn api(&self) -> Result<IDesktopWallpaper> {
        Ok(Com::create_instance(&DesktopWallpaper)?)
    }

    /// `None` if no slideshow is running.
    fn slideshow(api: &IDesktopWallpaper) -> Result<Option<Slideshow>> {
        let status = unsafe { api.GetStatus()? };
        if status.0 & DSS_SLIDESHOW.0 == 0 {
            return Ok(None);
        }

        let array = unsafe { api.GetSlideshow()? };
        let count = unsafe { array.GetCount()? };
        let mut items = Vec::new();
        for index in 0..count {
            let item = unsafe { array.GetItemAt(index)? };
            // items without a file system path (virtual folders) can't be persisted
            if let Ok(path) = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) } {
                items.push(take_pwstr(path));
            }
        }
        if items.is_empty() {
            return Ok(None);
        }

        let mut options = DESKTOP_SLIDESHOW_OPTIONS::default();
        let mut interval = 0u32;
        unsafe { api.GetSlideshowOptions(&mut options, &mut interval)? };
        Ok(Some(Slideshow {
            items,
            shuffle: options.0 & DSO_SHUFFLEIMAGES.0 != 0,
            interval,
        }))
    }
}

impl NativeWallpaperSystem for WindowsNativeWallpaper {
    fn create() -> Self {
        Self
    }

    fn init(&mut self) -> Result<()> {
        // S_FALSE (already initialized) is fine, it is our own thread.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok()?;
        Ok(())
    }

    fn wallpapers(&self) -> Result<Vec<(String, String)>> {
        let api = self.api()?;
        let count = unsafe { api.GetMonitorDevicePathCount() }.unwrap_or_default();
        Ok((0..count)
            .filter_map(|index| {
                let monitor = take_pwstr(unsafe { api.GetMonitorDevicePathAt(index) }.ok()?);
                let monitor_w = WindowsString::from_str(&monitor);
                // fails for monitors that are known but not attached
                let wallpaper =
                    take_pwstr(unsafe { api.GetWallpaper(monitor_w.as_pcwstr()) }.ok()?);
                Some((monitor, wallpaper))
            })
            .collect())
    }

    fn snapshot(&self, by_monitor: Vec<(String, String)>) -> Result<OriginalWallpaper> {
        let api = self.api()?;
        Ok(OriginalWallpaper {
            by_monitor,
            position: unsafe { api.GetPosition()? }.0,
            background_color: unsafe { api.GetBackgroundColor()? }.0,
            slideshow: Self::slideshow(&api)
                .inspect_err(|err| log::error!("Failed to read the slideshow: {err:?}"))
                .ok()
                .flatten(),
        })
    }

    fn set_filled_wallpaper(&self, path: &str) -> Result<()> {
        let api = self.api()?;
        let path = WindowsString::from_str(path);
        unsafe {
            api.SetPosition(DWPOS_FILL)?;
            api.SetWallpaper(PCWSTR::null(), path.as_pcwstr())?;
        }
        Ok(())
    }

    fn set_style(&self, original: &OriginalWallpaper) -> Result<()> {
        let api = self.api()?;
        unsafe {
            api.SetPosition(DESKTOP_WALLPAPER_POSITION(original.position))?;
            api.SetBackgroundColor(COLORREF(original.background_color))?;
        }
        Ok(())
    }

    fn set_slideshow(&self, slideshow: &Slideshow) -> Result<()> {
        let api = self.api()?;
        let pidls = slideshow
            .items
            .iter()
            .filter_map(|path| {
                let path = WindowsString::from_str(path);
                let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
                unsafe { SHParseDisplayName(path.as_pcwstr(), None, &mut pidl, 0, None) }
                    .inspect_err(|err| log::warn!("Slideshow item not found: {err:?}"))
                    .ok()
                    .filter(|_| !pidl.is_null())
                    .map(|_| pidl)
            })
            .collect::<Vec<_>>();

        let result = (|| -> Result<()> {
            if pidls.is_empty() {
                return Err("None of the slideshow items exist anymore".into());
            }
            let const_pidls: Vec<*const ITEMIDLIST> = pidls.iter().map(|p| *p as _).collect();
            let array = unsafe { SHCreateShellItemArrayFromIDLists(&const_pidls)? };
            unsafe { api.SetSlideshow(&array)? };
            let options = if slideshow.shuffle {
                DSO_SHUFFLEIMAGES
            } else {
                DESKTOP_SLIDESHOW_OPTIONS(0)
            };
            unsafe { api.SetSlideshowOptions(options, slideshow.interval)? };
            Ok(())
        })();

        for pidl in pidls {
            unsafe { ILFree(Some(pidl as _)) };
        }
        result
    }

    fn set_wallpaper(&self, monitor: Option<&str>, wallpaper: &str) -> Result<()> {
        let api = self.api()?;
        let monitor = monitor.map(WindowsString::from_str);
        let monitor = monitor
            .as_ref()
            .map_or(PCWSTR::null(), |monitor| monitor.as_pcwstr());
        let wallpaper = WindowsString::from_str(wallpaper);
        unsafe { api.SetWallpaper(monitor, wallpaper.as_pcwstr())? };
        Ok(())
    }
}

/// Reads and frees a COM allocated string.
fn take_pwstr(value: PWSTR) -> String {
    if value.is_null() {
        return String::new();
    }
    let result = unsafe { value.to_string() }.unwrap_or_default();
    unsafe { CoTaskMemFree(Some(value.0 as _)) };
    result
}
