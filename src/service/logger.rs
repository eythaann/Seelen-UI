use log::{Level, LevelFilter, Metadata, Record};
use windows::Win32::{
    Foundation::HANDLE,
    System::EventLog::{
        DeregisterEventSource, RegisterEventSourceW, ReportEventW, EVENTLOG_ERROR_TYPE,
        EVENTLOG_INFORMATION_TYPE, EVENTLOG_WARNING_TYPE,
    },
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{error::Result, is_local_dev, string_utils::WindowsString, SERVICE_DISPLAY_NAME};

pub struct SluServiceLogger {
    handle: HANDLE,
}

unsafe impl Send for SluServiceLogger {}
unsafe impl Sync for SluServiceLogger {}

impl SluServiceLogger {
    const REG_BASEKEY: &str = r"SYSTEM\CurrentControlSet\Services\EventLog\Application";

    pub fn install() -> Result<()> {
        let current_exe = std::env::current_exe()?;
        let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(Self::REG_BASEKEY)?;
        let (app_key, _) = key.create_subkey(SERVICE_DISPLAY_NAME.to_string())?;
        app_key.set_value("EventMessageFile", &current_exe.as_os_str())?;
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(Self::REG_BASEKEY)?;
        let path = SERVICE_DISPLAY_NAME.to_string();
        if key.open_subkey(&path).is_ok() {
            key.delete_subkey(&path)?;
        }
        Ok(())
    }

    fn create() -> Result<Self> {
        Ok(Self {
            handle: unsafe { RegisterEventSourceW(None, SERVICE_DISPLAY_NAME.as_pcwstr())? },
        })
    }

    pub fn init() -> Result<()> {
        let logger = Box::new(Self::create()?);
        log::set_boxed_logger(logger)?;
        log::set_max_level(LevelFilter::Trace);
        Ok(())
    }
}

impl Drop for SluServiceLogger {
    fn drop(&mut self) {
        let _ = unsafe { DeregisterEventSource(self.handle) };
    }
}

#[repr(u32)]
#[allow(dead_code)]
enum Severity {
    Success = 0,
    Information = 1,
    Warning = 2,
    Error = 3,
}

#[repr(u32)]
#[allow(dead_code)]
enum Customer {
    System = 0,
    Custom = 1,
}

/// Creates an event identifier based on the specified fields.
///
/// # Arguments
/// - `severity`: Severity level (Success, Information, Warning, Error)
/// - `customer`: Customer bit (0 for system code, 1 for custom code)
/// - `facility`: Facility code (up to 12 bits)
/// - `code`: Status code (up to 16 bits)
///
/// # Returns
/// An event identifier in `u32` format.
/// documentation: https://learn.microsoft.com/en-us/windows/win32/eventlog/event-identifiers
const fn event_id(severity: Severity, customer: Customer, facility: u16, code: u16) -> u32 {
    let severity = (severity as u32) << 30; // Most significant 2 bits
    let customer = (customer as u32) << 29; // Customer bit
    let facility = (facility as u32 & 0xFFF) << 16; // 12 bits for the facility
    let code = code as u32 & 0xFFFF; // 16 bits for the status code
    severity | customer | facility | code
}

pub const MSG_ERROR: u32 = event_id(Severity::Error, Customer::System, 0, 1);
pub const MSG_WARNING: u32 = event_id(Severity::Warning, Customer::System, 0, 2);
pub const MSG_INFO: u32 = event_id(Severity::Information, Customer::System, 0, 3);
pub const MSG_DEBUG: u32 = event_id(Severity::Information, Customer::System, 0, 4);
pub const MSG_TRACE: u32 = event_id(Severity::Information, Customer::System, 0, 5);

impl log::Log for SluServiceLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        !is_local_dev()
    }

    /// https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-reporteventw
    fn log(&self, record: &Record) {
        let message = format!("[{:?}] {}", record.level(), record.args());
        println!("{}", message);
        if !self.enabled(record.metadata()) {
            return;
        }

        let (event_type, event_id) = match record.level() {
            Level::Error => (EVENTLOG_ERROR_TYPE, MSG_ERROR),
            Level::Warn => (EVENTLOG_WARNING_TYPE, MSG_WARNING),
            Level::Info => (EVENTLOG_INFORMATION_TYPE, MSG_INFO),
            Level::Debug => (EVENTLOG_INFORMATION_TYPE, MSG_DEBUG),
            Level::Trace => (EVENTLOG_INFORMATION_TYPE, MSG_TRACE),
        };

        let message = WindowsString::from_str(&message);
        unsafe {
            let _ = ReportEventW(
                self.handle,
                event_type,
                0,
                event_id,
                None,
                0,
                Some(&[message.as_pcwstr()]),
                None,
            );
        };
    }

    fn flush(&self) {}
}
