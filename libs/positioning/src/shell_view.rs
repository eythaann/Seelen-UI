//! Undocumented shell COM interfaces used to cloak windows owned by other processes, as
//! `DwmSetWindowAttribute(DWMWA_CLOAK)` is rejected with E_ACCESSDENIED for foreign HWNDs.

use std::{
    ffi::c_void,
    sync::{Mutex, OnceLock, PoisonError},
};

use windows::{
    Win32::{
        Foundation::{HANDLE, HWND, RPC_E_DISCONNECTED, RPC_E_SERVER_DIED, RPC_E_SERVER_DIED_DNE},
        System::Com::{CLSCTX_ALL, CoCreateInstance, CoIncrementMTAUsage, IServiceProvider},
        UI::WindowsAndMessaging::{RemovePropW, SetPropW},
    },
    core::{GUID, HRESULT, IUnknown, IUnknown_Vtbl, Interface, PCWSTR, interface},
};

use crate::error::{Error, Result};

const CLSID_IMMERSIVE_SHELL: GUID = GUID::from_u128(0xC2F03A33_21F5_47FA_B4BB_156362A2F239);
const CLOAK_TYPE: u32 = 1;
const CLOAK_FLAG_HIDE: i32 = 2;
const CLOAK_FLAG_SHOW: i32 = 0;

/// Process-wide view collection, created once. It is an MTA proxy, so any thread of the implicit
/// MTA (every thread that never called CoInitialize, see [`ensure_mta`]) can use it, which spares
/// each animation a COM init plus an out-of-process CoCreateInstance + QueryService into explorer.
static VIEW_COLLECTION: Mutex<Option<SharedViewCollection>> = Mutex::new(None);

/// Cloaks a window while alive, uncloaking it on drop (including on errors/early returns).
///
/// While cloaked, the window carries the [`crate::CLOAKED_PROP`] prop so window trackers can
/// tell this transient cloak apart from app/shell cloaking (UWP suspension, virtual desktops).
/// The prop is set before cloaking and removed after uncloaking, so it covers the whole span.
pub struct CloakGuard {
    hwnd: isize,
    view: IApplicationView,
}

impl CloakGuard {
    pub fn cloak(hwnd: isize) -> Result<Self> {
        let view = get_application_view(hwnd)?;
        set_cloaked_prop(hwnd, true);
        if let Err(err) = unsafe { view.set_cloak(CLOAK_TYPE, CLOAK_FLAG_HIDE).ok() } {
            set_cloaked_prop(hwnd, false);
            return Err(err.into());
        }
        Ok(Self { hwnd, view })
    }
}

impl Drop for CloakGuard {
    fn drop(&mut self) {
        if let Err(err) = unsafe { self.view.set_cloak(CLOAK_TYPE, CLOAK_FLAG_SHOW).ok() } {
            log::error!("Failed to uncloak window: {err}");
        }
        set_cloaked_prop(self.hwnd, false);
    }
}

#[interface("372E1D3B-38D3-42E4-A15B-8AB2B178F513")]
unsafe trait IApplicationView: IUnknown {
    // IInspectable
    pub unsafe fn get_iids(&self, count: *mut u32, iids: *mut *mut GUID) -> HRESULT;
    pub unsafe fn get_runtime_class_name(&self, class_name: *mut *mut c_void) -> HRESULT;
    pub unsafe fn get_trust_level(&self, trust_level: *mut c_void) -> HRESULT;
    // IApplicationView
    pub unsafe fn set_focus(&self) -> HRESULT;
    pub unsafe fn switch_to(&self) -> HRESULT;
    pub unsafe fn try_invoke_back(&self, callback: *mut c_void) -> HRESULT;
    pub unsafe fn get_thumbnail_window(&self, hwnd: *mut HWND) -> HRESULT;
    pub unsafe fn get_monitor(&self, monitor: *mut *mut c_void) -> HRESULT;
    pub unsafe fn get_visibility(&self, visibility: *mut i32) -> HRESULT;
    pub unsafe fn set_cloak(&self, cloak_type: u32, flags: i32) -> HRESULT;
}

#[interface("1841C6D7-4F9D-42C0-AF41-8747538F10E5")]
unsafe trait IApplicationViewCollection: IUnknown {
    pub unsafe fn get_views(&self, views: *mut *mut c_void) -> HRESULT;
    pub unsafe fn get_views_by_zorder(&self, views: *mut *mut c_void) -> HRESULT;
    pub unsafe fn get_views_by_app_user_model_id(
        &self,
        id: PCWSTR,
        views: *mut *mut c_void,
    ) -> HRESULT;
    pub unsafe fn get_view_for_hwnd(
        &self,
        hwnd: HWND,
        view: *mut Option<IApplicationView>,
    ) -> HRESULT;
}

#[derive(Clone)]
struct SharedViewCollection(IApplicationViewCollection);

// SAFETY: obtained in the MTA and only used from MTA threads, where proxies are free-threaded.
unsafe impl Send for SharedViewCollection {}

/// Keeps the MTA alive for the whole process, so threads that never initialize COM are implicitly
/// part of it and can use MTA proxies without a per-thread CoInitializeEx/CoUninitialize.
fn ensure_mta() -> Result<()> {
    static MTA: OnceLock<std::result::Result<(), windows::core::Error>> = OnceLock::new();
    MTA.get_or_init(|| unsafe { CoIncrementMTAUsage() }.map(|_cookie| ()))
        .clone()?;
    Ok(())
}

fn get_application_view(hwnd: isize) -> Result<IApplicationView> {
    ensure_mta()?;
    // A cached proxy dies with explorer; recreate it once in that case.
    for retry in [false, true] {
        let collection = {
            let mut cached = VIEW_COLLECTION
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            match cached.as_ref() {
                Some(collection) => collection.clone(),
                None => cached.insert(create_view_collection()?).clone(),
            }
        };

        let mut view = None;
        match unsafe { collection.0.get_view_for_hwnd(HWND(hwnd as _), &mut view) }.ok() {
            Ok(()) => return view.ok_or(Error::NoApplicationView),
            Err(err) if !retry && is_disconnected(err.code()) => {
                *VIEW_COLLECTION
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner) = None;
            }
            Err(err) => return Err(err.into()),
        }
    }
    unreachable!("the retry iteration always returns")
}

fn create_view_collection() -> Result<SharedViewCollection> {
    unsafe {
        let provider: IServiceProvider =
            CoCreateInstance(&CLSID_IMMERSIVE_SHELL, None, CLSCTX_ALL)?;
        Ok(SharedViewCollection(
            provider.QueryService(&IApplicationViewCollection::IID)?,
        ))
    }
}

fn is_disconnected(code: HRESULT) -> bool {
    [
        RPC_E_DISCONNECTED,
        RPC_E_SERVER_DIED,
        RPC_E_SERVER_DIED_DNE,
        HRESULT(0x800706BA_u32 as _), // RPC_S_SERVER_UNAVAILABLE
        HRESULT(0x800706BF_u32 as _), // RPC_S_CALL_FAILED_DNE
    ]
    .contains(&code)
}

fn set_cloaked_prop(hwnd: isize, cloaked: bool) {
    let hwnd = HWND(hwnd as _);
    let result = unsafe {
        if cloaked {
            SetPropW(hwnd, crate::CLOAKED_PROP, Some(HANDLE(1 as _)))
        } else {
            RemovePropW(hwnd, crate::CLOAKED_PROP).map(|_| ())
        }
    };
    if let Err(err) = result {
        log::error!("Failed to update {hwnd:?} cloaked prop: {err}");
    }
}
