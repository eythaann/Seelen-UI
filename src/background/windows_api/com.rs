use crate::error_handler::Result;
use windows::{
    core::{Interface, GUID},
    Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    },
};

pub struct Com {}
impl Com {
    fn initialize() -> Result<()> {
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if result.is_err() {
            return Err("CoInitializeEx failed".into());
        }
        Ok(())
    }

    pub fn create_instance<T>(class_id: &GUID) -> Result<T>
    where
        T: Interface,
    {
        unsafe { Ok(CoCreateInstance(class_id, None, CLSCTX_ALL)?) }
    }

    /// Can panic if Com instances created between init and drop are still alive (no dropped yet)
    fn uninitialize() {
        unsafe { CoUninitialize() };
    }

    /// Will execute init and drop in a safe way, ensuring that all instances created between init and drop are dropped
    pub fn run_with_context<F, T>(f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        Self::initialize()?;
        let result = f();
        Self::uninitialize();
        result
    }
}
