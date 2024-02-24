param (
  [string]$ExeRoute
)

$runningProcesses = Get-Process | Where-Object { $_.ProcessName -eq "komorebi" }
if ($runningProcesses.Count -eq 0) {
  $task = Get-ScheduledTask -TaskName KomorebiUI -ErrorAction SilentlyContinue
  if ($null -ne $task) { 
    Start-ScheduledTask -TaskName KomorebiUI
  }
  else {
    $jsonPath = Join-Path $Env:USERPROFILE "\.config\komorebi-ui\settings.json"
    $jsonContent = Get-Content -Raw -Path $jsonPath | ConvertFrom-Json

    Start-Process powershell -WindowStyle Hidden -ArgumentList "-NoProfile -Command `"& '$ExeRoute' -c '$Env:USERPROFILE\.config\komorebi-ui\settings.json'`""

    if ($jsonContent.ahk_enabled -eq $true) {
      & (Join-Path $Env:USERPROFILE "\.config\komorebi-ui\komorebic.ahk")
    }
  }
}
