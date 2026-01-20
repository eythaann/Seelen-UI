param(
    [Parameter(Mandatory = $false)]
    [ValidateSet("x64", "arm64")]
    [string]$Architecture = "x64"
)

$ErrorActionPreference = "Stop"

Write-Host "Setting up Fixed Runtime for architecture: $Architecture" -ForegroundColor Cyan

$RuntimeDir = Resolve-Path ".\src\runtime" | Select-Object -ExpandProperty Path
$ConfigPath = ".\src\tauri.conf.json"

$CabFile = Get-ChildItem -Path $RuntimeDir -Filter "Microsoft.WebView2.FixedVersionRuntime.*.$Architecture.cab" | Select-Object -First 1

if (-not $CabFile) {
    Write-Error "Could not find .cab file for architecture $Architecture in $RuntimeDir"
    exit 1
}

Write-Host "File found: $($CabFile.Name)" -ForegroundColor Green

# Extract version from filename (e.g., "143.0.3650.139" from "Microsoft.WebView2.FixedVersionRuntime.143.0.3650.139.x64")
if ($CabFile.BaseName -match '(\d+\.\d+\.\d+\.\d+)') {
    $Version = $Matches[1]
}
else {
    Write-Error "Could not extract version from filename: $($CabFile.BaseName)"
    exit 1
}

$RuntimeFolderName = $Version

mkdir "$RuntimeDir/temp" -Force
Expand "$($CabFile.FullName)" -F:* "$RuntimeDir/temp/"
Move-Item "$RuntimeDir\temp\$($CabFile.BaseName)" "$RuntimeDir\$Version"

Write-Host "Runtime expanded successfully" -ForegroundColor Green

$RelativePath = "runtime/$RuntimeFolderName"

Write-Host "Updating tauri.conf.json..." -ForegroundColor Cyan

$Config = Get-Content $ConfigPath -Raw | ConvertFrom-Json

if (-not $Config.bundle.windows) {
    $Config.bundle | Add-Member -MemberType NoteProperty -Name "windows" -Value ([PSCustomObject]@{})
}

$WebviewConfig = [PSCustomObject]@{
    type = "fixedRuntime"
    path = $RelativePath
}

if ($Config.bundle.windows.webviewInstallMode) {
    $Config.bundle.windows.webviewInstallMode = $WebviewConfig
}
else {
    $Config.bundle.windows | Add-Member -MemberType NoteProperty -Name "webviewInstallMode" -Value $WebviewConfig
}

$Config | ConvertTo-Json -Depth 100 | Set-Content $ConfigPath -Encoding UTF8

Write-Host "Configuration updated successfully" -ForegroundColor Green
Write-Host "webviewInstallMode.path: $RelativePath" -ForegroundColor Green
