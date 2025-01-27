use std::thread::JoinHandle;

use crate::error::Result;
use windows::{
    core::{Interface, GUID},
    Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
        COINIT_MULTITHREADED,
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

    fn initialize_multithreaded() -> Result<()> {
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if result.is_err() {
            return Err("CoInitializeEx failed".into());
        }
        Ok(())
    }

    pub fn create_instance<T>(class_id: &GUID) -> Result<T>
    where
        T: Interface,
    {
        Ok(unsafe { CoCreateInstance(class_id, None, CLSCTX_ALL)? })
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

    pub fn run_threaded_with_context<F, T>(f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        std::thread::spawn(|| {
            Self::initialize_multithreaded().expect("failed to initialize multithreaded");
            let result = f();
            Self::uninitialize();
            result
        })
    }
}
