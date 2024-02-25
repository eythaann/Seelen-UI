param (
  [string]$AppPath,
  [bool]$Delete
)

$existingPath = [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User)

if ($Delete) {
  $newPath = $existingPath -replace [regex]::Escape($AppPath) + ';', ''
  [Environment]::SetEnvironmentVariable("PATH", $newPath, [System.EnvironmentVariableTarget]::User)
} else {
  if ($existingPath -split ';' -notcontains $AppPath) {
    $newPath = "$AppPath;$existingPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, [System.EnvironmentVariableTarget]::User)
  }
}
