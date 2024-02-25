param (
  [string]$VersionPath,
  [string]$ExeRoute
)

$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  $ownRoute = $MyInvocation.MyCommand.Definition
  Start-Process powershell -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$ownRoute`" -VersionPath `"$VersionPath`" -ExeRoute `"$ExeRoute`" " -Verb RunAs -WindowStyle Hidden
  Exit
}

$taskName = "KomorebiUI"
$jsonPath = Join-Path $Env:USERPROFILE "\.config\komorebi-ui\settings.json"

# ExeRoute => portable version
$command = "& '$ExeRoute' -c $jsonPath" 
# VersionPath => installed version
if ($null -ne $VersionPath) {
  $command = "Get-Content -Raw -Path $VersionPath | ConvertFrom-Json | % { & (Join-Path `$_.path `"komorebi.exe`") -c $jsonPath }"
}
$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -WindowStyle Hidden -Command `"$command`""
$trigger = New-ScheduledTaskTrigger -AtLogon
$settings = New-ScheduledTaskSettingsSet -Priority 2 -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -Hidden

Register-ScheduledTask -Force -Action $action -Trigger $trigger -Settings $settings -TaskName $taskName -User $env:USERNAME -RunLevel Highest

# No Admin Start Actions
$ahkPath = Join-Path $Env:USERPROFILE "\.config\komorebi-ui\komorebic.ahk"
$command = "Get-Content -Raw -Path $jsonPath | ConvertFrom-Json | % { if (`$_.ahk_enabled -eq `$true) { & '$ahkPath' } }"
$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -WindowStyle Hidden -Command `"$command`""
Register-ScheduledTask -Force -Action $action -Trigger $trigger -Settings $settings -TaskName "$taskName-limited" -User $env:USERNAME -RunLevel Limited
