param (
  [string]$SavePath = ".\uwp-manifests.json"
)

if (Test-Path $SavePath) {
  Remove-Item $SavePath
}

$packages = Get-AppxPackage
$output = @()

foreach ($package in $packages) {
  $manifest = Get-AppxPackageManifest -Package $package.PackageFullName
  $selected = [PSCustomObject]@{
    Name            = $package.Name
    Version         = $package.Version
    PublisherId     = $package.PublisherId
    AppId           = $manifest.Package.Applications.Application.Id
    Executable   = $manifest.Package.Applications.Application.Executable
    Logo            = $manifest.Package.Properties.Logo
    PackageFullName = $package.PackageFullName
    InstallLocation = $package.InstallLocation
  }
  $output += $selected
}

$output | ConvertTo-Json | Out-File -FilePath $SavePath -Encoding utf8