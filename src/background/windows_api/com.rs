use crate::error::Result;
use windows::{
    core::{Interface, GUID},
    Win32::{
        Foundation::RPC_E_CHANGED_MODE,
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
        },
    },
};

pub struct Com {}
impl Com {
    fn initialize() -> Result<ComGuard> {
        let hresult = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hresult.is_err() {
            if hresult == RPC_E_CHANGED_MODE {
                ComGuard { initialized: false };
            }
            return Err(format!("CoInitializeEx failed: {:?}", hresult.message()).into());
        }
        Ok(ComGuard { initialized: true })
    }

    pub fn create_instance<T>(class_id: &GUID) -> Result<T>
    where
        T: Interface,
    {
        unsafe { Ok(CoCreateInstance(class_id, None, CLSCTX_ALL)?) }
    }

    /// Will execute init and drop in a safe way, ensuring that all instances created between init and drop are dropped
    pub fn run_with_context<F, T>(f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let _guard = Self::initialize()?;
        f()
    }
}

struct ComGuard {
    initialized: bool,
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}
