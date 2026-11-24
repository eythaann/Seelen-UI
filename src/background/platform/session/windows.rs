use windows::Win32::System::{
    RemoteDesktop::{
        ProcessIdToSessionId, WTS_CURRENT_SERVER_HANDLE, WTSFreeMemory, WTSINFOW,
        WTSQuerySessionInformationW, WTSSessionInfo,
    },
    Threading::GetCurrentProcessId,
};
use windows_core::PWSTR;

use crate::error::Result;

/// `{session_id}_{logon_time}` (hex): the session id alone is reused by Windows after a reboot or
/// a log off, the logon time makes it unique per logon.
pub fn logon_key() -> Result<String> {
    unsafe {
        let mut session_id = 0;
        ProcessIdToSessionId(GetCurrentProcessId(), &mut session_id)?;

        let mut buffer = PWSTR::null();
        let mut bytes = 0;
        WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            session_id,
            WTSSessionInfo,
            &mut buffer,
            &mut bytes,
        )?;

        let logon_time = if buffer.is_null() || (bytes as usize) < size_of::<WTSINFOW>() {
            None
        } else {
            Some((*buffer.0.cast::<WTSINFOW>()).LogonTime)
        };
        if !buffer.is_null() {
            WTSFreeMemory(buffer.0.cast());
        }

        let logon_time = logon_time.ok_or("Invalid WTSSessionInfo buffer")?;
        Ok(format!("{session_id:x}_{logon_time:x}"))
    }
}
