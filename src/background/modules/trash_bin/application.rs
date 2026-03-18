use std::sync::LazyLock;

use seelen_core::system_state::TrashBinInfo;
use windows::Win32::UI::Shell::{
    SHEmptyRecycleBinW, SHQueryRecycleBinW, SHERB_NOSOUND, SHQUERYRBINFO,
};
use windows_core::PCWSTR;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::TracedMutex,
    windows_api::event_window::{subscribe_to_background_window, WM_TRASH_BIN_NOTIFY},
};

#[derive(Debug)]
pub struct TrashBinManager {
    pub info: TrashBinInfo,
}

#[derive(Debug, Clone)]
pub enum TrashBinManagerEvent {
    InfoChanged(TrashBinInfo),
}

event_manager!(TrashBinManager, TrashBinManagerEvent);

impl TrashBinManager {
    pub fn instance() -> &'static TracedMutex<Self> {
        static MANAGER: LazyLock<TracedMutex<TrashBinManager>> = LazyLock::new(|| {
            let mut m = TrashBinManager::new();
            m.init().log_error();
            TracedMutex::new(m)
        });
        &MANAGER
    }

    fn new() -> Self {
        Self {
            info: TrashBinInfo::default(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.info = Self::query()?;

        let eid = Self::subscribe(|event| {
            let TrashBinManagerEvent::InfoChanged(info) = event;
            Self::instance().lock().info = info;
        });
        Self::set_event_handler_priority(&eid, 1);

        subscribe_to_background_window(Self::on_bg_window_proc);
        Ok(())
    }

    pub fn query() -> Result<TrashBinInfo> {
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

    pub fn empty() -> Result<()> {
        unsafe {
            SHEmptyRecycleBinW(None, PCWSTR::null(), SHERB_NOSOUND)?;
        }
        Ok(())
    }

    fn on_bg_window_proc(msg: u32, _w_param: usize, _l_param: isize) -> Result<()> {
        if msg != WM_TRASH_BIN_NOTIFY {
            return Ok(());
        }

        let new_info = Self::query()?;
        let current = Self::instance().lock().info.clone();

        if new_info.item_count != current.item_count
            || new_info.size_in_bytes != current.size_in_bytes
        {
            Self::send(TrashBinManagerEvent::InfoChanged(new_info));
        }

        Ok(())
    }
}
