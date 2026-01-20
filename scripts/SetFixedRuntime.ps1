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

# Clean up old files
Remove-Item "$RuntimeDir\$($CabFile.BaseName)" -Force -Recurse -ErrorAction SilentlyContinue
Remove-Item "$RuntimeDir\$Version" -Force -Recurse -ErrorAction SilentlyContinue

# Extraction
$file = $CabFile.FullName
$destination = "$RuntimeDir"
Start-Process -FilePath "cmd.exe" -ArgumentList "/c expand.exe `"$file`" -f:* `"$destination`" > nul 2>&1" -Wait -WindowStyle Hidden | Out-Null
Move-Item "$RuntimeDir\$($CabFile.BaseName)" "$RuntimeDir\$Version"
Write-Host "Runtime expanded successfully" -ForegroundColor Green

# Update tauri config file
$RelativePath = "runtime/$RuntimeFolderName"
$Config = Get-Content $ConfigPath -Raw | ConvertFrom-Json
$WebviewConfig = [PSCustomObject]@{
    type = "fixedRuntime"
    path = $RelativePath
}
$Config.bundle.windows.webviewInstallMode = $WebviewConfig

$Config | ConvertTo-Json -Depth 100 | Set-Content $ConfigPath -Encoding UTF8

Write-Host "Configuration updated successfully" -ForegroundColor Green
Write-Host "webviewInstallMode.path: $RelativePath" -ForegroundColor Green
