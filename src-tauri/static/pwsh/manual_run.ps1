param (
  [string]$ExeRoute
)

$runningProcesses = Get-Process | Where-Object { $_.ProcessName -eq "komorebi" }
if ($runningProcesses.Count -eq 0) {
  if ($null -ne (Get-ScheduledTask -TaskName KomorebiUI -ErrorAction SilentlyContinue)) { 
    Start-ScheduledTask -TaskName "KomorebiUI"
    Start-ScheduledTask -TaskName "KomorebiUI-limited"
  }
  else {
    $jsonPath = Join-Path $Env:USERPROFILE "\.config\komorebi-ui\settings.json"
    $jsonContent = Get-Content -Raw -Path $jsonPath | ConvertFrom-Json

    Start-Process powershell -WindowStyle Hidden -ArgumentList "-NoProfile -Command `"& '$ExeRoute' -c $jsonPath`""

    if ($jsonContent.ahk_enabled -eq $true) {
      & (Join-Path $Env:USERPROFILE "\.config\komorebi-ui\komorebic.ahk")
    }
  }
}
