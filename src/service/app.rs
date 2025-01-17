/* use windows::{
    core::Result,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Security::{
            DuplicateTokenEx, SecurityImpersonation, TokenPrimary, TOKEN_ADJUST_DEFAULT,
            TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_QUERY,
        },
        System::{
            Threading::{CreateProcessAsUserW, PROCESS_INFORMATION, STARTUPINFOEXW, STARTUPINFOW},
            WTSSession::WTSEnumerateSessionsW,
            WindowsProgramming::WTSQueryUserToken,
        },
    },
};

pub fn start_gui_app(application_path: &str) -> Result<()> {
    unsafe {
        // Step 1: Enumerate active sessions
        let mut session_info = std::ptr::null_mut();
        let mut session_count = 0;

        let success = WTSEnumerateSessionsW(
            None,
            0,
            1, // Version 1
            &mut session_info,
            &mut session_count,
        );

        if !success.as_bool() {
            return Err(windows::core::Error::from_win32());
        }

        let sessions = std::slice::from_raw_parts(session_info, session_count as usize);

        // Step 2: Find an active session with a user
        let mut user_token = HANDLE::default();
        for session in sessions {
            if session.State == windows::Win32::WTSSession::WTSActive {
                if WTSQueryUserToken(session.SessionId, &mut user_token).as_bool() {
                    break;
                }
            }
        }

        // Free session info
        windows::Win32::WTSSession::WTSFreeMemory(session_info as _);

        if user_token.is_invalid() {
            return Err(windows::core::Error::from_win32());
        }

        // Step 3: Duplicate the token to get a primary token
        let mut primary_token = HANDLE::default();
        DuplicateTokenEx(
            user_token,
            TOKEN_ASSIGN_PRIMARY | TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ADJUST_DEFAULT,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &mut primary_token,
        )?;

        // Close the original user token
        CloseHandle(user_token);

        // Step 4: Set up process attributes
        let mut startup_info = STARTUPINFOW::default();
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        startup_info.lpDesktop = windows::core::PCWSTR::from_raw(
            "winsta0\\default\0"
                .encode_utf16()
                .collect::<Vec<u16>>()
                .as_ptr(),
        );

        let mut process_info = PROCESS_INFORMATION::default();

        // Step 5: Launch the GUI application
        CreateProcessAsUserW(
            primary_token,
            application_path,
            None,
            None,
            None,
            false,
            windows::Win32::System::Threading::CREATE_UNICODE_ENVIRONMENT,
            None,
            None,
            &startup_info,
            &mut process_info,
        )?;
    }

    Ok(())
}
 */
