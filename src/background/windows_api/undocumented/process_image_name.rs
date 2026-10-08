use std::ffi::OsString;

use windows::Wdk::System::SystemInformation::{NtQuerySystemInformation, SYSTEM_INFORMATION_CLASS};
use windows::Win32::{
    Foundation::{HANDLE, STATUS_SUCCESS, UNICODE_STRING},
    Storage::FileSystem::QueryDosDeviceW,
};

use crate::{error::Result, windows_api::string_utils::WindowsString};

use super::WindowsApi;

/// Undocumented `SystemProcessIdInformation` class (see phnt `ntexapi.h`).
const SYSTEM_PROCESS_ID_INFORMATION: SYSTEM_INFORMATION_CLASS = SYSTEM_INFORMATION_CLASS(88);
/// Chars that fit on an `UNICODE_STRING` (its length is an u16 in bytes), plus a null terminator.
const IMAGE_NAME_CAPACITY: usize = (u16::MAX / 2) as usize + 1;
/// NT prefix of network paths (`\Device\Mup\server\share` -> `\\server\share`).
const MUP_DEVICE_PREFIX: &str = r"\Device\Mup\";

#[repr(C)]
struct SystemProcessIdInformation {
    process_id: HANDLE,
    image_name: UNICODE_STRING,
}

impl WindowsApi {
    /// Image path of a process without opening it, so it also works for processes we
    /// are not allowed to open (e.g. running as SYSTEM or elevated). This is how the
    /// non-elevated Task Manager gets the path of those processes.
    pub fn get_process_path_by_pid(process_id: u32) -> Result<OsString> {
        // zero filled, the last char is never written so the result is always null terminated
        let mut image_name = WindowsString::new_to_fill(IMAGE_NAME_CAPACITY);
        let mut info = SystemProcessIdInformation {
            process_id: HANDLE(process_id as usize as _),
            image_name: UNICODE_STRING {
                Length: 0,
                MaximumLength: ((IMAGE_NAME_CAPACITY - 1) * 2) as u16,
                Buffer: image_name.as_pwstr(),
            },
        };

        let status = unsafe {
            NtQuerySystemInformation(
                SYSTEM_PROCESS_ID_INFORMATION,
                &mut info as *mut _ as _,
                std::mem::size_of::<SystemProcessIdInformation>() as u32,
                std::ptr::null_mut(),
            )
        };
        if status != STATUS_SUCCESS {
            return Err(format!(
                "NtQuerySystemInformation(SystemProcessIdInformation) failed with status: {:x}",
                status.0
            )
            .into());
        }

        nt_path_to_dos_path(&image_name.to_string())
    }
}

/// Converts a NT path (`\Device\HarddiskVolume3\dir\file.exe`) to a DOS path (`C:\dir\file.exe`).
fn nt_path_to_dos_path(nt_path: &str) -> Result<OsString> {
    if let Some(rest) = nt_path.strip_prefix(MUP_DEVICE_PREFIX) {
        return Ok(format!(r"\\{rest}").into());
    }

    for letter in 'A'..='Z' {
        let drive = format!("{letter}:");
        // the result is a list of null terminated strings, the first one is the current mapping
        let mut device = WindowsString::new_to_fill(1024);
        let written = unsafe {
            QueryDosDeviceW(
                WindowsString::from_str(&drive).as_pcwstr(),
                Some(device.as_mut_slice()),
            )
        };
        if written == 0 {
            continue;
        }

        if let Some(rest) = nt_path.strip_prefix(&device.to_string())
            && rest.starts_with('\\')
        {
            return Ok(format!("{drive}{rest}").into());
        }
    }

    Err(format!("No drive found for NT path: {nt_path}").into())
}
