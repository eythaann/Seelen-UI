param (
  [string]$ExeRoute
)

$runningProcesses = Get-Process | Where-Object { $_.Path -eq $ExeRoute }
if ($runningProcesses.Count -eq 0) {
  Start-Process powershell -WindowStyle Hidden -ArgumentList "-NoProfile -Command `"& '$ExeRoute' -c '$Env:USERPROFILE\.config\komorebi\settings.json'`""
}
