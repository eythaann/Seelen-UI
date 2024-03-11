# Stop Task if exist
$task = Get-ScheduledTask -TaskName KomorebiUI -ErrorAction SilentlyContinue
if ($null -ne $task -and $task.State -eq "Running") { 
  Stop-ScheduledTask -TaskName "KomorebiUI"
  Stop-ScheduledTask -TaskName "KomorebiUI-limited"
}

# Stop AHK script
$jsonPath = Join-Path $Env:USERPROFILE "\.config\komorebi-ui\settings.json"
$jsonContent = Get-Content -Raw -Path $jsonPath | ConvertFrom-Json
if ($jsonContent.ahk_enabled -eq $true) {
  wmic process where "commandline like '%komorebic.ahk%'" call terminate
}

# Check for others komorebi process caused by others sources.
$runningProcesses = Get-Process | Where-Object { $_.ProcessName -eq "komorebi" }
if ($runningProcesses.Count -gt 0) {
  Stop-Process -Id $runningProcesses.Id
}