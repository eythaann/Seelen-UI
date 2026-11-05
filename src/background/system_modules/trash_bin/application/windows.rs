use seelen_core::system_state::TrashBinInfo;
use windows::Win32::UI::Shell::{
    SHERB_NOSOUND, SHEmptyRecycleBinW, SHQUERYRBINFO, SHQueryRecycleBinW,
};
use windows_core::PCWSTR;

use crate::{
    error::Result,
    system_modules::trash_bin::domain::{TrashBinEvent, TrashBinSystem},
    windows_api::event_window::{WM_TRASH_BIN_NOTIFY, subscribe_to_background_window},
};

/// Stateless: the shell notifications are registered by the background window, whose
/// subscriptions live for the whole app lifetime (nothing to release on `Drop`).
pub struct WindowsTrashBin;

impl TrashBinSystem for WindowsTrashBin {
    fn create() -> Self {
        Self
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(TrashBinEvent) + Send + Sync + 'static,
    {
        subscribe_to_background_window(move |msg, _w_param, _l_param| {
            if msg == WM_TRASH_BIN_NOTIFY {
                on_event(TrashBinEvent::InfoChanged {
                    info: query_recycle_bin()?,
                });
            }
            Ok(())
        });
        Ok(())
    }

    fn query(&self) -> Result<TrashBinInfo> {
        query_recycle_bin()
    }

    fn empty(&self) -> Result<()> {
        unsafe { SHEmptyRecycleBinW(None, PCWSTR::null(), SHERB_NOSOUND)? };
        Ok(())
    }
}

/// Queries the recycle bin of all drives.
fn query_recycle_bin() -> Result<TrashBinInfo> {
    let mut rb_info = SHQUERYRBINFO {
        cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
        ..Default::default()
    };
    unsafe { SHQueryRecycleBinW(PCWSTR::null(), &mut rb_info)? };
    Ok(TrashBinInfo {
        item_count: rb_info.i64NumItems,
        size_in_bytes: rb_info.i64Size,
    })
}
