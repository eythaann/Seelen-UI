param (
  [string]$AppPath,
  [string]$Delete
)

$existingPath = [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User)

if ($Delete -eq "true") {
  $pathToDelete = [regex]::Escape($AppPath)
  $newPath = ($existingPath -split ';' | Where-Object {$_ -notmatch "^$pathToDelete$"}) -join ';'
  [Environment]::SetEnvironmentVariable("PATH", $newPath, [System.EnvironmentVariableTarget]::User)
} else {
  if ($existingPath -split ';' -notcontains $AppPath) {
    $newPath = "$AppPath;$existingPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, [System.EnvironmentVariableTarget]::User)
  }
}
