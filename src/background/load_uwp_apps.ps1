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

  $applications = @()
  foreach ($app in $manifest.Package.Applications.Application) {
    if ($null -eq $app.Executable) {
      continue
    }

    $applications += [PSCustomObject]@{
      AppId             = $app.Id
      Executable        = $app.Executable
      Square150x150Logo = $app.VisualElements.Square150x150Logo
      Square44x44Logo   = $app.VisualElements.Square44x44Logo
    }
  }

  if ($applications.Count -eq 0) {
    continue
  }

  # Resolve install location in case it's a symlink
  $resolvedInstallLocation = (Get-Item -Path $package.InstallLocation).Target[0]
  if ($null -eq $resolvedInstallLocation) {
    $resolvedInstallLocation = $package.InstallLocation
  }

  $selected = [PSCustomObject]@{
    Name            = $package.Name
    Version         = $package.Version
    PublisherId     = $package.PublisherId
    PackageFullName = $package.PackageFullName
    InstallLocation = $resolvedInstallLocation.TrimEnd('\')
    StoreLogo       = $manifest.Package.Properties.Logo
    Applications    = $applications
  }

  $output += $selected
}

$output | ConvertTo-Json -Depth 3 | Out-File -FilePath $SavePath -Encoding utf8