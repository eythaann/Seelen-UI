param(
    [Parameter(Mandatory = $false)]
    [ValidateSet("x64", "arm64")]
    [string]$Architecture = "x64",

    [Parameter(Mandatory = $false)]
    [string]$Version = "143.0.3650.139"
)

$ErrorActionPreference = "Stop"

Write-Host "Setting up Fixed Runtime for architecture: $Architecture, version: $Version"

$RuntimeDir = ".\src\runtime"
$ConfigPath = ".\src\tauri.conf.json"

mkdir $RuntimeDir -Force

# Build download URL
$FileName = "Microsoft.WebView2.FixedVersionRuntime.$Version.$Architecture.cab"
$DownloadUrl = "https://github.com/Seelen-Corp/webview2-archive/releases/download/$Version/$FileName"
$CabFilePath = Join-Path $RuntimeDir $FileName

# Download the .cab file if it doesn't exist
if (Test-Path $CabFilePath) {
    Write-Host "File already exists, skipping download: $FileName"
}
else {
    Write-Host "Downloading from: $DownloadUrl" -ForegroundColor Cyan
    try {
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $CabFilePath -UseBasicParsing
        Write-Host "Download completed: $FileName" -ForegroundColor Green
    }
    catch {
        Write-Error "Failed to download file from $DownloadUrl. Error: $_"
        exit 1
    }
}

$CabFile = Get-Item $CabFilePath

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
$Config.bundle.windows | Add-Member -Force -MemberType NoteProperty -Name "webviewInstallMode" -Value $WebviewConfig

$Config | ConvertTo-Json -Depth 100 | Set-Content $ConfigPath -Encoding UTF8

Write-Host "Configuration updated successfully" -ForegroundColor Green
Write-Host "webviewInstallMode.path: $RelativePath"
