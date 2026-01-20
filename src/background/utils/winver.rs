use std::path::PathBuf;

use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageId;

pub fn is_windows_10() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if (&10240..&22000).contains(&x))
}

#[allow(dead_code)]
pub fn is_windows_11() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if x >= &22000)
}

pub fn has_fixed_runtime() -> bool {
    std::env::var_os("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER").is_some()
}

pub fn get_fixed_runtime_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let folder = exe.parent().and_then(|p| p.parent())?;
    let read_dir = folder.join("runtime").read_dir().ok()?;
    let runtime = read_dir.last()?.ok()?.path();
    if runtime.join("msedgewebview2.exe").exists() {
        Some(runtime)
    } else {
        None
    }
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
    std::env::current_exe().is_ok_and(|p| p.with_file_name("AppxManifest.xml").exists())
}
