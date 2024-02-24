param (
  [string]$ExeRoute
)

$runningProcesses = Get-Process | Where-Object { $_.ProcessName -eq "komorebi" }
if ($runningProcesses.Count -eq 0) {
  $task = Get-ScheduledTask -TaskName KomorebiUI -ErrorAction SilentlyContinue
  if ($null -ne $task) { 
    Start-ScheduledTask -TaskName KomorebiUI
  } else {
    Start-Process powershell -WindowStyle Hidden -ArgumentList "-NoProfile -Command `"& '$ExeRoute' -c '$Env:USERPROFILE\.config\komorebi-ui\settings.json'`""
    & (Join-Path $Env:USERPROFILE "\.config\komorebi-ui\komorebic.ahk")
  }
}
