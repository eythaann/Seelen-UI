use interprocess::os::windows::security_descriptor::SecurityDescriptor;
use widestring::U16CString;
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree},
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, LookupAccountNameW, PSID,
            SID_NAME_USE, TOKEN_QUERY, TOKEN_USER, TokenUser,
        },
        System::{
            RemoteDesktop::{
                WTS_CURRENT_SERVER_HANDLE, WTS_INFO_CLASS, WTSDomainName, WTSFreeMemory,
                WTSQuerySessionInformationW, WTSUserName,
            },
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    },
    core::{PCWSTR, PWSTR},
};

use crate::{app::current_session_id, error::Result};

/// Max size of a SID in bytes (`SECURITY_MAX_SID_SIZE`)
const MAX_SID_SIZE: usize = 68;

/// Creates the security descriptor for the IPC pipes, only SYSTEM, elevated administrators and
/// the interactive user of the current session can connect to them, other users of the machine
/// are rejected by the OS.
///
/// The administrators group is only enabled on elevated tokens (deny-only otherwise), it covers
/// the service when it was started with credentials of another account (UAC over-the-shoulder).
pub fn create_security_descriptor() -> Result<SecurityDescriptor> {
    let user_sid = session_user_sid().or_else(|err| {
        log::warn!("Failed to get the session user, using the process user instead: {err}");
        process_user_sid()
    })?;
    let sddl = format!("D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;{user_sid})");
    let sddl = U16CString::from_str(sddl).map_err(std::io::Error::other)?;
    Ok(SecurityDescriptor::deserialize(&sddl)?)
}

/// SID of the user logged in the session of this process, as string.
fn session_user_sid() -> Result<String> {
    let session_id = current_session_id()?;
    let user = wts_session_string(session_id, WTSUserName)?;
    let domain = wts_session_string(session_id, WTSDomainName)?;
    if user.is_empty() {
        return Err(std::io::Error::other("No user logged in the session").into());
    }

    let account =
        U16CString::from_str(format!("{domain}\\{user}")).map_err(std::io::Error::other)?;
    let mut sid = [0u8; MAX_SID_SIZE];
    let mut sid_size = MAX_SID_SIZE as u32;
    let mut domain_buffer = [0u16; 256];
    let mut domain_size = domain_buffer.len() as u32;
    let mut sid_use = SID_NAME_USE::default();
    unsafe {
        LookupAccountNameW(
            PCWSTR::null(),
            PCWSTR(account.as_ptr()),
            Some(PSID(sid.as_mut_ptr().cast())),
            &mut sid_size,
            Some(PWSTR(domain_buffer.as_mut_ptr())),
            &mut domain_size,
            &mut sid_use,
        )?;
    }
    sid_to_string(PSID(sid.as_mut_ptr().cast()))
}

/// SID of the user owning this process, as string.
fn process_user_sid() -> Result<String> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        // TOKEN_USER is followed by the SID it points to, u64 keeps the buffer aligned
        let mut buffer = [0u64; 16];
        let mut size = 0;
        let result = GetTokenInformation(
            token,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            size_of_val(&buffer) as u32,
            &mut size,
        );
        let _ = CloseHandle(token);
        result?;

        let token_user = &*buffer.as_ptr().cast::<TOKEN_USER>();
        sid_to_string(token_user.User.Sid)
    }
}

fn wts_session_string(session_id: u32, info: WTS_INFO_CLASS) -> Result<String> {
    unsafe {
        let mut buffer = PWSTR::null();
        let mut bytes = 0;
        WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            session_id,
            info,
            &mut buffer,
            &mut bytes,
        )?;
        let value = buffer.to_string();
        WTSFreeMemory(buffer.0.cast());
        Ok(value.map_err(std::io::Error::other)?)
    }
}

fn sid_to_string(sid: PSID) -> Result<String> {
    unsafe {
        let mut string_sid = PWSTR::null();
        ConvertSidToStringSidW(sid, &mut string_sid)?;
        let value = string_sid.to_string();
        let _ = LocalFree(Some(HLOCAL(string_sid.0.cast())));
        Ok(value.map_err(std::io::Error::other)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // the session user is not checked as CI runners can have no interactive user
    #[test]
    fn creates_security_descriptor() -> Result<()> {
        process_user_sid()?;
        create_security_descriptor()?;
        Ok(())
    }
}
