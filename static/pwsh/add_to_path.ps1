param (
  [string]$AppPath
)

$existingPath = [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User)
if ($existingPath -split ';' -notcontains $AppPath) {
  $newPath = "$AppPath;$existingPath"
  [Environment]::SetEnvironmentVariable("PATH", $newPath, [System.EnvironmentVariableTarget]::User)
}