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

    tauri_build::build();
}
