$packages = Get-AppxPackage
$output = @()

foreach ($package in $packages) {
  $manifest = Get-AppxPackageManifest -Package $package.PackageFullName

  $applications = @()
  foreach ($app in $manifest.Package.Applications.Application) {
    if ($null -eq $app.Executable) {
      continue
    }

    $alias = $null
    if ($app.Extensions.Extension) {
      foreach ($extension in $app.Extensions.Extension) {
        if ($extension.Category -eq "windows.appExecutionAlias" -and $extension.AppExecutionAlias) {
          foreach ($executionAlias in $extension.AppExecutionAlias.ExecutionAlias) {
            if ($executionAlias.Alias) {
              $alias = $executionAlias.Alias
              break
            }
          }
        }
      }
    }

    $applications += [PSCustomObject]@{
      AppId             = $app.Id
      Executable        = $app.Executable
      Alias             = $alias
      Square150x150Logo = $app.VisualElements.Square150x150Logo
      Square44x44Logo   = $app.VisualElements.Square44x44Logo
    }
  }

  if ($applications.Count -eq 0) {
    continue
  }

  $resolvedInstallLocation = ""
  
  if ($null -ne $package.InstallLocation) {
    $resolvedInstallLocation = $package.InstallLocation
  }

  # Resolve install location in case it's a symlink
  $target = (Get-Item -Path $package.InstallLocation).Target
  if ($target -is [array]) {
    $target = $target[0]
  }

  if ($null -ne $target -and $target -ne "") {
    $resolvedInstallLocation = $target
  }

  # Convert to string if it's not already
  $selected = [PSCustomObject]@{
    Name            = $package.Name
    Version         = $package.Version
    PublisherId     = $package.PublisherId
    PackageFullName = $package.PackageFullName
    InstallLocation = $resolvedInstallLocation
    StoreLogo       = $manifest.Package.Properties.Logo
    Applications    = $applications
  }

  $output += $selected
}

$output | ConvertTo-Json -Depth 3 -Compress