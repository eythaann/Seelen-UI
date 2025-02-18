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

pub fn is_running_as_appx() -> bool {
    unsafe {
        let mut len = 0u32;
        let _ = GetCurrentPackageId(&mut len, None);
        let mut buffer = vec![0u8; len as usize];
        GetCurrentPackageId(&mut len, Some(buffer.as_mut_ptr())).is_ok()
    }
}

pub fn was_installed_using_msix() -> bool {
    std::env::current_exe().is_ok_and(|p| {
        p.with_file_name("AppxManifest.xml").exists()
            || p.starts_with("C:\\Program Files\\WindowsApps")
    })
}
