fn main() {
    std::process::Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-Command",
            "Get-WmiObject Win32_Process | Where-Object { $_.CommandLine -like '*seelen.ahk*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }",
        ])
        .spawn()
        .expect("Failed to close ahk")
        .wait()
        .expect("Failed to close ahk");

    if tauri_build::dev() {
        tauri_build::build();
    } else {
        let mut windows = tauri_build::WindowsAttributes::new();
        windows = windows.app_manifest(include_str!("manifest.xml"));

        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
            .expect("Failed to run build script");
    }
}
