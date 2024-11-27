use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageId;

pub fn is_windows_10() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if (&10240..&22000).contains(&x))
}

pub fn is_windows_11() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if x >= &22000)
}

/// this should be called before call any winvd function
pub fn is_virtual_desktop_supported() -> bool {
    // disable virtual desktop for 24h2
    // matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if (&10240..&26000).contains(&x))
    false
}

pub fn is_msix_intallation() -> bool {
    unsafe {
        let mut len = 0u32;
        let _ = GetCurrentPackageId(&mut len, None);
        let mut buffer = vec![0u8; len as usize];
        GetCurrentPackageId(&mut len, Some(buffer.as_mut_ptr())).is_ok()
    }
}
