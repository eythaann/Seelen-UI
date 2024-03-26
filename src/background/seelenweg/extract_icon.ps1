Param
(
  [string]$exe,
  [string]$ExtractDir
)

Add-Type -AssemblyName System.Drawing

$Filepath = (Get-ChildItem -Path $exe -Filter *.exe -ErrorAction SilentlyContinue) | Select-Object -First 1

[System.Reflection.Assembly]::LoadWithPartialName('System.Drawing') | Out-Null
If (-Not (Test-Path $ExtractDir)) {
  New-Item -Path $ExtractDir -ItemType Directory -ErrorAction SilentlyContinue | Out-Null
}
$baseName = [System.IO.Path]::GetFileNameWithoutExtension($Filepath.FullName)
[System.Drawing.Icon]::ExtractAssociatedIcon($Filepath.FullName).ToBitmap().Save("$ExtractDir\$BaseName.png", [System.Drawing.Imaging.ImageFormat]::Png)
